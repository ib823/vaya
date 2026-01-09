//! Retry logic and rate limiting

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::{CollectError, CollectResult};

/// Retry strategy
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay between retries (milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Add jitter to delays
    pub jitter: bool,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryStrategy {
    /// Create a new retry strategy
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retries
    pub fn max_retries(mut self, count: u32) -> Self {
        self.max_retries = count;
        self
    }

    /// Set initial delay
    pub fn initial_delay(mut self, ms: u64) -> Self {
        self.initial_delay_ms = ms;
        self
    }

    /// Set backoff multiplier
    pub fn backoff(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Disable jitter
    pub fn no_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }

    /// Calculate delay for a given attempt (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay =
            self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let delay_ms = base_delay.min(self.max_delay_ms as f64);

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (simple_random() * 0.25);
            delay_ms * jitter_factor
        } else {
            delay_ms
        };

        Duration::from_millis(final_delay as u64)
    }

    /// Check if an error should be retried
    pub fn should_retry(&self, error: &CollectError, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        matches!(
            error,
            CollectError::Io(_)
                | CollectError::ConnectionFailed(_)
                | CollectError::Timeout
                | CollectError::HttpError(500..=599, _)
                | CollectError::HttpError(429, _)
        )
    }
}

/// Simple pseudo-random number generator (0.0 to 1.0)
/// Not cryptographically secure, but good enough for jitter
fn simple_random() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

/// Rate limiter for API requests
pub struct RateLimiter {
    /// Requests per window
    requests_per_window: u32,
    /// Window duration
    window_duration: Duration,
    /// Tracking per host
    buckets: Mutex<HashMap<String, RateBucket>>,
}

/// Rate limit bucket for a single host
struct RateBucket {
    /// Timestamps of recent requests
    timestamps: Vec<Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_window: requests_per_second,
            window_duration: Duration::from_secs(1),
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Create with custom window
    pub fn with_window(requests: u32, window: Duration) -> Self {
        Self {
            requests_per_window: requests,
            window_duration: window,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Check if request is allowed and record it
    pub fn check(&self, host: &str) -> CollectResult<()> {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets.entry(host.to_string()).or_insert(RateBucket {
            timestamps: Vec::new(),
        });

        let now = Instant::now();
        let window_start = now - self.window_duration;

        // Remove old timestamps
        bucket.timestamps.retain(|&t| t > window_start);

        // Check if we can make a request
        if bucket.timestamps.len() >= self.requests_per_window as usize {
            // Calculate time until oldest request expires
            if let Some(&oldest) = bucket.timestamps.first() {
                let wait_time = self.window_duration - (now - oldest);
                return Err(CollectError::RateLimited(wait_time.as_secs()));
            }
        }

        // Record this request
        bucket.timestamps.push(now);
        Ok(())
    }

    /// Wait until a request is allowed
    pub fn wait(&self, host: &str) {
        loop {
            match self.check(host) {
                Ok(()) => return,
                Err(CollectError::RateLimited(seconds)) => {
                    std::thread::sleep(Duration::from_secs(seconds.max(1)));
                }
                _ => return,
            }
        }
    }

    /// Get current request count for a host
    pub fn current_count(&self, host: &str) -> u32 {
        let buckets = self.buckets.lock().unwrap();
        buckets
            .get(host)
            .map(|b| {
                let now = Instant::now();
                let window_start = now - self.window_duration;
                b.timestamps.iter().filter(|&&t| t > window_start).count() as u32
            })
            .unwrap_or(0)
    }

    /// Clear rate limit tracking for a host
    pub fn clear(&self, host: &str) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.remove(host);
    }

    /// Clear all rate limit tracking
    pub fn clear_all(&self) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.clear();
    }
}

/// Circuit breaker for failing services
pub struct CircuitBreaker {
    /// Failure threshold before opening circuit
    failure_threshold: u32,
    /// Reset timeout when circuit is open
    reset_timeout: Duration,
    /// State per host
    states: Mutex<HashMap<String, CircuitState>>,
}

/// Circuit breaker state for a single host
struct CircuitState {
    /// Current state
    state: CircuitStatus,
    /// Consecutive failure count
    failures: u32,
    /// Time of last state change
    last_change: Instant,
}

/// Circuit status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitStatus {
    /// Circuit is closed, requests allowed
    Closed,
    /// Circuit is open, requests blocked
    Open,
    /// Circuit is half-open, allowing test request
    HalfOpen,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            reset_timeout,
            states: Mutex::new(HashMap::new()),
        }
    }

    /// Check if request should be allowed
    pub fn check(&self, host: &str) -> CollectResult<()> {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(host.to_string()).or_insert(CircuitState {
            state: CircuitStatus::Closed,
            failures: 0,
            last_change: Instant::now(),
        });

        match state.state {
            CircuitStatus::Closed => Ok(()),
            CircuitStatus::Open => {
                if state.last_change.elapsed() >= self.reset_timeout {
                    state.state = CircuitStatus::HalfOpen;
                    state.last_change = Instant::now();
                    Ok(())
                } else {
                    Err(CollectError::ConnectionFailed(format!(
                        "Circuit breaker open for {}",
                        host
                    )))
                }
            }
            CircuitStatus::HalfOpen => Ok(()),
        }
    }

    /// Record a successful request
    pub fn record_success(&self, host: &str) {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.get_mut(host) {
            state.state = CircuitStatus::Closed;
            state.failures = 0;
            state.last_change = Instant::now();
        }
    }

    /// Record a failed request
    pub fn record_failure(&self, host: &str) {
        let mut states = self.states.lock().unwrap();
        let state = states.entry(host.to_string()).or_insert(CircuitState {
            state: CircuitStatus::Closed,
            failures: 0,
            last_change: Instant::now(),
        });

        state.failures += 1;

        if state.failures >= self.failure_threshold {
            state.state = CircuitStatus::Open;
            state.last_change = Instant::now();
        }
    }

    /// Get current status for a host
    pub fn status(&self, host: &str) -> CircuitStatus {
        let states = self.states.lock().unwrap();
        states
            .get(host)
            .map(|s| s.state)
            .unwrap_or(CircuitStatus::Closed)
    }

    /// Reset circuit breaker for a host
    pub fn reset(&self, host: &str) {
        let mut states = self.states.lock().unwrap();
        states.remove(host);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy::new()
            .max_retries(3)
            .initial_delay(100)
            .no_jitter();

        assert_eq!(strategy.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(strategy.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(strategy.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn test_retry_should_retry() {
        let strategy = RetryStrategy::new().max_retries(3);

        assert!(strategy.should_retry(&CollectError::Timeout, 0));
        assert!(strategy.should_retry(&CollectError::HttpError(500, "Server Error".into()), 1));
        assert!(!strategy.should_retry(&CollectError::HttpError(404, "Not Found".into()), 0));
        assert!(!strategy.should_retry(&CollectError::Timeout, 3)); // Exceeded max retries
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(2);

        // First two requests should succeed
        assert!(limiter.check("example.com").is_ok());
        assert!(limiter.check("example.com").is_ok());

        // Third should be rate limited
        assert!(limiter.check("example.com").is_err());

        // Different host should be allowed
        assert!(limiter.check("other.com").is_ok());
    }

    #[test]
    fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(100));

        // Circuit should be closed initially
        assert!(breaker.check("example.com").is_ok());

        // Record failures
        breaker.record_failure("example.com");
        assert!(breaker.check("example.com").is_ok()); // Still closed

        breaker.record_failure("example.com");
        // Now circuit should be open
        assert!(breaker.check("example.com").is_err());
        assert_eq!(breaker.status("example.com"), CircuitStatus::Open);
    }
}

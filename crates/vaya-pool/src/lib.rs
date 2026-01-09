//! vaya-pool: Group buying pools for travel deals
//!
//! This crate provides group buying functionality for travel bookings:
//!
//! - **Pool management**: Create and manage buying pools with state machine
//! - **Tiered pricing**: Automatic discounts based on group size
//! - **Member management**: Join, leave, and contribute to pools
//! - **Price locks**: Guaranteed pricing for members at join time
//!
//! # How It Works
//!
//! 1. An organizer creates a pool with a route and pricing structure
//! 2. Members join the pool, each getting a price lock
//! 3. When minimum members is reached, pool moves to Active
//! 4. Members contribute their share (payment)
//! 5. When all contributions are received, pool is Locked
//! 6. System books the flights and pool becomes Completed
//!
//! # Pricing Tiers
//!
//! Standard tiers provide increasing discounts:
//! - Silver (5+ members): 5% off
//! - Gold (10+ members): 10% off
//! - Platinum (20+ members): 15% off
//! - Diamond (50+ members): 20% off

mod error;
mod pool;
mod pricing;

pub use error::{PoolError, PoolResult};
pub use pool::{Pool, PoolMember, PoolRoute, PoolStatus, StatusChange};
pub use pricing::{PriceLock, PricingTier, TieredPricing};

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Default minimum members
    pub default_min_members: u32,
    /// Default maximum members
    pub default_max_members: u32,
    /// Default join deadline (seconds from creation)
    pub default_join_deadline_secs: i64,
    /// Default contribution deadline (seconds from creation)
    pub default_contribution_deadline_secs: i64,
    /// Price lock duration (seconds)
    pub price_lock_duration_secs: i64,
    /// Maximum spots per member
    pub max_spots_per_member: u32,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            default_min_members: 5,
            default_max_members: 50,
            default_join_deadline_secs: 7 * 24 * 3600, // 7 days
            default_contribution_deadline_secs: 10 * 24 * 3600, // 10 days
            price_lock_duration_secs: 24 * 3600,       // 24 hours
            max_spots_per_member: 10,
        }
    }
}

impl PoolConfig {
    /// Create a new pool configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default minimum members
    pub fn with_min_members(mut self, min: u32) -> Self {
        self.default_min_members = min;
        self
    }

    /// Set default maximum members
    pub fn with_max_members(mut self, max: u32) -> Self {
        self.default_max_members = max;
        self
    }

    /// Set default join deadline
    pub fn with_join_deadline(mut self, secs: i64) -> Self {
        self.default_join_deadline_secs = secs;
        self
    }

    /// Set default contribution deadline
    pub fn with_contribution_deadline(mut self, secs: i64) -> Self {
        self.default_contribution_deadline_secs = secs;
        self
    }
}

/// Validate pool member count
pub fn validate_member_count(current: u32, min: u32, max: u32) -> PoolResult<()> {
    if current > max {
        return Err(PoolError::MemberLimitReached);
    }

    if current < min {
        return Err(PoolError::MinMembersNotReached {
            required: min,
            current,
        });
    }

    Ok(())
}

/// Calculate pool progress percentage
pub fn calculate_progress(current_members: u32, target_members: u32) -> u8 {
    if target_members == 0 {
        return 100;
    }
    let progress = (current_members as f64 / target_members as f64 * 100.0) as u8;
    progress.min(100)
}

/// Pool summary statistics
#[derive(Debug, Clone)]
pub struct PoolSummary {
    /// Pool ID
    pub pool_id: String,
    /// Current member count
    pub member_count: u32,
    /// Total spots claimed
    pub total_spots: u32,
    /// Minimum required
    pub min_required: u32,
    /// Maximum allowed
    pub max_allowed: u32,
    /// Progress percentage
    pub progress_percent: u8,
    /// Current price per person
    pub current_price: vaya_common::MinorUnits,
    /// Current discount percentage
    pub discount_percent: u8,
    /// Total savings
    pub total_savings: vaya_common::MinorUnits,
    /// Members needed for next tier
    pub members_to_next_tier: Option<u32>,
    /// Time remaining to deadline (seconds)
    pub time_remaining: Option<i64>,
    /// Is pool joinable
    pub is_joinable: bool,
    /// Is pool full
    pub is_full: bool,
}

impl PoolSummary {
    /// Create summary from pool
    pub fn from_pool(pool: &Pool) -> Self {
        let next_tier = pool.pricing.get_next_tier(pool.total_spots());

        Self {
            pool_id: pool.id.clone(),
            member_count: pool.member_count(),
            total_spots: pool.total_spots(),
            min_required: pool.min_members,
            max_allowed: pool.max_members,
            progress_percent: calculate_progress(pool.total_spots(), pool.min_members),
            current_price: pool.current_price_per_person(),
            discount_percent: pool.current_discount(),
            total_savings: pool.potential_savings(),
            members_to_next_tier: next_tier.map(|(_, needed)| needed),
            time_remaining: pool.time_to_deadline(),
            is_joinable: pool.status.is_joinable() && !pool.is_full(),
            is_full: pool.is_full(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Date;
    use vaya_common::{CurrencyCode, IataCode, MinorUnits};

    fn create_test_pool() -> Pool {
        let route = PoolRoute::one_way(
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2025, time::Month::June, 15).unwrap(),
        );
        let pricing =
            TieredPricing::with_standard_tiers(MinorUnits::new(10000), CurrencyCode::SGD).unwrap();

        Pool::new("Test Pool", route, pricing, "organizer", 1).unwrap()
    }

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.default_min_members, 5);
        assert_eq!(config.default_max_members, 50);
        assert_eq!(config.price_lock_duration_secs, 24 * 3600);
    }

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::new()
            .with_min_members(3)
            .with_max_members(100)
            .with_join_deadline(3 * 24 * 3600);

        assert_eq!(config.default_min_members, 3);
        assert_eq!(config.default_max_members, 100);
        assert_eq!(config.default_join_deadline_secs, 3 * 24 * 3600);
    }

    #[test]
    fn test_validate_member_count() {
        // Valid
        assert!(validate_member_count(5, 3, 10).is_ok());

        // Too many
        assert!(matches!(
            validate_member_count(11, 3, 10),
            Err(PoolError::MemberLimitReached)
        ));

        // Too few
        assert!(matches!(
            validate_member_count(2, 3, 10),
            Err(PoolError::MinMembersNotReached { .. })
        ));
    }

    #[test]
    fn test_calculate_progress() {
        assert_eq!(calculate_progress(0, 10), 0);
        assert_eq!(calculate_progress(5, 10), 50);
        assert_eq!(calculate_progress(10, 10), 100);
        assert_eq!(calculate_progress(15, 10), 100); // Capped at 100
        assert_eq!(calculate_progress(5, 0), 100); // Edge case
    }

    #[test]
    fn test_pool_summary() {
        let pool = create_test_pool();
        let summary = PoolSummary::from_pool(&pool);

        assert_eq!(summary.pool_id, pool.id);
        assert_eq!(summary.member_count, 1);
        assert_eq!(summary.total_spots, 1);
        assert_eq!(summary.min_required, 5);
        assert!(summary.is_joinable);
        assert!(!summary.is_full);
    }

    #[test]
    fn test_pool_summary_with_members() {
        let mut pool = create_test_pool();

        // Add members to reach Silver tier (5 members)
        for i in 2..=5 {
            pool.join(&format!("user-{}", i), 1).unwrap();
        }

        let summary = PoolSummary::from_pool(&pool);

        assert_eq!(summary.total_spots, 5);
        assert_eq!(summary.discount_percent, 5); // Silver tier
        assert_eq!(summary.progress_percent, 100);

        // Should show members needed for Gold (10)
        assert_eq!(summary.members_to_next_tier, Some(5));
    }
}

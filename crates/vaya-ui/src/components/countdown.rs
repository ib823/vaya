//! Countdown Component
//!
//! Displays a countdown timer with warning state when time is low.

use gloo_timers::callback::Interval;
use leptos::*;

/// Countdown timer component
#[component]
pub fn Countdown(
    /// End timestamp (ISO 8601 string or Unix timestamp in seconds)
    #[prop(into)]
    end_time: String,
    /// Callback when countdown reaches zero
    #[prop(optional)]
    on_complete: Option<Callback<()>>,
    /// Seconds threshold for warning state (default: 300 = 5 minutes)
    #[prop(optional)]
    warning_threshold: Option<i64>,
    /// Show days in display
    #[prop(optional)]
    show_days: bool,
    /// Compact display (just numbers, no labels)
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let warning_secs = warning_threshold.unwrap_or(300);
    let (remaining, set_remaining) = create_signal(0i64);
    let (is_complete, set_complete) = create_signal(false);

    // Parse end time and calculate initial remaining
    let end_timestamp = parse_end_time(&end_time);

    // Update remaining time every second
    let _interval = store_value(Interval::new(1000, move || {
        let now = js_sys::Date::now() as i64 / 1000;
        let diff = end_timestamp - now;

        if diff <= 0 {
            set_remaining.set(0);
            if !is_complete.get() {
                set_complete.set(true);
                if let Some(cb) = on_complete {
                    cb.call(());
                }
            }
        } else {
            set_remaining.set(diff);
        }
    }));

    // Initial calculation
    {
        let now = js_sys::Date::now() as i64 / 1000;
        let diff = end_timestamp - now;
        set_remaining.set(diff.max(0));
    }

    let is_warning = move || remaining.get() > 0 && remaining.get() <= warning_secs;

    let countdown_class = move || {
        let mut classes = vec!["countdown"];
        if is_warning() {
            classes.push("countdown-warning");
        }
        if is_complete.get() {
            classes.push("countdown-complete");
        }
        if compact {
            classes.push("countdown-compact");
        }
        classes.join(" ")
    };

    view! {
        <div class=countdown_class>
            {move || {
                let secs = remaining.get();
                if secs <= 0 {
                    view! {
                        <span class="countdown-expired">"Expired"</span>
                    }.into_view()
                } else if compact {
                    view! {
                        <span class="countdown-time">{format_time_compact(secs, show_days)}</span>
                    }.into_view()
                } else {
                    format_time_segments(secs, show_days).into_view()
                }
            }}
        </div>
    }
}

/// Parse end time from string (ISO 8601 or Unix timestamp)
fn parse_end_time(end_time: &str) -> i64 {
    // Try parsing as Unix timestamp first
    if let Ok(ts) = end_time.parse::<i64>() {
        return ts;
    }

    // Try parsing as ISO 8601 date string using js_sys
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(end_time));
    (date.get_time() / 1000.0) as i64
}

/// Format time as "HH:MM:SS" or "D:HH:MM:SS"
fn format_time_compact(total_secs: i64, show_days: bool) -> String {
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if show_days && days > 0 {
        format!("{}:{:02}:{:02}:{:02}", days, hours, minutes, seconds)
    } else if hours > 0 || days > 0 {
        let total_hours = days * 24 + hours;
        format!("{:02}:{:02}:{:02}", total_hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Format time as separate segments with labels
fn format_time_segments(total_secs: i64, show_days: bool) -> impl IntoView {
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    view! {
        <div class="countdown-segments">
            {(show_days && days > 0).then(|| view! {
                <div class="countdown-segment">
                    <span class="countdown-value">{days}</span>
                    <span class="countdown-label">"d"</span>
                </div>
                <span class="countdown-separator">":"</span>
            })}
            {(hours > 0 || days > 0).then(|| view! {
                <div class="countdown-segment">
                    <span class="countdown-value">{format!("{:02}", hours)}</span>
                    <span class="countdown-label">"h"</span>
                </div>
                <span class="countdown-separator">":"</span>
            })}
            <div class="countdown-segment">
                <span class="countdown-value">{format!("{:02}", minutes)}</span>
                <span class="countdown-label">"m"</span>
            </div>
            <span class="countdown-separator">":"</span>
            <div class="countdown-segment">
                <span class="countdown-value">{format!("{:02}", seconds)}</span>
                <span class="countdown-label">"s"</span>
            </div>
        </div>
    }
}

/// Mini countdown for inline use
#[component]
pub fn CountdownMini(
    /// End timestamp
    #[prop(into)]
    end_time: String,
    /// Prefix text (e.g., "Expires in")
    #[prop(optional, into)]
    prefix: Option<String>,
) -> impl IntoView {
    let (remaining, set_remaining) = create_signal(0i64);
    let end_timestamp = parse_end_time(&end_time);

    let _interval = store_value(Interval::new(1000, move || {
        let now = js_sys::Date::now() as i64 / 1000;
        let diff = end_timestamp - now;
        set_remaining.set(diff.max(0));
    }));

    // Initial calculation
    {
        let now = js_sys::Date::now() as i64 / 1000;
        set_remaining.set((end_timestamp - now).max(0));
    }

    view! {
        <span class="countdown-mini">
            {prefix.map(|p| view! { <span class="countdown-mini-prefix">{p}" "</span> })}
            <span class="countdown-mini-time">{move || format_time_compact(remaining.get(), false)}</span>
        </span>
    }
}

/// Price lock countdown with special styling
#[component]
pub fn PriceLockCountdown(
    /// End timestamp
    #[prop(into)]
    end_time: String,
    /// Callback when expired
    #[prop(optional)]
    on_expire: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="price-lock-countdown">
            <span class="lock-countdown-label">"Price locked for"</span>
            <Countdown
                end_time=end_time
                on_complete=on_expire.unwrap_or_else(|| Callback::new(|_| {}))
                warning_threshold=3600  // 1 hour warning
                show_days=true
                compact=false
            />
        </div>
    }
}

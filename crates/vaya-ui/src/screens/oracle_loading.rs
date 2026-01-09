//! Oracle Loading Screen
//!
//! Displayed while the Oracle is processing the prediction.
//! Shows animated loading state with rotating messages.

use gloo_timers::callback::Interval;
use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::types::{OraclePrediction, OracleVerdict, Price, PriceTrend};

/// Loading messages to display during Oracle processing
const LOADING_MESSAGES: &[&str] = &[
    "Analyzing 847 million data points...",
    "Checking historical patterns...",
    "Calculating optimal booking window...",
    "Comparing airline pricing models...",
    "Finalizing recommendation...",
];

/// Generate mock prediction data for demo purposes
/// In production, this would be replaced by actual API call
fn generate_mock_prediction(origin: &str, destination: &str) -> OraclePrediction {
    // Deterministic "random" based on route to keep consistent during demo
    let hash = origin
        .bytes()
        .chain(destination.bytes())
        .fold(0u32, |acc, b| acc.wrapping_add(b as u32));

    let verdict = match hash % 4 {
        0 => OracleVerdict::BookNow,
        1 => OracleVerdict::Wait,
        2 => OracleVerdict::JoinPool,
        _ => OracleVerdict::Uncertain,
    };

    let confidence = 70 + (hash % 25) as u8; // 70-94%
    let base_price = 80000 + (hash % 120000) as i64; // 800-2000 MYR
    let price_delta = (hash % 30000) as i64 - 15000; // -150 to +150 MYR

    let wait_days = if verdict == OracleVerdict::Wait {
        Some(3 + (hash % 12))
    } else {
        None
    };

    let price_trend = match hash % 4 {
        0 => PriceTrend::Rising,
        1 => PriceTrend::Falling,
        2 => PriceTrend::Stable,
        _ => PriceTrend::Volatile,
    };

    OraclePrediction {
        id: format!("pred-{}-{}", origin, destination),
        verdict,
        confidence,
        current_price: Price::myr(base_price),
        predicted_price: Some(Price::myr(base_price + price_delta)),
        wait_days,
        price_trend: Some(price_trend),
        reasoning: vec![
            "Historical data analyzed from 847M+ data points".to_string(),
            format!(
                "Current price is {}% vs 30-day average",
                if price_delta > 0 { "above" } else { "below" }
            ),
            "Demand patterns suggest optimal booking window".to_string(),
            format!("Confidence based on {} similar routes", 1000 + hash % 5000),
        ],
    }
}

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Oracle loading screen component
#[component]
pub fn OracleLoading() -> impl IntoView {
    // Message rotation state
    let (message_index, set_message_index) = create_signal(0usize);

    // Set up interval for message rotation
    let _interval = store_value(Interval::new(2000, move || {
        set_message_index.update(|i| *i = (*i + 1) % LOADING_MESSAGES.len());
    }));

    // Get search params from URL
    let params = leptos_router::use_query_map();

    let origin = move || params.get().get("origin").cloned().unwrap_or_default();
    let destination = move || params.get().get("destination").cloned().unwrap_or_default();
    let date = move || params.get().get("date").cloned().unwrap_or_default();
    let pax = move || {
        params
            .get()
            .get("pax")
            .cloned()
            .unwrap_or_else(|| "1".to_string())
    };

    let route_display = move || {
        let o = origin();
        let d = destination();
        if o.is_empty() || d.is_empty() {
            "Your route".to_string()
        } else {
            format!("{} → {}", o, d)
        }
    };

    // Create prediction after loading animation completes
    let navigate = use_navigate();

    // After going through all messages, navigate to result
    create_effect(move |_| {
        if message_index.get() >= LOADING_MESSAGES.len() - 1 {
            // Slight delay after final message
            let nav = navigate.clone();
            let o = origin();
            let d = destination();
            let dt = date();
            let p = pax();

            set_timeout(
                move || {
                    // Generate prediction and store in session storage for result screen
                    let prediction = generate_mock_prediction(&o, &d);

                    // Store prediction data in sessionStorage
                    if let Some(storage) = get_session_storage() {
                        if let Ok(json) = serde_json::to_string(&prediction) {
                            let _ = storage.set_item("oracle_prediction", &json);
                            let _ = storage.set_item("oracle_origin", &o);
                            let _ = storage.set_item("oracle_destination", &d);
                            let _ = storage.set_item("oracle_date", &dt);
                            let _ = storage.set_item("oracle_pax", &p);
                        }
                    }

                    // Navigate to result
                    nav(
                        &format!("/oracle/result/{}", prediction.id),
                        Default::default(),
                    );
                },
                std::time::Duration::from_millis(1500),
            );
        }
    });

    view! {
        <div class="screen-oracle-loading">
            <div class="oracle-loading-container">
                // Animated Oracle visual
                <div class="oracle-visual">
                    <div class="oracle-ring oracle-ring-outer"></div>
                    <div class="oracle-ring oracle-ring-middle"></div>
                    <div class="oracle-ring oracle-ring-inner"></div>
                    <div class="oracle-core">
                        <div class="oracle-pulse"></div>
                    </div>
                </div>

                // Route being analyzed
                <div class="oracle-route">{route_display}</div>

                // Status text
                <h1 class="oracle-title">"The Oracle is thinking..."</h1>

                // Rotating message
                <p class="oracle-message">
                    {move || LOADING_MESSAGES[message_index.get()]}
                </p>

                // Progress dots
                <div class="oracle-progress">
                    {(0..5)
                        .map(|i| {
                            let is_active = move || message_index.get() >= i;
                            view! {
                                <div class=move || {
                                    if is_active() { "progress-dot progress-dot-active" } else { "progress-dot" }
                                }></div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

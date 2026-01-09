//! Oracle Result Screen
//!
//! Displays the Oracle's prediction verdict with confidence level.
//! This is the "money maker" screen - the core value proposition of VAYA.

use leptos::*;
use leptos_router::{use_navigate, use_params_map};
use web_sys::Storage;

use crate::types::{confidence_label, OraclePrediction, OracleVerdict, Price, PriceTrend};

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Load prediction from session storage (set by loading screen)
fn load_prediction_from_storage() -> Option<(OraclePrediction, String, String, String)> {
    let storage = get_session_storage()?;

    let json = storage.get_item("oracle_prediction").ok()??;
    let prediction: OraclePrediction = serde_json::from_str(&json).ok()?;

    let origin = storage.get_item("oracle_origin").ok()?.unwrap_or_default();
    let destination = storage
        .get_item("oracle_destination")
        .ok()?
        .unwrap_or_default();
    let date = storage.get_item("oracle_date").ok()?.unwrap_or_default();

    Some((prediction, origin, destination, date))
}

/// Generate fallback mock prediction if storage is empty
fn fallback_prediction() -> OraclePrediction {
    OraclePrediction {
        id: "fallback".to_string(),
        verdict: OracleVerdict::BookNow,
        confidence: 94,
        current_price: Price::myr(158900),
        predicted_price: Some(Price::myr(178500)),
        wait_days: None,
        price_trend: Some(PriceTrend::Rising),
        reasoning: vec![
            "Prices have increased 12% in the last 7 days".to_string(),
            "Historical data shows prices peak 3 weeks before departure".to_string(),
            "Current price is 8% below the 30-day average".to_string(),
            "High demand expected for this route next week".to_string(),
        ],
    }
}

/// Format date for display (YYYY-MM-DD to "Jan 15, 2026")
fn format_date_display(date: &str) -> String {
    if date.is_empty() {
        return "Select date".to_string();
    }

    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }

    let month = match parts.get(1).and_then(|m| m.parse::<u32>().ok()) {
        Some(1) => "Jan",
        Some(2) => "Feb",
        Some(3) => "Mar",
        Some(4) => "Apr",
        Some(5) => "May",
        Some(6) => "Jun",
        Some(7) => "Jul",
        Some(8) => "Aug",
        Some(9) => "Sep",
        Some(10) => "Oct",
        Some(11) => "Nov",
        Some(12) => "Dec",
        _ => return date.to_string(),
    };

    let day = parts
        .get(2)
        .and_then(|d| d.parse::<u32>().ok())
        .unwrap_or(1);
    let year = parts.first().unwrap_or(&"2026");

    format!("{} {}, {}", month, day, year)
}

/// Oracle result screen component
#[component]
pub fn OracleResult() -> impl IntoView {
    let navigate = use_navigate();

    // Get prediction ID from route params
    let params = use_params_map();
    let _prediction_id = move || params.get().get("id").cloned().unwrap_or_default();

    // Load prediction from session storage or use fallback
    let (prediction, origin, destination, date) =
        load_prediction_from_storage().unwrap_or_else(|| {
            (
                fallback_prediction(),
                "KUL".to_string(),
                "NRT".to_string(),
                "2026-01-15".to_string(),
            )
        });

    let route = format!("{} → {}", origin, destination);
    let date_display = format_date_display(&date);

    // Clone for closures
    let verdict = prediction.verdict.clone();
    let confidence = prediction.confidence;
    let current_price = prediction.current_price.clone();
    let predicted_price = prediction.predicted_price.clone();
    let wait_days = prediction.wait_days;
    let price_trend = prediction.price_trend.clone();
    let reasoning = prediction.reasoning.clone();

    // Generate verdict text
    let verdict_text = verdict.display_text_with_days(wait_days);
    let verdict_class = verdict.css_class();
    let cta_text = verdict.cta_text();
    let confidence_text = confidence_label(confidence);

    view! {
        <div class="screen-oracle-result">
            // Background gradient effect
            <div class="oracle-backdrop"></div>

            <div class="oracle-result-container">
                // Main verdict card
                <div class="oracle-card">
                    // Route header
                    <div class="oracle-card-header">
                        <span class="oracle-route-badge">{route}</span>
                        <span class="oracle-date">{date_display}</span>
                    </div>

                    // Price display
                    <div class="oracle-price-section">
                        <div class="oracle-current-price">
                            <span class="price-currency">{current_price.currency.clone()}</span>
                            <span class="price-amount">{format!("{:.0}", current_price.display_amount())}</span>
                        </div>

                        {predicted_price.as_ref().map(|pp| {
                            let diff = pp.amount - current_price.amount;
                            let diff_display = Price::new(diff.abs(), &current_price.currency);
                            let is_increase = diff > 0;

                            view! {
                                <div class="oracle-price-prediction">
                                    <span class="prediction-arrow">
                                        {if is_increase { "↗" } else { "↘" }}
                                    </span>
                                    <span class="prediction-amount">
                                        {if is_increase { "+" } else { "-" }}
                                        {diff_display.format()}
                                    </span>
                                    <span class="prediction-label">"predicted"</span>
                                </div>
                            }
                        })}
                    </div>

                    // Verdict banner
                    <div class=format!("oracle-verdict {}", verdict_class)>
                        <span class="verdict-text">{verdict_text}</span>
                    </div>

                    // Confidence meter
                    <div class="oracle-confidence">
                        <div class="confidence-header">
                            <span class="confidence-label">"Confidence"</span>
                            <span class="confidence-value">{confidence}"%"</span>
                        </div>
                        <div class="confidence-bar">
                            <div
                                class="confidence-fill"
                                style=format!("width: {}%", confidence)
                            ></div>
                        </div>
                        <span class="confidence-level">{confidence_text}</span>
                    </div>

                    // Price trend indicator
                    {price_trend.map(|trend| {
                        view! {
                            <div class="oracle-trend">
                                <span class="trend-icon">{trend.icon()}</span>
                                <span class="trend-text">{trend.display_text()}</span>
                            </div>
                        }
                    })}

                    // Primary CTA
                    <div class="oracle-cta">
                        <button
                            class="btn btn-primary btn-lg btn-full"
                            on:click={
                                let nav = navigate.clone();
                                move |_| nav("/booking/flights", Default::default())
                            }
                        >
                            {cta_text}
                        </button>
                    </div>

                    // Secondary actions
                    <div class="oracle-secondary-actions">
                        <button
                            class="link-secondary"
                            on:click={
                                let nav = navigate.clone();
                                move |_| nav("/booking/flights", Default::default())
                            }
                        >
                            "See all flights"
                        </button>
                        <span class="action-divider">"•"</span>
                        <button class="link-secondary">"Set price alert"</button>
                    </div>
                </div>

                // Reasoning section
                <div class="oracle-reasoning">
                    <h3 class="reasoning-title">"Why this recommendation?"</h3>
                    <ul class="reasoning-list">
                        {reasoning.iter().map(|reason| {
                            view! { <li>{reason.clone()}</li> }
                        }).collect_view()}
                    </ul>
                </div>
            </div>
        </div>
    }
}

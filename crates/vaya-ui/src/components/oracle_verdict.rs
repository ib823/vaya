//! Oracle Verdict Card Component
//!
//! Displays the Oracle's recommendation with animated confidence bar,
//! price prediction, and contextual CTA. Four variants based on verdict type.

use leptos::*;
use crate::types::{OracleVerdict, OraclePrediction, confidence_label};

/// Oracle verdict card with full prediction display
#[component]
pub fn OracleVerdictCard(
    /// The oracle prediction data
    prediction: OraclePrediction,
    /// Callback for primary CTA
    #[prop(optional)]
    on_primary_action: Option<Callback<OracleVerdict>>,
    /// Callback for secondary action
    #[prop(optional)]
    on_secondary_action: Option<Callback<()>>,
    /// Animate on mount
    #[prop(optional)]
    animate: bool,
) -> impl IntoView {
    let verdict = prediction.verdict.clone();
    let verdict_for_cta = prediction.verdict.clone();
    let verdict_for_class = prediction.verdict.clone();
    let confidence = prediction.confidence;

    // Card CSS class based on verdict
    let card_class = move || {
        let mut classes = vec!["verdict-card"];
        classes.push(verdict_for_class.css_class());
        if animate {
            classes.push("verdict-card-animate");
        }
        classes.join(" ")
    };

    // Handle CTA click
    let handle_primary = move |_| {
        if let Some(cb) = on_primary_action {
            cb.call(verdict_for_cta.clone());
        }
    };

    let handle_secondary = move |_| {
        if let Some(cb) = on_secondary_action {
            cb.call(());
        }
    };

    // Format price
    let format_price = |amount: i64, currency: &str| -> String {
        format!("{} {}", currency, amount)
    };

    let current_price_display = format_price(
        prediction.current_price.amount,
        &prediction.current_price.currency,
    );

    let predicted_price_display = prediction.predicted_price.as_ref().map(|p| {
        format_price(p.amount, &p.currency)
    });

    // Price change calculation
    let price_change = prediction.predicted_price.as_ref().map(|predicted| {
        let diff = predicted.amount - prediction.current_price.amount;
        let pct = (diff as f64 / prediction.current_price.amount as f64 * 100.0).round() as i64;
        (diff, pct)
    });

    let wait_days = prediction.wait_days;
    let reasoning = prediction.reasoning.clone();
    let cta_text = prediction.verdict.cta_text();
    let verdict_text = prediction.verdict.display_text_with_days(wait_days);

    view! {
        <div class=card_class>
            // Verdict banner
            <div class="verdict-banner">
                <span class="verdict-icon">
                    {verdict_icon(&verdict)}
                </span>
                <span class="verdict-text">
                    {verdict_text}
                </span>
            </div>

            // Price section
            <div class="verdict-prices">
                <div class="verdict-current-price">
                    <span class="verdict-price-label">"Current Price"</span>
                    <span class="verdict-price-value">{current_price_display}</span>
                </div>

                {predicted_price_display.map(|predicted| {
                    let (diff, pct) = price_change.unwrap_or((0, 0));
                    let change_class = if diff > 0 { "price-up" } else if diff < 0 { "price-down" } else { "" };
                    let arrow = if diff > 0 { "↑" } else if diff < 0 { "↓" } else { "" };

                    view! {
                        <div class="verdict-predicted-price">
                            <span class="verdict-price-label">"Predicted Price"</span>
                            <div class="verdict-price-row">
                                <span class="verdict-price-value">{predicted}</span>
                                <span class=format!("verdict-price-change {}", change_class)>
                                    {arrow} {pct.abs()}"%"
                                </span>
                            </div>
                        </div>
                    }
                })}
            </div>

            // Confidence meter
            <div class="verdict-confidence">
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
                <span class="confidence-text">{confidence_label(confidence)}</span>
            </div>

            // Price trend indicator
            {prediction.price_trend.map(|trend| {
                view! {
                    <div class="verdict-trend">
                        <span class="trend-icon">{trend.icon()}</span>
                        <span class="trend-text">{trend.display_text()}</span>
                    </div>
                }
            })}

            // Reasoning section
            {(!reasoning.is_empty()).then(|| {
                view! {
                    <div class="verdict-reasoning">
                        <h4 class="reasoning-title">"Why this recommendation?"</h4>
                        <ul class="reasoning-list">
                            {reasoning.iter().map(|reason| {
                                view! { <li>{reason.clone()}</li> }
                            }).collect_view()}
                        </ul>
                    </div>
                }
            })}

            // Actions
            <div class="verdict-actions">
                <button
                    class="btn btn-large verdict-cta"
                    on:click=handle_primary
                >
                    {cta_text}
                </button>

                {on_secondary_action.is_some().then(|| {
                    view! {
                        <button
                            class="btn btn-ghost verdict-secondary"
                            on:click=handle_secondary
                        >
                            "See all options"
                        </button>
                    }
                })}
            </div>
        </div>
    }
}

/// Get icon for verdict type
fn verdict_icon(verdict: &OracleVerdict) -> &'static str {
    match verdict {
        OracleVerdict::BookNow => "🎯",
        OracleVerdict::Wait => "⏳",
        OracleVerdict::JoinPool => "👥",
        OracleVerdict::Uncertain => "🔍",
    }
}

/// Simplified verdict banner for compact displays
#[component]
pub fn VerdictBanner(
    /// The verdict type
    verdict: OracleVerdict,
    /// Optional wait days for Wait verdict
    #[prop(optional)]
    wait_days: Option<u32>,
    /// Size variant
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let verdict_for_class = verdict.clone();
    let banner_class = move || {
        let mut classes = vec!["verdict-banner-standalone"];
        classes.push(verdict_for_class.css_class());
        if compact {
            classes.push("verdict-banner-compact");
        }
        classes.join(" ")
    };

    view! {
        <div class=banner_class>
            <span class="verdict-icon">{verdict_icon(&verdict)}</span>
            <span class="verdict-text">{verdict.display_text_with_days(wait_days)}</span>
        </div>
    }
}

/// Animated confidence bar component
#[component]
pub fn ConfidenceBar(
    /// Confidence percentage (0-100)
    confidence: u8,
    /// Show label text
    #[prop(optional)]
    show_label: bool,
    /// Animate fill on mount
    #[prop(optional)]
    animate: bool,
) -> impl IntoView {
    let bar_class = if animate {
        "confidence-bar confidence-bar-animate"
    } else {
        "confidence-bar"
    };

    view! {
        <div class="confidence-container">
            <div class="confidence-header">
                <span class="confidence-label">"Confidence"</span>
                <span class="confidence-value">{confidence}"%"</span>
            </div>
            <div class=bar_class>
                <div
                    class="confidence-fill"
                    style=format!("width: {}%", confidence)
                ></div>
            </div>
            {show_label.then(|| {
                view! {
                    <span class="confidence-text">{confidence_label(confidence)}</span>
                }
            })}
        </div>
    }
}

/// Price comparison display
#[component]
pub fn PriceComparison(
    /// Current price amount
    current: i64,
    /// Predicted price amount
    predicted: i64,
    /// Currency code
    #[prop(into)]
    currency: String,
) -> impl IntoView {
    let diff = predicted - current;
    let pct = ((diff as f64 / current as f64) * 100.0).round() as i64;

    let change_class = if diff > 0 {
        "price-comparison-up"
    } else if diff < 0 {
        "price-comparison-down"
    } else {
        "price-comparison-stable"
    };

    let arrow = if diff > 0 { "↑" } else if diff < 0 { "↓" } else { "→" };

    view! {
        <div class=format!("price-comparison {}", change_class)>
            <div class="price-current">
                <span class="price-label">"Now"</span>
                <span class="price-amount">{format!("{} {}", currency, current)}</span>
            </div>
            <div class="price-arrow">{arrow}</div>
            <div class="price-predicted">
                <span class="price-label">"Predicted"</span>
                <span class="price-amount">{format!("{} {}", currency, predicted)}</span>
            </div>
            <div class="price-change">
                <span class="price-pct">{pct.abs()}"%"</span>
            </div>
        </div>
    }
}

//! Price Breakdown Component
//!
//! Displays itemized pricing with subtotals, taxes, and discounts.

use leptos::*;

/// Type of price line item
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PriceItemType {
    Base,
    Tax,
    Fee,
    Extra,
    Discount,
    Subtotal,
}

impl PriceItemType {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Base => "price-line-base",
            Self::Tax => "price-line-tax",
            Self::Fee => "price-line-fee",
            Self::Extra => "price-line-extra",
            Self::Discount => "price-line-discount",
            Self::Subtotal => "price-line-subtotal",
        }
    }
}

/// A single line item in price breakdown
#[derive(Clone, Debug)]
pub struct PriceLineItem {
    pub label: String,
    pub amount: i64,         // In minor units (sen/cents)
    pub item_type: PriceItemType,
    pub quantity: Option<u32>,
}

impl PriceLineItem {
    pub fn new(label: impl Into<String>, amount: i64, item_type: PriceItemType) -> Self {
        Self {
            label: label.into(),
            amount,
            item_type,
            quantity: None,
        }
    }

    pub fn with_quantity(mut self, qty: u32) -> Self {
        self.quantity = Some(qty);
        self
    }

    pub fn base(label: impl Into<String>, amount: i64) -> Self {
        Self::new(label, amount, PriceItemType::Base)
    }

    pub fn tax(label: impl Into<String>, amount: i64) -> Self {
        Self::new(label, amount, PriceItemType::Tax)
    }

    pub fn fee(label: impl Into<String>, amount: i64) -> Self {
        Self::new(label, amount, PriceItemType::Fee)
    }

    pub fn extra(label: impl Into<String>, amount: i64) -> Self {
        Self::new(label, amount, PriceItemType::Extra)
    }

    pub fn discount(label: impl Into<String>, amount: i64) -> Self {
        // Discounts should be negative
        Self::new(label, -amount.abs(), PriceItemType::Discount)
    }
}

/// Format amount for display
fn format_amount(amount: i64, currency: &str) -> String {
    let symbol = match currency {
        "MYR" => "RM",
        "USD" => "$",
        "SGD" => "S$",
        "EUR" => "E",
        "GBP" => "£",
        _ => currency,
    };

    let major = amount.abs() / 100;
    let minor = amount.abs() % 100;

    if amount < 0 {
        format!("-{} {}.{:02}", symbol, major, minor)
    } else {
        format!("{} {}.{:02}", symbol, major, minor)
    }
}

/// Price breakdown component
#[component]
pub fn PriceBreakdown(
    /// Line items to display
    #[prop(into)]
    items: Vec<PriceLineItem>,
    /// Total amount (in minor units)
    total: i64,
    /// Currency code (default "MYR")
    #[prop(optional, into)]
    currency: Option<String>,
    /// Show per-person breakdown
    #[prop(optional)]
    passengers: Option<u8>,
    /// Compact mode for smaller displays
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let currency = currency.unwrap_or_else(|| "MYR".to_string());
    let currency_for_total = currency.clone();
    let currency_for_pp = currency.clone();

    let wrapper_class = if compact {
        "price-breakdown price-breakdown-compact"
    } else {
        "price-breakdown"
    };

    view! {
        <div class=wrapper_class>
            <div class="price-breakdown-items">
                {items.iter().map(|item| {
                    let label = if let Some(qty) = item.quantity {
                        format!("{} × {}", item.label, qty)
                    } else {
                        item.label.clone()
                    };
                    let amount_display = format_amount(item.amount, &currency);
                    let line_class = format!("price-line {}", item.item_type.css_class());

                    view! {
                        <div class=line_class>
                            <span class="price-line-label">{label}</span>
                            <span class="price-line-amount">{amount_display}</span>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="price-breakdown-divider"></div>

            <div class="price-breakdown-total">
                <span class="price-total-label">"Total"</span>
                <span class="price-total-amount">{format_amount(total, &currency_for_total)}</span>
            </div>

            {passengers.map(|pax| {
                if pax > 1 {
                    let per_person = total / pax as i64;
                    Some(view! {
                        <div class="price-per-person">
                            <span class="price-pp-label">{format!("{} passengers", pax)}</span>
                            <span class="price-pp-amount">{format!("{}/person", format_amount(per_person, &currency_for_pp))}</span>
                        </div>
                    })
                } else {
                    None
                }
            })}
        </div>
    }
}

/// Simplified price display for summaries
#[component]
pub fn PriceSummary(
    /// Total amount
    total: i64,
    /// Currency
    #[prop(optional, into)]
    currency: Option<String>,
    /// Label (e.g., "Total", "From")
    #[prop(optional, into)]
    label: Option<String>,
) -> impl IntoView {
    let currency = currency.unwrap_or_else(|| "MYR".to_string());
    let label = label.unwrap_or_else(|| "Total".to_string());

    view! {
        <div class="price-summary">
            <span class="price-summary-label">{label}</span>
            <span class="price-summary-amount">{format_amount(total, &currency)}</span>
        </div>
    }
}

/// Price lock fee display
#[component]
pub fn PriceLockFee(
    /// Lock fee amount
    fee: i64,
    /// Whether fee is refundable
    #[prop(optional)]
    refundable: bool,
    /// Currency
    #[prop(optional, into)]
    currency: Option<String>,
) -> impl IntoView {
    let currency = currency.unwrap_or_else(|| "MYR".to_string());

    view! {
        <div class="price-lock-fee">
            <div class="lock-fee-row">
                <span class="lock-fee-label">"Price lock fee"</span>
                <span class="lock-fee-amount">{format_amount(fee, &currency)}</span>
            </div>
            {refundable.then(|| view! {
                <span class="lock-fee-note">"Fully refundable if you book"</span>
            })}
        </div>
    }
}

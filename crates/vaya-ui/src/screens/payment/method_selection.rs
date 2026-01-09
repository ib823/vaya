//! Payment Method Selection Screen
//!
//! Displays available payment methods including cards, FPX, and e-wallets.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::types::PaymentMethod;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Payment method selection screen
#[component]
pub fn MethodSelection() -> impl IntoView {
    let navigate = use_navigate();

    // Get total from storage
    let total: i64 = get_session_storage()
        .and_then(|s| s.get_item("booking_total").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // State
    let (selected_method, set_method) = create_signal::<Option<PaymentMethod>>(None);

    // Handle continue
    let handle_continue = {
        let nav = navigate.clone();
        move |_| {
            if let Some(method) = selected_method.get() {
                // Store selected method
                if let Some(storage) = get_session_storage() {
                    let _ = storage.set_item("payment_method", &format!("{:?}", method));
                }

                // Navigate based on method
                let route = match method {
                    PaymentMethod::Card => "/payment/card",
                    PaymentMethod::Fpx => "/payment/fpx",
                    _ => "/payment/processing", // E-wallets go straight to processing
                };
                nav(route, Default::default());
            }
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/booking/review", Default::default());
        }
    };

    view! {
        <div class="screen-payment-method">
            // Header
            <header class="payment-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Payment Method"</h1>
                <p class="page-subtitle">"Select how you'd like to pay"</p>
            </header>

            // Total display
            <div class="payment-total-banner">
                <span class="total-label">"Total to pay"</span>
                <span class="total-amount">{format!("RM {:.2}", total as f64 / 100.0)}</span>
            </div>

            // Payment methods
            <div class="payment-methods">
                // Cards section
                <section class="method-section">
                    <h2 class="method-section-title">"Cards"</h2>
                    <PaymentMethodCard
                        method=PaymentMethod::Card
                        selected=selected_method
                        on_select=move |m| set_method.set(Some(m))
                    />
                </section>

                // Online Banking section
                <section class="method-section">
                    <h2 class="method-section-title">"Online Banking"</h2>
                    <PaymentMethodCard
                        method=PaymentMethod::Fpx
                        selected=selected_method
                        on_select=move |m| set_method.set(Some(m))
                    />
                </section>

                // E-Wallets section
                <section class="method-section">
                    <h2 class="method-section-title">"E-Wallets"</h2>
                    <div class="ewallet-grid">
                        <PaymentMethodCard
                            method=PaymentMethod::GrabPay
                            selected=selected_method
                            on_select=move |m| set_method.set(Some(m))
                            compact=true
                        />
                        <PaymentMethodCard
                            method=PaymentMethod::TouchNGo
                            selected=selected_method
                            on_select=move |m| set_method.set(Some(m))
                            compact=true
                        />
                        <PaymentMethodCard
                            method=PaymentMethod::Boost
                            selected=selected_method
                            on_select=move |m| set_method.set(Some(m))
                            compact=true
                        />
                        <PaymentMethodCard
                            method=PaymentMethod::ShopeePay
                            selected=selected_method
                            on_select=move |m| set_method.set(Some(m))
                            compact=true
                        />
                    </div>
                </section>
            </div>

            // Continue button
            <div class="payment-footer">
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || selected_method.get().is_none()
                    on:click=handle_continue
                >
                    "Continue"
                </button>
            </div>
        </div>
    }
}

/// Individual payment method card
#[component]
fn PaymentMethodCard(
    method: PaymentMethod,
    selected: ReadSignal<Option<PaymentMethod>>,
    on_select: impl Fn(PaymentMethod) + 'static,
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let is_selected = move || selected.get() == Some(method);

    let card_class = move || {
        let mut classes = vec!["payment-method-card"];
        if is_selected() {
            classes.push("payment-method-card-selected");
        }
        if compact {
            classes.push("payment-method-card-compact");
        }
        classes.join(" ")
    };

    view! {
        <button
            class=card_class
            on:click=move |_| on_select(method)
        >
            <span class="method-icon">{method.icon()}</span>
            <span class="method-name">{method.display_text()}</span>
            <span class="method-check">
                {move || if is_selected() { "✓" } else { "" }}
            </span>
        </button>
    }
}

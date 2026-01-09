//! FPX Bank Selection Screen
//!
//! Displays available FPX banks for online banking payment.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::types::FpxBank;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// FPX bank selection screen
#[component]
pub fn FpxBankSelection() -> impl IntoView {
    let navigate = use_navigate();

    // Get total from storage
    let total: i64 = get_session_storage()
        .and_then(|s| s.get_item("booking_total").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Load banks (in production: fetch from API)
    let banks = FpxBank::mock_banks();

    // State
    let (selected_bank, set_bank) = create_signal::<Option<String>>(None);

    // Handle continue
    let handle_continue = {
        let nav = navigate.clone();
        move |_| {
            if let Some(bank_code) = selected_bank.get() {
                // Store selected bank
                if let Some(storage) = get_session_storage() {
                    let _ = storage.set_item("fpx_bank", &bank_code);
                }
                // Redirect to bank (in demo: go to processing)
                nav("/payment/processing", Default::default());
            }
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/payment/method", Default::default());
        }
    };

    view! {
        <div class="screen-fpx">
            // Header
            <header class="fpx-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Select Your Bank"</h1>
                <p class="page-subtitle">"FPX Online Banking"</p>
            </header>

            // Total display
            <div class="payment-total-banner">
                <span class="total-label">"Total to pay"</span>
                <span class="total-amount">{format!("RM {:.2}", total as f64 / 100.0)}</span>
            </div>

            // Bank list
            <div class="bank-list">
                {banks.into_iter().map(|bank| {
                    let bank_code = bank.code.clone();
                    let bank_code_for_class = bank.code.clone();
                    let bank_code_for_check = bank.code.clone();
                    let is_online = bank.online;

                    let option_class = move || {
                        let mut classes = vec!["bank-option"];
                        if selected_bank.get().as_ref() == Some(&bank_code_for_class) {
                            classes.push("bank-option-selected");
                        }
                        if !is_online {
                            classes.push("bank-option-offline");
                        }
                        classes.join(" ")
                    };

                    view! {
                        <button
                            class=option_class
                            disabled=!is_online
                            on:click=move |_| set_bank.set(Some(bank_code.clone()))
                        >
                            <span class="bank-icon">"🏦"</span>
                            <div class="bank-info">
                                <span class="bank-name">{bank.name}</span>
                                {(!is_online).then(|| view! {
                                    <span class="bank-status">"Currently unavailable"</span>
                                })}
                            </div>
                            <span class="bank-check">
                                {move || if selected_bank.get().as_ref() == Some(&bank_code_for_check) { "✓" } else { "" }}
                            </span>
                        </button>
                    }
                }).collect_view()}
            </div>

            // Info note
            <div class="fpx-info">
                <p class="info-text">
                    "You will be redirected to your bank's secure site to complete payment."
                </p>
            </div>

            // Continue button
            <div class="fpx-footer">
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || selected_bank.get().is_none()
                    on:click=handle_continue
                >
                    "Continue to Bank"
                </button>
            </div>
        </div>
    }
}

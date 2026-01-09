//! Payment Failure Screen
//!
//! Displays payment failure with appropriate error message and retry options.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::components::CountdownMini;
use crate::types::PaymentError;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Parse payment error from storage
fn get_payment_error() -> PaymentError {
    get_session_storage()
        .and_then(|s| s.get_item("payment_error").ok().flatten())
        .map(|s| match s.as_str() {
            "CardDeclined" => PaymentError::CardDeclined,
            "InsufficientFunds" => PaymentError::InsufficientFunds,
            "ExpiredCard" => PaymentError::ExpiredCard,
            "InvalidCard" => PaymentError::InvalidCard,
            "FraudSuspected" => PaymentError::FraudSuspected,
            "BankUnavailable" => PaymentError::BankUnavailable,
            "NetworkError" => PaymentError::NetworkError,
            "ThreeDsFailed" => PaymentError::ThreeDsFailed,
            "CurrencyNotSupported" => PaymentError::CurrencyNotSupported,
            "LimitExceeded" => PaymentError::LimitExceeded,
            "CardNotSupported" => PaymentError::CardNotSupported,
            "ProcessorError" => PaymentError::ProcessorError,
            "TimeoutError" => PaymentError::TimeoutError,
            "DuplicateTransaction" => PaymentError::DuplicateTransaction,
            "InvalidCvv" => PaymentError::InvalidCvv,
            "AddressVerificationFailed" => PaymentError::AddressVerificationFailed,
            "RiskBlocked" => PaymentError::RiskBlocked,
            _ => PaymentError::GeneralError,
        })
        .unwrap_or(PaymentError::GeneralError)
}

/// Get error icon based on type
fn error_icon(error: &PaymentError) -> &'static str {
    match error {
        PaymentError::CardDeclined | PaymentError::InsufficientFunds => "💳",
        PaymentError::ExpiredCard => "📅",
        PaymentError::FraudSuspected | PaymentError::RiskBlocked => "🛡️",
        PaymentError::BankUnavailable => "🏦",
        PaymentError::NetworkError | PaymentError::TimeoutError => "📡",
        PaymentError::ThreeDsFailed => "🔐",
        _ => "⚠️",
    }
}

/// Payment failure screen
#[component]
pub fn Failure() -> impl IntoView {
    let navigate = use_navigate();

    // Get error
    let error = get_payment_error();
    let error_message = error.display_message();
    let is_retryable = error.is_retryable();
    let icon = error_icon(&error);

    // Check for price lock
    let has_price_lock = get_session_storage()
        .and_then(|s| s.get_item("price_lock_duration").ok().flatten())
        .is_some();

    // Calculate lock expiry (mock - 24h from now)
    let lock_expiry = {
        let now = js_sys::Date::now() as i64;
        let expiry = now + (24 * 60 * 60 * 1000); // 24 hours
        expiry.to_string()
    };

    // Handle retry
    let handle_retry = {
        let nav = navigate.clone();
        move |_| {
            nav("/payment/method", Default::default());
        }
    };

    // Handle try different method
    let handle_different = {
        let nav = navigate.clone();
        move |_| {
            nav("/payment/method", Default::default());
        }
    };

    // Handle contact support
    let handle_support = move |_| {
        // In production: open support chat or page
        web_sys::window().and_then(|w| {
            w.open_with_url_and_target("mailto:support@vaya.my", "_blank")
                .ok()
        });
    };

    // Handle go home
    let handle_home = {
        let nav = navigate;
        move |_| {
            nav("/", Default::default());
        }
    };

    view! {
        <div class="screen-failure">
            <div class="failure-container">
                // Error icon
                <div class="failure-icon">
                    <span class="icon-emoji">{icon}</span>
                </div>

                // Title
                <h1 class="failure-title">"Payment Failed"</h1>

                // Error message
                <p class="failure-message">{error_message}</p>

                // Price lock reassurance
                {has_price_lock.then(|| view! {
                    <div class="price-lock-reassurance">
                        <span class="lock-icon">"🔒"</span>
                        <div class="lock-info">
                            <span class="lock-title">"Your price is still locked"</span>
                            <CountdownMini
                                end_time=lock_expiry.clone()
                                prefix="Expires in"
                            />
                        </div>
                    </div>
                })}

                // Actions
                <div class="failure-actions">
                    {is_retryable.then(|| view! {
                        <button
                            class="btn btn-primary btn-lg btn-full"
                            on:click=handle_retry
                        >
                            "Try Again"
                        </button>
                    })}

                    <button
                        class="btn btn-secondary btn-full"
                        on:click=handle_different
                    >
                        "Try Different Payment Method"
                    </button>

                    <button
                        class="btn btn-ghost btn-full"
                        on:click=handle_support
                    >
                        "Contact Support"
                    </button>
                </div>

                // Error code (for support reference)
                <div class="error-code">
                    <span class="code-label">"Error code: "</span>
                    <span class="code-value">{format!("{:?}", error)}</span>
                </div>

                // Home link
                <button
                    class="btn btn-ghost home-link"
                    on:click=handle_home
                >
                    "← Back to Home"
                </button>
            </div>
        </div>
    }
}

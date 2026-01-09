//! Payment Processing Screen
//!
//! Shows payment processing status with animated loading.

use gloo_timers::callback::Timeout;
use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::types::PaymentStatus;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Processing messages
const PROCESSING_MESSAGES: &[&str] = &[
    "Processing payment...",
    "Verifying transaction...",
    "Confirming with airline...",
    "Almost done...",
];

/// Payment processing screen
#[component]
pub fn Processing() -> impl IntoView {
    let navigate = use_navigate();

    // State
    let (message_index, set_message_index) = create_signal(0usize);
    let (_status, set_status) = create_signal(PaymentStatus::Processing);

    // Rotate messages
    let _msg_interval = store_value(gloo_timers::callback::Interval::new(1500, move || {
        set_message_index.update(|i| *i = (*i + 1) % PROCESSING_MESSAGES.len());
    }));

    // Simulate payment completion (in production: poll payment status API)
    let nav = navigate.clone();
    let _timeout = store_value(Timeout::new(5000, move || {
        // Simulate 90% success rate
        let success = js_sys::Math::random() > 0.1;

        if success {
            set_status.set(PaymentStatus::Succeeded);

            // Generate booking reference
            let reference = format!("VY{:06}", (js_sys::Math::random() * 1000000.0) as u32);
            if let Some(storage) = get_session_storage() {
                let _ = storage.set_item("booking_reference", &reference);
            }

            nav("/payment/success", Default::default());
        } else {
            set_status.set(PaymentStatus::Failed);

            // Store error type
            if let Some(storage) = get_session_storage() {
                let _ = storage.set_item("payment_error", "CardDeclined");
            }

            nav("/payment/failed", Default::default());
        }
    }));

    view! {
        <div class="screen-processing">
            <div class="processing-container">
                // Animated orb
                <div class="processing-visual">
                    <div class="processing-orb">
                        <div class="orb-ring orb-ring-1"></div>
                        <div class="orb-ring orb-ring-2"></div>
                        <div class="orb-ring orb-ring-3"></div>
                        <div class="orb-core"></div>
                    </div>
                </div>

                // Status text
                <h1 class="processing-title">"Processing Payment"</h1>
                <p class="processing-message">
                    {move || PROCESSING_MESSAGES[message_index.get()]}
                </p>

                // Progress dots
                <div class="processing-dots">
                    <span class="dot dot-1"></span>
                    <span class="dot dot-2"></span>
                    <span class="dot dot-3"></span>
                </div>

                // Warning
                <p class="processing-warning">
                    "Please don't close this page or press back"
                </p>
            </div>
        </div>
    }
}

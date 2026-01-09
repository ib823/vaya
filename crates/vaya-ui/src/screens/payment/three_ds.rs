//! 3D Secure Challenge Screen
//!
//! Handles 3D Secure authentication iframe.

use gloo_timers::callback::Timeout;
use leptos::*;
use leptos_router::use_navigate;

/// 3D Secure challenge screen
#[component]
pub fn ThreeDsChallenge() -> impl IntoView {
    let navigate = use_navigate();

    // In production: load actual 3DS URL from payment initiation response
    // For demo: simulate 3DS with a timeout
    let (is_loading, set_loading) = create_signal(true);

    // Simulate 3DS completion after 3 seconds
    let nav = navigate.clone();
    let _timeout = store_value(Timeout::new(3000, move || {
        set_loading.set(false);
        // Simulate success - in production, this would be handled by 3DS callback
        nav("/payment/processing", Default::default());
    }));

    // Handle cancel
    let handle_cancel = {
        let nav = navigate;
        move |_| {
            nav("/payment/card", Default::default());
        }
    };

    view! {
        <div class="screen-three-ds">
            // Header
            <header class="three-ds-header">
                <h1 class="page-title">"Verify Your Card"</h1>
                <p class="page-subtitle">"3D Secure Authentication"</p>
            </header>

            // 3DS iframe container
            <div class="three-ds-container">
                {move || if is_loading.get() {
                    view! {
                        <div class="three-ds-loading">
                            <div class="three-ds-spinner"></div>
                            <p class="three-ds-message">"Connecting to your bank..."</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="three-ds-frame-placeholder">
                            // In production: actual iframe
                            // <iframe
                            //     class="three-ds-iframe"
                            //     src=three_ds_url
                            //     title="3D Secure Verification"
                            // />
                            <p>"Bank verification page would appear here"</p>
                        </div>
                    }.into_view()
                }}
            </div>

            // Info
            <div class="three-ds-info">
                <div class="info-item">
                    <span class="info-icon">"🔒"</span>
                    <span class="info-text">"This is a secure verification by your bank"</span>
                </div>
                <div class="info-item">
                    <span class="info-icon">"📱"</span>
                    <span class="info-text">"You may receive an SMS or app notification"</span>
                </div>
            </div>

            // Cancel option
            <div class="three-ds-footer">
                <button
                    class="btn btn-ghost"
                    on:click=handle_cancel
                >
                    "Cancel and go back"
                </button>
            </div>
        </div>
    }
}

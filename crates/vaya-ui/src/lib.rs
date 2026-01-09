//! VAYA UI - Frontend for the Oracle flight booking system
//!
//! This crate provides the Leptos-based user interface for VAYA.
//!
//! # Architecture
//!
//! - `tokens/` - Design system tokens (colors, spacing, typography, animations)
//! - `components/` - Reusable UI components (buttons, inputs, cards)
//! - `screens/` - Full page screens (home, oracle results, booking)
//! - `hooks/` - API integration and shared state hooks
//!
//! # Usage
//!
//! Build with Trunk:
//! ```bash
//! cd crates/vaya-ui
//! trunk serve
//! ```

use leptos::*;
use leptos_router::*;

pub mod components;
#[allow(dead_code)]
pub mod generated_types;
pub mod hooks;
pub mod screens;
pub mod tokens;
pub mod types;

use screens::{
    CardEntry,
    ContactDetails,
    ExtrasSelection,
    Failure,
    // Booking flow
    FlightSelection,
    FpxBankSelection,
    Home,
    // Payment flow
    MethodSelection,
    NotFound,
    OracleLoading,
    OracleResult,
    OrderReview,
    PassengerDetails,
    PriceLock,
    Processing,
    Success,
    ThreeDsChallenge,
};

/// Main application component with router and error boundary
#[component]
pub fn App() -> impl IntoView {
    // Provide booking state at app root
    hooks::provide_booking_state();

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <div class="error-page">
                    <div class="error-container">
                        <h1 class="error-title">"Something went wrong"</h1>
                        <div class="error-list">
                            {move || {
                                errors.get()
                                    .into_iter()
                                    .map(|(_, e)| {
                                        view! { <p class="error-message">{e.to_string()}</p> }
                                    })
                                    .collect_view()
                            }}
                        </div>
                        <a href="/" class="btn btn-primary">"Return Home"</a>
                    </div>
                </div>
            }
        }>
            <Router>
                <main class="vaya-app">
                    <Routes>
                        // Core routes
                        <Route path="/" view=Home />
                        <Route path="/oracle/loading" view=OracleLoading />
                        <Route path="/oracle/result/:id" view=OracleResult />

                        // Booking flow
                        <Route path="/booking/flights" view=FlightSelection />
                        <Route path="/booking/price-lock" view=PriceLock />
                        <Route path="/booking/passengers" view=PassengerDetails />
                        <Route path="/booking/extras" view=ExtrasSelection />
                        <Route path="/booking/contact" view=ContactDetails />
                        <Route path="/booking/review" view=OrderReview />

                        // Payment flow
                        <Route path="/payment/method" view=MethodSelection />
                        <Route path="/payment/card" view=CardEntry />
                        <Route path="/payment/fpx" view=FpxBankSelection />
                        <Route path="/payment/3ds" view=ThreeDsChallenge />
                        <Route path="/payment/processing" view=Processing />
                        <Route path="/payment/success" view=Success />
                        <Route path="/payment/failed" view=Failure />

                        // Catch-all
                        <Route path="/*any" view=NotFound />
                    </Routes>
                </main>
            </Router>
        </ErrorBoundary>
    }
}

/// WASM entry point - called when the module loads
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    // Set up better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Remove the loading indicator
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(loading) = document.get_element_by_id("vaya-loading") {
                loading.remove();
            }
        }
    }

    // Mount the Leptos app to the body
    mount_to_body(|| view! { <App /> });
}

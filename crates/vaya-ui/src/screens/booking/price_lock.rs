//! Price Lock Screen
//!
//! Allows users to lock the current price for 24/48/72 hours.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::components::{FlightMini, PriceLockFee};
use crate::hooks::{use_booking_state, set_price_lock, mock_price_lock};
use crate::types::{Flight, PriceLockDuration};

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Load selected flight from storage
fn load_selected_flight() -> Option<Flight> {
    let storage = get_session_storage()?;
    let json = storage.get_item("selected_flight").ok()??;
    serde_json::from_str(&json).ok()
}

/// Price lock screen
#[component]
pub fn PriceLock() -> impl IntoView {
    let navigate = use_navigate();
    let booking_state = use_booking_state();

    // Load flight data from state or fallback to storage
    let flight = {
        let state = booking_state.get();
        state.selected_flight.clone().or_else(load_selected_flight)
    };
    let flight_exists = flight.is_some();
    let flight_clone = flight.clone();

    // State
    let (selected_duration, set_duration) = create_signal::<Option<PriceLockDuration>>(None);
    let (_skip_lock, set_skip_lock) = create_signal(false);

    // Handle continue
    let flight_for_lock = flight_clone.clone();
    let handle_continue = {
        let nav = navigate.clone();
        move |_| {
            if let Some(duration) = selected_duration.get() {
                // Create and store price lock
                if let Some(ref flight) = flight_for_lock {
                    let lock = mock_price_lock(flight, duration);
                    set_price_lock(lock);
                }

                // Also store in session storage for backward compatibility
                if let Some(storage) = get_session_storage() {
                    let _ = storage.set_item("price_lock_duration", &format!("{:?}", duration));
                    let _ = storage.set_item("price_lock_fee", &duration.fee_myr().to_string());
                }
            }
            nav("/booking/passengers", Default::default());
        }
    };

    // Handle skip
    let handle_skip = {
        let nav = navigate.clone();
        move |_| {
            set_skip_lock.set(true);
            // Clear any price lock
            if let Some(storage) = get_session_storage() {
                let _ = storage.remove_item("price_lock_duration");
                let _ = storage.remove_item("price_lock_fee");
            }
            nav("/booking/passengers", Default::default());
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/booking/flights", Default::default());
        }
    };

    if !flight_exists {
        return view! {
            <div class="screen-price-lock">
                <div class="error-state">
                    <p>"No flight selected"</p>
                    <button class="btn btn-primary" on:click=handle_back>
                        "Select a flight"
                    </button>
                </div>
            </div>
        }.into_view();
    }

    let flight = flight_clone.unwrap();
    let origin = flight.origin.clone();
    let destination = flight.destination.clone();
    let price_display = flight.price.format();

    view! {
        <div class="screen-price-lock">
            // Header
            <header class="price-lock-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Lock Your Price"</h1>
            </header>

            // Flight summary
            <div class="price-lock-flight">
                <FlightMini
                    origin=origin
                    destination=destination
                    price=price_display.clone()
                />
            </div>

            // Value proposition
            <div class="price-lock-promo">
                <div class="promo-icon">"🔒"</div>
                <h2 class="promo-title">"Secure today's price"</h2>
                <p class="promo-text">
                    "Lock this price while you finalize your plans. If prices go up, you're protected."
                </p>
            </div>

            // Duration options
            <div class="price-lock-options">
                {PriceLockDuration::all().into_iter().map(|duration| {
                    let is_selected = move || selected_duration.get() == Some(duration);
                    let option_class = move || {
                        if is_selected() {
                            "lock-option lock-option-selected"
                        } else {
                            "lock-option"
                        }
                    };

                    view! {
                        <button
                            class=option_class
                            on:click=move |_| set_duration.set(Some(duration))
                        >
                            <div class="lock-option-header">
                                <span class="lock-duration">{duration.display_text()}</span>
                                <span class="lock-check">
                                    {move || if is_selected() { "✓" } else { "" }}
                                </span>
                            </div>
                            <div class="lock-option-price">
                                <PriceLockFee
                                    fee=duration.fee_myr()
                                    refundable=true
                                />
                            </div>
                        </button>
                    }
                }).collect_view()}
            </div>

            // Guarantee info
            <div class="price-lock-guarantee">
                <div class="guarantee-item">
                    <span class="guarantee-icon">"✓"</span>
                    <span class="guarantee-text">"Fee refunded when you book"</span>
                </div>
                <div class="guarantee-item">
                    <span class="guarantee-icon">"✓"</span>
                    <span class="guarantee-text">"Cancel anytime, no questions asked"</span>
                </div>
                <div class="guarantee-item">
                    <span class="guarantee-icon">"✓"</span>
                    <span class="guarantee-text">"Price guaranteed even if it increases"</span>
                </div>
            </div>

            // Actions
            <div class="price-lock-actions">
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || selected_duration.get().is_none()
                    on:click=handle_continue
                >
                    {move || {
                        if let Some(duration) = selected_duration.get() {
                            format!("Lock for {} - RM {:.2}", duration.display_text(), duration.fee_myr() as f64 / 100.0)
                        } else {
                            "Select duration".to_string()
                        }
                    }}
                </button>

                <button
                    class="btn btn-ghost btn-full"
                    on:click=handle_skip
                >
                    "Skip - I'll book now"
                </button>
            </div>
        </div>
    }.into_view()
}

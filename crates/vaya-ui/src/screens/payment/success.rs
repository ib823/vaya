//! Payment Success Screen
//!
//! Displays successful payment confirmation with booking reference.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::types::Flight;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Load booking data for confirmation
fn load_confirmation_data() -> ConfirmationData {
    let storage = get_session_storage();

    let reference = storage
        .as_ref()
        .and_then(|s| s.get_item("booking_reference").ok().flatten())
        .unwrap_or_else(|| "VY000000".to_string());

    let flight: Option<Flight> = storage
        .as_ref()
        .and_then(|s| s.get_item("selected_flight").ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok());

    let email = storage
        .as_ref()
        .and_then(|s| s.get_item("contact_email").ok().flatten())
        .unwrap_or_default();

    let total: i64 = storage
        .as_ref()
        .and_then(|s| s.get_item("booking_total").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    ConfirmationData {
        reference,
        flight,
        email,
        total,
    }
}

struct ConfirmationData {
    reference: String,
    flight: Option<Flight>,
    email: String,
    total: i64,
}

/// Payment success screen
#[component]
pub fn Success() -> impl IntoView {
    let navigate = use_navigate();

    // Load data
    let data = load_confirmation_data();
    let reference = data.reference;
    let flight = data.flight;
    let email = data.email;
    let total = data.total;

    // Animation trigger
    let (show_confetti, set_confetti) = create_signal(false);
    let (show_content, set_content) = create_signal(false);

    // Trigger animations on mount
    request_animation_frame(move || {
        set_confetti.set(true);
        set_timeout(
            move || set_content.set(true),
            std::time::Duration::from_millis(500),
        );
    });

    // Handle done
    let handle_done = {
        let nav = navigate.clone();
        move |_| {
            // Clear booking data
            if let Some(storage) = get_session_storage() {
                let _ = storage.clear();
            }
            nav("/", Default::default());
        }
    };

    // Handle view booking
    let handle_view = {
        let nav = navigate;
        move |_| {
            nav("/bookings", Default::default());
        }
    };

    view! {
        <div class="screen-success">
            // Confetti container
            {move || show_confetti.get().then(|| view! {
                <div class="confetti-container">
                    {(0..50).map(|i| {
                        let left = format!("{}%", (i * 2) % 100);
                        let delay = format!("{}ms", (i * 100) % 1000);
                        let color = match i % 4 {
                            0 => "var(--mint-500)",
                            1 => "var(--cyan-500)",
                            2 => "var(--warning)",
                            _ => "var(--purple)",
                        };
                        view! {
                            <div
                                class="confetti-piece"
                                style=format!("left: {}; animation-delay: {}; background: {};", left, delay, color)
                            ></div>
                        }
                    }).collect_view()}
                </div>
            })}

            <div class="success-container" class:show=move || show_content.get()>
                // Success checkmark
                <div class="success-checkmark">
                    <svg viewBox="0 0 52 52" class="checkmark-svg">
                        <circle class="checkmark-circle" cx="26" cy="26" r="25" fill="none"/>
                        <path class="checkmark-check" fill="none" d="M14.1 27.2l7.1 7.2 16.7-16.8"/>
                    </svg>
                </div>

                // Title
                <h1 class="success-title">"Payment Successful!"</h1>
                <p class="success-subtitle">"Your booking is confirmed"</p>

                // Booking reference
                <div class="booking-reference">
                    <span class="reference-label">"Booking Reference"</span>
                    <span class="reference-code">{reference}</span>
                </div>

                // Flight summary
                {flight.map(|f| view! {
                    <div class="success-flight">
                        <div class="flight-route">
                            <span class="route-code">{f.origin}</span>
                            <span class="route-arrow">"→"</span>
                            <span class="route-code">{f.destination}</span>
                        </div>
                        <div class="flight-details">
                            <span>{f.airline_name}</span>
                            <span>" • "</span>
                            <span>{f.departure_time}</span>
                        </div>
                    </div>
                })}

                // Total paid
                <div class="success-total">
                    <span class="total-label">"Total Paid"</span>
                    <span class="total-amount">{format!("RM {:.2}", total as f64 / 100.0)}</span>
                </div>

                // Confirmation email
                <div class="success-email">
                    <span class="email-icon">"📧"</span>
                    <span class="email-text">
                        "Confirmation sent to "
                        <strong>{email}</strong>
                    </span>
                </div>

                // Actions
                <div class="success-actions">
                    <button
                        class="btn btn-primary btn-lg btn-full"
                        on:click=handle_view
                    >
                        "View Booking"
                    </button>
                    <button
                        class="btn btn-ghost btn-full"
                        on:click=handle_done
                    >
                        "Done"
                    </button>
                </div>
            </div>
        </div>
    }
}

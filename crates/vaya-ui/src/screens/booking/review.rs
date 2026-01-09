//! Order Review Screen
//!
//! Shows complete booking summary before payment.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::components::{FlightCard, PriceBreakdown, PriceLineItem, TermsCheckbox};
use crate::types::{Flight, Passenger};

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Load booking data from session storage
fn load_booking_data() -> BookingData {
    let storage = get_session_storage();

    let flight: Option<Flight> = storage
        .as_ref()
        .and_then(|s| s.get_item("selected_flight").ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok());

    let passengers: Vec<Passenger> = storage
        .as_ref()
        .and_then(|s| s.get_item("booking_passengers").ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();

    let contact_email = storage
        .as_ref()
        .and_then(|s| s.get_item("contact_email").ok().flatten())
        .unwrap_or_default();

    let contact_phone = storage
        .as_ref()
        .and_then(|s| s.get_item("contact_phone").ok().flatten())
        .unwrap_or_default();

    let extras_total: i64 = storage
        .as_ref()
        .and_then(|s| s.get_item("extras_total").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let price_lock_fee: i64 = storage
        .as_ref()
        .and_then(|s| s.get_item("price_lock_fee").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    BookingData {
        flight,
        passengers,
        contact_email,
        contact_phone,
        extras_total,
        price_lock_fee,
    }
}

struct BookingData {
    flight: Option<Flight>,
    passengers: Vec<Passenger>,
    contact_email: String,
    contact_phone: String,
    extras_total: i64,
    price_lock_fee: i64,
}

/// Order review screen
#[component]
pub fn OrderReview() -> impl IntoView {
    let navigate = use_navigate();

    // Load data
    let data = load_booking_data();
    let flight = data.flight.clone();
    let passengers = data.passengers.clone();
    let contact_email = data.contact_email.clone();
    let contact_phone = data.contact_phone.clone();
    let extras_total = data.extras_total;
    let price_lock_fee = data.price_lock_fee;

    // Calculate totals
    let base_fare = flight
        .as_ref()
        .map(|f| f.price.amount * passengers.len() as i64)
        .unwrap_or(0);
    let taxes = (base_fare as f64 * 0.06) as i64; // 6% tax
    let fees = 1500; // RM 15 booking fee
    let total = base_fare + taxes + fees + extras_total - price_lock_fee; // Lock fee credited back

    // Build price breakdown items
    let price_items = {
        let mut items = vec![
            PriceLineItem::base(format!("Flight × {}", passengers.len()), base_fare),
            PriceLineItem::tax("Taxes & fees", taxes),
            PriceLineItem::fee("Booking fee", fees),
        ];
        if extras_total > 0 {
            items.push(PriceLineItem::extra("Extras", extras_total));
        }
        if price_lock_fee > 0 {
            items.push(PriceLineItem::discount("Price lock credit", price_lock_fee));
        }
        items
    };

    // Terms acceptance
    let terms_accepted = create_rw_signal(false);

    // Handle payment
    let handle_payment = {
        let nav = navigate.clone();
        move |_| {
            if terms_accepted.get() {
                // Store final booking data
                if let Some(storage) = get_session_storage() {
                    let _ = storage.set_item("booking_total", &total.to_string());
                }
                nav("/payment/method", Default::default());
            }
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate.clone();
        move |_| {
            nav("/booking/contact", Default::default());
        }
    };

    // Check if we have required data
    if flight.is_none() {
        let nav = navigate;
        return view! {
            <div class="screen-review">
                <div class="error-state">
                    <p>"No booking data found"</p>
                    <button class="btn btn-primary" on:click=move |_| {
                        nav("/booking/flights", Default::default());
                    }>
                        "Start over"
                    </button>
                </div>
            </div>
        }
        .into_view();
    }

    let flight_data = flight.unwrap();

    view! {
        <div class="screen-review">
            // Header
            <header class="review-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Review Your Booking"</h1>
            </header>

            // Flight section
            <section class="review-section">
                <h2 class="section-title">"Flight"</h2>
                <FlightCard
                    flight=flight_data.clone()
                    compact=true
                />
            </section>

            // Passengers section
            <section class="review-section">
                <h2 class="section-title">"Passengers"</h2>
                <div class="passenger-list">
                    {passengers.iter().enumerate().map(|(i, pax)| {
                        let name = format!("{} {} {}",
                            pax.title.clone().unwrap_or_default(),
                            pax.first_name,
                            pax.last_name
                        );
                        view! {
                            <div class="passenger-item">
                                <span class="passenger-number">{i + 1}</span>
                                <span class="passenger-name">{name}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </section>

            // Contact section
            <section class="review-section">
                <h2 class="section-title">"Contact"</h2>
                <div class="contact-summary">
                    <div class="contact-row">
                        <span class="contact-label">"Email"</span>
                        <span class="contact-value">{contact_email}</span>
                    </div>
                    <div class="contact-row">
                        <span class="contact-label">"Phone"</span>
                        <span class="contact-value">{contact_phone}</span>
                    </div>
                </div>
            </section>

            // Price breakdown section
            <section class="review-section">
                <h2 class="section-title">"Price Breakdown"</h2>
                <PriceBreakdown
                    items=price_items
                    total=total
                    passengers=passengers.len() as u8
                />
            </section>

            // Terms section
            <section class="review-section terms-section">
                <TermsCheckbox
                    checked=terms_accepted
                    terms_url="/terms"
                    privacy_url="/privacy"
                />
            </section>

            // Payment button
            <div class="review-footer">
                <div class="total-display">
                    <span class="total-label">"Total"</span>
                    <span class="total-amount">{format!("RM {:.2}", total as f64 / 100.0)}</span>
                </div>
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || !terms_accepted.get()
                    on:click=handle_payment
                >
                    "Continue to Payment"
                </button>
            </div>
        </div>
    }
    .into_view()
}

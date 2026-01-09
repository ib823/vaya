//! Passenger Details Screen
//!
//! Collects passenger information for all travelers.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::components::{CountrySelect, DateInput, TextInput, TitleSelect};
use crate::types::{Passenger, PassengerType};

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Passenger form for a single passenger
#[component]
fn PassengerForm(
    /// Passenger index (1-based)
    index: usize,
    /// Passenger type
    passenger_type: PassengerType,
    /// Passenger data signal
    passenger: RwSignal<Passenger>,
    /// Whether international (requires passport)
    international: bool,
) -> impl IntoView {
    // Create signals for each field
    let title = create_rw_signal(passenger.get().title.unwrap_or_default());
    let first_name = create_rw_signal(passenger.get().first_name.clone());
    let last_name = create_rw_signal(passenger.get().last_name.clone());
    let dob = create_rw_signal(passenger.get().date_of_birth.unwrap_or_default());
    let nationality = create_rw_signal(passenger.get().nationality.unwrap_or_default());
    let passport_number = create_rw_signal(passenger.get().passport_number.unwrap_or_default());
    let passport_expiry = create_rw_signal(passenger.get().passport_expiry.unwrap_or_default());

    // Update main passenger signal when fields change
    create_effect(move |_| {
        passenger.update(|p| {
            p.title = if title.get().is_empty() {
                None
            } else {
                Some(title.get())
            };
            p.first_name = first_name.get();
            p.last_name = last_name.get();
            p.date_of_birth = if dob.get().is_empty() {
                None
            } else {
                Some(dob.get())
            };
            p.nationality = if nationality.get().is_empty() {
                None
            } else {
                Some(nationality.get())
            };
            p.passport_number = if passport_number.get().is_empty() {
                None
            } else {
                Some(passport_number.get())
            };
            p.passport_expiry = if passport_expiry.get().is_empty() {
                None
            } else {
                Some(passport_expiry.get())
            };
        });
    });

    let type_label = passenger_type.display_text();

    view! {
        <div class="passenger-form">
            <div class="passenger-header">
                <h3 class="passenger-title">"Passenger "{index}</h3>
                <span class="passenger-type">{type_label}</span>
            </div>

            <div class="form-grid">
                // Title
                <div class="form-row">
                    <TitleSelect value=title />
                </div>

                // Name row
                <div class="form-row two-col">
                    <TextInput
                        label="First Name"
                        placeholder="As on passport"
                        value=first_name
                    />
                    <TextInput
                        label="Last Name"
                        placeholder="As on passport"
                        value=last_name
                    />
                </div>

                // Date of birth
                <div class="form-row">
                    <DateInput
                        label="Date of Birth"
                        value=dob
                        max=get_today()
                    />
                </div>

                // Nationality (for international)
                {international.then(|| view! {
                    <div class="form-row">
                        <CountrySelect
                            label="Nationality"
                            value=nationality
                        />
                    </div>
                })}

                // Passport section (for international)
                {international.then(|| view! {
                    <div class="passport-section">
                        <h4 class="section-title">"Passport Details"</h4>
                        <div class="form-row two-col">
                            <TextInput
                                label="Passport Number"
                                placeholder="A12345678"
                                value=passport_number
                                uppercase=true
                            />
                            <DateInput
                                label="Passport Expiry"
                                value=passport_expiry
                                min=get_today()
                            />
                        </div>
                    </div>
                })}
            </div>
        </div>
    }
}

/// Get today's date
fn get_today() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}

/// Passenger details screen
#[component]
pub fn PassengerDetails() -> impl IntoView {
    let navigate = use_navigate();

    // Get passenger count from storage
    let pax_count: u8 = get_session_storage()
        .and_then(|s| s.get_item("oracle_pax").ok().flatten())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    // Check if international flight
    let is_international = {
        let storage = get_session_storage();
        let origin = storage
            .as_ref()
            .and_then(|s| s.get_item("oracle_origin").ok().flatten())
            .unwrap_or_default();
        let dest = storage
            .as_ref()
            .and_then(|s| s.get_item("oracle_destination").ok().flatten())
            .unwrap_or_default();
        // Simple check: different first letter usually means international
        !origin.is_empty() && !dest.is_empty() && origin.chars().next() != dest.chars().next()
    };

    // Create passenger signals
    let passengers: Vec<RwSignal<Passenger>> = (0..pax_count)
        .map(|i| {
            create_rw_signal(Passenger {
                id: format!("pax-{}", i + 1),
                passenger_type: PassengerType::Adult,
                ..Default::default()
            })
        })
        .collect();

    // Clone for different closures
    let passengers_for_valid = passengers.clone();
    let passengers_for_continue = passengers.clone();

    // Validation
    let is_valid = move || {
        passengers_for_valid.iter().all(|p| {
            let pax = p.get();
            !pax.first_name.is_empty() && !pax.last_name.is_empty()
        })
    };

    // Handle continue
    let handle_continue = {
        let nav = navigate.clone();
        move |_| {
            // Store passengers in session storage
            if let Some(storage) = get_session_storage() {
                let pax_data: Vec<Passenger> =
                    passengers_for_continue.iter().map(|p| p.get()).collect();
                if let Ok(json) = serde_json::to_string(&pax_data) {
                    let _ = storage.set_item("booking_passengers", &json);
                }
            }
            nav("/booking/extras", Default::default());
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/booking/price-lock", Default::default());
        }
    };

    view! {
        <div class="screen-passengers">
            // Header
            <header class="passengers-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Passenger Details"</h1>
                <p class="page-subtitle">"Enter details as they appear on passport/ID"</p>
            </header>

            // Passenger forms
            <div class="passengers-list">
                {passengers.into_iter().enumerate().map(|(i, pax)| {
                    view! {
                        <PassengerForm
                            index=i + 1
                            passenger_type=PassengerType::Adult
                            passenger=pax
                            international=is_international
                        />
                    }
                }).collect_view()}
            </div>

            // Continue button
            <div class="passengers-footer">
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || !is_valid()
                    on:click=handle_continue
                >
                    "Continue to Extras"
                </button>
            </div>
        </div>
    }
}

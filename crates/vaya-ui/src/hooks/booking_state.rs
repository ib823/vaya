//! Booking State Management
//!
//! Provides centralized state management for the booking flow using Leptos context.

use crate::types::*;
use leptos::*;

/// Search parameters from home screen
#[derive(Clone, Debug, Default)]
pub struct SearchParams {
    pub origin: String,
    pub origin_city: String,
    pub destination: String,
    pub destination_city: String,
    pub date: String,
    pub passengers: u8,
}

/// Contact information collected during booking
#[derive(Clone, Debug, Default)]
pub struct ContactInfo {
    pub email: String,
    pub phone_country: String,
    pub phone_number: String,
    pub marketing_opt_in: bool,
}

/// Central booking state
#[derive(Clone, Debug, Default)]
pub struct BookingState {
    pub search: SearchParams,
    pub selected_flight: Option<Flight>,
    pub price_lock: Option<PriceLock>,
    pub passengers: Vec<Passenger>,
    pub extras: BookingExtras,
    pub contact: ContactInfo,
    pub payment_method: Option<PaymentMethod>,
    pub booking_reference: Option<String>,
    pub terms_accepted: bool,
}

impl BookingState {
    /// Calculate total price in sen
    pub fn calculate_total(&self) -> i64 {
        let base = self
            .selected_flight
            .as_ref()
            .map(|f| f.price.amount)
            .unwrap_or(0);

        let lock_fee = self.price_lock.as_ref().map(|l| l.fee).unwrap_or(0);

        let pax_count = self.search.passengers.max(1) as i64;

        // Per passenger costs
        let bags = self.extras.checked_bags as i64 * 8000; // RM80 per bag in sen
        let seat = if self.extras.seat_selection.is_some() {
            4500
        } else {
            0
        }; // RM45
        let meal = if self.extras.meal.is_some() { 3500 } else { 0 }; // RM35

        // Insurance is per booking, not per passenger
        let insurance = match self.extras.insurance {
            Some(InsuranceType::Basic) => 2500,    // RM25
            Some(InsuranceType::Standard) => 4500, // RM45
            Some(InsuranceType::Premium) => 8500,  // RM85
            None => 0,
        };

        (base * pax_count)
            + lock_fee
            + (bags * pax_count)
            + (seat * pax_count)
            + (meal * pax_count)
            + insurance
    }

    /// Check if booking is ready for payment
    pub fn is_ready_for_payment(&self) -> bool {
        self.selected_flight.is_some()
            && !self.passengers.is_empty()
            && self
                .passengers
                .iter()
                .all(|p| !p.first_name.is_empty() && !p.last_name.is_empty())
            && !self.contact.email.is_empty()
            && !self.contact.phone_number.is_empty()
            && self.terms_accepted
    }
}

/// Provide booking state at app root
pub fn provide_booking_state() {
    let state = create_rw_signal(BookingState::default());
    provide_context(state);
}

/// Use booking state in any component
pub fn use_booking_state() -> RwSignal<BookingState> {
    use_context::<RwSignal<BookingState>>().expect("BookingState must be provided at app root")
}

/// Try to get booking state (returns None if not in context)
pub fn try_use_booking_state() -> Option<RwSignal<BookingState>> {
    use_context::<RwSignal<BookingState>>()
}

// ============================================================================
// State Actions
// ============================================================================

/// Set search parameters from home screen
pub fn set_search_params(
    origin: &str,
    origin_city: &str,
    destination: &str,
    destination_city: &str,
    date: &str,
    passengers: u8,
) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.search = SearchParams {
                origin: origin.to_string(),
                origin_city: origin_city.to_string(),
                destination: destination.to_string(),
                destination_city: destination_city.to_string(),
                date: date.to_string(),
                passengers: passengers.max(1),
            };
            // Initialize passengers based on count
            s.passengers = (0..passengers.max(1))
                .map(|i| Passenger {
                    id: format!("PAX{}", i + 1),
                    passenger_type: PassengerType::Adult,
                    ..Default::default()
                })
                .collect();
        });
    }
}

/// Select a flight
pub fn select_flight(flight: Flight) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.selected_flight = Some(flight);
        });
    }
}

/// Set price lock
pub fn set_price_lock(lock: PriceLock) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.price_lock = Some(lock);
        });
    }
}

/// Update a specific passenger
pub fn update_passenger(index: usize, passenger: Passenger) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            if index < s.passengers.len() {
                s.passengers[index] = passenger;
            }
        });
    }
}

/// Set all passengers
pub fn set_passengers(passengers: Vec<Passenger>) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.passengers = passengers;
        });
    }
}

/// Set extras
pub fn set_extras(extras: BookingExtras) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.extras = extras;
        });
    }
}

/// Set contact info
pub fn set_contact(contact: ContactInfo) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.contact = contact;
        });
    }
}

/// Set payment method
pub fn set_payment_method(method: PaymentMethod) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.payment_method = Some(method);
        });
    }
}

/// Accept terms
pub fn accept_terms(accepted: bool) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.terms_accepted = accepted;
        });
    }
}

/// Complete booking with reference
pub fn complete_booking(reference: String) {
    if let Some(state) = try_use_booking_state() {
        state.update(|s| {
            s.booking_reference = Some(reference);
        });
    }
}

/// Reset booking state
pub fn reset_booking() {
    if let Some(state) = try_use_booking_state() {
        state.set(BookingState::default());
    }
}

/// Get current search summary string
pub fn get_search_summary() -> String {
    if let Some(state) = try_use_booking_state() {
        let s = state.get();
        format!(
            "{} → {} · {} passenger{}",
            s.search.origin,
            s.search.destination,
            s.search.passengers,
            if s.search.passengers == 1 { "" } else { "s" }
        )
    } else {
        String::new()
    }
}

//! Flight Selection Screen
//!
//! Displays available flights with filtering and sorting options.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::components::FlightCard;
use crate::hooks::{mock_flights, use_booking_state, select_flight as set_flight};
use crate::types::Flight;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Sort options
#[derive(Clone, Copy, PartialEq, Default)]
pub enum SortOption {
    #[default]
    PriceLow,
    PriceHigh,
    Duration,
    Departure,
}

impl SortOption {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::PriceLow => "Price (Low to High)",
            Self::PriceHigh => "Price (High to Low)",
            Self::Duration => "Duration (Shortest)",
            Self::Departure => "Departure (Earliest)",
        }
    }
}

/// Filter options
#[derive(Clone, Default)]
pub struct FlightFilters {
    pub direct_only: bool,
    pub max_price: Option<i64>,
    pub airlines: Vec<String>,
}

/// Flight selection screen
#[component]
pub fn FlightSelection() -> impl IntoView {
    let navigate = use_navigate();
    let booking_state = use_booking_state();

    // Get search params from booking state or session storage fallback
    let (origin, destination, date, pax) = {
        let state = booking_state.get();
        if !state.search.origin.is_empty() {
            (
                state.search.origin.clone(),
                state.search.destination.clone(),
                state.search.date.clone(),
                state.search.passengers.to_string(),
            )
        } else {
            // Fallback to session storage
            let storage = get_session_storage();
            let origin = storage.as_ref().and_then(|s| s.get_item("oracle_origin").ok().flatten()).unwrap_or_else(|| "KUL".to_string());
            let destination = storage.as_ref().and_then(|s| s.get_item("oracle_destination").ok().flatten()).unwrap_or_else(|| "NRT".to_string());
            let date = storage.as_ref().and_then(|s| s.get_item("oracle_date").ok().flatten()).unwrap_or_default();
            let pax = storage.as_ref().and_then(|s| s.get_item("oracle_pax").ok().flatten()).unwrap_or_else(|| "1".to_string());
            (origin, destination, date, pax)
        }
    };

    // Use mock flights instead of generating
    let all_flights = mock_flights();

    // State
    let (sort_by, set_sort_by) = create_signal(SortOption::PriceLow);
    let (direct_only, set_direct_only) = create_signal(false);
    let (selected_flight, set_selected_flight) = create_signal::<Option<Flight>>(None);

    // Filter and sort flights
    let get_filtered_flights = {
        let all_flights = all_flights.clone();
        move || {
            let mut flights = all_flights.clone();

            // Apply direct filter
            if direct_only.get() {
                flights.retain(|f| f.stops == 0);
            }

            // Apply sort
            match sort_by.get() {
                SortOption::PriceLow => flights.sort_by_key(|f| f.price.amount),
                SortOption::PriceHigh => flights.sort_by_key(|f| std::cmp::Reverse(f.price.amount)),
                SortOption::Duration => flights.sort_by_key(|f| f.duration_minutes),
                SortOption::Departure => flights.sort_by(|a, b| a.departure_time.cmp(&b.departure_time)),
            }

            flights
        }
    };

    let all_flights_for_count = all_flights.clone();
    let flights_count = move || {
        let mut flights = all_flights_for_count.clone();
        if direct_only.get() {
            flights.retain(|f| f.stops == 0);
        }
        flights.len()
    };

    // Handle flight selection
    let handle_select = move |flight: Flight| {
        set_selected_flight.set(Some(flight));
    };

    // Clone navigate for different closures
    let nav_continue = navigate.clone();
    let nav_back = navigate;

    // Handle continue to price lock
    let handle_continue = move |_| {
        if let Some(flight) = selected_flight.get() {
            // Store in booking state
            set_flight(flight.clone());

            // Also store in session storage for backward compatibility
            if let Some(storage) = get_session_storage() {
                if let Ok(json) = serde_json::to_string(&flight) {
                    let _ = storage.set_item("selected_flight", &json);
                }
            }
            nav_continue("/booking/price-lock", Default::default());
        }
    };

    let route_display = format!("{} → {}", origin, destination);

    view! {
        <div class="screen-flight-selection">
            // Header
            <header class="flight-selection-header">
                <button
                    class="back-btn"
                    on:click=move |_| nav_back("/oracle/result/1", Default::default())
                >
                    "← Back"
                </button>
                <div class="header-route">
                    <h1 class="header-title">{route_display}</h1>
                    <span class="header-date">{date}</span>
                    <span class="header-pax">{pax}" passenger(s)"</span>
                </div>
            </header>

            // Filters and Sort
            <div class="flight-controls">
                <div class="flight-filters">
                    <label class="filter-checkbox">
                        <input
                            type="checkbox"
                            checked=move || direct_only.get()
                            on:change=move |ev| set_direct_only.set(event_target_checked(&ev))
                        />
                        <span>"Direct flights only"</span>
                    </label>
                </div>

                <div class="flight-sort">
                    <label class="sort-label">"Sort by:"</label>
                    <select
                        class="sort-select"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            let sort = match val.as_str() {
                                "price_high" => SortOption::PriceHigh,
                                "duration" => SortOption::Duration,
                                "departure" => SortOption::Departure,
                                _ => SortOption::PriceLow,
                            };
                            set_sort_by.set(sort);
                        }
                    >
                        <option value="price_low">"Price (Low to High)"</option>
                        <option value="price_high">"Price (High to Low)"</option>
                        <option value="duration">"Duration (Shortest)"</option>
                        <option value="departure">"Departure (Earliest)"</option>
                    </select>
                </div>
            </div>

            // Results count
            <div class="flight-results-count">
                {move || format!("{} flights found", flights_count())}
            </div>

            // Flight list
            <div class="flight-list-container">
                {move || {
                    let flights = get_filtered_flights();
                    if flights.is_empty() {
                        view! {
                            <div class="flight-list-empty">
                                <p>"No flights match your filters"</p>
                                <button
                                    class="btn btn-secondary"
                                    on:click=move |_| set_direct_only.set(false)
                                >
                                    "Clear filters"
                                </button>
                            </div>
                        }.into_view()
                    } else {
                        flights.into_iter().map(|flight| {
                            let is_selected = selected_flight.get().as_ref().map_or(false, |f| f.id == flight.id);
                            view! {
                                <FlightCard
                                    flight=flight
                                    selected=is_selected
                                    on_click=Callback::new(move |f: Flight| handle_select(f))
                                />
                            }
                        }).collect_view()
                    }
                }}
            </div>

            // Continue button (sticky footer)
            <div class="flight-selection-footer">
                <div class="selected-summary">
                    {move || selected_flight.get().map(|f| {
                        view! {
                            <div class="selected-flight-summary">
                                <span class="summary-airline">{f.airline_name}</span>
                                <span class="summary-price">{f.price.format()}</span>
                            </div>
                        }
                    })}
                </div>
                <button
                    class="btn btn-primary btn-lg"
                    disabled=move || selected_flight.get().is_none()
                    on:click=handle_continue
                >
                    "Continue"
                </button>
            </div>
        </div>
    }
}

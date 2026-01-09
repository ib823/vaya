//! Flight Card Component
//!
//! Displays flight information in a card format with route, price,
//! airline, and timing details. Supports selection state and click handling.

use crate::types::Flight;
use leptos::*;

/// Flight card displaying route, price, and flight details
#[component]
pub fn FlightCard(
    /// Flight data to display
    flight: Flight,
    /// Whether this card is selected
    #[prop(optional)]
    selected: bool,
    /// Click handler
    #[prop(optional)]
    on_click: Option<Callback<Flight>>,
    /// Compact mode for lists
    #[prop(optional)]
    compact: bool,
) -> impl IntoView {
    let flight_clone = flight.clone();
    let flight_for_click = flight.clone();
    let has_click = on_click.is_some();

    let handle_click = move |_| {
        if let Some(cb) = on_click {
            cb.call(flight_for_click.clone());
        }
    };

    let card_class = move || {
        let mut classes = vec!["flight-card"];
        if selected {
            classes.push("flight-card-selected");
        }
        if compact {
            classes.push("flight-card-compact");
        }
        if has_click {
            classes.push("flight-card-clickable");
        }
        classes.join(" ")
    };

    // Format duration (e.g., "7h 30m")
    let format_duration = |minutes: u32| -> String {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if hours > 0 && mins > 0 {
            format!("{}h {}m", hours, mins)
        } else if hours > 0 {
            format!("{}h", hours)
        } else {
            format!("{}m", mins)
        }
    };

    // Format time (e.g., "14:30")
    let format_time = |time: &str| -> String {
        // Assuming time is in HH:MM format or ISO format
        if time.len() >= 5 {
            time[..5].to_string()
        } else {
            time.to_string()
        }
    };

    // Stops display
    let stops_text = match flight_clone.stops {
        0 => "Direct".to_string(),
        1 => "1 stop".to_string(),
        n => format!("{} stops", n),
    };

    let stops_class = if flight_clone.stops == 0 {
        "flight-stops flight-stops-direct"
    } else {
        "flight-stops"
    };

    view! {
        <article
            class=card_class
            on:click=handle_click
            tabindex=move || if has_click { Some(0) } else { None }
            role=move || if has_click { Some("button") } else { None }
        >
            // Airline logo and name
            <div class="flight-card-header">
                <div class="flight-airline">
                    <span class="flight-airline-code">{flight.airline.clone()}</span>
                    <span class="flight-airline-name">{flight.airline_name.clone()}</span>
                </div>
                <span class="flight-number">{flight.flight_number.clone()}</span>
            </div>

            // Route and timing
            <div class="flight-card-route">
                <div class="flight-endpoint flight-origin">
                    <span class="flight-time">{format_time(&flight.departure_time)}</span>
                    <span class="flight-code">{flight.origin.clone()}</span>
                </div>

                <div class="flight-journey">
                    <span class="flight-duration">{format_duration(flight.duration_minutes)}</span>
                    <div class="flight-line">
                        <div class="flight-line-bar"></div>
                        {(flight.stops > 0).then(|| view! {
                            <div class="flight-line-stops">
                                {(0..flight.stops).map(|_| view! { <span class="flight-line-dot"></span> }).collect_view()}
                            </div>
                        })}
                    </div>
                    <span class=stops_class>{stops_text}</span>
                </div>

                <div class="flight-endpoint flight-destination">
                    <span class="flight-time">{format_time(&flight.arrival_time)}</span>
                    <span class="flight-code">{flight.destination.clone()}</span>
                </div>
            </div>

            // Price
            <div class="flight-card-footer">
                <div class="flight-price">
                    <span class="flight-price-currency">{flight.price.currency.clone()}</span>
                    <span class="flight-price-amount">{flight.price.amount.to_string()}</span>
                </div>
                <span class="flight-cabin">{flight.cabin_class.clone()}</span>
            </div>
        </article>
    }
}

/// List of flight cards
#[component]
pub fn FlightList(
    /// List of flights to display
    flights: Vec<Flight>,
    /// Currently selected flight ID
    #[prop(optional)]
    selected_id: Option<String>,
    /// Show compact cards
    #[prop(optional)]
    compact: bool,
    /// Empty state message
    #[prop(optional, into)]
    empty_message: Option<String>,
) -> impl IntoView {
    let empty_msg = empty_message.unwrap_or_else(|| "No flights found".to_string());

    view! {
        <div class="flight-list">
            {if flights.is_empty() {
                view! {
                    <div class="flight-list-empty">
                        <p>{empty_msg}</p>
                    </div>
                }.into_view()
            } else {
                flights.into_iter().map(|flight| {
                    let is_selected = selected_id.as_ref().is_some_and(|id| id == &flight.id);
                    view! {
                        <FlightCard
                            flight=flight
                            selected=is_selected
                            compact=compact
                        />
                    }
                }).collect_view()
            }}
        </div>
    }
}

/// Mini flight card for summaries (e.g., in oracle result)
#[component]
pub fn FlightMini(
    /// Origin code
    #[prop(into)]
    origin: String,
    /// Destination code
    #[prop(into)]
    destination: String,
    /// Price display
    #[prop(optional, into)]
    price: Option<String>,
) -> impl IntoView {
    view! {
        <div class="flight-mini">
            <span class="flight-mini-route">
                <span class="flight-mini-code">{origin}</span>
                <span class="flight-mini-arrow">"→"</span>
                <span class="flight-mini-code">{destination}</span>
            </span>
            {price.map(|p| view! {
                <span class="flight-mini-price">{p}</span>
            })}
        </div>
    }
}

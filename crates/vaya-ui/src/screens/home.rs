//! Home Screen - Intent Declaration
//!
//! The primary landing screen where users declare their travel intent.
//! This is the entry point to the Oracle flow.

use leptos::*;
use leptos_router::use_navigate;
use crate::components::{AirportPicker, SwapButton, DateInput};
use crate::hooks::set_search_params;

/// Home screen component
#[component]
pub fn Home() -> impl IntoView {
    // Get navigate function at component setup time (must be inside Router)
    let navigate = use_navigate();

    // Form state using RwSignal for component compatibility
    let origin = create_rw_signal(String::new());
    let destination = create_rw_signal(String::new());
    let departure_date = create_rw_signal(String::new());
    let (passengers, set_passengers) = create_signal(1u8);

    // Swap animation state
    let (_is_swapping, set_swapping) = create_signal(false);

    // Validation state
    let is_valid = move || {
        !origin.get().is_empty()
            && !destination.get().is_empty()
            && !departure_date.get().is_empty()
            && origin.get().len() == 3
            && destination.get().len() == 3
    };

    // Handle route swap
    let handle_swap = move |_| {
        set_swapping.set(true);
        let o = origin.get();
        let d = destination.get();
        origin.set(d);
        destination.set(o);

        // Reset animation state
        set_timeout(
            move || set_swapping.set(false),
            std::time::Duration::from_millis(300),
        );
    };

    // Handle search submission
    let on_search = move |_| {
        if is_valid() {
            // Set booking state
            set_search_params(
                &origin.get(),
                "", // origin city - will be resolved from airport picker
                &destination.get(),
                "", // destination city
                &departure_date.get(),
                passengers.get(),
            );

            // Navigate to oracle loading
            let query = format!(
                "?origin={}&destination={}&date={}&pax={}",
                origin.get(),
                destination.get(),
                departure_date.get(),
                passengers.get()
            );
            navigate(&format!("/oracle/loading{}", query), Default::default());
        }
    };

    // Get today's date for min value
    let today = js_sys::Date::new_0();
    let today_str = format!(
        "{:04}-{:02}-{:02}",
        today.get_full_year(),
        today.get_month() + 1,
        today.get_date()
    );

    view! {
        <div class="screen-home">
            // Hero section
            <section class="hero">
                <div class="hero-content">
                    <h1 class="hero-title">
                        <span class="hero-title-gradient">"Know exactly"</span>
                        <br />
                        "when to book"
                    </h1>
                    <p class="hero-subtitle">
                        "The Oracle analyzes millions of data points to tell you: book now, or wait?"
                    </p>
                </div>
            </section>

            // Search form section
            <section class="search-section">
                <div class="search-card">
                    <h2 class="search-title">"Where do you want to go?"</h2>

                    <div class="search-form">
                        // Route inputs with AirportPicker
                        <div class="search-row search-row-route">
                            <AirportPicker
                                label="FROM"
                                code=origin
                            />

                            <div class="route-swap">
                                <SwapButton
                                    on_swap=Callback::new(handle_swap)
                                />
                            </div>

                            <AirportPicker
                                label="TO"
                                code=destination
                            />
                        </div>

                        // Date and passengers row
                        <div class="search-row">
                            <div class="input-group input-group-date">
                                <DateInput
                                    label="When"
                                    value=departure_date
                                    min=today_str
                                />
                            </div>

                            <div class="input-group input-group-pax">
                                <label class="input-label" for="passengers">"Travelers"</label>
                                <select
                                    id="passengers"
                                    class="input"
                                    on:change=move |ev| {
                                        if let Ok(n) = event_target_value(&ev).parse::<u8>() {
                                            set_passengers.set(n);
                                        }
                                    }
                                >
                                    <option value="1" selected>"1 Adult"</option>
                                    <option value="2">"2 Adults"</option>
                                    <option value="3">"3 Adults"</option>
                                    <option value="4">"4 Adults"</option>
                                </select>
                            </div>
                        </div>

                        // Search button
                        <div class="search-action">
                            <button
                                class="btn btn-primary btn-lg btn-full"
                                type="button"
                                disabled=move || !is_valid()
                                on:click=on_search
                            >
                                "Ask the Oracle"
                            </button>
                        </div>
                    </div>
                </div>
            </section>

            // Features section
            <section class="features-section">
                <div class="features-grid">
                    <div class="feature-card">
                        <div class="feature-icon">"🔮"</div>
                        <h3 class="feature-title">"Price Predictions"</h3>
                        <p class="feature-description">
                            "ML-powered forecasts tell you if prices will rise or fall"
                        </p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">"👥"</div>
                        <h3 class="feature-title">"Demand Pools"</h3>
                        <p class="feature-description">
                            "Join others to unlock group discounts on your route"
                        </p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">"🔔"</div>
                        <h3 class="feature-title">"Smart Alerts"</h3>
                        <p class="feature-description">
                            "Get notified the moment it's time to book"
                        </p>
                    </div>
                </div>
            </section>
        </div>
    }
}

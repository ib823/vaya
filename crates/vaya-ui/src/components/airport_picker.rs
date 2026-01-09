//! Airport Picker Component
//!
//! A stylized airport selection component with autocomplete dropdown.
//! Shows IATA code prominently with city name below.

use leptos::*;

/// Airport data for autocomplete
#[derive(Clone, Debug)]
pub struct Airport {
    pub code: &'static str,
    pub city: &'static str,
    pub country: &'static str,
}

/// List of popular airports for autocomplete
const AIRPORTS: &[Airport] = &[
    Airport { code: "KUL", city: "Kuala Lumpur", country: "Malaysia" },
    Airport { code: "SIN", city: "Singapore", country: "Singapore" },
    Airport { code: "NRT", city: "Tokyo Narita", country: "Japan" },
    Airport { code: "HND", city: "Tokyo Haneda", country: "Japan" },
    Airport { code: "HKG", city: "Hong Kong", country: "Hong Kong" },
    Airport { code: "BKK", city: "Bangkok", country: "Thailand" },
    Airport { code: "ICN", city: "Seoul Incheon", country: "South Korea" },
    Airport { code: "TPE", city: "Taipei", country: "Taiwan" },
    Airport { code: "MNL", city: "Manila", country: "Philippines" },
    Airport { code: "CGK", city: "Jakarta", country: "Indonesia" },
    Airport { code: "SYD", city: "Sydney", country: "Australia" },
    Airport { code: "MEL", city: "Melbourne", country: "Australia" },
    Airport { code: "PER", city: "Perth", country: "Australia" },
    Airport { code: "LAX", city: "Los Angeles", country: "USA" },
    Airport { code: "JFK", city: "New York JFK", country: "USA" },
    Airport { code: "SFO", city: "San Francisco", country: "USA" },
    Airport { code: "LHR", city: "London Heathrow", country: "UK" },
    Airport { code: "CDG", city: "Paris CDG", country: "France" },
    Airport { code: "AMS", city: "Amsterdam", country: "Netherlands" },
    Airport { code: "FRA", city: "Frankfurt", country: "Germany" },
    Airport { code: "DXB", city: "Dubai", country: "UAE" },
    Airport { code: "DOH", city: "Doha", country: "Qatar" },
    Airport { code: "PEN", city: "Penang", country: "Malaysia" },
    Airport { code: "LGK", city: "Langkawi", country: "Malaysia" },
    Airport { code: "BKI", city: "Kota Kinabalu", country: "Malaysia" },
    Airport { code: "KCH", city: "Kuching", country: "Malaysia" },
    Airport { code: "JHB", city: "Johor Bahru", country: "Malaysia" },
    Airport { code: "DMK", city: "Bangkok Don Mueang", country: "Thailand" },
    Airport { code: "CNX", city: "Chiang Mai", country: "Thailand" },
    Airport { code: "HKT", city: "Phuket", country: "Thailand" },
    Airport { code: "SGN", city: "Ho Chi Minh City", country: "Vietnam" },
    Airport { code: "HAN", city: "Hanoi", country: "Vietnam" },
    Airport { code: "DPS", city: "Bali Denpasar", country: "Indonesia" },
    Airport { code: "SUB", city: "Surabaya", country: "Indonesia" },
    Airport { code: "PNH", city: "Phnom Penh", country: "Cambodia" },
    Airport { code: "REP", city: "Siem Reap", country: "Cambodia" },
    Airport { code: "RGN", city: "Yangon", country: "Myanmar" },
    Airport { code: "CMB", city: "Colombo", country: "Sri Lanka" },
    Airport { code: "DEL", city: "New Delhi", country: "India" },
    Airport { code: "BOM", city: "Mumbai", country: "India" },
    Airport { code: "KIX", city: "Osaka Kansai", country: "Japan" },
    Airport { code: "NGO", city: "Nagoya", country: "Japan" },
    Airport { code: "CTS", city: "Sapporo", country: "Japan" },
    Airport { code: "FUK", city: "Fukuoka", country: "Japan" },
    Airport { code: "PVG", city: "Shanghai Pudong", country: "China" },
    Airport { code: "PEK", city: "Beijing", country: "China" },
    Airport { code: "CAN", city: "Guangzhou", country: "China" },
    Airport { code: "SZX", city: "Shenzhen", country: "China" },
    Airport { code: "MFM", city: "Macau", country: "Macau" },
];

/// Get city name from IATA code
pub fn get_city_name(code: &str) -> String {
    AIRPORTS
        .iter()
        .find(|a| a.code.eq_ignore_ascii_case(code))
        .map(|a| a.city.to_string())
        .unwrap_or_default()
}

/// Filter airports based on search query
fn filter_airports(query: &str) -> Vec<&'static Airport> {
    if query.is_empty() {
        return AIRPORTS.iter().take(8).collect();
    }

    let query_upper = query.to_uppercase();
    let query_lower = query.to_lowercase();

    AIRPORTS
        .iter()
        .filter(|a| {
            a.code.starts_with(&query_upper)
                || a.city.to_lowercase().contains(&query_lower)
                || a.country.to_lowercase().contains(&query_lower)
        })
        .take(8)
        .collect()
}

/// Airport picker with autocomplete dropdown
#[component]
pub fn AirportPicker(
    /// Label text ("FROM" or "TO")
    #[prop(into)]
    label: String,
    /// IATA code signal (e.g., "KUL")
    code: RwSignal<String>,
    /// City name to display (e.g., "Kuala Lumpur")
    #[prop(optional, into)]
    city: Option<String>,
    /// Callback when airport is selected/changed
    #[prop(optional)]
    on_select: Option<Callback<String>>,
    /// Whether picker is disabled
    #[prop(optional)]
    disabled: bool,
    /// Error state
    #[prop(optional, into)]
    error: Option<String>,
) -> impl IntoView {
    let (is_open, set_open) = create_signal(false);
    let (search_text, set_search_text) = create_signal(String::new());
    let (highlighted_index, set_highlighted_index) = create_signal(0usize);

    let input_ref = create_node_ref::<html::Input>();

    let has_error = error.is_some();

    let displayed_city = move || {
        city.clone().unwrap_or_else(|| get_city_name(&code.get()))
    };

    let filtered_airports = move || filter_airports(&search_text.get());

    let wrapper_class = move || {
        let mut classes = vec!["airport-picker"];
        if disabled {
            classes.push("airport-picker-disabled");
        }
        if is_open.get() {
            classes.push("airport-picker-focus");
        }
        if has_error {
            classes.push("airport-picker-error");
        }
        classes.join(" ")
    };

    let select_airport = move |airport_code: &str| {
        code.set(airport_code.to_string());
        set_search_text.set(String::new());
        set_open.set(false);
        if let Some(cb) = on_select {
            cb.call(airport_code.to_string());
        }
    };

    let handle_input = move |ev: web_sys::Event| {
        let val = event_target_value(&ev).to_uppercase();
        set_search_text.set(val.clone());
        set_highlighted_index.set(0);

        // If user types exactly 3 chars that match a code, auto-select
        if val.len() == 3 {
            if let Some(airport) = AIRPORTS.iter().find(|a| a.code == val) {
                select_airport(airport.code);
                return;
            }
        }

        // Update code as they type for preview
        code.set(val);
    };

    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        let airports = filtered_airports();
        let len = airports.len();

        match ev.key().as_str() {
            "ArrowDown" => {
                ev.prevent_default();
                set_highlighted_index.update(|i| *i = (*i + 1).min(len.saturating_sub(1)));
            }
            "ArrowUp" => {
                ev.prevent_default();
                set_highlighted_index.update(|i| *i = i.saturating_sub(1));
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(airport) = airports.get(highlighted_index.get()) {
                    select_airport(airport.code);
                }
            }
            "Escape" => {
                set_open.set(false);
                set_search_text.set(String::new());
            }
            "Tab" => {
                set_open.set(false);
            }
            _ => {}
        }
    };

    let handle_focus = move |_| {
        set_open.set(true);
        set_search_text.set(code.get());
    };

    let handle_blur = move |_| {
        // Delay to allow click on dropdown item
        set_timeout(
            move || {
                set_open.set(false);
                // If search text doesn't match a valid code, revert
                let search = search_text.get();
                if search.len() != 3 || !AIRPORTS.iter().any(|a| a.code == search) {
                    set_search_text.set(String::new());
                }
            },
            std::time::Duration::from_millis(150),
        );
    };

    let input_id = format!("airport-{}", label.to_lowercase().replace(' ', "-"));

    view! {
        <div class=wrapper_class>
            <label class="airport-picker-label" for=input_id.clone()>
                {label}
            </label>

            <div class="airport-picker-container">
                <div class="airport-picker-display">
                    <input
                        node_ref=input_ref
                        id=input_id.clone()
                        class="airport-picker-input"
                        type="text"
                        maxlength=3
                        placeholder="---"
                        autocomplete="off"
                        disabled=disabled
                        value=move || if is_open.get() { search_text.get() } else { code.get() }
                        on:input=handle_input
                        on:focus=handle_focus
                        on:blur=handle_blur
                        on:keydown=handle_keydown
                    />

                    <span class="airport-city">
                        {move || {
                            let city_name = displayed_city();
                            if city_name.is_empty() {
                                "Select airport".to_string()
                            } else {
                                city_name
                            }
                        }}
                    </span>
                </div>

                // Autocomplete dropdown
                <Show when=move || is_open.get() && !disabled>
                    <div class="airport-dropdown">
                        {move || {
                            let airports = filtered_airports();
                            if airports.is_empty() {
                                view! {
                                    <div class="airport-dropdown-empty">
                                        "No airports found"
                                    </div>
                                }.into_view()
                            } else {
                                airports.iter().enumerate().map(|(i, airport)| {
                                    let is_highlighted = move || highlighted_index.get() == i;
                                    let airport_code = airport.code;
                                    let airport_city = airport.city;
                                    let airport_country = airport.country;

                                    view! {
                                        <button
                                            type="button"
                                            class="airport-dropdown-item"
                                            class:airport-dropdown-item-highlighted=is_highlighted
                                            on:mousedown=move |ev| {
                                                ev.prevent_default();
                                                select_airport(airport_code);
                                            }
                                            on:mouseenter=move |_| set_highlighted_index.set(i)
                                        >
                                            <span class="dropdown-code">{airport_code}</span>
                                            <span class="dropdown-city">{airport_city}</span>
                                            <span class="dropdown-country">{airport_country}</span>
                                        </button>
                                    }
                                }).collect_view()
                            }
                        }}
                    </div>
                </Show>
            </div>

            {error.map(|err| {
                view! {
                    <span class="airport-picker-error-text" role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

/// Swap button component for switching origin/destination
#[component]
pub fn SwapButton(
    /// Callback when swap is clicked
    on_swap: Callback<()>,
    /// Disabled state
    #[prop(optional)]
    disabled: bool,
) -> impl IntoView {
    let (is_animating, set_animating) = create_signal(false);

    let handle_click = move |_| {
        if !disabled && !is_animating.get() {
            set_animating.set(true);
            on_swap.call(());

            set_timeout(
                move || set_animating.set(false),
                std::time::Duration::from_millis(300),
            );
        }
    };

    let button_class = move || {
        let mut classes = vec!["swap-button"];
        if is_animating.get() {
            classes.push("swap-button-animating");
        }
        if disabled {
            classes.push("swap-button-disabled");
        }
        classes.join(" ")
    };

    view! {
        <button
            type="button"
            class=button_class
            disabled=disabled
            aria-label="Swap origin and destination"
            on:click=handle_click
        >
            <svg
                class="swap-icon"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <path d="M7 16V4M7 4L3 8M7 4L11 8" />
                <path d="M17 8V20M17 20L21 16M17 20L13 16" />
            </svg>
        </button>
    }
}

/// Route picker combining two airport pickers with a swap button
#[component]
pub fn RoutePicker(
    /// Origin airport code
    origin: RwSignal<String>,
    /// Destination airport code
    destination: RwSignal<String>,
    /// Disabled state
    #[prop(optional)]
    disabled: bool,
) -> impl IntoView {
    let handle_swap = move |_| {
        let orig = origin.get();
        let dest = destination.get();
        origin.set(dest);
        destination.set(orig);
    };

    view! {
        <div class="route-picker">
            <AirportPicker
                label="FROM"
                code=origin
                disabled=disabled
            />

            <SwapButton
                on_swap=Callback::new(handle_swap)
                disabled=disabled
            />

            <AirportPicker
                label="TO"
                code=destination
                disabled=disabled
            />
        </div>
    }
}

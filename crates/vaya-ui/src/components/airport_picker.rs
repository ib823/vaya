//! Airport Picker Component
//!
//! A stylized airport selection component displaying IATA code prominently
//! with city name below. Designed for the VAYA search interface.

use leptos::*;

/// Airport picker with large code display and city name
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
    let (is_focused, set_focused) = create_signal(false);
    let (is_editing, set_editing) = create_signal(false);

    // Clone error for closure
    let has_error = error.is_some();

    // Get city name from code (simplified mapping)
    let get_city_name = move |code: &str| -> String {
        match code.to_uppercase().as_str() {
            "KUL" => "Kuala Lumpur".to_string(),
            "SIN" => "Singapore".to_string(),
            "NRT" => "Tokyo Narita".to_string(),
            "HND" => "Tokyo Haneda".to_string(),
            "HKG" => "Hong Kong".to_string(),
            "BKK" => "Bangkok".to_string(),
            "ICN" => "Seoul Incheon".to_string(),
            "TPE" => "Taipei".to_string(),
            "MNL" => "Manila".to_string(),
            "CGK" => "Jakarta".to_string(),
            "SYD" => "Sydney".to_string(),
            "MEL" => "Melbourne".to_string(),
            "LAX" => "Los Angeles".to_string(),
            "JFK" => "New York JFK".to_string(),
            "LHR" => "London Heathrow".to_string(),
            "CDG" => "Paris CDG".to_string(),
            "DXB" => "Dubai".to_string(),
            "DOH" => "Doha".to_string(),
            _ => String::new(),
        }
    };

    let displayed_city = move || {
        city.clone().unwrap_or_else(|| get_city_name(&code.get()))
    };

    let wrapper_class = move || {
        let mut classes = vec!["airport-picker"];
        if disabled {
            classes.push("airport-picker-disabled");
        }
        if is_focused.get() {
            classes.push("airport-picker-focus");
        }
        if has_error {
            classes.push("airport-picker-error");
        }
        classes.join(" ")
    };

    let input_id = format!("airport-{}", label.to_lowercase());

    view! {
        <div class=wrapper_class>
            <label class="airport-picker-label" for=input_id.clone()>
                {label}
            </label>

            <div class="airport-picker-display" on:click=move |_| {
                if !disabled {
                    set_editing.set(true);
                }
            }>
                {move || {
                    if is_editing.get() {
                        view! {
                            <input
                                id=input_id.clone()
                                class="airport-picker-input"
                                type="text"
                                maxlength=3
                                placeholder="---"
                                value=code.get()
                                autofocus=true
                                on:input=move |ev| {
                                    let val = event_target_value(&ev).to_uppercase();
                                    code.set(val.clone());
                                    if let Some(cb) = on_select {
                                        cb.call(val);
                                    }
                                }
                                on:focus=move |_| set_focused.set(true)
                                on:blur=move |_| {
                                    set_focused.set(false);
                                    set_editing.set(false);
                                }
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" || ev.key() == "Escape" {
                                        set_editing.set(false);
                                    }
                                }
                            />
                        }.into_view()
                    } else {
                        let code_display = code.get();
                        let code_text = if code_display.is_empty() {
                            "---".to_string()
                        } else {
                            code_display
                        };

                        view! {
                            <span class="airport-code" class:airport-code-empty=move || code.get().is_empty()>
                                {code_text}
                            </span>
                        }.into_view()
                    }
                }}

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

            // Reset animation state after animation completes
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

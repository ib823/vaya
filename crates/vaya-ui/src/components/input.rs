//! Text Input Component
//!
//! A styled text input with label, validation states, and accessibility support.

use leptos::*;

/// Input field states
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum InputState {
    #[default]
    Default,
    Focus,
    Filled,
    Error,
    Success,
    Disabled,
}

impl InputState {
    /// Get CSS class for this state
    pub fn css_class(&self) -> &'static str {
        match self {
            InputState::Default => "",
            InputState::Focus => "input-focus",
            InputState::Filled => "input-filled",
            InputState::Error => "input-error",
            InputState::Success => "input-success",
            InputState::Disabled => "input-disabled",
        }
    }
}

/// Text input component with full state management
#[component]
pub fn TextInput(
    /// Label text displayed above the input
    #[prop(into)]
    label: String,
    /// Placeholder text when empty
    #[prop(into)]
    placeholder: String,
    /// Optional controlled value signal
    #[prop(optional)]
    value: Option<RwSignal<String>>,
    /// Error message to display (also sets error state)
    #[prop(default = None)]
    error: Option<String>,
    /// Whether the input is disabled
    #[prop(optional)]
    disabled: bool,
    /// Input type (text, email, password, etc.)
    #[prop(optional, into)]
    input_type: Option<String>,
    /// Maximum length
    #[prop(optional)]
    maxlength: Option<u32>,
    /// Callback when input value changes
    #[prop(optional)]
    on_input: Option<Callback<String>>,
    /// Callback when input loses focus
    #[prop(optional)]
    on_blur: Option<Callback<String>>,
    /// Optional ID for the input element
    #[prop(optional, into)]
    id: Option<String>,
    /// Auto-uppercase the input
    #[prop(optional)]
    uppercase: bool,
) -> impl IntoView {
    // Internal state if no external signal provided
    let internal_value = create_rw_signal(String::new());
    let current_value = value.unwrap_or(internal_value);

    let (is_focused, set_focused) = create_signal(false);

    // Clone error for state closure
    let has_error = error.is_some();

    // Compute the current state
    let state = move || {
        if disabled {
            InputState::Disabled
        } else if has_error {
            InputState::Error
        } else if is_focused.get() {
            InputState::Focus
        } else if !current_value.get().is_empty() {
            InputState::Filled
        } else {
            InputState::Default
        }
    };

    // Build CSS classes
    let input_class = move || {
        let mut classes = vec!["input"];
        let s = state();
        if !s.css_class().is_empty() {
            classes.push(s.css_class());
        }
        classes.join(" ")
    };

    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let input_id =
        id.unwrap_or_else(|| format!("input-{}", label.to_lowercase().replace(' ', "-")));
    let error_id = format!("{}-error", input_id);

    // Clone values for view
    let label_clone = label.clone();
    let error_id_clone = error_id.clone();

    view! {
        <div class="input-wrapper">
            <label class="input-label" for=input_id.clone()>
                {label_clone}
            </label>
            <input
                id=input_id
                class=input_class
                type=input_type
                placeholder=placeholder
                disabled=disabled
                maxlength=maxlength
                value=move || current_value.get()
                aria-invalid=has_error
                aria-describedby=move || if has_error { Some(error_id_clone.clone()) } else { None }
                on:input=move |ev| {
                    let mut val = event_target_value(&ev);
                    if uppercase {
                        val = val.to_uppercase();
                    }
                    current_value.set(val.clone());
                    if let Some(cb) = on_input {
                        cb.call(val);
                    }
                }
                on:focus=move |_| set_focused.set(true)
                on:blur=move |ev| {
                    set_focused.set(false);
                    if let Some(cb) = on_blur {
                        cb.call(event_target_value(&ev));
                    }
                }
            />
            {error.map(|err| {
                view! {
                    <span class="input-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

/// Specialized input for IATA airport codes (3 letters, uppercase)
#[component]
pub fn AirportCodeInput(
    /// Label text
    #[prop(into)]
    label: String,
    /// Placeholder (e.g., "KUL")
    #[prop(into)]
    placeholder: String,
    /// Controlled value signal
    value: RwSignal<String>,
    /// Error message
    #[prop(optional, into)]
    error: Option<String>,
    /// Disabled state
    #[prop(optional)]
    disabled: bool,
    /// Change callback
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    let (is_focused, set_focused) = create_signal(false);
    let has_error = error.is_some();
    let error_id = format!("airport-{}-error", label.to_lowercase().replace(' ', "-"));
    let error_id_attr = error_id.clone();

    let input_class = move || {
        let mut classes = vec!["input"];
        if disabled {
            classes.push("input-disabled");
        } else if has_error {
            classes.push("input-error");
        } else if is_focused.get() {
            classes.push("input-focus");
        } else if !value.get().is_empty() {
            classes.push("input-filled");
        }
        classes.join(" ")
    };

    let input_id = format!("airport-{}", label.to_lowercase().replace(' ', "-"));

    view! {
        <div class="input-wrapper">
            <label class="input-label" for=input_id.clone()>
                {label.clone()}
            </label>
            <input
                id=input_id
                class=input_class
                type="text"
                placeholder=placeholder
                disabled=disabled
                maxlength=3
                value=move || value.get()
                aria-invalid=has_error
                aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }
                on:input=move |ev| {
                    let val = event_target_value(&ev).to_uppercase();
                    value.set(val.clone());
                    if let Some(cb) = on_change {
                        cb.call(val);
                    }
                }
                on:focus=move |_| set_focused.set(true)
                on:blur=move |_| set_focused.set(false)
            />
            {error.map(|err| {
                view! {
                    <span class="input-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

/// Date input component
#[component]
pub fn DateInput(
    /// Label text
    #[prop(into)]
    label: String,
    /// Controlled value signal (YYYY-MM-DD format)
    value: RwSignal<String>,
    /// Minimum date (YYYY-MM-DD)
    #[prop(optional, into)]
    min: Option<String>,
    /// Maximum date (YYYY-MM-DD)
    #[prop(optional, into)]
    max: Option<String>,
    /// Error message
    #[prop(optional, into)]
    error: Option<String>,
    /// Disabled state
    #[prop(optional)]
    disabled: bool,
    /// Change callback
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    let (is_focused, set_focused) = create_signal(false);
    let has_error = error.is_some();
    let error_id_base = format!("date-{}-error", label.to_lowercase().replace(' ', "-"));
    let error_id = error_id_base.clone();
    let error_id_attr = error_id_base.clone();

    let input_class = move || {
        let mut classes = vec!["input", "input-date"];
        if disabled {
            classes.push("input-disabled");
        } else if has_error {
            classes.push("input-error");
        } else if is_focused.get() {
            classes.push("input-focus");
        } else if !value.get().is_empty() {
            classes.push("input-filled");
        }
        classes.join(" ")
    };

    let input_id = format!("date-{}", label.to_lowercase().replace(' ', "-"));

    view! {
        <div class="input-wrapper">
            <label class="input-label" for=input_id.clone()>
                {label.clone()}
            </label>
            <input
                id=input_id
                class=input_class
                type="date"
                disabled=disabled
                min=min
                max=max
                value=move || value.get()
                aria-invalid=has_error
                aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    value.set(val.clone());
                    if let Some(cb) = on_change {
                        cb.call(val);
                    }
                }
                on:focus=move |_| set_focused.set(true)
                on:blur=move |_| set_focused.set(false)
            />
            {error.map(|err| {
                view! {
                    <span class="input-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

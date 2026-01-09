//! Select Input Component
//!
//! A styled select dropdown with label, validation states, and accessibility support.

use leptos::*;

/// Option for select input
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Select input component with full state management
#[component]
pub fn SelectInput(
    /// Label text displayed above the select
    #[prop(into)]
    label: String,
    /// Available options
    #[prop(into)]
    options: Vec<SelectOption>,
    /// Controlled value signal
    value: RwSignal<String>,
    /// Placeholder text when no selection
    #[prop(optional, into)]
    placeholder: Option<String>,
    /// Error message to display
    #[prop(default = None)]
    error: Option<String>,
    /// Whether the select is disabled
    #[prop(optional)]
    disabled: bool,
    /// Callback when selection changes
    #[prop(optional)]
    on_change: Option<Callback<String>>,
) -> impl IntoView {
    let (is_focused, set_focused) = create_signal(false);
    let has_error = error.is_some();

    let select_class = move || {
        let mut classes = vec!["select-field"];
        if disabled {
            classes.push("select-disabled");
        } else if has_error {
            classes.push("select-error");
        } else if is_focused.get() {
            classes.push("select-focus");
        } else if !value.get().is_empty() {
            classes.push("select-filled");
        }
        classes.join(" ")
    };

    let select_id = format!("select-{}", label.to_lowercase().replace(' ', "-"));
    let error_id = format!("{}-error", select_id);
    let error_id_attr = error_id.clone();
    let placeholder_text = placeholder.unwrap_or_else(|| "Select...".to_string());

    view! {
        <div class="select-wrapper">
            <label class="select-label" for=select_id.clone()>
                {label}
            </label>
            <div class="select-container">
                <select
                    id=select_id
                    class=select_class
                    disabled=disabled
                    aria-invalid=has_error
                    aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        value.set(val.clone());
                        if let Some(cb) = on_change {
                            cb.call(val);
                        }
                    }
                    on:focus=move |_| set_focused.set(true)
                    on:blur=move |_| set_focused.set(false)
                >
                    <option value="" disabled selected=move || value.get().is_empty()>
                        {placeholder_text.clone()}
                    </option>
                    {options.iter().map(|opt| {
                        let opt_value = opt.value.clone();
                        let opt_label = opt.label.clone();
                        let is_selected = {
                            let v = opt.value.clone();
                            move || value.get() == v
                        };
                        view! {
                            <option value=opt_value selected=is_selected>
                                {opt_label}
                            </option>
                        }
                    }).collect_view()}
                </select>
                <div class="select-arrow">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path d="M2.5 4.5L6 8L9.5 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                </div>
            </div>
            {error.map(|err| {
                view! {
                    <span class="select-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

/// Title select for passenger forms (Mr, Mrs, Ms, etc.)
#[component]
pub fn TitleSelect(
    /// Controlled value signal
    value: RwSignal<String>,
    /// Error message
    #[prop(default = None)]
    error: Option<String>,
    /// Disabled state
    #[prop(optional)]
    disabled: bool,
) -> impl IntoView {
    let options = vec![
        SelectOption::new("mr", "Mr"),
        SelectOption::new("mrs", "Mrs"),
        SelectOption::new("ms", "Ms"),
        SelectOption::new("dr", "Dr"),
    ];

    view! {
        <SelectInput
            label="Title"
            options=options
            value=value
            placeholder="Title"
            error=error
            disabled=disabled
        />
    }
}

/// Country select for nationality/passport
#[component]
pub fn CountrySelect(
    /// Label text
    #[prop(into)]
    label: String,
    /// Controlled value signal
    value: RwSignal<String>,
    /// Error message
    #[prop(default = None)]
    error: Option<String>,
    /// Disabled state
    #[prop(optional)]
    disabled: bool,
) -> impl IntoView {
    // Common countries for flight bookings (abbreviated list)
    let options = vec![
        SelectOption::new("MY", "Malaysia"),
        SelectOption::new("SG", "Singapore"),
        SelectOption::new("TH", "Thailand"),
        SelectOption::new("ID", "Indonesia"),
        SelectOption::new("PH", "Philippines"),
        SelectOption::new("VN", "Vietnam"),
        SelectOption::new("JP", "Japan"),
        SelectOption::new("KR", "South Korea"),
        SelectOption::new("CN", "China"),
        SelectOption::new("TW", "Taiwan"),
        SelectOption::new("HK", "Hong Kong"),
        SelectOption::new("IN", "India"),
        SelectOption::new("AU", "Australia"),
        SelectOption::new("NZ", "New Zealand"),
        SelectOption::new("GB", "United Kingdom"),
        SelectOption::new("US", "United States"),
        SelectOption::new("CA", "Canada"),
        SelectOption::new("DE", "Germany"),
        SelectOption::new("FR", "France"),
        SelectOption::new("AE", "UAE"),
    ];

    view! {
        <SelectInput
            label=label
            options=options
            value=value
            placeholder="Select country"
            error=error
            disabled=disabled
        />
    }
}

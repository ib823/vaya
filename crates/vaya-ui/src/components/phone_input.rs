//! Phone Input Component
//!
//! A phone number input with country code selector and formatting.

use leptos::*;

/// Country code option
#[derive(Clone, Debug)]
pub struct CountryCode {
    pub code: String,      // "+60"
    pub country: String,   // "MY"
    pub name: String,      // "Malaysia"
    pub flag: String,      // "🇲🇾"
}

impl CountryCode {
    pub fn new(code: &str, country: &str, name: &str, flag: &str) -> Self {
        Self {
            code: code.to_string(),
            country: country.to_string(),
            name: name.to_string(),
            flag: flag.to_string(),
        }
    }
}

/// Get default country codes for phone input
pub fn default_country_codes() -> Vec<CountryCode> {
    vec![
        CountryCode::new("+60", "MY", "Malaysia", "🇲🇾"),
        CountryCode::new("+65", "SG", "Singapore", "🇸🇬"),
        CountryCode::new("+66", "TH", "Thailand", "🇹🇭"),
        CountryCode::new("+62", "ID", "Indonesia", "🇮🇩"),
        CountryCode::new("+63", "PH", "Philippines", "🇵🇭"),
        CountryCode::new("+84", "VN", "Vietnam", "🇻🇳"),
        CountryCode::new("+81", "JP", "Japan", "🇯🇵"),
        CountryCode::new("+82", "KR", "South Korea", "🇰🇷"),
        CountryCode::new("+86", "CN", "China", "🇨🇳"),
        CountryCode::new("+886", "TW", "Taiwan", "🇹🇼"),
        CountryCode::new("+852", "HK", "Hong Kong", "🇭🇰"),
        CountryCode::new("+91", "IN", "India", "🇮🇳"),
        CountryCode::new("+61", "AU", "Australia", "🇦🇺"),
        CountryCode::new("+64", "NZ", "New Zealand", "🇳🇿"),
        CountryCode::new("+44", "GB", "United Kingdom", "🇬🇧"),
        CountryCode::new("+1", "US", "United States", "🇺🇸"),
    ]
}

/// Phone input component with country code selector
#[component]
pub fn PhoneInput(
    /// Label text
    #[prop(into)]
    label: String,
    /// Country code signal (e.g., "+60")
    country_code: RwSignal<String>,
    /// Phone number signal (without country code)
    number: RwSignal<String>,
    /// Error message to display
    #[prop(default = None)]
    error: Option<String>,
    /// Whether the input is disabled
    #[prop(optional)]
    disabled: bool,
    /// Callback when phone number changes
    #[prop(optional)]
    on_change: Option<Callback<(String, String)>>,
) -> impl IntoView {
    let (is_focused, set_focused) = create_signal(false);
    let has_error = error.is_some();
    let country_codes = default_country_codes();

    // Set default country code if empty
    if country_code.get().is_empty() {
        country_code.set("+60".to_string());
    }

    let wrapper_class = move || {
        let mut classes = vec!["phone-wrapper"];
        if disabled {
            classes.push("phone-disabled");
        }
        if has_error {
            classes.push("phone-error");
        }
        if is_focused.get() {
            classes.push("phone-focus");
        }
        classes.join(" ")
    };

    let input_id = format!("phone-{}", label.to_lowercase().replace(' ', "-"));
    let error_id = format!("{}-error", input_id);
    let error_id_attr = error_id.clone();

    // Format phone number as user types (remove non-digits)
    let format_phone = move |input: String| -> String {
        input.chars().filter(|c| c.is_ascii_digit()).collect()
    };

    view! {
        <div class="phone-input-group">
            <label class="phone-label" for=input_id.clone()>
                {label}
            </label>
            <div class=wrapper_class>
                <select
                    class="country-code-select"
                    disabled=disabled
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        country_code.set(val.clone());
                        if let Some(cb) = on_change {
                            cb.call((val, number.get()));
                        }
                    }
                >
                    {country_codes.iter().map(|cc| {
                        let code = cc.code.clone();
                        let display = format!("{} {}", cc.flag, cc.code);
                        let is_selected = {
                            let c = cc.code.clone();
                            move || country_code.get() == c
                        };
                        view! {
                            <option value=code selected=is_selected>
                                {display}
                            </option>
                        }
                    }).collect_view()}
                </select>
                <input
                    id=input_id
                    type="tel"
                    class="phone-number-input"
                    placeholder="12 345 6789"
                    disabled=disabled
                    value=move || number.get()
                    aria-invalid=has_error
                    aria-describedby=move || if has_error { Some(error_id_attr.clone()) } else { None }
                    on:input=move |ev| {
                        let val = format_phone(event_target_value(&ev));
                        number.set(val.clone());
                        if let Some(cb) = on_change {
                            cb.call((country_code.get(), val));
                        }
                    }
                    on:focus=move |_| set_focused.set(true)
                    on:blur=move |_| set_focused.set(false)
                />
            </div>
            {error.map(|err| {
                view! {
                    <span class="phone-error-text" id=error_id role="alert">
                        {err}
                    </span>
                }
            })}
        </div>
    }
}

/// Get full phone number from country code and number
pub fn get_full_phone(country_code: &str, number: &str) -> String {
    format!("{}{}", country_code, number)
}

/// Validate Malaysian phone number format
pub fn validate_my_phone(number: &str) -> bool {
    // Malaysian numbers: 9-10 digits starting with 1
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 9 && digits.len() <= 10 && digits.starts_with('1')
}

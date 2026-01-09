//! Contact Details Screen
//!
//! Collects contact information for booking confirmation.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::components::{TextInput, PhoneInput};
use crate::hooks::validation::{validate, email_rules, phone_rules};

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Contact details screen
#[component]
pub fn ContactDetails() -> impl IntoView {
    let navigate = use_navigate();

    // State
    let email = create_rw_signal(String::new());
    let email_confirm = create_rw_signal(String::new());
    let country_code = create_rw_signal("+60".to_string());
    let phone_number = create_rw_signal(String::new());

    // Validation errors
    let (email_error, set_email_error) = create_signal::<Option<String>>(None);
    let (email_confirm_error, set_email_confirm_error) = create_signal::<Option<String>>(None);
    let (phone_error, set_phone_error) = create_signal::<Option<String>>(None);

    // Validate email
    let validate_email = move || {
        match validate(&email.get(), &email_rules()) {
            Ok(_) => {
                set_email_error.set(None);
                true
            }
            Err(e) => {
                set_email_error.set(Some(e));
                false
            }
        }
    };

    // Validate email confirmation
    let validate_email_confirm = move || {
        if email_confirm.get() != email.get() {
            set_email_confirm_error.set(Some("Emails do not match".to_string()));
            false
        } else {
            set_email_confirm_error.set(None);
            true
        }
    };

    // Validate phone
    let validate_phone = move || {
        match validate(&phone_number.get(), &phone_rules()) {
            Ok(_) => {
                set_phone_error.set(None);
                true
            }
            Err(e) => {
                set_phone_error.set(Some(e));
                false
            }
        }
    };

    // Overall validation
    let is_valid = move || {
        !email.get().is_empty()
            && !email_confirm.get().is_empty()
            && !phone_number.get().is_empty()
            && email.get() == email_confirm.get()
    };

    // Handle continue
    let handle_continue = {
        let nav = navigate.clone();
        move |_| {
            let email_ok = validate_email();
            let confirm_ok = validate_email_confirm();
            let phone_ok = validate_phone();

            if email_ok && confirm_ok && phone_ok {
                // Store contact info
                if let Some(storage) = get_session_storage() {
                    let _ = storage.set_item("contact_email", &email.get());
                    let _ = storage.set_item("contact_phone", &format!("{}{}", country_code.get(), phone_number.get()));
                }
                nav("/booking/review", Default::default());
            }
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/booking/extras", Default::default());
        }
    };

    view! {
        <div class="screen-contact">
            // Header
            <header class="contact-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Contact Details"</h1>
                <p class="page-subtitle">"We'll send your booking confirmation here"</p>
            </header>

            // Form
            <div class="contact-form">
                // Email section
                <section class="form-section">
                    <h2 class="section-title">"Email Address"</h2>
                    <div class="form-field">
                        <TextInput
                            label="Email"
                            placeholder="you@example.com"
                            input_type="email"
                            value=email
                            error=email_error.get()
                            on_blur=Callback::new(move |_| { validate_email(); })
                        />
                    </div>
                    <div class="form-field">
                        <TextInput
                            label="Confirm Email"
                            placeholder="Confirm your email"
                            input_type="email"
                            value=email_confirm
                            error=email_confirm_error.get()
                            on_blur=Callback::new(move |_| { validate_email_confirm(); })
                        />
                    </div>
                </section>

                // Phone section
                <section class="form-section">
                    <h2 class="section-title">"Phone Number"</h2>
                    <p class="section-hint">"For flight updates and emergencies"</p>
                    <div class="form-field">
                        <PhoneInput
                            label="Phone"
                            country_code=country_code
                            number=phone_number
                            error=phone_error.get()
                        />
                    </div>
                </section>

                // Marketing opt-in
                <section class="form-section">
                    <label class="checkbox-inline">
                        <input type="checkbox" class="checkbox-input" />
                        <span class="checkbox-box"></span>
                        <span class="checkbox-text">
                            "Send me exclusive deals and travel inspiration"
                        </span>
                    </label>
                </section>
            </div>

            // Continue button
            <div class="contact-footer">
                <button
                    class="btn btn-primary btn-lg btn-full"
                    disabled=move || !is_valid()
                    on:click=handle_continue
                >
                    "Continue to Review"
                </button>
            </div>
        </div>
    }
}

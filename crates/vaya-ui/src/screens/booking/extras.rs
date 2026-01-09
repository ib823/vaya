//! Extras Selection Screen
//!
//! Allows passengers to add baggage, seats, meals, and insurance.

use leptos::*;
use leptos_router::use_navigate;
use web_sys::Storage;

use crate::types::InsuranceType;

/// Get session storage
fn get_session_storage() -> Option<Storage> {
    web_sys::window()?.session_storage().ok()?
}

/// Baggage option
#[derive(Clone, Copy, PartialEq)]
pub struct BaggageOption {
    pub kg: u8,
    pub price: i64,
}

impl BaggageOption {
    pub fn options() -> Vec<Self> {
        vec![
            BaggageOption { kg: 0, price: 0 },
            BaggageOption {
                kg: 20,
                price: 8000,
            },
            BaggageOption {
                kg: 25,
                price: 10000,
            },
            BaggageOption {
                kg: 30,
                price: 12000,
            },
        ]
    }
}

/// Meal option
#[derive(Clone)]
pub struct MealOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: i64,
}

impl MealOption {
    pub fn options() -> Vec<Self> {
        vec![
            MealOption {
                id: "none".to_string(),
                name: "No meal".to_string(),
                description: "Skip in-flight meal".to_string(),
                price: 0,
            },
            MealOption {
                id: "standard".to_string(),
                name: "Standard Meal".to_string(),
                description: "Chicken rice or pasta".to_string(),
                price: 3500,
            },
            MealOption {
                id: "vegetarian".to_string(),
                name: "Vegetarian".to_string(),
                description: "Plant-based meal".to_string(),
                price: 3500,
            },
            MealOption {
                id: "premium".to_string(),
                name: "Premium Meal".to_string(),
                description: "Chef's special selection".to_string(),
                price: 6500,
            },
        ]
    }
}

/// Format price for display
fn format_price(amount: i64) -> String {
    if amount == 0 {
        "Free".to_string()
    } else {
        format!("+ RM {:.2}", amount as f64 / 100.0)
    }
}

/// Extras selection screen
#[component]
pub fn ExtrasSelection() -> impl IntoView {
    let navigate = use_navigate();

    // State
    let (selected_baggage, set_baggage) = create_signal(0u8); // kg
    let (selected_meal, set_meal) = create_signal::<Option<String>>(None);
    let (selected_insurance, set_insurance) = create_signal::<Option<InsuranceType>>(None);

    // Calculate total extras cost
    let extras_total = move || {
        let bag_cost = BaggageOption::options()
            .iter()
            .find(|b| b.kg == selected_baggage.get())
            .map(|b| b.price)
            .unwrap_or(0);

        let meal_cost = selected_meal
            .get()
            .and_then(|id| {
                MealOption::options()
                    .iter()
                    .find(|m| m.id == id)
                    .map(|m| m.price)
            })
            .unwrap_or(0);

        let insurance_cost = selected_insurance.get().map(|i| i.price_myr()).unwrap_or(0);

        bag_cost + meal_cost + insurance_cost
    };

    // Handle continue
    let handle_continue = {
        let nav = navigate.clone();
        move |_| {
            // Store extras in session storage
            if let Some(storage) = get_session_storage() {
                let _ = storage.set_item("extras_baggage", &selected_baggage.get().to_string());
                if let Some(meal) = selected_meal.get() {
                    let _ = storage.set_item("extras_meal", &meal);
                }
                if let Some(ins) = selected_insurance.get() {
                    let _ = storage.set_item("extras_insurance", &format!("{:?}", ins));
                }
                let _ = storage.set_item("extras_total", &extras_total().to_string());
            }
            nav("/booking/contact", Default::default());
        }
    };

    // Handle back
    let handle_back = {
        let nav = navigate;
        move |_| {
            nav("/booking/passengers", Default::default());
        }
    };

    view! {
        <div class="screen-extras">
            // Header
            <header class="extras-header">
                <button class="back-button" on:click=handle_back aria-label="Go back">
                    "← Back"
                </button>
                <h1 class="page-title">"Add Extras"</h1>
                <p class="page-subtitle">"Enhance your journey (all optional)"</p>
            </header>

            // Baggage section
            <section class="extras-section">
                <h2 class="section-title">"Checked Baggage"</h2>
                <div class="extras-grid baggage-grid">
                    {BaggageOption::options().into_iter().map(|opt| {
                        let is_selected = move || selected_baggage.get() == opt.kg;
                        let option_class = move || {
                            if is_selected() {
                                "extras-option extras-option-selected"
                            } else {
                                "extras-option"
                            }
                        };

                        view! {
                            <button
                                class=option_class
                                on:click=move |_| set_baggage.set(opt.kg)
                            >
                                <span class="option-main">
                                    {if opt.kg == 0 { "No bag".to_string() } else { format!("{} kg", opt.kg) }}
                                </span>
                                <span class="option-price">{format_price(opt.price)}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </section>

            // Meals section
            <section class="extras-section">
                <h2 class="section-title">"In-flight Meal"</h2>
                <div class="extras-list meal-list">
                    {MealOption::options().into_iter().map(|opt| {
                        let opt_id = opt.id.clone();
                        let opt_id_check = opt.id.clone();
                        let is_selected = move || selected_meal.get().as_ref() == Some(&opt_id_check);
                        let option_class = move || {
                            if is_selected() {
                                "extras-option extras-option-selected"
                            } else {
                                "extras-option"
                            }
                        };

                        view! {
                            <button
                                class=option_class
                                on:click=move |_| set_meal.set(Some(opt_id.clone()))
                            >
                                <div class="option-content">
                                    <span class="option-name">{opt.name}</span>
                                    <span class="option-desc">{opt.description}</span>
                                </div>
                                <span class="option-price">{format_price(opt.price)}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </section>

            // Insurance section
            <section class="extras-section">
                <h2 class="section-title">"Travel Insurance"</h2>
                <div class="extras-list insurance-list">
                    <button
                        class=move || if selected_insurance.get().is_none() { "extras-option extras-option-selected" } else { "extras-option" }
                        on:click=move |_| set_insurance.set(None)
                    >
                        <div class="option-content">
                            <span class="option-name">"No insurance"</span>
                            <span class="option-desc">"Travel at your own risk"</span>
                        </div>
                        <span class="option-price">"Free"</span>
                    </button>
                    {[InsuranceType::Basic, InsuranceType::Standard, InsuranceType::Premium].into_iter().map(|ins| {
                        let is_selected = move || selected_insurance.get() == Some(ins);
                        let option_class = move || {
                            if is_selected() {
                                "extras-option extras-option-selected"
                            } else {
                                "extras-option"
                            }
                        };

                        view! {
                            <button
                                class=option_class
                                on:click=move |_| set_insurance.set(Some(ins))
                            >
                                <div class="option-content">
                                    <span class="option-name">{ins.display_text()}</span>
                                    <span class="option-desc">{match ins {
                                        InsuranceType::Basic => "Trip cancellation up to RM 5,000",
                                        InsuranceType::Standard => "Trip cancellation + medical up to RM 50,000",
                                        InsuranceType::Premium => "Full coverage up to RM 500,000",
                                    }}</span>
                                </div>
                                <span class="option-price">{format_price(ins.price_myr())}</span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </section>

            // Footer with total
            <div class="extras-footer">
                <div class="extras-summary">
                    <span class="summary-label">"Extras total"</span>
                    <span class="summary-amount">
                        {move || format!("RM {:.2}", extras_total() as f64 / 100.0)}
                    </span>
                </div>
                <button
                    class="btn btn-primary btn-lg btn-full"
                    on:click=handle_continue
                >
                    "Continue"
                </button>
            </div>
        </div>
    }
}

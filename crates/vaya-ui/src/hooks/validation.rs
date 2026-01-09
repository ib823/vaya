//! Validation Module
//!
//! Provides validation rules and helpers for form fields.

/// Validation rule types
#[derive(Clone, Debug)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Email,
    Phone,
    Alphanumeric,
    LettersOnly,
    FutureDate,
    PastDate,
    MinAge(u8),
    MaxAge(u8),
    AgeBetween(u8, u8),
    MinMonthsFromDate(String, u8),
    Luhn,  // Credit card validation
    ExpiryDate,
}

/// Validate a value against a set of rules
pub fn validate(value: &str, rules: &[ValidationRule]) -> Result<(), String> {
    for rule in rules {
        match rule {
            ValidationRule::Required => {
                if value.trim().is_empty() {
                    return Err("This field is required".to_string());
                }
            }
            ValidationRule::MinLength(min) => {
                if value.len() < *min {
                    return Err(format!("Must be at least {} characters", min));
                }
            }
            ValidationRule::MaxLength(max) => {
                if value.len() > *max {
                    return Err(format!("Must be at most {} characters", max));
                }
            }
            ValidationRule::Pattern(pattern) => {
                // Simple pattern matching (not full regex)
                if !matches_pattern(value, pattern) {
                    return Err("Invalid format".to_string());
                }
            }
            ValidationRule::Email => {
                if !is_valid_email(value) {
                    return Err("Please enter a valid email address".to_string());
                }
            }
            ValidationRule::Phone => {
                if !is_valid_phone(value) {
                    return Err("Please enter a valid phone number".to_string());
                }
            }
            ValidationRule::Alphanumeric => {
                if !value.chars().all(|c| c.is_alphanumeric()) {
                    return Err("Only letters and numbers allowed".to_string());
                }
            }
            ValidationRule::LettersOnly => {
                if !value.chars().all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-' || c == '\'') {
                    return Err("Only letters allowed".to_string());
                }
            }
            ValidationRule::FutureDate => {
                if !is_future_date(value) {
                    return Err("Date must be in the future".to_string());
                }
            }
            ValidationRule::PastDate => {
                if !is_past_date(value) {
                    return Err("Date must be in the past".to_string());
                }
            }
            ValidationRule::MinAge(min_age) => {
                if !meets_min_age(value, *min_age) {
                    return Err(format!("Must be at least {} years old", min_age));
                }
            }
            ValidationRule::MaxAge(max_age) => {
                if !meets_max_age(value, *max_age) {
                    return Err(format!("Must be at most {} years old", max_age));
                }
            }
            ValidationRule::AgeBetween(min, max) => {
                if !age_between(value, *min, *max) {
                    return Err(format!("Age must be between {} and {}", min, max));
                }
            }
            ValidationRule::MinMonthsFromDate(from_date, months) => {
                if !min_months_from(value, from_date, *months) {
                    return Err(format!("Must be at least {} months from {}", months, from_date));
                }
            }
            ValidationRule::Luhn => {
                if !is_valid_luhn(value) {
                    return Err("Invalid card number".to_string());
                }
            }
            ValidationRule::ExpiryDate => {
                if !is_valid_expiry(value) {
                    return Err("Invalid or expired card".to_string());
                }
            }
        }
    }
    Ok(())
}

// ============================================================================
// PRE-BUILT RULE SETS
// ============================================================================

/// Rules for name fields
pub fn name_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::MinLength(2),
        ValidationRule::MaxLength(50),
        ValidationRule::LettersOnly,
    ]
}

/// Rules for email fields
pub fn email_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::Email,
    ]
}

/// Rules for phone number fields
pub fn phone_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::Phone,
    ]
}

/// Rules for passport number fields
pub fn passport_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::MinLength(6),
        ValidationRule::MaxLength(20),
        ValidationRule::Alphanumeric,
    ]
}

/// Rules for credit card number fields
pub fn card_number_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::Luhn,
    ]
}

/// Rules for CVV fields
pub fn cvv_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::MinLength(3),
        ValidationRule::MaxLength(4),
    ]
}

/// Rules for card expiry fields (MM/YY)
pub fn expiry_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::ExpiryDate,
    ]
}

/// Rules for adult passenger date of birth
pub fn adult_dob_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::PastDate,
        ValidationRule::MinAge(12),
    ]
}

/// Rules for child passenger date of birth
pub fn child_dob_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::PastDate,
        ValidationRule::AgeBetween(2, 11),
    ]
}

/// Rules for infant passenger date of birth
pub fn infant_dob_rules() -> Vec<ValidationRule> {
    vec![
        ValidationRule::Required,
        ValidationRule::PastDate,
        ValidationRule::MaxAge(2),
    ]
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/// Simple email validation
fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.is_empty() {
        return false;
    }

    // Basic email format check
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() || domain.is_empty() {
        return false;
    }

    // Domain must have at least one dot
    if !domain.contains('.') {
        return false;
    }

    // Basic character validation
    local.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '+')
        && domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-')
}

/// Phone number validation (digits only, 9-15 chars)
fn is_valid_phone(phone: &str) -> bool {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 9 && digits.len() <= 15
}

/// Simple pattern matching
fn matches_pattern(value: &str, _pattern: &str) -> bool {
    // Simplified - would need regex for full support
    !value.is_empty()
}

/// Check if date is in the future
fn is_future_date(date: &str) -> bool {
    let today = get_today();
    date > today.as_str()
}

/// Check if date is in the past
fn is_past_date(date: &str) -> bool {
    let today = get_today();
    date < today.as_str()
}

/// Check minimum age from date of birth
fn meets_min_age(dob: &str, min_age: u8) -> bool {
    if let Some(age) = calculate_age(dob) {
        age >= min_age as i32
    } else {
        false
    }
}

/// Check maximum age from date of birth
fn meets_max_age(dob: &str, max_age: u8) -> bool {
    if let Some(age) = calculate_age(dob) {
        age <= max_age as i32
    } else {
        false
    }
}

/// Check age is between min and max
fn age_between(dob: &str, min: u8, max: u8) -> bool {
    if let Some(age) = calculate_age(dob) {
        age >= min as i32 && age <= max as i32
    } else {
        false
    }
}

/// Check minimum months from a reference date
fn min_months_from(date: &str, from_date: &str, min_months: u8) -> bool {
    // Parse dates and calculate difference
    let date_parts: Vec<i32> = date.split('-').filter_map(|s| s.parse().ok()).collect();
    let from_parts: Vec<i32> = from_date.split('-').filter_map(|s| s.parse().ok()).collect();

    if date_parts.len() != 3 || from_parts.len() != 3 {
        return false;
    }

    let date_months = date_parts[0] * 12 + date_parts[1];
    let from_months = from_parts[0] * 12 + from_parts[1];

    (date_months - from_months) >= min_months as i32
}

/// Luhn algorithm for credit card validation
fn is_valid_luhn(card_number: &str) -> bool {
    let digits: Vec<u32> = card_number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for digit in digits.iter().rev() {
        let mut d = *digit;
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }

    sum % 10 == 0
}

/// Validate card expiry date (MM/YY format)
fn is_valid_expiry(expiry: &str) -> bool {
    let parts: Vec<&str> = expiry.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let month: u32 = match parts[0].parse() {
        Ok(m) => m,
        Err(_) => return false,
    };

    let year: u32 = match parts[1].parse() {
        Ok(y) => y,
        Err(_) => return false,
    };

    if month < 1 || month > 12 {
        return false;
    }

    // Get current date
    let now = js_sys::Date::new_0();
    let current_year = (now.get_full_year() as u32) % 100;
    let current_month = now.get_month() as u32 + 1;

    // Check if expired
    if year < current_year {
        return false;
    }
    if year == current_year && month < current_month {
        return false;
    }

    true
}

/// Get today's date as YYYY-MM-DD
fn get_today() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}

/// Calculate age from date of birth (YYYY-MM-DD)
fn calculate_age(dob: &str) -> Option<i32> {
    let parts: Vec<i32> = dob.split('-').filter_map(|s| s.parse().ok()).collect();
    if parts.len() != 3 {
        return None;
    }

    let now = js_sys::Date::new_0();
    let current_year = now.get_full_year() as i32;
    let current_month = now.get_month() as i32 + 1;
    let current_day = now.get_date() as i32;

    let birth_year = parts[0];
    let birth_month = parts[1];
    let birth_day = parts[2];

    let mut age = current_year - birth_year;

    // Adjust if birthday hasn't occurred this year
    if current_month < birth_month || (current_month == birth_month && current_day < birth_day) {
        age -= 1;
    }

    Some(age)
}

/// Format card number with spaces (4-digit groups)
pub fn format_card_number(number: &str) -> String {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    digits
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format expiry as MM/YY
pub fn format_expiry(input: &str) -> String {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 2 {
        format!("{}/{}", &digits[..2], &digits[2..].chars().take(2).collect::<String>())
    } else {
        digits
    }
}

/// Get card type from number
pub fn get_card_type(number: &str) -> Option<&'static str> {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }

    let first = digits.chars().next()?;
    let first_two: String = digits.chars().take(2).collect();

    match first {
        '4' => Some("Visa"),
        '5' => {
            if let Ok(num) = first_two.parse::<u32>() {
                if (51..=55).contains(&num) {
                    return Some("Mastercard");
                }
            }
            None
        }
        '3' => {
            if first_two == "34" || first_two == "37" {
                Some("American Express")
            } else {
                None
            }
        }
        _ => None,
    }
}

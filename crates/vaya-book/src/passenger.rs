//! Passenger data types with validation

use time::Date;
use vaya_common::Gender;
use vaya_search::PassengerType;

use crate::{BookError, BookResult};

/// A passenger in a booking
#[derive(Debug, Clone)]
pub struct Passenger {
    /// Passenger ID within booking
    pub id: u8,
    /// Passenger type
    pub pax_type: PassengerType,
    /// Title (Mr, Mrs, Ms, etc.)
    pub title: Title,
    /// First/given name (as on passport)
    pub first_name: String,
    /// Last/family name (as on passport)
    pub last_name: String,
    /// Middle name (optional)
    pub middle_name: Option<String>,
    /// Date of birth
    pub date_of_birth: Date,
    /// Gender
    pub gender: Gender,
    /// Nationality (ISO 3166-1 alpha-2)
    pub nationality: CountryCode,
    /// Document details
    pub document: Option<TravelDocument>,
    /// Contact details (primary passenger only)
    pub contact: Option<ContactDetails>,
    /// Frequent flyer numbers
    pub frequent_flyer: Vec<FrequentFlyer>,
    /// Special requests
    pub special_requests: Vec<SpecialRequest>,
    /// Meal preference
    pub meal_preference: Option<MealPreference>,
    /// Seat preference
    pub seat_preference: Option<SeatPreference>,
    /// Redress number (TSA)
    pub redress_number: Option<String>,
    /// Known traveler number
    pub known_traveler_number: Option<String>,
}

impl Passenger {
    /// Create a new adult passenger
    pub fn adult(
        first_name: impl Into<String>,
        last_name: impl Into<String>,
        dob: Date,
        gender: Gender,
    ) -> Self {
        Self {
            id: 0,
            pax_type: PassengerType::Adult,
            title: if gender == Gender::Male {
                Title::Mr
            } else {
                Title::Ms
            },
            first_name: first_name.into().to_uppercase(),
            last_name: last_name.into().to_uppercase(),
            middle_name: None,
            date_of_birth: dob,
            gender,
            nationality: CountryCode::new("SG"),
            document: None,
            contact: None,
            frequent_flyer: Vec::new(),
            special_requests: Vec::new(),
            meal_preference: None,
            seat_preference: None,
            redress_number: None,
            known_traveler_number: None,
        }
    }

    /// Validate passenger data
    pub fn validate(&self, departure_date: Date) -> BookResult<()> {
        // Validate name
        self.validate_name()?;

        // Validate age for passenger type
        self.validate_age(departure_date)?;

        // Validate document if present
        if let Some(ref doc) = self.document {
            doc.validate(departure_date)?;
        }

        // Validate contact if present
        if let Some(ref contact) = self.contact {
            contact.validate()?;
        }

        Ok(())
    }

    /// Validate name fields
    fn validate_name(&self) -> BookResult<()> {
        // First name required
        if self.first_name.is_empty() {
            return Err(BookError::MissingField("first_name".into()));
        }

        // Last name required
        if self.last_name.is_empty() {
            return Err(BookError::MissingField("last_name".into()));
        }

        // Name length check (airline systems typically max 30 chars)
        if self.first_name.len() > 30 {
            return Err(BookError::InvalidPassenger(
                "First name too long (max 30 chars)".into(),
            ));
        }

        if self.last_name.len() > 30 {
            return Err(BookError::InvalidPassenger(
                "Last name too long (max 30 chars)".into(),
            ));
        }

        // Name characters check (letters, space, hyphen, apostrophe only)
        if !is_valid_name(&self.first_name) {
            return Err(BookError::InvalidPassenger(
                "First name contains invalid characters".into(),
            ));
        }

        if !is_valid_name(&self.last_name) {
            return Err(BookError::InvalidPassenger(
                "Last name contains invalid characters".into(),
            ));
        }

        if let Some(ref middle) = self.middle_name {
            if !is_valid_name(middle) {
                return Err(BookError::InvalidPassenger(
                    "Middle name contains invalid characters".into(),
                ));
            }
        }

        Ok(())
    }

    /// Validate age matches passenger type
    fn validate_age(&self, departure_date: Date) -> BookResult<()> {
        let age = calculate_age(self.date_of_birth, departure_date);

        match self.pax_type {
            PassengerType::Adult => {
                if age < 12 {
                    return Err(BookError::InvalidPassenger(format!(
                        "Adult must be 12+ years old, passenger is {} years",
                        age
                    )));
                }
            }
            PassengerType::Child => {
                if !(2..12).contains(&age) {
                    return Err(BookError::InvalidPassenger(format!(
                        "Child must be 2-11 years old, passenger is {} years",
                        age
                    )));
                }
            }
            PassengerType::Infant => {
                if age >= 2 {
                    return Err(BookError::InvalidPassenger(format!(
                        "Infant must be under 2 years old, passenger is {} years",
                        age
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get full name
    pub fn full_name(&self) -> String {
        match &self.middle_name {
            Some(middle) => format!("{} {} {}", self.first_name, middle, self.last_name),
            None => format!("{} {}", self.first_name, self.last_name),
        }
    }

    /// Get name for PNR (last/first format)
    pub fn pnr_name(&self) -> String {
        format!("{}/{}", self.last_name, self.first_name)
    }
}

/// Passenger title
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Title {
    Mr,
    Mrs,
    Ms,
    Miss,
    Mstr, // Master (child)
    Dr,
    Prof,
}

impl Title {
    pub fn as_str(&self) -> &'static str {
        match self {
            Title::Mr => "MR",
            Title::Mrs => "MRS",
            Title::Ms => "MS",
            Title::Miss => "MISS",
            Title::Mstr => "MSTR",
            Title::Dr => "DR",
            Title::Prof => "PROF",
        }
    }
}

/// Country code (ISO 3166-1 alpha-2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountryCode([u8; 2]);

impl CountryCode {
    pub fn new(code: &str) -> Self {
        let bytes = code.as_bytes();
        let mut arr = [b' '; 2];
        for (i, b) in bytes.iter().take(2).enumerate() {
            arr[i] = b.to_ascii_uppercase();
        }
        Self(arr)
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("XX").trim()
    }
}

/// Travel document
#[derive(Debug, Clone)]
pub struct TravelDocument {
    /// Document type
    pub doc_type: DocumentType,
    /// Document number
    pub number: String,
    /// Issuing country
    pub issuing_country: CountryCode,
    /// Issue date
    pub issue_date: Option<Date>,
    /// Expiry date
    pub expiry_date: Date,
}

impl TravelDocument {
    /// Create a passport document
    pub fn passport(number: &str, country: CountryCode, expiry: Date) -> Self {
        Self {
            doc_type: DocumentType::Passport,
            number: number.to_uppercase().replace(" ", ""),
            issuing_country: country,
            issue_date: None,
            expiry_date: expiry,
        }
    }

    /// Validate document
    pub fn validate(&self, departure_date: Date) -> BookResult<()> {
        // Check document number format
        if self.number.is_empty() {
            return Err(BookError::MissingField("document_number".into()));
        }

        if self.number.len() < 5 || self.number.len() > 20 {
            return Err(BookError::InvalidPassenger(
                "Document number must be 5-20 characters".into(),
            ));
        }

        // Check alphanumeric
        if !self.number.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(BookError::InvalidPassenger(
                "Document number must be alphanumeric".into(),
            ));
        }

        // Check expiry - most countries require 6 months validity
        let min_expiry = departure_date + time::Duration::days(180);
        if self.expiry_date < min_expiry {
            return Err(BookError::InvalidPassenger(
                "Document must be valid for at least 6 months from travel date".into(),
            ));
        }

        Ok(())
    }
}

/// Document type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    Passport,
    NationalId,
    DrivingLicense,
    Other,
}

impl DocumentType {
    pub fn code(&self) -> &'static str {
        match self {
            DocumentType::Passport => "P",
            DocumentType::NationalId => "I",
            DocumentType::DrivingLicense => "D",
            DocumentType::Other => "O",
        }
    }
}

/// Contact details
#[derive(Debug, Clone)]
pub struct ContactDetails {
    /// Email address
    pub email: String,
    /// Phone country code
    pub phone_country: String,
    /// Phone number
    pub phone_number: String,
    /// Emergency contact name
    pub emergency_name: Option<String>,
    /// Emergency contact phone
    pub emergency_phone: Option<String>,
}

impl ContactDetails {
    /// Create contact with email and phone
    pub fn new(
        email: impl Into<String>,
        phone_country: impl Into<String>,
        phone: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into().to_lowercase(),
            phone_country: phone_country.into(),
            phone_number: phone.into(),
            emergency_name: None,
            emergency_phone: None,
        }
    }

    /// Validate contact details
    pub fn validate(&self) -> BookResult<()> {
        // Validate email
        if !is_valid_email(&self.email) {
            return Err(BookError::InvalidContact("Invalid email format".into()));
        }

        // Validate phone
        if !is_valid_phone(&self.phone_number) {
            return Err(BookError::InvalidContact("Invalid phone number".into()));
        }

        Ok(())
    }
}

/// Frequent flyer program
#[derive(Debug, Clone)]
pub struct FrequentFlyer {
    /// Airline code
    pub airline: String,
    /// Member number
    pub number: String,
}

/// Special service request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialRequest {
    Wheelchair,
    WheelchairRamp,
    WheelchairSteps,
    WheelchairCabin,
    BlindPassenger,
    DeafPassenger,
    UnaccompaniedMinor,
    MeetAssist,
    ExtraLegroom,
    BassinetRequired,
    OxygenRequired,
    StretcherRequired,
    ServiceAnimal,
}

impl SpecialRequest {
    pub fn code(&self) -> &'static str {
        match self {
            SpecialRequest::Wheelchair => "WCHR",
            SpecialRequest::WheelchairRamp => "WCHC",
            SpecialRequest::WheelchairSteps => "WCHS",
            SpecialRequest::WheelchairCabin => "WCHC",
            SpecialRequest::BlindPassenger => "BLND",
            SpecialRequest::DeafPassenger => "DEAF",
            SpecialRequest::UnaccompaniedMinor => "UMNR",
            SpecialRequest::MeetAssist => "MAAS",
            SpecialRequest::ExtraLegroom => "EXST",
            SpecialRequest::BassinetRequired => "BSCT",
            SpecialRequest::OxygenRequired => "OXYG",
            SpecialRequest::StretcherRequired => "STCR",
            SpecialRequest::ServiceAnimal => "SVAN",
        }
    }
}

/// Meal preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealPreference {
    Regular,
    Vegetarian,
    VeganMeal,
    Kosher,
    Halal,
    Hindu,
    GlutenFree,
    LowSodium,
    LowFat,
    Diabetic,
    ChildMeal,
    InfantMeal,
    SeafoodMeal,
    FruitPlatter,
}

impl MealPreference {
    pub fn code(&self) -> &'static str {
        match self {
            MealPreference::Regular => "AVML",
            MealPreference::Vegetarian => "VGML",
            MealPreference::VeganMeal => "VGML",
            MealPreference::Kosher => "KSML",
            MealPreference::Halal => "MOML",
            MealPreference::Hindu => "HNML",
            MealPreference::GlutenFree => "GFML",
            MealPreference::LowSodium => "LSML",
            MealPreference::LowFat => "LFML",
            MealPreference::Diabetic => "DBML",
            MealPreference::ChildMeal => "CHML",
            MealPreference::InfantMeal => "BBML",
            MealPreference::SeafoodMeal => "SFML",
            MealPreference::FruitPlatter => "FPML",
        }
    }
}

/// Seat preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatPreference {
    Window,
    Aisle,
    Middle,
    FrontOfCabin,
    RearOfCabin,
    ExitRow,
    Bulkhead,
    NoPreference,
}

impl SeatPreference {
    pub fn code(&self) -> &'static str {
        match self {
            SeatPreference::Window => "W",
            SeatPreference::Aisle => "A",
            SeatPreference::Middle => "M",
            SeatPreference::FrontOfCabin => "F",
            SeatPreference::RearOfCabin => "R",
            SeatPreference::ExitRow => "E",
            SeatPreference::Bulkhead => "B",
            SeatPreference::NoPreference => "N",
        }
    }
}

// === Validation helpers ===

/// Check if name contains only valid characters
fn is_valid_name(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '-' || c == '\'')
}

/// Basic email validation
fn is_valid_email(email: &str) -> bool {
    // Must contain exactly one @
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let (local, domain) = (parts[0], parts[1]);

    // Local part not empty and reasonable length
    if local.is_empty() || local.len() > 64 {
        return false;
    }

    // Domain must have at least one dot
    if !domain.contains('.') {
        return false;
    }

    // Domain not empty and reasonable length
    if domain.is_empty() || domain.len() > 255 {
        return false;
    }

    // Basic character check
    email
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-_@+".contains(c))
}

/// Basic phone number validation
fn is_valid_phone(phone: &str) -> bool {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.len() >= 7 && digits.len() <= 15
}

/// Calculate age from date of birth
fn calculate_age(dob: Date, reference: Date) -> i32 {
    let years = reference.year() - dob.year();

    // Adjust if birthday hasn't occurred yet this year
    if (reference.month() as u8) < (dob.month() as u8) {
        years - 1
    } else if (reference.month() as u8) == (dob.month() as u8) && reference.day() < dob.day() {
        years - 1
    } else {
        years
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passenger_creation() {
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let pax = Passenger::adult("John", "Doe", dob, Gender::Male);

        assert_eq!(pax.first_name, "JOHN");
        assert_eq!(pax.last_name, "DOE");
        assert_eq!(pax.pax_type, PassengerType::Adult);
    }

    #[test]
    fn test_passenger_validation() {
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let pax = Passenger::adult("John", "Doe", dob, Gender::Male);

        assert!(pax.validate(dep).is_ok());
    }

    #[test]
    fn test_age_validation() {
        let dob = Date::from_calendar_date(2020, time::Month::January, 15).unwrap();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let mut pax = Passenger::adult("Child", "Name", dob, Gender::Male);

        // 5 year old can't be adult
        assert!(pax.validate(dep).is_err());

        // Change to child
        pax.pax_type = PassengerType::Child;
        assert!(pax.validate(dep).is_ok());
    }

    #[test]
    fn test_document_validation() {
        let expiry = Date::from_calendar_date(2030, time::Month::January, 1).unwrap();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();

        let doc = TravelDocument::passport("E12345678", CountryCode::new("SG"), expiry);
        assert!(doc.validate(dep).is_ok());

        // Expired document
        let expired = TravelDocument::passport("E12345678", CountryCode::new("SG"), dep);
        assert!(expired.validate(dep).is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("test.name@example.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("no@domain"));
        assert!(!is_valid_email("@nodomain.com"));
    }

    #[test]
    fn test_phone_validation() {
        assert!(is_valid_phone("12345678"));
        assert!(is_valid_phone("+65 9123 4567"));
        assert!(!is_valid_phone("123"));
        assert!(!is_valid_phone(""));
    }

    #[test]
    fn test_age_calculation() {
        let dob = Date::from_calendar_date(1990, time::Month::June, 15).unwrap();

        let before_birthday = Date::from_calendar_date(2025, time::Month::January, 1).unwrap();
        assert_eq!(calculate_age(dob, before_birthday), 34);

        let after_birthday = Date::from_calendar_date(2025, time::Month::July, 1).unwrap();
        assert_eq!(calculate_age(dob, after_birthday), 35);
    }

    #[test]
    fn test_name_validation() {
        assert!(is_valid_name("John"));
        assert!(is_valid_name("Mary-Jane"));
        assert!(is_valid_name("O'Brien"));
        assert!(!is_valid_name("John123"));
        assert!(!is_valid_name("Name!"));
    }

    #[test]
    fn test_pnr_name() {
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let pax = Passenger::adult("John", "Doe", dob, Gender::Male);
        assert_eq!(pax.pnr_name(), "DOE/JOHN");
    }
}

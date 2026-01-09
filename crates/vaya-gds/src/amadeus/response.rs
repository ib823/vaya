//! Amadeus API response types
//!
//! These types map to Amadeus API responses and are converted
//! to VAYA types for internal use.

use serde::Deserialize;

/// Amadeus error response
#[derive(Debug, Deserialize)]
pub struct AmadeusError {
    /// Error list
    pub errors: Vec<AmadeusErrorDetail>,
}

/// Amadeus error detail
#[derive(Debug, Deserialize)]
pub struct AmadeusErrorDetail {
    /// Error code
    pub code: Option<i32>,
    /// Error title
    pub title: Option<String>,
    /// Error detail
    pub detail: Option<String>,
    /// HTTP status
    pub status: Option<i32>,
}

/// Flight offers search response
#[derive(Debug, Deserialize)]
pub struct FlightOffersResponse {
    /// Response data
    pub data: Vec<AmadeusFlightOffer>,
    /// Dictionaries for lookup
    pub dictionaries: Option<Dictionaries>,
}

/// Dictionaries for airline/airport names
#[derive(Debug, Deserialize)]
pub struct Dictionaries {
    /// Carrier codes to names
    pub carriers: Option<std::collections::HashMap<String, String>>,
    /// Aircraft codes to names
    pub aircraft: Option<std::collections::HashMap<String, String>>,
    /// Location codes to details
    pub locations: Option<std::collections::HashMap<String, LocationDict>>,
}

/// Location dictionary entry
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationDict {
    /// City code
    pub city_code: Option<String>,
    /// Country code
    pub country_code: Option<String>,
}

/// Amadeus flight offer
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusFlightOffer {
    /// Offer type
    #[serde(rename = "type")]
    pub offer_type: String,
    /// Unique offer ID
    pub id: String,
    /// Source (GDS)
    pub source: Option<String>,
    /// Instant ticketing required
    pub instant_ticketing_required: Option<bool>,
    /// Number of bookable seats
    pub number_of_bookable_seats: Option<u32>,
    /// Itineraries
    pub itineraries: Vec<AmadeusItinerary>,
    /// Price information
    pub price: AmadeusPrice,
    /// Pricing options
    pub pricing_options: Option<PricingOptions>,
    /// Validating airline codes
    pub validating_airline_codes: Option<Vec<String>>,
    /// Traveler pricings
    pub traveler_pricings: Option<Vec<TravelerPricing>>,
    /// Last ticketing date
    pub last_ticketing_date: Option<String>,
}

/// Amadeus itinerary
#[derive(Debug, Deserialize)]
pub struct AmadeusItinerary {
    /// Duration (ISO 8601)
    pub duration: Option<String>,
    /// Segments
    pub segments: Vec<AmadeusSegment>,
}

/// Amadeus segment
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusSegment {
    /// Departure info
    pub departure: AmadeusFlightEndpoint,
    /// Arrival info
    pub arrival: AmadeusFlightEndpoint,
    /// Carrier code
    pub carrier_code: String,
    /// Flight number
    pub number: String,
    /// Aircraft
    pub aircraft: Option<Aircraft>,
    /// Operating carrier
    pub operating: Option<OperatingCarrier>,
    /// Duration
    pub duration: Option<String>,
    /// Segment ID
    pub id: Option<String>,
    /// Number of stops
    pub number_of_stops: Option<u32>,
    /// Blacklisted in EU
    pub blacklisted_in_eu: Option<bool>,
}

/// Flight endpoint (departure/arrival)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusFlightEndpoint {
    /// IATA code
    pub iata_code: String,
    /// Terminal
    pub terminal: Option<String>,
    /// Date and time
    pub at: String,
}

/// Aircraft info
#[derive(Debug, Deserialize)]
pub struct Aircraft {
    /// Aircraft code
    pub code: String,
}

/// Operating carrier
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatingCarrier {
    /// Carrier code
    pub carrier_code: String,
}

/// Price information
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmadeusPrice {
    /// Currency
    pub currency: String,
    /// Total price
    pub total: String,
    /// Base price
    pub base: Option<String>,
    /// Fees
    pub fees: Option<Vec<Fee>>,
    /// Grand total
    pub grand_total: Option<String>,
}

/// Fee information
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    /// Amount
    pub amount: String,
    /// Fee type
    #[serde(rename = "type")]
    pub fee_type: String,
}

/// Pricing options
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingOptions {
    /// Fare type
    pub fare_type: Option<Vec<String>>,
    /// Include checked bags
    pub included_checked_bags_only: Option<bool>,
}

/// Traveler pricing
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelerPricing {
    /// Traveler ID
    pub traveler_id: String,
    /// Fare option
    pub fare_option: Option<String>,
    /// Traveler type
    pub traveler_type: String,
    /// Price
    pub price: TravelerPrice,
    /// Fare details by segment
    pub fare_details_by_segment: Option<Vec<FareDetail>>,
}

/// Traveler price
#[derive(Debug, Deserialize)]
pub struct TravelerPrice {
    /// Currency
    pub currency: String,
    /// Total
    pub total: String,
    /// Base
    pub base: Option<String>,
}

/// Fare detail
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FareDetail {
    /// Segment ID
    pub segment_id: String,
    /// Cabin
    pub cabin: Option<String>,
    /// Fare basis
    pub fare_basis: Option<String>,
    /// Booking class
    pub class: Option<String>,
    /// Included checked bags
    pub included_checked_bags: Option<IncludedBags>,
}

/// Included bags
#[derive(Debug, Deserialize)]
pub struct IncludedBags {
    /// Weight
    pub weight: Option<u32>,
    /// Weight unit
    #[serde(rename = "weightUnit")]
    pub weight_unit: Option<String>,
    /// Quantity
    pub quantity: Option<u32>,
}

/// Booking request
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightOrderRequest {
    /// Type
    #[serde(rename = "type")]
    pub request_type: String,
    /// Flight offers
    pub flight_offers: Vec<serde_json::Value>,
    /// Travelers
    pub travelers: Vec<TravelerRequest>,
    /// Remarks
    pub remarks: Option<RemarksRequest>,
    /// Contact
    pub contacts: Vec<ContactRequest>,
}

/// Traveler request
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelerRequest {
    /// Traveler ID
    pub id: String,
    /// Date of birth
    pub date_of_birth: String,
    /// Gender
    pub gender: String,
    /// Name
    pub name: TravelerName,
    /// Documents
    pub documents: Option<Vec<TravelerDocument>>,
    /// Contact
    pub contact: Option<TravelerContact>,
}

/// Traveler name
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelerName {
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
}

/// Traveler document
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelerDocument {
    /// Document type
    pub document_type: String,
    /// Birth place
    pub birth_place: Option<String>,
    /// Issuance location
    pub issuance_location: Option<String>,
    /// Issuance date
    pub issuance_date: Option<String>,
    /// Number
    pub number: String,
    /// Expiry date
    pub expiry_date: String,
    /// Issuance country
    pub issuance_country: String,
    /// Validity country
    pub validity_country: Option<String>,
    /// Nationality
    pub nationality: String,
    /// Holder
    pub holder: bool,
}

/// Traveler contact
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TravelerContact {
    /// Email address
    pub email_address: Option<String>,
    /// Phones
    pub phones: Option<Vec<Phone>>,
}

/// Phone
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Phone {
    /// Device type
    pub device_type: String,
    /// Country calling code
    pub country_calling_code: String,
    /// Number
    pub number: String,
}

/// Remarks request
#[derive(Debug, serde::Serialize)]
pub struct RemarksRequest {
    /// General remarks
    pub general: Option<Vec<GeneralRemark>>,
}

/// General remark
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralRemark {
    /// Subtype
    pub subtype: String,
    /// Text
    pub text: String,
}

/// Contact request
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactRequest {
    /// Address
    pub address_eename: Option<AddressName>,
    /// Purpose
    pub purpose: String,
    /// Phones
    pub phones: Vec<Phone>,
    /// Email address
    pub email_address: String,
}

/// Address name
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressName {
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
}

/// Flight order response
#[derive(Debug, Deserialize)]
pub struct FlightOrderResponse {
    /// Response data
    pub data: FlightOrderData,
}

/// Flight order data
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightOrderData {
    /// Type
    #[serde(rename = "type")]
    pub order_type: String,
    /// Order ID
    pub id: String,
    /// Associated records (PNRs)
    pub associated_records: Option<Vec<AssociatedRecord>>,
    /// Flight offers
    pub flight_offers: Vec<serde_json::Value>,
    /// Travelers
    pub travelers: Option<Vec<serde_json::Value>>,
    /// Ticketing agreement
    pub ticketing_agreement: Option<TicketingAgreement>,
}

/// Associated record (PNR)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociatedRecord {
    /// Reference
    pub reference: String,
    /// Creation date
    pub creation_date: Option<String>,
    /// Origin system code
    pub origin_system_code: Option<String>,
    /// Airline
    pub fligh_offer_id: Option<String>,
}

/// Ticketing agreement
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketingAgreement {
    /// Option
    pub option: Option<String>,
    /// Date time
    pub date_time: Option<String>,
}

/// Airport search response
#[derive(Debug, Deserialize)]
pub struct AirportSearchResponse {
    /// Data
    pub data: Vec<AirportData>,
}

/// Airport data
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirportData {
    /// Type
    #[serde(rename = "type")]
    pub data_type: String,
    /// Subtype
    pub subtype: Option<String>,
    /// Name
    pub name: String,
    /// IATA code
    pub iata_code: String,
    /// Address
    pub address: Option<AirportAddress>,
}

/// Airport address
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirportAddress {
    /// City name
    pub city_name: Option<String>,
    /// City code
    pub city_code: Option<String>,
    /// Country name
    pub country_name: Option<String>,
    /// Country code
    pub country_code: Option<String>,
}

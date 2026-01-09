//! VAYA UI Components
//!
//! Reusable components following the VAYA Design System.
//! Each component implements all 6 states: default, hover, active, focus, disabled, loading.

pub mod button;
pub mod loading;
pub mod input;
pub mod airport_picker;
pub mod flight_card;
pub mod oracle_verdict;
pub mod select_input;
pub mod checkbox;
pub mod phone_input;
pub mod price_breakdown;
pub mod countdown;

// Re-export commonly used components
pub use button::{Button, IconButton};
pub use loading::{LoadingSpinner, Skeleton, SkeletonText, PageLoading};
pub use input::{TextInput, AirportCodeInput, DateInput};
pub use airport_picker::{AirportPicker, SwapButton, RoutePicker};
pub use flight_card::{FlightCard, FlightList, FlightMini};
pub use oracle_verdict::{OracleVerdictCard, VerdictBanner, ConfidenceBar, PriceComparison};
pub use select_input::{SelectInput, SelectOption, TitleSelect, CountrySelect};
pub use checkbox::{Checkbox, CheckboxGroup, TermsCheckbox};
pub use phone_input::{PhoneInput, CountryCode};
pub use price_breakdown::{PriceBreakdown, PriceLineItem, PriceItemType, PriceSummary, PriceLockFee};
pub use countdown::{Countdown, CountdownMini, PriceLockCountdown};

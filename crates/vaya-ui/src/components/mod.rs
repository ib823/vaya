//! VAYA UI Components
//!
//! Reusable components following the VAYA Design System.
//! Each component implements all 6 states: default, hover, active, focus, disabled, loading.

pub mod airport_picker;
pub mod button;
pub mod checkbox;
pub mod countdown;
pub mod flight_card;
pub mod input;
pub mod loading;
pub mod oracle_verdict;
pub mod phone_input;
pub mod price_breakdown;
pub mod select_input;

// Re-export commonly used components
pub use airport_picker::{AirportPicker, RoutePicker, SwapButton};
pub use button::{Button, IconButton};
pub use checkbox::{Checkbox, CheckboxGroup, TermsCheckbox};
pub use countdown::{Countdown, CountdownMini, PriceLockCountdown};
pub use flight_card::{FlightCard, FlightList, FlightMini};
pub use input::{AirportCodeInput, DateInput, TextInput};
pub use loading::{LoadingSpinner, PageLoading, Skeleton, SkeletonText};
pub use oracle_verdict::{ConfidenceBar, OracleVerdictCard, PriceComparison, VerdictBanner};
pub use phone_input::{CountryCode, PhoneInput};
pub use price_breakdown::{
    PriceBreakdown, PriceItemType, PriceLineItem, PriceLockFee, PriceSummary,
};
pub use select_input::{CountrySelect, SelectInput, SelectOption, TitleSelect};

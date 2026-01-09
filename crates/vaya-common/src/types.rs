//! Core types for VAYA - Zero external database dependencies
//!
//! All types use rkyv for zero-copy serialization.
//! Storage is handled by VayaDB (LSM-tree + B+Tree) and VayaCache.

use rkyv::{Archive, Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

// ============================================================================
// PRIMITIVE TYPES
// ============================================================================

/// Airport/City Code (3-4 chars, null-padded)
/// Uses fixed-size array for zero-copy access
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(C)]
pub struct IataCode([u8; 4]);

impl IataCode {
    /// Create a new IATA code from a string
    pub fn new(code: &str) -> Self {
        let mut bytes = [0u8; 4];
        let code_upper = code.to_uppercase();
        let code_bytes = code_upper.as_bytes();
        let len = code_bytes.len().min(4);
        bytes[..len].copy_from_slice(&code_bytes[..len]);
        Self(bytes)
    }

    /// Get the code as a string slice
    pub fn as_str(&self) -> &str {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(4);
        // SAFETY: We only store valid UTF-8 uppercase ASCII
        unsafe { std::str::from_utf8_unchecked(&self.0[..len]) }
    }

    /// Check if this is a valid 3-letter IATA code
    pub fn is_valid(&self) -> bool {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(4);
        len == 3 && self.0[..3].iter().all(|&b| b.is_ascii_uppercase())
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    // Common airports
    /// Kuala Lumpur International Airport
    pub const KUL: Self = Self(*b"KUL\0");
    /// Singapore Changi Airport
    pub const SIN: Self = Self(*b"SIN\0");
    /// Bangkok Suvarnabhumi Airport
    pub const BKK: Self = Self(*b"BKK\0");
    /// Tokyo Narita International Airport
    pub const NRT: Self = Self(*b"NRT\0");
    /// Tokyo Haneda Airport
    pub const HND: Self = Self(*b"HND\0");
    /// Hong Kong International Airport
    pub const HKG: Self = Self(*b"HKG\0");
    /// Seoul Incheon International Airport
    pub const ICN: Self = Self(*b"ICN\0");
    /// Sydney Kingsford Smith Airport
    pub const SYD: Self = Self(*b"SYD\0");
    /// Melbourne Tullamarine Airport
    pub const MEL: Self = Self(*b"MEL\0");
    /// London Heathrow Airport
    pub const LHR: Self = Self(*b"LHR\0");
    /// Paris Charles de Gaulle Airport
    pub const CDG: Self = Self(*b"CDG\0");
    /// Dubai International Airport
    pub const DXB: Self = Self(*b"DXB\0");
    /// New York John F. Kennedy Airport
    pub const JFK: Self = Self(*b"JFK\0");
    /// Los Angeles International Airport
    pub const LAX: Self = Self(*b"LAX\0");
}

impl fmt::Debug for IataCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IataCode(\"{}\")", self.as_str())
    }
}

impl fmt::Display for IataCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Currency Code (3 chars, ISO 4217)
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(C)]
pub struct CurrencyCode([u8; 4]);

impl CurrencyCode {
    /// Creates a new currency code from a string.
    ///
    /// The code is automatically converted to uppercase.
    /// Only the first 3 characters are used.
    pub fn new(code: &str) -> Self {
        let mut bytes = [0u8; 4];
        let code_upper = code.to_uppercase();
        let code_bytes = code_upper.as_bytes();
        let len = code_bytes.len().min(3);
        bytes[..len].copy_from_slice(&code_bytes[..len]);
        Self(bytes)
    }

    /// Returns the currency code as a string slice.
    pub fn as_str(&self) -> &str {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(3);
        unsafe { std::str::from_utf8_unchecked(&self.0[..len]) }
    }

    /// Get decimal places for this currency
    pub fn decimals(&self) -> u8 {
        match self.as_str() {
            "JPY" | "KRW" | "VND" => 0,
            "BHD" | "KWD" | "OMR" => 3,
            _ => 2,
        }
    }

    // Common currencies
    /// Malaysian Ringgit
    pub const MYR: Self = Self(*b"MYR\0");
    /// US Dollar
    pub const USD: Self = Self(*b"USD\0");
    /// Singapore Dollar
    pub const SGD: Self = Self(*b"SGD\0");
    /// Thai Baht
    pub const THB: Self = Self(*b"THB\0");
    /// Indonesian Rupiah
    pub const IDR: Self = Self(*b"IDR\0");
    /// Philippine Peso
    pub const PHP: Self = Self(*b"PHP\0");
    /// Vietnamese Dong
    pub const VND: Self = Self(*b"VND\0");
    /// Japanese Yen
    pub const JPY: Self = Self(*b"JPY\0");
    /// Korean Won
    pub const KRW: Self = Self(*b"KRW\0");
    /// Chinese Yuan
    pub const CNY: Self = Self(*b"CNY\0");
    /// Hong Kong Dollar
    pub const HKD: Self = Self(*b"HKD\0");
    /// Taiwan Dollar
    pub const TWD: Self = Self(*b"TWD\0");
    /// Australian Dollar
    pub const AUD: Self = Self(*b"AUD\0");
    /// New Zealand Dollar
    pub const NZD: Self = Self(*b"NZD\0");
    /// Euro
    pub const EUR: Self = Self(*b"EUR\0");
    /// British Pound
    pub const GBP: Self = Self(*b"GBP\0");
}

impl fmt::Debug for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CurrencyCode(\"{}\")", self.as_str())
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Monetary amount in minor units (cents/sen)
/// Using i64 to handle all currencies including IDR, VND
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub struct MinorUnits(i64);

impl MinorUnits {
    /// Zero value constant for convenience.
    pub const ZERO: Self = Self(0);

    /// Creates a new MinorUnits from an i64 value.
    ///
    /// The value represents the smallest currency unit (e.g., cents, sen).
    pub fn new(amount: i64) -> Self {
        Self(amount)
    }

    /// Returns the raw i64 value.
    pub fn as_i64(&self) -> i64 {
        self.0
    }

    /// Convert to major units (e.g., dollars from cents)
    pub fn to_major(&self, decimals: u8) -> f64 {
        self.0 as f64 / 10f64.powi(decimals as i32)
    }

    /// Create from major units
    pub fn from_major(amount: f64, decimals: u8) -> Self {
        Self((amount * 10f64.powi(decimals as i32)).round() as i64)
    }

    /// Add two amounts
    pub fn add(&self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Subtract
    pub fn sub(&self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Multiply by a factor
    pub fn mul(&self, factor: i64) -> Self {
        Self(self.0.saturating_mul(factor))
    }
}

impl fmt::Debug for MinorUnits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MinorUnits({})", self.0)
    }
}

impl fmt::Display for MinorUnits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Price with currency - the standard money type
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq))]
#[repr(C)]
pub struct Price {
    /// Amount in minor units (cents/sen)
    pub amount: MinorUnits,
    /// Currency code (ISO 4217)
    pub currency: CurrencyCode,
}

impl Price {
    /// Creates a new Price with the given amount and currency.
    pub fn new(amount: MinorUnits, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    /// Create price in MYR (sen)
    pub fn myr(sen: i64) -> Self {
        Self {
            amount: MinorUnits::new(sen),
            currency: CurrencyCode::MYR,
        }
    }

    /// Create price in USD (cents)
    pub fn usd(cents: i64) -> Self {
        Self {
            amount: MinorUnits::new(cents),
            currency: CurrencyCode::USD,
        }
    }

    /// Get display amount
    pub fn display_amount(&self) -> f64 {
        self.amount.to_major(self.currency.decimals())
    }

    /// Format for display
    pub fn format(&self) -> String {
        let decimals = self.currency.decimals();
        let major = self.amount.to_major(decimals);
        format!(
            "{} {:.prec$}",
            self.currency.as_str(),
            major,
            prec = decimals as usize
        )
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.amount.as_i64() == 0
    }

    /// Add prices (must be same currency)
    pub fn add(&self, other: &Self) -> Option<Self> {
        if self.currency != other.currency {
            return None;
        }
        Some(Self {
            amount: self.amount.add(other.amount),
            currency: self.currency,
        })
    }
}

impl fmt::Debug for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Price({} {})", self.currency.as_str(), self.amount.0)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Unix timestamp (seconds since epoch)
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Unix epoch (1970-01-01 00:00:00 UTC).
    pub const EPOCH: Self = Self(0);

    /// Returns the current timestamp.
    pub fn now() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        )
    }

    /// Creates a timestamp from Unix seconds.
    pub fn from_unix(secs: i64) -> Self {
        Self(secs)
    }

    /// Returns the Unix timestamp value in seconds.
    pub fn as_unix(&self) -> i64 {
        self.0
    }

    /// Add seconds
    pub fn add_secs(&self, secs: i64) -> Self {
        Self(self.0.saturating_add(secs))
    }

    /// Add minutes
    pub fn add_mins(&self, mins: i64) -> Self {
        self.add_secs(mins * 60)
    }

    /// Add hours
    pub fn add_hours(&self, hours: i64) -> Self {
        self.add_secs(hours * 3600)
    }

    /// Add days
    pub fn add_days(&self, days: i64) -> Self {
        self.add_secs(days * 86400)
    }

    /// Check if in the past
    pub fn is_past(&self) -> bool {
        self.0 < Self::now().0
    }

    /// Check if in the future
    pub fn is_future(&self) -> bool {
        self.0 > Self::now().0
    }

    /// Duration until this timestamp (0 if past)
    pub fn until(&self) -> std::time::Duration {
        let now = Self::now().0;
        if self.0 > now {
            std::time::Duration::from_secs((self.0 - now) as u64)
        } else {
            std::time::Duration::ZERO
        }
    }

    /// Duration since this timestamp (0 if future)
    pub fn since(&self) -> std::time::Duration {
        let now = Self::now().0;
        if now > self.0 {
            std::time::Duration::from_secs((now - self.0) as u64)
        } else {
            std::time::Duration::ZERO
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Timestamp({})", self.0)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format as ISO 8601
        let secs = self.0;
        let days = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        // Simple days-since-epoch to date (ignoring leap seconds)
        let mut y = 1970;
        let mut remaining_days = days;
        loop {
            let days_in_year = if is_leap_year(y) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            y += 1;
        }

        let (m, d) = days_to_month_day(remaining_days as u32, is_leap_year(y));

        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            y, m, d, hours, minutes, seconds
        )
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_to_month_day(day_of_year: u32, leap: bool) -> (u32, u32) {
    let days_in_months: [u32; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut remaining = day_of_year;
    for (i, &days) in days_in_months.iter().enumerate() {
        if remaining < days {
            return ((i + 1) as u32, remaining + 1);
        }
        remaining -= days;
    }
    (12, 31) // Fallback
}

/// Date (year, month, day) - compact date without time
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub struct Date {
    /// Year (e.g., 2025)
    pub year: i16,
    /// Month (1-12)
    pub month: u8,
    /// Day of month (1-31)
    pub day: u8,
}

impl Date {
    /// Creates a new Date with the given year, month, and day.
    pub fn new(year: i16, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Get today's date
    pub fn today() -> Self {
        let ts = Timestamp::now();
        let secs = ts.as_unix();
        let days = secs / 86400;

        let mut y = 1970i16;
        let mut remaining_days = days;
        loop {
            let days_in_year = if is_leap_year(y as i64) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            y += 1;
        }

        let (m, d) = days_to_month_day(remaining_days as u32, is_leap_year(y as i64));

        Self {
            year: y,
            month: m as u8,
            day: d as u8,
        }
    }

    /// Check if valid date
    pub fn is_valid(&self) -> bool {
        if self.month < 1 || self.month > 12 || self.day < 1 {
            return false;
        }

        let days_in_month = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if is_leap_year(self.year as i64) {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };

        self.day <= days_in_month
    }

    /// Add days
    pub fn add_days(&self, days: i32) -> Self {
        let ts = self.to_timestamp();
        let new_ts = ts.add_days(days as i64);
        Self::from_timestamp(new_ts)
    }

    /// Convert to timestamp (midnight UTC)
    pub fn to_timestamp(&self) -> Timestamp {
        let mut days: i64 = 0;

        // Years since 1970
        for y in 1970..self.year as i64 {
            days += if is_leap_year(y) { 366 } else { 365 };
        }

        // Months
        let days_in_months: [i64; 12] = if is_leap_year(self.year as i64) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        for &month_days in days_in_months
            .iter()
            .take(self.month.saturating_sub(1) as usize)
        {
            days += month_days;
        }

        // Days
        days += (self.day.saturating_sub(1)) as i64;

        Timestamp::from_unix(days * 86400)
    }

    /// Create from timestamp
    pub fn from_timestamp(ts: Timestamp) -> Self {
        let secs = ts.as_unix();
        let days = secs / 86400;

        let mut y = 1970i16;
        let mut remaining_days = days;
        loop {
            let days_in_year = if is_leap_year(y as i64) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            y += 1;
        }

        let (m, d) = days_to_month_day(remaining_days as u32, is_leap_year(y as i64));

        Self {
            year: y,
            month: m as u8,
            day: d as u8,
        }
    }

    /// Days until this date
    pub fn days_from_now(&self) -> i32 {
        let today = Self::today();
        let today_ts = today.to_timestamp().as_unix();
        let self_ts = self.to_timestamp().as_unix();
        ((self_ts - today_ts) / 86400) as i32
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Date({:04}-{:02}-{:02})",
            self.year, self.month, self.day
        )
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Route (origin -> destination)
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(C)]
pub struct Route {
    /// Origin airport code
    pub origin: IataCode,
    /// Destination airport code
    pub destination: IataCode,
}

impl Route {
    /// Creates a new Route with the given origin and destination codes.
    pub fn new(origin: IataCode, destination: IataCode) -> Self {
        Self {
            origin,
            destination,
        }
    }

    /// Create from string codes
    pub fn from_codes(origin: &str, destination: &str) -> Self {
        Self {
            origin: IataCode::new(origin),
            destination: IataCode::new(destination),
        }
    }

    /// Check if valid (both codes are 3-letter IATA codes and different)
    pub fn is_valid(&self) -> bool {
        self.origin.is_valid() && self.destination.is_valid() && self.origin != self.destination
    }

    /// Get the reverse route
    pub fn reverse(&self) -> Self {
        Self {
            origin: self.destination,
            destination: self.origin,
        }
    }

    /// Format as "KUL-NRT"
    pub fn to_string_compact(&self) -> String {
        format!("{}-{}", self.origin.as_str(), self.destination.as_str())
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Route({} -> {})",
            self.origin.as_str(),
            self.destination.as_str()
        )
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.origin.as_str(),
            self.destination.as_str()
        )
    }
}

/// UUID v4 - stored as two u64s for alignment
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(C, align(16))]
pub struct Uuid {
    /// High 64 bits of the UUID
    pub high: u64,
    /// Low 64 bits of the UUID
    pub low: u64,
}

impl Uuid {
    /// The nil UUID (all zeros)
    pub const NIL: Self = Self { high: 0, low: 0 };

    /// Generate a new UUID v4 using ring's SystemRandom
    pub fn new_v4() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let mut bytes = [0u8; 16];
        ring::rand::SecureRandom::fill(&rng, &mut bytes).expect("RNG failure");

        // Set version 4 and variant 1
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        // SAFETY: bytes is [u8; 16], so these slices are always exactly 8 bytes
        Self {
            high: u64::from_be_bytes(
                bytes[0..8]
                    .try_into()
                    .expect("slice from 16-byte array is always 8 bytes"),
            ),
            low: u64::from_be_bytes(
                bytes[8..16]
                    .try_into()
                    .expect("slice from 16-byte array is always 8 bytes"),
            ),
        }
    }

    /// Check if this is the nil UUID
    pub fn is_nil(&self) -> bool {
        self.high == 0 && self.low == 0
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.high.to_be_bytes());
        bytes[8..16].copy_from_slice(&self.low.to_be_bytes());
        bytes
    }

    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        // SAFETY: bytes is [u8; 16], so these slices are always exactly 8 bytes
        Self {
            high: u64::from_be_bytes(
                bytes[0..8]
                    .try_into()
                    .expect("slice from 16-byte array is always 8 bytes"),
            ),
            low: u64::from_be_bytes(
                bytes[8..16]
                    .try_into()
                    .expect("slice from 16-byte array is always 8 bytes"),
            ),
        }
    }

    /// Parse from string (with or without hyphens)
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.replace('-', "");
        if s.len() != 32 {
            return None;
        }

        let mut bytes = [0u8; 16];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hex_str = std::str::from_utf8(chunk).ok()?;
            bytes[i] = u8::from_str_radix(hex_str, 16).ok()?;
        }

        Some(Self::from_bytes(bytes))
    }

    /// Format as hyphenated string
    pub fn to_string_hyphenated(&self) -> String {
        let bytes = self.as_bytes();
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        )
    }
}

impl fmt::Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uuid({})", self.to_string_hyphenated())
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_hyphenated())
    }
}

/// Fixed-size string for efficient storage
/// Uses inline storage up to N bytes
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FixedString<const N: usize> {
    data: [u8; N],
    len: u8,
}

impl<const N: usize> FixedString<N> {
    /// Creates an empty FixedString.
    pub const fn empty() -> Self {
        Self {
            data: [0u8; N],
            len: 0,
        }
    }

    /// Creates a new FixedString from a string slice.
    ///
    /// If the string exceeds N bytes, it is truncated.
    pub fn new(s: &str) -> Self {
        let mut data = [0u8; N];
        let len = s.len().min(N);
        data[..len].copy_from_slice(&s.as_bytes()[..len]);
        Self {
            data,
            len: len as u8,
        }
    }

    /// Returns the string content as a string slice.
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.data[..self.len as usize]) }
    }

    /// Returns the length of the string in bytes.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns true if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the maximum capacity in bytes.
    pub fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> Default for FixedString<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> fmt::Debug for FixedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixedString({:?})", self.as_str())
    }
}

impl<const N: usize> fmt::Display for FixedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Type aliases for common fixed string sizes
/// Fixed string with 16-byte capacity
pub type String16 = FixedString<16>;
/// Fixed string with 32-byte capacity
pub type String32 = FixedString<32>;
/// Fixed string with 64-byte capacity
pub type String64 = FixedString<64>;
/// Fixed string with 128-byte capacity
pub type String128 = FixedString<128>;
/// Fixed string with 256-byte capacity
pub type String256 = FixedString<256>;

/// Airline code (2-letter IATA)
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(C)]
pub struct AirlineCode([u8; 2]);

impl AirlineCode {
    /// Creates a new airline code from a string.
    ///
    /// The code is automatically converted to uppercase.
    /// Only the first 2 characters are used.
    pub fn new(code: &str) -> Self {
        let mut bytes = [b' '; 2];
        let code_upper = code.to_uppercase();
        let code_bytes = code_upper.as_bytes();
        let len = code_bytes.len().min(2);
        bytes[..len].copy_from_slice(&code_bytes[..len]);
        Self(bytes)
    }

    /// Returns the airline code as a string slice.
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }

    // Common airlines
    /// Malaysia Airlines
    pub const MH: Self = Self(*b"MH");
    /// AirAsia
    pub const AK: Self = Self(*b"AK");
    /// Singapore Airlines
    pub const SQ: Self = Self(*b"SQ");
    /// Thai Airways
    pub const TG: Self = Self(*b"TG");
    /// Cathay Pacific
    pub const CX: Self = Self(*b"CX");
    /// All Nippon Airways (ANA)
    pub const NH: Self = Self(*b"NH");
    /// Japan Airlines
    pub const JL: Self = Self(*b"JL");
    /// Korean Air
    pub const KE: Self = Self(*b"KE");
    /// Asiana Airlines
    pub const OZ: Self = Self(*b"OZ");
    /// Emirates
    pub const EK: Self = Self(*b"EK");
    /// Qatar Airways
    pub const QR: Self = Self(*b"QR");
    /// Scoot
    pub const TR: Self = Self(*b"TR");
    /// Thai AirAsia
    pub const FD: Self = Self(*b"FD");
    /// VietJet Air
    pub const VJ: Self = Self(*b"VJ");
}

impl fmt::Debug for AirlineCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AirlineCode(\"{}\")", self.as_str())
    }
}

impl fmt::Display for AirlineCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iata_code() {
        let kul = IataCode::new("kul");
        assert_eq!(kul.as_str(), "KUL");
        assert!(kul.is_valid());

        let invalid = IataCode::new("AB");
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_currency() {
        let myr = CurrencyCode::MYR;
        assert_eq!(myr.as_str(), "MYR");
        assert_eq!(myr.decimals(), 2);

        let jpy = CurrencyCode::JPY;
        assert_eq!(jpy.decimals(), 0);
    }

    #[test]
    fn test_price() {
        let price = Price::myr(15000); // RM 150.00
        assert_eq!(price.display_amount(), 150.0);
        assert_eq!(price.format(), "MYR 150.00");
    }

    #[test]
    fn test_uuid() {
        let id = Uuid::new_v4();
        assert!(!id.is_nil());

        let s = id.to_string_hyphenated();
        let parsed = Uuid::parse(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_date() {
        let date = Date::new(2026, 1, 8);
        assert!(date.is_valid());
        assert_eq!(date.to_string(), "2026-01-08");

        let tomorrow = date.add_days(1);
        assert_eq!(tomorrow.day, 9);
    }

    #[test]
    fn test_route() {
        let route = Route::from_codes("KUL", "NRT");
        assert!(route.is_valid());
        assert_eq!(route.to_string_compact(), "KUL-NRT");

        let reverse = route.reverse();
        assert_eq!(reverse.origin.as_str(), "NRT");
    }
}

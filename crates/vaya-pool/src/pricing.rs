//! Tiered pricing for group buying pools

use vaya_common::{CurrencyCode, MinorUnits};

use crate::{PoolError, PoolResult};

/// A pricing tier based on group size
#[derive(Debug, Clone)]
pub struct PricingTier {
    /// Tier name/identifier
    pub name: String,
    /// Minimum members required for this tier
    pub min_members: u32,
    /// Maximum members for this tier (exclusive, None = unlimited)
    pub max_members: Option<u32>,
    /// Price per person in this tier
    pub price_per_person: MinorUnits,
    /// Discount percentage from base price
    pub discount_percent: u8,
}

impl PricingTier {
    /// Create a new pricing tier
    pub fn new(
        name: impl Into<String>,
        min_members: u32,
        max_members: Option<u32>,
        price_per_person: MinorUnits,
        discount_percent: u8,
    ) -> Self {
        Self {
            name: name.into(),
            min_members,
            max_members,
            price_per_person,
            discount_percent,
        }
    }

    /// Check if a member count qualifies for this tier
    pub fn matches(&self, member_count: u32) -> bool {
        if member_count < self.min_members {
            return false;
        }
        match self.max_members {
            Some(max) => member_count < max,
            None => true,
        }
    }
}

/// Tiered pricing structure for a pool
#[derive(Debug, Clone)]
pub struct TieredPricing {
    /// Base price (single person, no discount)
    pub base_price: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Pricing tiers (sorted by min_members ascending)
    pub tiers: Vec<PricingTier>,
    /// Maximum pool size
    pub max_pool_size: u32,
}

impl TieredPricing {
    /// Create a new tiered pricing structure
    pub fn new(base_price: MinorUnits, currency: CurrencyCode) -> Self {
        Self {
            base_price,
            currency,
            tiers: Vec::new(),
            max_pool_size: 50, // Default max
        }
    }

    /// Add a pricing tier
    pub fn add_tier(&mut self, tier: PricingTier) -> PoolResult<()> {
        // Validate tier
        if tier.min_members == 0 {
            return Err(PoolError::InvalidConfig(
                "Tier min_members must be > 0".into(),
            ));
        }

        if tier.discount_percent > 100 {
            return Err(PoolError::InvalidConfig(
                "Discount cannot exceed 100%".into(),
            ));
        }

        // Check for overlapping tiers
        for existing in &self.tiers {
            let overlaps = match (existing.max_members, tier.max_members) {
                (Some(ex_max), Some(t_max)) => {
                    !(tier.min_members >= ex_max || t_max <= existing.min_members)
                }
                (Some(ex_max), None) => tier.min_members < ex_max,
                (None, Some(t_max)) => t_max > existing.min_members,
                (None, None) => true, // Both unlimited, overlap
            };

            if overlaps {
                return Err(PoolError::InvalidConfig(format!(
                    "Tier '{}' overlaps with existing tier '{}'",
                    tier.name, existing.name
                )));
            }
        }

        self.tiers.push(tier);
        self.tiers.sort_by_key(|t| t.min_members);
        Ok(())
    }

    /// Get the applicable tier for a member count
    pub fn get_tier(&self, member_count: u32) -> Option<&PricingTier> {
        // Find the highest tier that matches
        self.tiers
            .iter()
            .rev()
            .find(|tier| tier.matches(member_count))
    }

    /// Get price per person for a member count
    pub fn get_price_per_person(&self, member_count: u32) -> MinorUnits {
        match self.get_tier(member_count) {
            Some(tier) => tier.price_per_person,
            None => self.base_price,
        }
    }

    /// Get discount percentage for a member count
    pub fn get_discount_percent(&self, member_count: u32) -> u8 {
        match self.get_tier(member_count) {
            Some(tier) => tier.discount_percent,
            None => 0,
        }
    }

    /// Calculate total price for a pool size
    pub fn calculate_total(&self, member_count: u32) -> MinorUnits {
        let price_per_person = self.get_price_per_person(member_count);
        MinorUnits::new(price_per_person.as_i64() * member_count as i64)
    }

    /// Get savings compared to individual bookings
    pub fn calculate_savings(&self, member_count: u32) -> MinorUnits {
        let individual_total = self.base_price.as_i64() * member_count as i64;
        let pool_total = self.calculate_total(member_count).as_i64();
        MinorUnits::new(individual_total - pool_total)
    }

    /// Get next tier (if any) and members needed
    pub fn get_next_tier(&self, current_members: u32) -> Option<(&PricingTier, u32)> {
        for tier in &self.tiers {
            if tier.min_members > current_members {
                return Some((tier, tier.min_members - current_members));
            }
        }
        None
    }

    /// Validate the pricing structure
    pub fn validate(&self) -> PoolResult<()> {
        if self.base_price.as_i64() <= 0 {
            return Err(PoolError::InvalidConfig(
                "Base price must be positive".into(),
            ));
        }

        if self.max_pool_size == 0 {
            return Err(PoolError::InvalidConfig("Max pool size must be > 0".into()));
        }

        // Check that discounts increase with tier level
        let mut last_discount = 0u8;
        for tier in &self.tiers {
            if tier.discount_percent < last_discount {
                return Err(PoolError::InvalidConfig(
                    "Discounts should increase with larger pool sizes".into(),
                ));
            }
            last_discount = tier.discount_percent;

            // Verify price matches discount
            let expected_price = self.base_price.as_i64()
                - (self.base_price.as_i64() * tier.discount_percent as i64 / 100);
            let actual_price = tier.price_per_person.as_i64();

            // Allow small rounding differences
            if (expected_price - actual_price).abs() > 1 {
                return Err(PoolError::InvalidConfig(format!(
                    "Tier '{}' price {} doesn't match {}% discount from base {}",
                    tier.name,
                    actual_price,
                    tier.discount_percent,
                    self.base_price.as_i64()
                )));
            }
        }

        Ok(())
    }

    /// Create standard tiers from base price
    pub fn with_standard_tiers(base_price: MinorUnits, currency: CurrencyCode) -> PoolResult<Self> {
        let mut pricing = Self::new(base_price, currency);
        let base = base_price.as_i64();

        // Tier 1: 5+ members = 5% off
        pricing.add_tier(PricingTier::new(
            "Silver",
            5,
            Some(10),
            MinorUnits::new(base - base * 5 / 100),
            5,
        ))?;

        // Tier 2: 10+ members = 10% off
        pricing.add_tier(PricingTier::new(
            "Gold",
            10,
            Some(20),
            MinorUnits::new(base - base * 10 / 100),
            10,
        ))?;

        // Tier 3: 20+ members = 15% off
        pricing.add_tier(PricingTier::new(
            "Platinum",
            20,
            Some(50),
            MinorUnits::new(base - base * 15 / 100),
            15,
        ))?;

        // Tier 4: 50+ members = 20% off
        pricing.add_tier(PricingTier::new(
            "Diamond",
            50,
            None,
            MinorUnits::new(base - base * 20 / 100),
            20,
        ))?;

        pricing.max_pool_size = 100;
        pricing.validate()?;

        Ok(pricing)
    }
}

/// Price lock for a pool (snapshot of price at join time)
#[derive(Debug, Clone)]
pub struct PriceLock {
    /// Locked price per person
    pub price_per_person: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Tier name at lock time
    pub tier_name: Option<String>,
    /// Member count at lock time
    pub member_count: u32,
    /// Lock timestamp
    pub locked_at: i64,
    /// Expiry timestamp
    pub expires_at: i64,
}

impl PriceLock {
    /// Create a new price lock
    pub fn new(
        price_per_person: MinorUnits,
        currency: CurrencyCode,
        tier_name: Option<String>,
        member_count: u32,
        ttl_seconds: i64,
    ) -> Self {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        Self {
            price_per_person,
            currency,
            tier_name,
            member_count,
            locked_at: now,
            expires_at: now + ttl_seconds,
        }
    }

    /// Check if price lock is still valid
    pub fn is_valid(&self) -> bool {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        now < self.expires_at
    }

    /// Get time remaining until expiry
    pub fn time_remaining(&self) -> i64 {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        (self.expires_at - now).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard_pricing() -> TieredPricing {
        TieredPricing::with_standard_tiers(
            MinorUnits::new(10000), // $100 base price
            CurrencyCode::SGD,
        )
        .unwrap()
    }

    #[test]
    fn test_tier_matching() {
        let pricing = standard_pricing();

        // 1 member = base price
        assert_eq!(pricing.get_price_per_person(1).as_i64(), 10000);
        assert_eq!(pricing.get_discount_percent(1), 0);

        // 5 members = Silver (5% off)
        assert_eq!(pricing.get_price_per_person(5).as_i64(), 9500);
        assert_eq!(pricing.get_discount_percent(5), 5);

        // 10 members = Gold (10% off)
        assert_eq!(pricing.get_price_per_person(10).as_i64(), 9000);
        assert_eq!(pricing.get_discount_percent(10), 10);

        // 25 members = Platinum (15% off)
        assert_eq!(pricing.get_price_per_person(25).as_i64(), 8500);
        assert_eq!(pricing.get_discount_percent(25), 15);

        // 50 members = Diamond (20% off)
        assert_eq!(pricing.get_price_per_person(50).as_i64(), 8000);
        assert_eq!(pricing.get_discount_percent(50), 20);
    }

    #[test]
    fn test_total_calculation() {
        let pricing = standard_pricing();

        // 1 member = $100
        assert_eq!(pricing.calculate_total(1).as_i64(), 10000);

        // 5 members @ $95 = $475
        assert_eq!(pricing.calculate_total(5).as_i64(), 47500);

        // 10 members @ $90 = $900
        assert_eq!(pricing.calculate_total(10).as_i64(), 90000);
    }

    #[test]
    fn test_savings_calculation() {
        let pricing = standard_pricing();

        // 1 member = no savings
        assert_eq!(pricing.calculate_savings(1).as_i64(), 0);

        // 5 members: $500 - $475 = $25 savings
        assert_eq!(pricing.calculate_savings(5).as_i64(), 2500);

        // 10 members: $1000 - $900 = $100 savings
        assert_eq!(pricing.calculate_savings(10).as_i64(), 10000);
    }

    #[test]
    fn test_next_tier() {
        let pricing = standard_pricing();

        // At 3 members, next tier is Silver at 5 (need 2 more)
        let (tier, needed) = pricing.get_next_tier(3).unwrap();
        assert_eq!(tier.name, "Silver");
        assert_eq!(needed, 2);

        // At 8 members, next tier is Gold at 10 (need 2 more)
        let (tier, needed) = pricing.get_next_tier(8).unwrap();
        assert_eq!(tier.name, "Gold");
        assert_eq!(needed, 2);

        // At 50+ members, no next tier (Diamond is the highest)
        assert!(pricing.get_next_tier(50).is_none());
    }

    #[test]
    fn test_tier_validation() {
        let mut pricing = TieredPricing::new(MinorUnits::new(10000), CurrencyCode::SGD);

        // Invalid: 0 min members
        let tier = PricingTier::new("Bad", 0, Some(5), MinorUnits::new(9500), 5);
        assert!(pricing.add_tier(tier).is_err());

        // Invalid: > 100% discount
        let tier = PricingTier::new("Bad", 5, Some(10), MinorUnits::new(9500), 150);
        assert!(pricing.add_tier(tier).is_err());
    }

    #[test]
    fn test_price_lock() {
        let lock = PriceLock::new(
            MinorUnits::new(9500),
            CurrencyCode::SGD,
            Some("Silver".into()),
            5,
            3600, // 1 hour
        );

        assert!(lock.is_valid());
        assert!(lock.time_remaining() > 0);
        assert!(lock.time_remaining() <= 3600);
    }

    #[test]
    fn test_overlapping_tiers() {
        let mut pricing = TieredPricing::new(MinorUnits::new(10000), CurrencyCode::SGD);

        // Add first tier
        let tier1 = PricingTier::new("First", 5, Some(10), MinorUnits::new(9500), 5);
        assert!(pricing.add_tier(tier1).is_ok());

        // Overlapping tier should fail
        let tier2 = PricingTier::new("Overlap", 8, Some(15), MinorUnits::new(9000), 10);
        assert!(pricing.add_tier(tier2).is_err());

        // Non-overlapping tier should succeed
        let tier3 = PricingTier::new("NonOverlap", 10, Some(20), MinorUnits::new(9000), 10);
        assert!(pricing.add_tier(tier3).is_ok());
    }
}

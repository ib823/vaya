//! Pool types and state machine

use ring::rand::{SecureRandom, SystemRandom};
use time::OffsetDateTime;
use vaya_common::{MinorUnits, IataCode};
use vaya_search::FlightOffer;

use crate::pricing::{PriceLock, TieredPricing};
use crate::{PoolError, PoolResult};

/// Pool status (state machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolStatus {
    /// Pool is forming, accepting members
    Forming,
    /// Minimum members reached, collecting contributions
    Active,
    /// Contributions complete, awaiting booking
    Locked,
    /// Booking complete, pool successful
    Completed,
    /// Pool expired (deadline passed)
    Expired,
    /// Pool cancelled by organizer
    Cancelled,
    /// Pool failed (booking failed, refunds pending)
    Failed,
}

impl PoolStatus {
    /// Get status as string
    pub fn as_str(&self) -> &'static str {
        match self {
            PoolStatus::Forming => "FORMING",
            PoolStatus::Active => "ACTIVE",
            PoolStatus::Locked => "LOCKED",
            PoolStatus::Completed => "COMPLETED",
            PoolStatus::Expired => "EXPIRED",
            PoolStatus::Cancelled => "CANCELLED",
            PoolStatus::Failed => "FAILED",
        }
    }

    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            PoolStatus::Completed | PoolStatus::Expired | PoolStatus::Cancelled | PoolStatus::Failed
        )
    }

    /// Check if pool is joinable
    pub fn is_joinable(&self) -> bool {
        matches!(self, PoolStatus::Forming | PoolStatus::Active)
    }

    /// Check if pool can receive contributions
    pub fn can_contribute(&self) -> bool {
        matches!(self, PoolStatus::Active)
    }

    /// Validate state transition
    pub fn can_transition_to(&self, target: PoolStatus) -> bool {
        match (self, target) {
            // From Forming
            (PoolStatus::Forming, PoolStatus::Active) => true,
            (PoolStatus::Forming, PoolStatus::Expired) => true,
            (PoolStatus::Forming, PoolStatus::Cancelled) => true,

            // From Active
            (PoolStatus::Active, PoolStatus::Locked) => true,
            (PoolStatus::Active, PoolStatus::Expired) => true,
            (PoolStatus::Active, PoolStatus::Cancelled) => true,
            (PoolStatus::Active, PoolStatus::Forming) => true, // Member left, below minimum

            // From Locked
            (PoolStatus::Locked, PoolStatus::Completed) => true,
            (PoolStatus::Locked, PoolStatus::Failed) => true,

            // All other transitions invalid
            _ => false,
        }
    }
}

/// Pool member
#[derive(Debug, Clone)]
pub struct PoolMember {
    /// User ID
    pub user_id: String,
    /// Number of seats/spots claimed
    pub spots: u32,
    /// Join timestamp
    pub joined_at: i64,
    /// Contribution amount (if paid)
    pub contribution: Option<MinorUnits>,
    /// Contribution timestamp
    pub contributed_at: Option<i64>,
    /// Price lock at join time
    pub price_lock: Option<PriceLock>,
    /// Is pool organizer
    pub is_organizer: bool,
}

impl PoolMember {
    /// Create a new pool member
    pub fn new(user_id: impl Into<String>, spots: u32) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            user_id: user_id.into(),
            spots,
            joined_at: now,
            contribution: None,
            contributed_at: None,
            price_lock: None,
            is_organizer: false,
        }
    }

    /// Create pool organizer
    pub fn organizer(user_id: impl Into<String>, spots: u32) -> Self {
        let mut member = Self::new(user_id, spots);
        member.is_organizer = true;
        member
    }

    /// Check if member has contributed
    pub fn has_contributed(&self) -> bool {
        self.contribution.is_some()
    }

    /// Record contribution
    pub fn record_contribution(&mut self, amount: MinorUnits) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.contribution = Some(amount);
        self.contributed_at = Some(now);
    }
}

/// Pool route
#[derive(Debug, Clone)]
pub struct PoolRoute {
    /// Origin airport
    pub origin: IataCode,
    /// Destination airport
    pub destination: IataCode,
    /// Departure date
    pub departure_date: time::Date,
    /// Return date (if round-trip)
    pub return_date: Option<time::Date>,
}

impl PoolRoute {
    /// Create a one-way route
    pub fn one_way(origin: IataCode, destination: IataCode, departure: time::Date) -> Self {
        Self {
            origin,
            destination,
            departure_date: departure,
            return_date: None,
        }
    }

    /// Create a round-trip route
    pub fn round_trip(
        origin: IataCode,
        destination: IataCode,
        departure: time::Date,
        return_date: time::Date,
    ) -> Self {
        Self {
            origin,
            destination,
            departure_date: departure,
            return_date: Some(return_date),
        }
    }
}

/// A buying pool
#[derive(Debug, Clone)]
pub struct Pool {
    /// Unique pool ID
    pub id: String,
    /// Pool name/title
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Current status
    pub status: PoolStatus,
    /// Route
    pub route: PoolRoute,
    /// Tiered pricing
    pub pricing: TieredPricing,
    /// Minimum members required
    pub min_members: u32,
    /// Maximum members allowed
    pub max_members: u32,
    /// Current members
    pub members: Vec<PoolMember>,
    /// Flight offer (if locked to specific offer)
    pub offer: Option<FlightOffer>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Join deadline (Unix timestamp)
    pub join_deadline: i64,
    /// Contribution deadline (Unix timestamp)
    pub contribution_deadline: i64,
    /// Booking reference (after completion)
    pub booking_ref: Option<String>,
    /// Status history
    pub history: Vec<StatusChange>,
    /// Version for optimistic locking
    pub version: u32,
}

impl Pool {
    /// Create a new pool
    pub fn new(
        name: impl Into<String>,
        route: PoolRoute,
        pricing: TieredPricing,
        organizer_id: impl Into<String>,
        organizer_spots: u32,
    ) -> PoolResult<Self> {
        let id = generate_pool_id()?;
        let now = OffsetDateTime::now_utc().unix_timestamp();

        // Default deadlines: 7 days to join, 3 days to contribute after
        let join_deadline = now + (7 * 24 * 3600);
        let contribution_deadline = now + (10 * 24 * 3600);

        let organizer = PoolMember::organizer(organizer_id, organizer_spots);

        let mut pool = Self {
            id: id.clone(),
            name: name.into(),
            description: None,
            status: PoolStatus::Forming,
            route,
            pricing,
            min_members: 5, // Default minimum
            max_members: 50,
            members: vec![organizer],
            offer: None,
            created_at: now,
            updated_at: now,
            join_deadline,
            contribution_deadline,
            booking_ref: None,
            history: Vec::new(),
            version: 1,
        };

        // Record initial state
        pool.history.push(StatusChange {
            from: None,
            to: PoolStatus::Forming,
            timestamp: now,
            reason: "Pool created".into(),
            actor: "SYSTEM".into(),
        });

        Ok(pool)
    }

    /// Get total spots claimed
    pub fn total_spots(&self) -> u32 {
        self.members.iter().map(|m| m.spots).sum()
    }

    /// Get member count
    pub fn member_count(&self) -> u32 {
        self.members.len() as u32
    }

    /// Check if pool is full
    pub fn is_full(&self) -> bool {
        self.total_spots() >= self.max_members
    }

    /// Check if minimum reached
    pub fn min_reached(&self) -> bool {
        self.total_spots() >= self.min_members
    }

    /// Get current price per person
    pub fn current_price_per_person(&self) -> MinorUnits {
        self.pricing.get_price_per_person(self.total_spots())
    }

    /// Get current discount percentage
    pub fn current_discount(&self) -> u8 {
        self.pricing.get_discount_percent(self.total_spots())
    }

    /// Get potential savings
    pub fn potential_savings(&self) -> MinorUnits {
        self.pricing.calculate_savings(self.total_spots())
    }

    /// Join pool
    pub fn join(&mut self, user_id: &str, spots: u32) -> PoolResult<()> {
        // Check status
        if !self.status.is_joinable() {
            return Err(PoolError::PoolNotJoinable(format!(
                "Pool is in {} status",
                self.status.as_str()
            )));
        }

        // Check deadline
        let now = OffsetDateTime::now_utc().unix_timestamp();
        if now > self.join_deadline {
            self.transition(PoolStatus::Expired, "Join deadline passed", "SYSTEM")?;
            return Err(PoolError::PoolExpired);
        }

        // Check if already a member
        if self.get_member(user_id).is_some() {
            return Err(PoolError::AlreadyMember);
        }

        // Check capacity
        if self.total_spots() + spots > self.max_members {
            return Err(PoolError::MemberLimitReached);
        }

        // Add member with price lock
        let mut member = PoolMember::new(user_id, spots);
        let new_total = self.total_spots() + spots;
        let tier = self.pricing.get_tier(new_total);

        member.price_lock = Some(PriceLock::new(
            self.pricing.get_price_per_person(new_total),
            self.pricing.currency,
            tier.map(|t| t.name.clone()),
            new_total,
            24 * 3600, // 24 hour lock
        ));

        self.members.push(member);
        self.updated_at = now;
        self.version += 1;

        // Check if minimum reached (transition to Active)
        if self.status == PoolStatus::Forming && self.min_reached() {
            self.transition(PoolStatus::Active, "Minimum members reached", "SYSTEM")?;
        }

        Ok(())
    }

    /// Leave pool
    pub fn leave(&mut self, user_id: &str) -> PoolResult<()> {
        // Check status
        if self.status == PoolStatus::Locked {
            return Err(PoolError::CannotLeave("Pool is locked".into()));
        }

        if self.status.is_terminal() {
            return Err(PoolError::CannotLeave("Pool is closed".into()));
        }

        // Find member
        let pos = self.members.iter().position(|m| m.user_id == user_id);
        let member = match pos {
            Some(i) => &self.members[i],
            None => return Err(PoolError::NotAMember),
        };

        // Cannot leave if already contributed
        if member.has_contributed() {
            return Err(PoolError::CannotLeave(
                "Already contributed, request refund instead".into(),
            ));
        }

        // Organizer cannot leave (must cancel pool)
        if member.is_organizer {
            return Err(PoolError::CannotLeave(
                "Organizer must cancel the pool".into(),
            ));
        }

        // Remove member
        self.members.remove(pos.unwrap());

        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.updated_at = now;
        self.version += 1;

        // Check if we dropped below minimum (revert to Forming)
        if self.status == PoolStatus::Active && !self.min_reached() {
            self.transition(PoolStatus::Forming, "Dropped below minimum", "SYSTEM")?;
        }

        Ok(())
    }

    /// Record contribution from member
    pub fn contribute(&mut self, user_id: &str, amount: MinorUnits) -> PoolResult<()> {
        // Check status
        if !self.status.can_contribute() {
            return Err(PoolError::InvalidContribution(format!(
                "Cannot contribute in {} status",
                self.status.as_str()
            )));
        }

        // Check deadline
        let now = OffsetDateTime::now_utc().unix_timestamp();
        if now > self.contribution_deadline {
            return Err(PoolError::ContributionDeadlinePassed);
        }

        // Get current price before borrowing member
        let current_price = self.current_price_per_person();

        // Find member and get required info
        let member_idx = self
            .members
            .iter()
            .position(|m| m.user_id == user_id)
            .ok_or(PoolError::NotAMember)?;

        let member = &self.members[member_idx];

        // Check if already contributed
        if member.has_contributed() {
            return Err(PoolError::ContributionAlreadyProcessed);
        }

        // Calculate required amount
        let price_per_person = member
            .price_lock
            .as_ref()
            .map(|lock| lock.price_per_person)
            .unwrap_or(current_price);

        let spots = member.spots;
        let required = price_per_person.as_i64() * spots as i64;

        if amount.as_i64() < required {
            return Err(PoolError::InsufficientContribution {
                required,
                provided: amount.as_i64(),
            });
        }

        // Record contribution (mutable borrow)
        self.members[member_idx].record_contribution(amount);
        self.updated_at = now;
        self.version += 1;

        // Check if all members have contributed
        if self.all_contributed() {
            self.transition(PoolStatus::Locked, "All contributions received", "SYSTEM")?;
        }

        Ok(())
    }

    /// Check if all members have contributed
    pub fn all_contributed(&self) -> bool {
        self.members.iter().all(|m| m.has_contributed())
    }

    /// Get total contributions
    pub fn total_contributions(&self) -> MinorUnits {
        let sum: i64 = self
            .members
            .iter()
            .filter_map(|m| m.contribution.as_ref())
            .map(|c| c.as_i64())
            .sum();
        MinorUnits::new(sum)
    }

    /// Get member by user ID
    pub fn get_member(&self, user_id: &str) -> Option<&PoolMember> {
        self.members.iter().find(|m| m.user_id == user_id)
    }

    /// Get mutable member by user ID
    pub fn get_member_mut(&mut self, user_id: &str) -> Option<&mut PoolMember> {
        self.members.iter_mut().find(|m| m.user_id == user_id)
    }

    /// Transition to a new status
    pub fn transition(&mut self, new_status: PoolStatus, reason: &str, actor: &str) -> PoolResult<()> {
        if !self.status.can_transition_to(new_status) {
            return Err(PoolError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: new_status.as_str().to_string(),
            });
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();

        self.history.push(StatusChange {
            from: Some(self.status),
            to: new_status,
            timestamp: now,
            reason: reason.to_string(),
            actor: actor.to_string(),
        });

        self.status = new_status;
        self.updated_at = now;
        self.version += 1;

        Ok(())
    }

    /// Cancel pool (organizer only)
    pub fn cancel(&mut self, user_id: &str, reason: &str) -> PoolResult<()> {
        // Verify organizer
        let member = self.get_member(user_id).ok_or(PoolError::NotAMember)?;
        if !member.is_organizer {
            return Err(PoolError::CannotLeave(
                "Only organizer can cancel the pool".into(),
            ));
        }

        // Cannot cancel locked/completed pools
        if self.status == PoolStatus::Locked || self.status == PoolStatus::Completed {
            return Err(PoolError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "CANCELLED".to_string(),
            });
        }

        self.transition(PoolStatus::Cancelled, reason, user_id)
    }

    /// Mark as completed
    pub fn complete(&mut self, booking_ref: &str, actor: &str) -> PoolResult<()> {
        if self.status != PoolStatus::Locked {
            return Err(PoolError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "COMPLETED".to_string(),
            });
        }

        self.booking_ref = Some(booking_ref.to_string());
        self.transition(PoolStatus::Completed, "Booking completed", actor)
    }

    /// Mark as failed
    pub fn fail(&mut self, reason: &str, actor: &str) -> PoolResult<()> {
        if self.status != PoolStatus::Locked {
            return Err(PoolError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "FAILED".to_string(),
            });
        }

        self.transition(PoolStatus::Failed, reason, actor)
    }

    /// Check if pool has expired
    pub fn check_expiry(&mut self) -> bool {
        if self.status.is_terminal() {
            return false;
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();

        // Check join deadline for Forming status
        if self.status == PoolStatus::Forming && now > self.join_deadline {
            let _ = self.transition(PoolStatus::Expired, "Join deadline passed", "SYSTEM");
            return true;
        }

        // Check contribution deadline for Active status
        if self.status == PoolStatus::Active && now > self.contribution_deadline {
            let _ = self.transition(PoolStatus::Expired, "Contribution deadline passed", "SYSTEM");
            return true;
        }

        false
    }

    /// Get time remaining until deadline
    pub fn time_to_deadline(&self) -> Option<i64> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let deadline = match self.status {
            PoolStatus::Forming => self.join_deadline,
            PoolStatus::Active => self.contribution_deadline,
            _ => return None,
        };
        Some((deadline - now).max(0))
    }
}

/// Status change record
#[derive(Debug, Clone)]
pub struct StatusChange {
    /// Previous status
    pub from: Option<PoolStatus>,
    /// New status
    pub to: PoolStatus,
    /// Timestamp
    pub timestamp: i64,
    /// Reason for change
    pub reason: String,
    /// Actor who made the change
    pub actor: String,
}

/// Generate a unique pool ID (8 alphanumeric characters)
fn generate_pool_id() -> PoolResult<String> {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 8];

    rng.fill(&mut bytes)
        .map_err(|_| PoolError::Internal("Failed to generate pool ID".into()))?;

    let id: String = bytes
        .iter()
        .map(|b| CHARS[(*b as usize) % CHARS.len()] as char)
        .collect();

    Ok(format!("POOL-{}", id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::TieredPricing;
    use vaya_common::CurrencyCode;

    fn test_route() -> PoolRoute {
        PoolRoute::one_way(
            IataCode::SIN,
            IataCode::BKK,
            time::Date::from_calendar_date(2025, time::Month::June, 15).unwrap(),
        )
    }

    fn test_pricing() -> TieredPricing {
        TieredPricing::with_standard_tiers(MinorUnits::new(10000), CurrencyCode::SGD).unwrap()
    }

    #[test]
    fn test_pool_creation() {
        let pool = Pool::new(
            "SIN-BKK Group Deal",
            test_route(),
            test_pricing(),
            "user-organizer",
            1,
        ).unwrap();

        assert!(pool.id.starts_with("POOL-"));
        assert_eq!(pool.status, PoolStatus::Forming);
        assert_eq!(pool.member_count(), 1);
        assert_eq!(pool.total_spots(), 1);
        assert!(pool.members[0].is_organizer);
    }

    #[test]
    fn test_join_pool() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        // Join as new member
        assert!(pool.join("user-2", 1).is_ok());
        assert_eq!(pool.member_count(), 2);
        assert_eq!(pool.total_spots(), 2);

        // Cannot join twice
        assert!(pool.join("user-2", 1).is_err());

        // Can join with multiple spots
        assert!(pool.join("user-3", 2).is_ok());
        assert_eq!(pool.total_spots(), 4);
    }

    #[test]
    fn test_minimum_members() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        pool.min_members = 3;

        assert_eq!(pool.status, PoolStatus::Forming);
        assert!(!pool.min_reached());

        // Add members
        pool.join("user-2", 1).unwrap();
        assert!(!pool.min_reached());
        assert_eq!(pool.status, PoolStatus::Forming);

        pool.join("user-3", 1).unwrap();
        assert!(pool.min_reached());
        assert_eq!(pool.status, PoolStatus::Active);
    }

    #[test]
    fn test_leave_pool() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        pool.join("user-2", 1).unwrap();

        // Regular member can leave
        assert!(pool.leave("user-2").is_ok());
        assert_eq!(pool.member_count(), 1);

        // Organizer cannot leave
        assert!(pool.leave("organizer").is_err());

        // Non-member cannot leave
        assert!(pool.leave("user-3").is_err());
    }

    #[test]
    fn test_contribution() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        pool.min_members = 1;
        pool.join("user-2", 1).unwrap();

        // Force to Active status
        pool.status = PoolStatus::Active;

        // Contribute
        let amount = MinorUnits::new(10000);
        assert!(pool.contribute("user-2", amount).is_ok());

        let member = pool.get_member("user-2").unwrap();
        assert!(member.has_contributed());
        assert_eq!(member.contribution.unwrap().as_i64(), 10000);
    }

    #[test]
    fn test_insufficient_contribution() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            2,
        ).unwrap();

        pool.min_members = 1;
        pool.status = PoolStatus::Active;

        // Try to contribute less than required (2 spots @ $100 = $200)
        let amount = MinorUnits::new(10000); // Only $100
        let result = pool.contribute("organizer", amount);
        assert!(matches!(result, Err(PoolError::InsufficientContribution { .. })));
    }

    #[test]
    fn test_all_contributed_locks_pool() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        pool.min_members = 1;
        pool.status = PoolStatus::Active;

        // Contribute
        pool.contribute("organizer", MinorUnits::new(10000)).unwrap();

        assert_eq!(pool.status, PoolStatus::Locked);
    }

    #[test]
    fn test_status_transitions() {
        assert!(PoolStatus::Forming.can_transition_to(PoolStatus::Active));
        assert!(PoolStatus::Forming.can_transition_to(PoolStatus::Expired));
        assert!(PoolStatus::Active.can_transition_to(PoolStatus::Locked));
        assert!(PoolStatus::Locked.can_transition_to(PoolStatus::Completed));
        assert!(PoolStatus::Locked.can_transition_to(PoolStatus::Failed));

        // Invalid transitions
        assert!(!PoolStatus::Forming.can_transition_to(PoolStatus::Completed));
        assert!(!PoolStatus::Completed.can_transition_to(PoolStatus::Active));
    }

    #[test]
    fn test_cancel_pool() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        // Non-organizer cannot cancel
        pool.join("user-2", 1).unwrap();
        assert!(pool.cancel("user-2", "Changed mind").is_err());

        // Organizer can cancel
        assert!(pool.cancel("organizer", "Changed mind").is_ok());
        assert_eq!(pool.status, PoolStatus::Cancelled);
    }

    #[test]
    fn test_complete_pool() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        pool.status = PoolStatus::Locked;

        pool.complete("BKG-123", "SYSTEM").unwrap();
        assert_eq!(pool.status, PoolStatus::Completed);
        assert_eq!(pool.booking_ref, Some("BKG-123".to_string()));
    }

    #[test]
    fn test_pool_id_generation() {
        let id = generate_pool_id().unwrap();
        assert!(id.starts_with("POOL-"));
        assert_eq!(id.len(), 13); // POOL- + 8 chars
    }

    #[test]
    fn test_price_lock_on_join() {
        let mut pool = Pool::new(
            "Test Pool",
            test_route(),
            test_pricing(),
            "organizer",
            1,
        ).unwrap();

        pool.join("user-2", 1).unwrap();

        let member = pool.get_member("user-2").unwrap();
        assert!(member.price_lock.is_some());

        let lock = member.price_lock.as_ref().unwrap();
        assert!(lock.is_valid());
    }
}

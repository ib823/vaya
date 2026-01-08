# ðŸ”´ SCHEMA DESIGN OVERRIDE

**CRITICAL: This document SUPERSEDES all SQL schemas in other documents.**

---

## âš ï¸ IGNORE THESE (DEPRECATED)

The following documents contain **OUTDATED** SQL/PostgreSQL references that are **NO LONGER VALID**:

| Document | Contains | Status |
|----------|----------|--------|
| `VAYA_Master_Specification.html` | PostgreSQL schemas, CREATE TABLE | âŒ **DEPRECATED** |
| `VAYA_Architecture_v1.html` | ScyllaDB, Redis, PostgreSQL | âŒ **DEPRECATED** |
| `001_initial_schema.sql` | SQL migration | âŒ **DEPRECATED** |

**These documents were created BEFORE the zero-dependency redesign.**

---

## âœ… USE THIS INSTEAD (CURRENT)

VAYA uses **100% custom storage** with:

### 1. Schema = Rust Structs

```rust
// Schema IS the code. No separate DDL. No SQL.
#[derive(Archive, Serialize, Deserialize)]
#[repr(C)]  // Fixed memory layout
pub struct User {
    pub id: Uuid,           // 16 bytes
    pub email: [u8; 256],   // Fixed size
    pub status: UserStatus, // 1 byte enum
    pub created_at: i64,    // 8 bytes timestamp
    // ... all fixed-size, cache-aligned
}
```

### 2. Storage = VayaDB (Custom LSM + B+Tree)

- **NO PostgreSQL**
- **NO ScyllaDB**  
- **NO Redis**
- **NO SQL language**

### 3. Serialization = rkyv (Zero-Copy)

```rust
// Zero-copy deserialization - data is read directly from disk
let archived = unsafe { rkyv::archived_root::<User>(&bytes) };
// No parsing, no allocation, instant access
```

### 4. Security Benefits

| SQL Approach | Our Approach |
|--------------|--------------|
| SQL injection possible | **No query language = No injection** |
| Network protocol exploits | **Embedded = No network attack surface** |
| Schema drift | **Compile-time validation** |
| ORM overhead | **Zero-copy direct access** |

---

## ðŸ“‹ CORRECT SCHEMA DEFINITIONS

All schemas are defined in Rust in `vaya-common/src/types.rs`:

### Core Types (Use These)

```rust
// From vaya-common/src/types.rs - THE CANONICAL SOURCE

pub struct IataCode([u8; 4]);      // Airport: KUL, SIN
pub struct CurrencyCode([u8; 4]); // Currency: MYR, USD
pub struct MinorUnits(i64);        // Money in cents/sen
pub struct Timestamp(i64);         // Unix epoch
pub struct Date { year: i16, month: u8, day: u8 }

pub struct Price {
    pub amount: MinorUnits,
    pub currency: CurrencyCode,
    pub decimals: u8,
}

pub struct Route {
    pub origin: IataCode,
    pub destination: IataCode,
}
```

### Domain Types

```rust
// User - stored in VayaStore (B+Tree)
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub status: UserStatus,
    pub tier: UserTier,
    pub created_at: Timestamp,
}

// PricePoint - stored in VayaDB (LSM-tree)
pub struct PricePoint {
    pub route: Route,
    pub departure_date: Date,
    pub price: Price,
    pub source: OfferSource,
    pub collected_at: Timestamp,
}

// Prediction - stored in VayaDB (LSM-tree)
pub struct PricePrediction {
    pub route: Route,
    pub departure_date: Date,
    pub predicted_price: Price,
    pub confidence: f32,
    pub predicted_at: Timestamp,
}

// Pool - stored in VayaStore (B+Tree)
pub struct DemandPool {
    pub id: Uuid,
    pub route: Route,
    pub departure_window: (Date, Date),
    pub status: PoolStatus,
    pub member_count: u32,
    pub target_price: Price,
}

// Booking - stored in VayaStore (B+Tree)
pub struct Booking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pool_id: Option<Uuid>,
    pub status: BookingStatus,
    pub total_price: Price,
    pub created_at: Timestamp,
}
```

---

## ðŸ”§ STORAGE ENGINE MAPPING

| Data Type | Storage Engine | Why |
|-----------|---------------|-----|
| Price observations | VayaDB LSM-tree | Time-series, write-heavy |
| Predictions | VayaDB LSM-tree | Time-series, temporal queries |
| Users | VayaStore B+Tree | Relational, read-heavy |
| Bookings | VayaStore B+Tree | Relational, ACID required |
| Pools | VayaStore B+Tree | Relational, complex queries |
| Sessions | VayaCache | Ephemeral, fast access |

---

## ðŸš« DO NOT

1. **DO NOT** use SQL syntax anywhere
2. **DO NOT** reference PostgreSQL, ScyllaDB, Redis
3. **DO NOT** create .sql migration files
4. **DO NOT** use ORM patterns
5. **DO NOT** trust the old HTML specs for database design

---

## âœ… DO

1. **DO** define schemas as Rust structs with `#[derive(Archive, Serialize, Deserialize)]`
2. **DO** use `vaya-common` types (IataCode, Price, etc.)
3. **DO** use VayaDB for time-series data
4. **DO** use VayaStore for relational data
5. **DO** use VayaCache for hot data

---

## ðŸ“š REFERENCE DOCUMENTS (CORRECT)

For database design, ONLY use these documents:

1. `SOVEREIGN_ARCHITECTURE.md` - Complete custom stack design
2. `ZERO_DEPENDENCY_ARCHITECTURE.md` - Philosophy and approach
3. `NUCLEAR_ARCHITECTURE_V3.md` - Architecture overview
4. `vaya-common/src/types.rs` - Canonical type definitions
5. `vaya-db/src/lib.rs` - Storage engine implementation
6. `vaya-store/src/lib.rs` - Relational layer implementation

---

**This document is the FINAL AUTHORITY on schema design.**

*Created: January 8, 2025*
*Reason: Resolve conflict between old SQL specs and new zero-dependency implementation*

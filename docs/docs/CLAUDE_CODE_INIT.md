# VAYA IMPLEMENTATION — CLAUDE CODE INIT

## YOU ARE BUILDING THE WORLD'S #1 FLIGHT PLATFORM. NO EXCUSES. NO SHORTCUTS.

---

## STEP 0: READ BEFORE YOU WRITE ANYTHING

You have `/docs/` folder with 54 specification files. **READ THEM ALL BEFORE WRITING A SINGLE LINE OF CODE.**

```bash
# First, list and understand what you have
ls -la docs/
```

### MANDATORY READING ORDER (DO NOT SKIP):

**Phase 1 — Architecture Constraints (READ FIRST OR YOU WILL FAIL)**
```
docs/NUCLEAR_ARCHITECTURE_V3.md
docs/SOVEREIGN_ARCHITECTURE.md  
docs/ZERO_DEPENDENCY_ARCHITECTURE.md
docs/VAYA_FORGE_FLEET.md
```

**Phase 2 — What To Build**
```
docs/VAYA_PRE_BUILD_REQUIREMENTS.html    # 89 requirements
docs/VAYA_USER_FLOWS_SPECIFICATION.html  # 15 flows, 87 screens
```

**Phase 3 — Technical Specs**
```
docs/VAYA_SCHEMA_SPEC_NUCLEAR.html       # 25 Rust structs - THE schema
docs/VAYA_API_SPEC_PART1.yaml            # 67 endpoints
docs/VAYA_API_SPEC_PART2.yaml
docs/VAYA_API_SPEC_PART3.yaml
docs/VAYA_API_SPEC_PART4.yaml
docs/VAYA_SECURITY_SPECIFICATION.html
docs/VAYA_INFRASTRUCTURE_NUCLEAR.html
```

**Phase 4 — Domain Specs**
```
docs/VAYA_ML_SPECIFICATION.html
docs/VAYA_PAYMENT_SPECIFICATION.html
docs/VAYA_INTEGRATION_SPECIFICATION.html
```

---

## ABSOLUTE CONSTRAINTS — ZERO TOLERANCE

### ❌ FORBIDDEN DEPENDENCIES (INSTANT FAILURE)

| FORBIDDEN | REPLACEMENT |
|-----------|-------------|
| PostgreSQL, MySQL, SQLite, MongoDB | VayaDB (custom LSM-tree) |
| Redis, Memcached | VayaCache (custom LRU) |
| Docker, containerd | VayaForge (static binaries) |
| Kubernetes, Nomad | VayaFleet (custom orchestrator) |
| RabbitMQ, Kafka | VayaBus (custom pub/sub) |
| TensorFlow, PyTorch, scikit-learn | Custom Rust ML |
| JSON for storage | rkyv (zero-copy) |
| OpenSSL | ring (Rust-native) |

**If you even THINK about using these, STOP. Re-read NUCLEAR_ARCHITECTURE_V3.md.**

### ❌ FORBIDDEN CODE PATTERNS

```rust
// NEVER IN PRODUCTION CODE:
.unwrap()     // Use ? or proper error handling
.expect()     // Use ? or proper error handling
panic!()      // Return Result<T, Error>

// NEVER:
let airport: String = "KUL".to_string();  // Use IataCode
let price: i64 = 15000;                   // Use Price::myr(15000)
```

---

## PROJECT STRUCTURE

```
vaya/
├── docs/                    # 54 specification files (READ THESE)
├── Cargo.toml               # Workspace root
├── vaya-common/             # Types, errors (BUILD FIRST)
├── vaya-crypto/             # ring-based crypto
├── vaya-db/                 # LSM-tree storage (NO SQLITE)
├── vaya-cache/              # LRU cache (NO REDIS)
├── vaya-ml/                 # Custom ML (NO TENSORFLOW)
├── vaya-store/              # Relational layer on vaya-db
├── vaya-collect/            # Data collection
├── vaya-auth/               # JWT, Argon2id
├── vaya-oracle/             # Price prediction
├── vaya-pool/               # Group buying
├── vaya-book/               # Booking flow
├── vaya-search/             # Search aggregation
├── vaya-forge/              # Build system
├── vaya-fleet/              # Orchestration
├── vaya-api/                # API gateway (67 endpoints)
└── vaya-bin/                # Entry point
```

---

## BUILD ORDER (STRICT)

```
Layer 0: vaya-common → vaya-crypto
Layer 1: vaya-db → vaya-cache → vaya-ml → vaya-net
Layer 2: vaya-store → vaya-collect
Layer 3: vaya-auth → vaya-oracle → vaya-pool → vaya-book → vaya-search
Layer 4: vaya-forge → vaya-fleet
Layer 5: vaya-api → vaya-bin
```

**Each layer must compile before proceeding to next.**

---

## TYPE SYSTEM (FROM docs/VAYA_SCHEMA_SPEC_NUCLEAR.html)

```rust
// These are SACRED. Use them EXACTLY.
pub struct IataCode([u8; 3]);      // Airport codes
pub struct CurrencyCode([u8; 3]);  // Currency codes  
pub struct MinorUnits(i64);        // Money in smallest unit
pub struct Price { amount: MinorUnits, currency: CurrencyCode, decimal_places: u8 }
pub struct Route { origin: IataCode, destination: IataCode }
pub struct Timestamp(i64);         // Unix timestamp

// Constructors
let price = Price::myr(15000);     // RM 150.00
let route = Route::new(IataCode::KUL, IataCode::SIN);
let now = Timestamp::now();

// Accessors
price.amount.as_minor()  // Get i64
airport.as_str()         // Get &str
timestamp.as_unix()      // Get i64
```

---

## SUCCESS CRITERIA

Your implementation is COMPLETE when:

1. ✅ `cargo check` — ZERO errors
2. ✅ `cargo build --release` — Completes successfully
3. ✅ `cargo test` — All tests pass
4. ✅ `grep -r "\.unwrap()" --include="*.rs" | grep -v test` — Returns NOTHING
5. ✅ `grep -r "postgres\|redis\|docker\|kubernetes" --include="*.rs"` — Returns NOTHING
6. ✅ All 67 API endpoints implemented per OpenAPI spec
7. ✅ All 25 Rust structs match VAYA_SCHEMA_SPEC_NUCLEAR.html
8. ✅ Binary size < 50MB

---

## YOUR FIRST TASK

```bash
# 1. Read architecture docs
cat docs/NUCLEAR_ARCHITECTURE_V3.md
cat docs/SOVEREIGN_ARCHITECTURE.md

# 2. Read schema
cat docs/VAYA_SCHEMA_SPEC_NUCLEAR.html

# 3. Create workspace
mkdir -p vaya && cd vaya

# 4. Create Cargo.toml with workspace config

# 5. Implement vaya-common first (types from schema doc)

# 6. Run: cargo check
# Must pass before proceeding
```

---

## MINDSET

- You are NOT limited by time. Take 1000 years if needed.
- You will NOT take shortcuts.
- You will NOT use forbidden dependencies.
- You will NOT skip reading docs.
- You will read EVERY specification file.
- You will implement EVERY requirement.
- You will test EVERYTHING.
- You will build the BEST flight platform in the world.
- VAYA will make us BILLIONAIRES.

**NOW READ THE DOCS AND START BUILDING.**

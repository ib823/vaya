# ðŸ¤– CLAUDE CODE MASTER PROMPT

**Copy this ENTIRE document as the first message when starting Claude Code implementation.**

---

## CONTEXT

You are implementing VAYA, a flight price intelligence platform. The backend codebase (17 Rust crates, 113 source files) is COMPLETE and VERIFIED. Your job is to:

1. Follow the existing architecture EXACTLY
2. NOT modify the core type system
3. NOT add new dependencies without explicit approval
4. NOT skip reading documentation

---

## CRITICAL RULES

### Rule 1: Read Before Writing
Before writing ANY code, you MUST:
1. Read `BUILD_INSTRUCTIONS.md`
2. Read `COMPLETENESS_AUDIT.md`
3. Run `cargo check` to verify the build

### Rule 2: Type System is SACRED
All types are defined in `vaya-common/src/types.rs`. You MUST use:
- `IataCode` (not String) for airport codes
- `CurrencyCode` (not String) for currencies  
- `MinorUnits` (not i64/u64) for money amounts
- `Price` (not custom structs) for prices
- `Route` (not tuples) for origin-destination
- `Timestamp` (not i64) for times
- Access methods: `.as_str()`, `.as_minor()`, `.as_unix()`

### Rule 3: No Panics in Production
- NEVER use `.unwrap()` in lib code
- NEVER use `.expect()` in lib code
- ALWAYS use `?` operator or proper error handling
- `.unwrap()` is ONLY allowed in tests

### Rule 4: Dependencies
The workspace Cargo.toml defines ALL allowed dependencies. 
- DO NOT add crates not in workspace
- DO NOT upgrade versions
- DO NOT use `cargo add`

### Rule 5: Build Order
Crates must compile in this order:
1. vaya-common, vaya-crypto
2. vaya-net, vaya-db, vaya-cache, vaya-ml
3. vaya-store, vaya-collect
4. vaya-auth, vaya-oracle, vaya-pool, vaya-book, vaya-search
5. vaya-forge, vaya-fleet
6. vaya-api, vaya-bin

---

## CRATE RESPONSIBILITIES

```
vaya-common     â†’ Types, errors, constants (DO NOT MODIFY without approval)
vaya-crypto     â†’ Base64, JWT signing, Argon2, HMAC, constant-time ops
vaya-net        â†’ HTTP server, request parsing, response building, routing
vaya-db         â†’ LSM-tree storage, WAL, memtable, SSTable, compaction
vaya-cache      â†’ In-memory LRU cache, TTL, sharding
vaya-ml         â†’ Tensor operations, model inference
vaya-store      â†’ Relational tables on top of vaya-db, indexes, transactions
vaya-collect    â†’ Data collection from APIs (Kiwi, Travelpayouts, currency)
vaya-auth       â†’ JWT tokens, password hashing, sessions, OAuth
vaya-oracle     â†’ Price prediction, trend analysis, explanation generation
vaya-pool       â†’ Group buying pools, bidding, member management
vaya-book       â†’ Booking flow, payment, tickets, refunds
vaya-search     â†’ Multi-source search aggregation, ranking
vaya-forge      â†’ Build system, hermetic builds, artifact registry
vaya-fleet      â†’ Orchestration, node management, consensus
vaya-api        â†’ API gateway, handlers, middleware
vaya-bin        â†’ Main binary entry point
```

---

## COMMON MISTAKES TO AVOID

### Mistake 1: Wrong Type Access
```rust
// WRONG
let code = airport.0;  // IataCode.0 is private!

// CORRECT
let code = airport.as_str();
```

### Mistake 2: Wrong Money Handling
```rust
// WRONG
let amount: u64 = price.amount;  // amount is MinorUnits, not u64!

// CORRECT
let amount: i64 = price.amount.as_minor();
```

### Mistake 3: Creating Types Wrong
```rust
// WRONG
let price = Price { amount: 15000, currency: "MYR" };

// CORRECT
let price = Price::myr(15000);  // 150.00 MYR in sen
// OR
let price = Price::new(MinorUnits::new(15000), CurrencyCode::MYR, 2);
```

### Mistake 4: Wrong Route Creation
```rust
// WRONG
let route = ("KUL", "SIN");

// CORRECT
let route = Route::new(IataCode::KUL, IataCode::SIN);
```

### Mistake 5: Using unwrap()
```rust
// WRONG
let user = get_user(id).unwrap();

// CORRECT
let user = get_user(id)?;
// OR
let user = get_user(id).ok_or(ApiError::NotFound)?;
```

---

## BUILD COMMANDS

```bash
# First time setup
cd vaya-oracle
cargo check              # Verify compilation

# Development
cargo build              # Debug build
cargo test               # Run tests

# Production
cargo build --release    # Release build
./target/release/vaya    # Run server

# Specific crate
cargo build -p vaya-api  # Build single crate
cargo test -p vaya-auth  # Test single crate
```

---

## ENVIRONMENT VARIABLES

```bash
# Required for runtime
export VAYA_ENV=development
export VAYA_DATA_DIR=/var/lib/vaya

# API keys (for data collection)
export KIWI_API_KEY=your_key
export TRAVELPAYOUTS_TOKEN=your_token

# Optional
export RUST_LOG=info
```

---

## FILE STRUCTURE

```
vaya-oracle/
â”œâ”€â”€ Cargo.toml                  # Workspace root
â”œâ”€â”€ vaya-common/
â”‚   â”œâ”€â”€ Cargo.toml
â”‚   â””â”€â”€ src/
â”‚       â”œâ”€â”€ lib.rs              # Exports types, error, constants
â”‚       â”œâ”€â”€ types.rs            # ALL core types
â”‚       â”œâ”€â”€ error.rs            # VayaError, VayaResult
â”‚       â””â”€â”€ constants.rs        # Constants
â”œâ”€â”€ vaya-*/                     # 16 more crates following same pattern
â””â”€â”€ docs/
    â”œâ”€â”€ BUILD_INSTRUCTIONS.md
    â”œâ”€â”€ COMPLETENESS_AUDIT.md
    â””â”€â”€ KNOWLEDGE_TRANSFER.md
```

---

## WHEN STUCK

1. Read the error message completely
2. Check the type definitions in `vaya-common/src/types.rs`
3. Check if the module exists (`ls -la src/`)
4. Check if dependencies are in Cargo.toml
5. DO NOT randomly modify code
6. DO NOT add new dependencies

---

## SUCCESS CRITERIA

Your implementation is successful when:
1. `cargo check` passes with NO errors
2. `cargo build --release` completes
3. `cargo test` passes (some may be skipped)
4. No `.unwrap()` in lib code
5. All types match `vaya-common` definitions

---

## CURRENT STATE

The backend is COMPLETE with:
- âœ… All 17 crates created
- âœ… All Cargo.toml configured
- âœ… All lib.rs with proper module declarations
- âœ… All modules have source files
- âœ… Type system is consistent
- âœ… Dependencies are declared

What needs work:
- ðŸ”² Integration testing
- ðŸ”² Live API connections
- ðŸ”² ML model training
- ðŸ”² Frontend development
- ðŸ”² Deployment configuration

---

## START HERE

```bash
# 1. Extract the codebase
tar -xzf vaya-oracle-complete.tar.gz
cd vaya-oracle

# 2. Verify everything compiles
cargo check

# 3. If errors, read them carefully and fix

# 4. Once cargo check passes, start your task
```

**DO NOT PROCEED until `cargo check` passes.**

---

*This prompt was generated by Claude (Kiasu Mode) to ensure implementation success.*

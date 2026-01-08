# VAYA Build Instructions for Claude Code

**READ THIS ENTIRE DOCUMENT BEFORE DOING ANYTHING**

This document contains **MANDATORY** instructions for building VAYA. Failure to follow these instructions will result in build failures. Do not skip any step. Do not assume anything. Do not improvise.

## Prerequisites

```bash
# Install Rust (MANDATORY - no alternatives)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Verify Rust version (MUST be 1.75 or higher)
rustc --version
# Expected: rustc 1.75.0 or higher

# Install musl target for static binaries (MANDATORY for VayaForge)
rustup target add x86_64-unknown-linux-musl
```

## Build Order (CRITICAL - DO NOT CHANGE)

The crates MUST be built in dependency order. Cargo handles this automatically, but if you're debugging, this is the order:

1. `vaya-common` - Core types (no dependencies)
2. `vaya-crypto` - Crypto primitives (no VAYA dependencies)
3. `vaya-net` - HTTP server (depends on vaya-common)
4. `vaya-db` - LSM-tree database (depends on vaya-common)
5. `vaya-cache` - In-memory cache (depends on vaya-common)
6. `vaya-ml` - ML inference (depends on vaya-common)
7. `vaya-store` - Relational layer (depends on vaya-db, vaya-cache)
8. `vaya-auth` - Authentication (depends on vaya-cache, vaya-store)
9. `vaya-oracle` - Price prediction (depends on vaya-db, vaya-cache, vaya-ml)
10. `vaya-collect` - Data collection (depends on vaya-db)
11. `vaya-pool` - Group buying (depends on vaya-store, vaya-auth)
12. `vaya-book` - Booking engine (depends on vaya-pool, vaya-auth)
13. `vaya-search` - Search engine (depends on vaya-cache)
14. `vaya-forge` - Build system (depends on vaya-common, vaya-crypto)
15. `vaya-fleet` - Orchestration (depends on vaya-forge, vaya-net)
16. `vaya-api` - API gateway (depends on ALL service crates)
17. `vaya-bin` - Main binary (depends on ALL crates)

## Build Commands

### Development Build (Fast, with debug symbols)

```bash
cd /path/to/vaya-oracle
cargo build
```

### Release Build (Optimized, for production)

```bash
cd /path/to/vaya-oracle
cargo build --release
```

### Check Only (Verify compilation without building)

```bash
cd /path/to/vaya-oracle
cargo check
```

### Run Tests

```bash
cd /path/to/vaya-oracle
cargo test
```

## Common Build Errors and Solutions

### Error: "unresolved import"

**Cause**: Missing module or incorrect path.

**Solution**: Check that:
1. The module file exists (e.g., `src/module_name.rs`)
2. The module is declared in `lib.rs` with `pub mod module_name;`
3. The `use` statement matches the actual exports

### Error: "no field `X` on type `Y`"

**Cause**: Type mismatch between crates.

**Solution**: 
1. Check `vaya-common/src/types.rs` for the correct field names
2. Use `.as_minor()` for `MinorUnits` (not direct access)
3. Use `.as_str()` for `IataCode` (not `.0`)

### Error: "trait `X` is not implemented for `Y`"

**Cause**: Missing derive or impl.

**Solution**: Check the type definition has the required derives (Debug, Clone, etc.)

### Error: "cannot find crate for `xyz`"

**Cause**: Missing dependency in Cargo.toml.

**Solution**: Add the dependency to both:
1. Workspace `Cargo.toml` (in `[workspace.dependencies]`)
2. Crate's `Cargo.toml` (in `[dependencies]`)

## Type Reference (CRITICAL)

These are the EXACT types from `vaya-common`. Do NOT use alternatives:

| Type | Usage | Access |
|------|-------|--------|
| `IataCode` | Airport codes (KUL, SIN) | `.as_str()` to get string |
| `CurrencyCode` | Currency codes (MYR, USD) | Constants: `CurrencyCode::MYR` |
| `MinorUnits` | Money amounts in cents/sen | `.as_minor()` to get i64 |
| `Price` | Money with currency | Use `Price::myr(sen)` or `Price::new(MinorUnits, CurrencyCode, decimals)` |
| `Route` | Origin-destination pair | Fields: `.origin`, `.destination` |
| `Timestamp` | Unix timestamp | `Timestamp::now()`, `.as_unix()` |
| `Date` | Calendar date | `Date::new(year, month, day)`, `Date::from_days_since_epoch(days)` |
| `OfferSource` | Fare source | `.name()` to get string |

## Workspace Structure

```
vaya-oracle/
â”œâ”€â”€ Cargo.toml              # Workspace root - DO NOT MODIFY members list
â”œâ”€â”€ vaya-common/            # Core types (lib.rs exports: types, error, constants)
â”œâ”€â”€ vaya-crypto/            # Crypto (lib.rs exports: base64, jwt, argon2, token, hmac)
â”œâ”€â”€ vaya-net/               # HTTP server (lib.rs exports: server, router, middleware)
â”œâ”€â”€ vaya-db/                # Database (lib.rs exports: engine, wal, memtable, sstable)
â”œâ”€â”€ vaya-cache/             # Cache (lib.rs - single file)
â”œâ”€â”€ vaya-store/             # Relational (lib.rs exports: schema, index, table, query, row)
â”œâ”€â”€ vaya-ml/                # ML (lib.rs exports: tensor, ops, model, models)
â”œâ”€â”€ vaya-oracle/            # Predictor (lib.rs exports: predictor, analyzer, explainer)
â”œâ”€â”€ vaya-collect/           # Collection (lib.rs exports: sources, scheduler, pipeline)
â”œâ”€â”€ vaya-auth/              # Auth (lib.rs exports: jwt, password, totp, session, oauth)
â”œâ”€â”€ vaya-pool/              # Pool (lib.rs exports: state, member, bid, engine)
â”œâ”€â”€ vaya-book/              # Booking (lib.rs exports: state, payment, stripe, ticket)
â”œâ”€â”€ vaya-search/            # Search (lib.rs exports: request, result, route, source, aggregator)
â”œâ”€â”€ vaya-forge/             # Build (lib.rs exports: artifact, manifest, builder, registry, delta)
â”œâ”€â”€ vaya-fleet/             # Orchestration (lib.rs exports: node, service, scheduler, consensus)
â”œâ”€â”€ vaya-api/               # API (lib.rs exports: server, router, handlers, middleware)
â””â”€â”€ vaya-bin/               # Binary (main.rs only)
```

## Environment Variables (for runtime)

```bash
# Required
export VAYA_ENV=development  # or: staging, production
export VAYA_DATA_DIR=/var/lib/vaya

# API Keys (for data collection)
export KIWI_API_KEY=your_key_here
export TRAVELPAYOUTS_TOKEN=your_token_here

# Optional
export RUST_LOG=info  # Logging level: trace, debug, info, warn, error
```

## GitHub Codespaces Specific

If building in GitHub Codespaces:

1. The devcontainer should have Rust pre-installed
2. If not, run the prerequisites above
3. Use `cargo build --release` for production builds
4. Binary will be at `target/release/vaya`

## Verification Checklist

Before considering the build "complete", verify:

- [ ] `cargo check` passes with no errors
- [ ] `cargo build --release` completes successfully
- [ ] `cargo test` passes (some tests may need database)
- [ ] Binary exists at `target/release/vaya`
- [ ] Binary runs: `./target/release/vaya --help`

## If Build Fails

1. **DO NOT** randomly modify code
2. **DO** read the error message carefully
3. **DO** check this document for the error
4. **DO** check that all files exist using: `find . -name "*.rs" | head -50`
5. **DO** check dependencies are correct in Cargo.toml files

## Final Notes

- This codebase uses **zero external runtime dependencies** philosophy
- All types are custom-built in `vaya-common`
- Security is paramount - no unsafe code without explicit review
- Performance matters - every allocation is intentional

**DO NOT** add new dependencies without explicit approval.
**DO NOT** use `unwrap()` in production code - handle errors properly.
**DO NOT** skip reading error messages - they tell you exactly what's wrong.

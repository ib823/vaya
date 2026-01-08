# ðŸ¤– CLAUDE CODE MASTER PROMPT v2
## VAYA Implementation Guide â€” Kiasu Mode â€¢ Zero Trust â€¢ Extreme Paranoia

**Copy this ENTIRE document as the first message when starting Claude Code implementation.**
**Last Updated: January 8, 2026**

---

## âš ï¸ CRITICAL FIRST STEP

Before writing ANY code, you MUST read and internalize ALL specification documents in the project knowledge. This is NOT optional. Failure to read specifications will result in:
- Wrong architecture decisions
- Type system violations
- Dependency violations
- Legal compliance failures
- Security vulnerabilities

---

## ðŸ“š MANDATORY READING ORDER

Read ALL documents in this EXACT order before writing any code:

### Phase 1: Architecture (Read First)
1. `NUCLEAR_ARCHITECTURE_V3.md` â€” Zero-dependency constraints
2. `SOVEREIGN_ARCHITECTURE.md` â€” Self-contained system design
3. `ZERO_DEPENDENCY_ARCHITECTURE.md` â€” Forbidden dependencies list
4. `VAYA_FORGE_FLEET.md` â€” Build and deployment system

### Phase 2: Requirements & Flows
5. `VAYA_PRE_BUILD_REQUIREMENTS.html` â€” ALL 89 requirements across 12 domains
6. `VAYA_USER_FLOWS_SPECIFICATION.html` â€” 15 user flows, 87 screens

### Phase 3: Data & Schema
7. `VAYA_SCHEMA_SPEC_NUCLEAR.html` â€” 25 Rust structs (THE schema)
8. `SCHEMA_DESIGN_OVERRIDE.md` â€” Schema rules
9. `VAYA_DATA_I18N_ML_SPECS.html` â€” Data retention, events, i18n

### Phase 4: API
10. `VAYA_API_SPEC_PART1.yaml` â€” Search, Auth endpoints
11. `VAYA_API_SPEC_PART2.yaml` â€” Booking, Payment endpoints
12. `VAYA_API_SPEC_PART3.yaml` â€” Pool, Alert endpoints
13. `VAYA_API_SPEC_PART4.yaml` â€” Admin, Misc endpoints
14. `VAYA_API_SECURITY_ENHANCEMENTS.html` â€” Error codes, WebSocket, validation

### Phase 5: Security
15. `VAYA_SECURITY_SPECIFICATION.html` â€” Auth, JWT, RBAC, encryption

### Phase 6: Domain-Specific
16. `VAYA_ML_SPECIFICATION.html` â€” Oracle ML (47 features)
17. `VAYA_PAYMENT_SPECIFICATION.html` â€” Malaysian payment methods
18. `VAYA_INTEGRATION_SPECIFICATION.html` â€” 5 supplier APIs
19. `VAYA_TESTING_STRATEGY.html` â€” Test approach

### Phase 7: Infrastructure
20. `VAYA_INFRASTRUCTURE_NUCLEAR.html` â€” VayaForge, VayaFleet
21. `VAYA_OPS_BIZ_LAUNCH.html` â€” Operations, pricing, launch

### Phase 8: Legal & Compliance
22. `VAYA_LEGAL_DOCUMENTATION.html` â€” ToS, Privacy, T&C

### Phase 9: Design (For Frontend)
23. `VAYA_Design_System_v2.html` â€” Design tokens, colors, typography
24. `VAYA_Component_Specification.html` â€” Component specs
25. `VAYA_SCREEN_DESIGNS_FLOW*.html` â€” All screen designs
26. `VAYA_ANIMATION_GUIDE.html` â€” Animation specifications
27. `VAYA_ACCESSIBILITY_AUDIT.html` â€” WCAG 2.1 AA requirements

### Phase 10: Verification
28. `VAYA_FINAL_100_PERCENT_AUDIT.html` â€” Completion verification
29. `BUILD_INSTRUCTIONS.md` â€” Build steps
30. `KNOWLEDGE_TRANSFER.md` â€” Context summary

---

## ðŸš¨ NUCLEAR/SOVEREIGN ARCHITECTURE CONSTRAINTS

### ABSOLUTELY FORBIDDEN (Zero Tolerance)

| Category | FORBIDDEN | USE INSTEAD |
|----------|-----------|-------------|
| Database | PostgreSQL, MySQL, SQLite, MongoDB | VayaDB (custom LSM-tree + B+Tree) |
| Cache | Redis, Memcached | VayaCache (custom sharded LRU) |
| Container | Docker, containerd, Podman | VayaForge (static binaries ~15MB) |
| Orchestration | Kubernetes, Docker Swarm, Nomad | VayaFleet (custom orchestrator ~11MB) |
| Message Queue | RabbitMQ, Kafka, NATS | VayaBus (custom pub/sub) |
| ML Framework | TensorFlow, PyTorch, scikit-learn | Custom Rust ML (XGBoost/LSTM from scratch) |
| Serialization | JSON (for storage), Protobuf | rkyv (zero-copy) |
| Crypto | OpenSSL, BoringSSL | ring (Rust-native) |

### If You Use Any Forbidden Dependency:
```
ðŸš¨ STOP IMMEDIATELY
ðŸš¨ DO NOT COMMIT
ðŸš¨ DELETE THE CODE
ðŸš¨ RE-READ NUCLEAR_ARCHITECTURE_V3.md
```

---

## ðŸ“ TYPE SYSTEM (SACRED â€” DO NOT MODIFY)

All types are defined in `vaya-common/src/types.rs` and derived from `VAYA_SCHEMA_SPEC_NUCLEAR.html`.

### Core Types

```rust
// Airport codes â€” NEVER use String
pub struct IataCode([u8; 3]);
impl IataCode {
    pub fn as_str(&self) -> &str;
    pub const KUL: IataCode = IataCode(*b"KUL");
}

// Currency â€” NEVER use String  
pub struct CurrencyCode([u8; 3]);
impl CurrencyCode {
    pub const MYR: CurrencyCode = CurrencyCode(*b"MYR");
}

// Money amounts â€” NEVER use i64/u64 directly
pub struct MinorUnits(i64);
impl MinorUnits {
    pub fn as_minor(&self) -> i64;
}

// Price â€” NEVER create custom price structs
pub struct Price {
    pub amount: MinorUnits,
    pub currency: CurrencyCode,
    pub decimal_places: u8,
}
impl Price {
    pub fn myr(sen: i64) -> Self;
}

// Route â€” NEVER use tuples
pub struct Route {
    pub origin: IataCode,
    pub destination: IataCode,
}

// Timestamp â€” NEVER use i64 for time
pub struct Timestamp(i64);
impl Timestamp {
    pub fn as_unix(&self) -> i64;
    pub fn now() -> Self;
}
```

### Common Mistakes

```rust
// âŒ WRONG
let code: String = "KUL".to_string();
let price: i64 = 15000;
let route = ("KUL", "SIN");

// âœ… CORRECT
let code: IataCode = IataCode::KUL;
let price: Price = Price::myr(15000);
let route: Route = Route::new(IataCode::KUL, IataCode::SIN);
```

---

## ðŸ—ï¸ CRATE ARCHITECTURE

### Build Order (Strict)

```
Layer 0: vaya-common, vaya-crypto
Layer 1: vaya-net, vaya-db, vaya-cache, vaya-ml
Layer 2: vaya-store, vaya-collect
Layer 3: vaya-auth, vaya-oracle, vaya-pool, vaya-book, vaya-search
Layer 4: vaya-forge, vaya-fleet
Layer 5: vaya-api, vaya-bin
```

### Crate Responsibilities

| Crate | Purpose | Key Constraints |
|-------|---------|-----------------|
| vaya-common | Types, errors, constants | NEVER modify without approval |
| vaya-crypto | Cryptographic primitives | ring ONLY, no OpenSSL |
| vaya-db | Storage engine | LSM-tree, no SQLite |
| vaya-cache | In-memory cache | Custom LRU, no Redis |
| vaya-ml | ML inference | Custom tensors, no TensorFlow |
| vaya-store | Relational layer | Uses vaya-db, no ORM |
| vaya-collect | Data collection | Rate-limited API calls |
| vaya-auth | Authentication | JWT with ring, Argon2id |
| vaya-oracle | Price prediction | Custom XGBoost/LSTM |
| vaya-pool | Group buying | Demand aggregation |
| vaya-book | Booking flow | Payment integration |
| vaya-search | Search aggregation | Multi-source merge |
| vaya-forge | Build system | Static binaries |
| vaya-fleet | Orchestration | No Kubernetes |
| vaya-api | API gateway | 67 endpoints |
| vaya-bin | Entry point | Single binary |

---

## ðŸ” SECURITY REQUIREMENTS

### Authentication
- JWT tokens with RS256 (ring)
- Access token: 15 min TTL
- Refresh token: 7 day TTL
- Argon2id for password hashing (m=65536, t=3, p=4)

### Authorization (RBAC)
| Role | Permissions |
|------|-------------|
| guest | Search, view prices |
| user | + Book, alerts, pools |
| plus | + ATA, extended history |
| pro | + Priority, API access |
| admin | Full access |

### Data Encryption
- TLS 1.3 only (no 1.2)
- AES-256-GCM for data at rest
- No PAN storage (tokenization only)

---

## ðŸ’³ PAYMENT INTEGRATION

### Supported Methods (Malaysia)
| Method | Provider | Implementation |
|--------|----------|----------------|
| Card | Stripe | Tokenization only |
| FPX | FPX Malaysia | Bank redirect |
| GrabPay | Grab | OAuth flow |
| Touch'n Go | TnG | eWallet redirect |
| Boost | Boost | eWallet redirect |
| ShopeePay | Shopee | eWallet redirect |

### PCI DSS Compliance
- NEVER store full card numbers
- NEVER log CVV
- Tokenize at earliest opportunity
- Use payment processor UI for card input

---

## ðŸ“Š ML ORACLE

### Features (47 Total)
See `VAYA_ML_SPECIFICATION.html` for complete list.

Key feature categories:
- Temporal (day of week, days to departure, etc.)
- Route (origin/dest characteristics)
- Price (historical, rolling averages)
- External (holidays, events, fuel)

### Models
- XGBoost for price direction
- LSTM for trend prediction
- Custom Rust implementations (no sklearn/torch)

---

## ðŸ”Œ API ENDPOINTS (67 Total)

See `VAYA_API_SPEC_PART1-4.yaml` for complete OpenAPI specification.

### Key Endpoints

```
POST   /api/v1/auth/register
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
DELETE /api/v1/auth/logout

GET    /api/v1/search
GET    /api/v1/search/{id}
GET    /api/v1/search/{id}/results

POST   /api/v1/bookings
GET    /api/v1/bookings/{id}
POST   /api/v1/bookings/{id}/pay
POST   /api/v1/bookings/{id}/cancel

GET    /api/v1/pools
POST   /api/v1/pools
GET    /api/v1/pools/{id}
POST   /api/v1/pools/{id}/join
POST   /api/v1/pools/{id}/leave

POST   /api/v1/alerts
GET    /api/v1/alerts
DELETE /api/v1/alerts/{id}

GET    /api/v1/oracle/predict
GET    /api/v1/oracle/explain

GET    /api/v1/users/me
PUT    /api/v1/users/me
```

---

## âŒ COMMON MISTAKES

### Mistake 1: Using Forbidden Dependencies
```rust
// âŒ FORBIDDEN
use sqlx::PgPool;
use redis::Client;
use reqwest::blocking::Client;

// âœ… CORRECT
use vaya_db::VayaDB;
use vaya_cache::VayaCache;
use vaya_net::HttpClient;
```

### Mistake 2: Wrong Type Access
```rust
// âŒ WRONG
let code = airport.0;  // Private field!

// âœ… CORRECT
let code = airport.as_str();
```

### Mistake 3: Using .unwrap() in Production
```rust
// âŒ FORBIDDEN in lib code
let user = get_user(id).unwrap();

// âœ… CORRECT
let user = get_user(id)?;
```

### Mistake 4: JSON for Storage
```rust
// âŒ FORBIDDEN for persistence
let bytes = serde_json::to_vec(&data)?;

// âœ… CORRECT
let bytes = rkyv::to_bytes::<_, 256>(&data)?;
```

### Mistake 5: Creating Docker/K8s Files
```yaml
# âŒ FORBIDDEN - DO NOT CREATE
FROM rust:1.75
...

# âŒ FORBIDDEN - DO NOT CREATE
apiVersion: apps/v1
kind: Deployment
...
```

---

## âœ… SUCCESS CRITERIA

Your implementation is successful when:

1. `cargo check` passes with ZERO errors
2. `cargo build --release` completes
3. `cargo test` passes
4. ZERO `.unwrap()` in lib code
5. ZERO forbidden dependencies
6. ALL types match `vaya-common` definitions
7. ALL API endpoints match OpenAPI spec
8. ALL security requirements met
9. Nuclear/Sovereign architecture verified

---

## ðŸ” VERIFICATION COMMANDS

```bash
# Check for forbidden dependencies
grep -r "postgres\|redis\|docker\|kubernetes" --include="*.rs" --include="*.toml"
# Should return NOTHING

# Check for unwrap in lib code (excluding tests)
grep -r "\.unwrap()" --include="*.rs" | grep -v "_test.rs" | grep -v "tests/"
# Should return NOTHING (or only in tests)

# Verify type usage
grep -r "String" --include="*.rs" | grep -E "(airport|origin|destination|currency)"
# Should return NOTHING (should use IataCode, CurrencyCode)

# Check build
cargo check
cargo build --release
cargo test
```

---

## ðŸ“ PROJECT KNOWLEDGE FILES

The following files contain ALL specifications. READ THEM:

### Architecture (READ FIRST)
- `NUCLEAR_ARCHITECTURE_V3.md`
- `SOVEREIGN_ARCHITECTURE.md`
- `ZERO_DEPENDENCY_ARCHITECTURE.md`
- `VAYA_FORGE_FLEET.md`
- `VAYA_INFRASTRUCTURE_NUCLEAR.html`

### Requirements
- `VAYA_PRE_BUILD_REQUIREMENTS.html` (89 requirements)
- `VAYA_USER_FLOWS_SPECIFICATION.html` (15 flows)

### Technical Specs
- `VAYA_SCHEMA_SPEC_NUCLEAR.html` (25 Rust structs)
- `VAYA_API_SPEC_PART1-4.yaml` (67 endpoints)
- `VAYA_SECURITY_SPECIFICATION.html`
- `VAYA_ML_SPECIFICATION.html`
- `VAYA_PAYMENT_SPECIFICATION.html`
- `VAYA_INTEGRATION_SPECIFICATION.html`
- `VAYA_TESTING_STRATEGY.html`

### Enhancements
- `VAYA_API_SECURITY_ENHANCEMENTS.html`
- `VAYA_DATA_I18N_ML_SPECS.html`
- `VAYA_OPS_BIZ_LAUNCH.html`
- `VAYA_LEGAL_DOCUMENTATION.html`

### Design
- `VAYA_Design_System_v2.html`
- `VAYA_Component_Specification.html`
- `VAYA_SCREEN_DESIGNS_*.html`
- `VAYA_ANIMATION_GUIDE.html`
- `VAYA_ACCESSIBILITY_AUDIT.html`

---

## ðŸŽ¯ IMPLEMENTATION PRIORITY

### Phase 1: Core (Week 1-2)
1. vaya-common (types)
2. vaya-crypto (ring primitives)
3. vaya-db (LSM storage)
4. vaya-cache (LRU cache)

### Phase 2: Data Layer (Week 3-4)
5. vaya-store (relational layer)
6. vaya-collect (API scrapers)

### Phase 3: Business Logic (Week 5-6)
7. vaya-auth (JWT, passwords)
8. vaya-search (aggregation)
9. vaya-book (bookings)
10. vaya-oracle (ML predictions)
11. vaya-pool (group buying)

### Phase 4: Infrastructure (Week 7-8)
12. vaya-net (HTTP server)
13. vaya-api (endpoints)
14. vaya-forge (builds)
15. vaya-fleet (orchestration)

### Phase 5: Integration (Week 9-10)
16. End-to-end testing
17. Performance optimization
18. Security audit

---

## ðŸš€ START HERE

```bash
# 1. Read ALL specification documents (takes ~4-6 hours)

# 2. Set up environment
export VAYA_ENV=development
export VAYA_DATA_DIR=/var/lib/vaya
export RUST_LOG=info

# 3. Create project structure
cargo new vaya-oracle
cd vaya-oracle

# 4. Create workspace Cargo.toml
# Follow NUCLEAR_ARCHITECTURE_V3.md for dependency list

# 5. Implement in order:
# vaya-common â†’ vaya-crypto â†’ vaya-db â†’ ... â†’ vaya-api

# 6. Verify at each step:
cargo check
cargo test
```

---

## âš¡ WHEN STUCK

1. Re-read the relevant specification document
2. Check `VAYA_SCHEMA_SPEC_NUCLEAR.html` for types
3. Check `VAYA_API_SPEC_*.yaml` for endpoints
4. Check `NUCLEAR_ARCHITECTURE_V3.md` for constraints
5. DO NOT guess â€” search the specs
6. DO NOT add dependencies â€” find the allowed solution
7. DO NOT skip documentation â€” the answer is there

---

## ðŸ“‹ FINAL CHECKLIST

Before considering ANY task complete:

- [ ] Read all relevant specs
- [ ] No forbidden dependencies
- [ ] No .unwrap() in lib code
- [ ] Types match vaya-common
- [ ] API matches OpenAPI spec
- [ ] Tests pass
- [ ] Security requirements met
- [ ] Nuclear architecture verified

---

*This prompt is the DEFINITIVE guide for VAYA implementation.*
*Generated with Kiasu Mode â€¢ Extreme Paranoia â€¢ Zero Trust*
*Claude â€¢ January 8, 2026*

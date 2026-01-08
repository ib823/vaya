# VayaPay Operational Deep-Dive: Your Questions Answered

## Table of Contents
1. [How Can VayaPay Track User Bank Balance (Travel Fund)?](#1-travel-fund-balance-tracking)
2. [Escrow Service Costs in Malaysia](#2-escrow-service-costs)
3. [Do Trustees Have APIs?](#3-trustee-api-availability)
4. [How Easy to Put Money Into Trustee Account?](#4-funding-trustee-accounts)
5. [KYC Process in VAYA](#5-kyc-process)
6. [What Is Escrow For?](#6-escrow-purposes)

---

## 1. Travel Fund Balance Tracking

### The Reality Check: VAYA CANNOT Directly Track Bank Balances

**Important clarification:** In the original architecture, I oversimplified this. Let me correct:

### Option A: User Self-Reports (Phase 1 - Simplest)
```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                 â”‚
â”‚   USER'S BANK ACCOUNT                    VAYA TRAVEL FUND       â”‚
â”‚   (Maybank, CIMB, etc.)                  (Goal Tracker)         â”‚
â”‚                                                                 â”‚
â”‚   User's actual balance: RM 5,000        Target: RM 2,400       â”‚
â”‚   (VAYA cannot see this)                 Progress: RM 1,600     â”‚
â”‚                                          (User-reported)        â”‚
â”‚                                                                 â”‚
â”‚   User MANUALLY updates                                         â”‚
â”‚   progress in VAYA app     â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â–º  VAYA tracks progress   â”‚
â”‚                                          toward goal            â”‚
â”‚                                                                 â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

**How it works:**
- User sets goal (e.g., "Tokyo trip, RM 2,400")
- User manually updates how much they've saved
- VAYA sends reminders, tracks progress, notifies when goal reached
- **No API connection to bank needed**
- **No license required** - just goal-tracking software

**Cost:** RM 0 (pure software)

---

### Option B: Open Banking via Account Aggregator (Phase 2)

Malaysia has emerging Open Banking infrastructure, but it's NOT like Plaid in the US yet.

**Southeast Asia Open Banking Providers:**

| Provider | Coverage | Malaysia Support | Pricing Model |
|----------|----------|------------------|---------------|
| **Finverse** | HK, PH, SG, VN, MY, ID | âœ… Yes - Top banks | Pay per API call |
| **Brankas** | ID, PH, TH, SG | âš ï¸ Expanding to MY | Pay per usage |
| **Finantier** | ID, SG | âŒ Not yet | SaaS model |
| **Brick** | ID | âŒ Indonesia only | Pay per usage |

**Finverse (Most Relevant for Malaysia):**
- Covers **40+ banks** including Malaysia's major banks
- Provides: Account balances, transactions, statements, identity, income
- API-based, consent-driven
- Contact: https://finverse.com

**How Open Banking Integration Works:**

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   USER                   VAYA                   FINVERSE              BANK â”‚
â”‚                                                 (Aggregator)               â”‚
â”‚                                                                            â”‚
â”‚   1. "Link my           2. Redirect to                                     â”‚
â”‚      Maybank"   â”€â”€â”€â”€â–º      Finverse     â”€â”€â”€â”€â–º  3. Bank login               â”‚
â”‚                                                   portal        â”€â”€â”€â”€â–º  4. User â”‚
â”‚                                                                     authenticates â”‚
â”‚                                                                            â”‚
â”‚                         6. Receive              5. Return                  â”‚
â”‚                            balance data  â—„â”€â”€â”€â”€     consent +    â—„â”€â”€â”€â”€      â”‚
â”‚                                                    balance                 â”‚
â”‚                                                                            â”‚
â”‚   7. Display                                                               â”‚
â”‚      "RM 1,600 saved"                                                      â”‚
â”‚      (auto-updated)                                                        â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

**Estimated Costs (Open Banking):**
- Finverse/Brankas: ~$0.30-$1.00 per API call
- Monthly active user: ~$0.50-$1.00/user/month
- For 10,000 users: ~RM 25,000-50,000/month

**License Required:** Potentially needs registration as AISP (Account Information Service Provider) with BNM, or partner with licensed aggregator.

---

### Option C: FPX Direct Debit / eMandate (Phase 2 - Recommended)

This is the **most practical approach** for Malaysia.

**What is eMandate?**
- PayNet's electronic direct debit authorization system
- User authorizes recurring debits from their bank account
- Processing: Same-day (vs. 2 weeks for paper mandate)
- Supported by: All major MY banks (Maybank, CIMB, Public Bank, RHB, Hong Leong, etc.)

**How eMandate Works for Travel Fund:**

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   STEP 1: USER SETS UP AUTO-SAVE                                           â”‚
â”‚                                                                            â”‚
â”‚   User: "Auto-save RM 200/month for Tokyo trip"                            â”‚
â”‚                                                                            â”‚
â”‚         â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”        â”‚
â”‚         â”‚                                                         â”‚        â”‚
â”‚         â”‚   VAYA redirects to eMandate authorization              â”‚        â”‚
â”‚         â”‚   (via PayNet/bank's secure portal)                     â”‚        â”‚
â”‚         â”‚                                                         â”‚        â”‚
â”‚         â”‚   User logs into Maybank2u                              â”‚        â”‚
â”‚         â”‚   Authorizes: "VAYA SDN BHD may debit RM 200/month"     â”‚        â”‚
â”‚         â”‚   Duration: 12 months                                   â”‚        â”‚
â”‚         â”‚   Purpose: Travel savings                               â”‚        â”‚
â”‚         â”‚                                                         â”‚        â”‚
â”‚         â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜        â”‚
â”‚                                                                            â”‚
â”‚   STEP 2: MONTHLY AUTO-DEBIT                                               â”‚
â”‚                                                                            â”‚
â”‚         â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”         â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”                        â”‚
â”‚         â”‚               â”‚         â”‚               â”‚                        â”‚
â”‚         â”‚  USER'S BANK  â”‚ â”€â”€â”€â”€â”€â”€â–º â”‚  VAYA'S PSP   â”‚                        â”‚
â”‚         â”‚  (Maybank)    â”‚ RM 200  â”‚  (iPay88)     â”‚                        â”‚
â”‚         â”‚               â”‚         â”‚               â”‚                        â”‚
â”‚         â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜         â””â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”˜                        â”‚
â”‚                                           â”‚                                â”‚
â”‚                                           â–¼                                â”‚
â”‚                                   â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”                        â”‚
â”‚                                   â”‚               â”‚                        â”‚
â”‚                                   â”‚  VAYA HOLDING â”‚                        â”‚
â”‚                                   â”‚  ACCOUNT      â”‚                        â”‚
â”‚                                   â”‚  (Client $)   â”‚                        â”‚
â”‚                                   â”‚               â”‚                        â”‚
â”‚                                   â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜                        â”‚
â”‚                                                                            â”‚
â”‚   STEP 3: VAYA TRACKS PROGRESS                                             â”‚
â”‚                                                                            â”‚
â”‚   Since VAYA initiated the debit, VAYA knows exactly how much              â”‚
â”‚   has been collected. No need to "read" bank balance.                      â”‚
â”‚                                                                            â”‚
â”‚   Progress: â–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–ˆâ–‘â–‘â–‘â–‘â–‘â–‘â–‘â–‘ 67% (RM 1,600 of RM 2,400)            â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

**âš ï¸ IMPORTANT LICENSE IMPLICATION:**

If VAYA collects and **HOLDS** user's savings, this may require:
- **E-Money License** (if funds held > 7 days)
- OR structure via **Trust Account** at licensed trustee

**SOLUTION: Don't hold the funds yourself.**

### Option D: Partner with Licensed Entity for Trust Account

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   USER                  VAYA              TRUSTEE            USER'S BANK   â”‚
â”‚                      (Platform)        (AmanahRaya)          (Maybank)     â”‚
â”‚                                                                            â”‚
â”‚   1. Set goal:      2. Create           3. Open trust                      â”‚
â”‚      RM 2,400          Travel Fund         sub-account                     â”‚
â”‚                        ID                  for user                        â”‚
â”‚                                                                            â”‚
â”‚   4. Authorize      5. Process          6. Receive         7. Debit       â”‚
â”‚      eMandate          debit               funds               funds       â”‚
â”‚                        instruction                                         â”‚
â”‚                                                                            â”‚
â”‚                                         8. Hold in trust                   â”‚
â”‚                                            account                         â”‚
â”‚                                            (User's $)                      â”‚
â”‚                                                                            â”‚
â”‚   9. View           10. Query           11. Report                         â”‚
â”‚      progress           balance              balance                       â”‚
â”‚                                                                            â”‚
â”‚   12. Book!         13. Instruct        14. Release to                     â”‚
â”‚                         release             airline                        â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜

VAYA LICENSE: âŒ NONE (Trustee holds funds)
TRUSTEE LICENSE: âœ… Trust Companies Act 1949
```

---

## 2. Escrow Service Costs in Malaysia

### Malaysian Trustee Fee Structures

Based on research, Malaysian trustees typically charge:

| Fee Type | Typical Range | Notes |
|----------|---------------|-------|
| **Setup Fee** | RM 500 - RM 5,000 | One-time, depends on complexity |
| **Annual Administration** | 0.1% - 0.5% of AUM | Minimum RM 1,000-5,000/year |
| **Per Transaction** | RM 50 - RM 200 | For each release/refund |
| **Custody Fee** | 0.05% - 0.15% p.a. | For holding funds |

### Estimated Costs for VayaPay Use Cases

**Scenario 1: Pool Commitment Escrow (4,000 users)**
```
Pool size: 4,000 users Ã— RM 200 = RM 800,000
Duration: 60 days average

Setup fee:                    RM 2,000
Custody fee (0.1% p.a.):      RM 132 (60 days pro-rated)
Per-transaction (4,000 Ã— RM50): RM 200,000 (if each user = 1 txn)

PROBLEM: Per-transaction fees make this uneconomical!
```

**Solution: Negotiate Bulk/Enterprise Deal**

| Tier | Monthly Volume | Per-Transaction | Custody |
|------|----------------|-----------------|---------|
| Startup | <RM 1M | RM 50/txn | 0.1% p.a. |
| Growth | RM 1-10M | RM 10-20/txn | 0.05% p.a. |
| Enterprise | >RM 10M | RM 2-5/txn | Negotiable |

**Realistic Cost for Pool Feature:**
- Negotiate enterprise rate: RM 5/transaction
- 4,000 pool members = RM 20,000
- Split across users = RM 5/user
- **Add to pool commitment fee** (user pays)

**Scenario 2: Travel Fund Escrow (Personal Savings)**

This is trickier because it's many small accounts:

```
10,000 users Ã— RM 200/month = RM 2M monthly flow
Average balance: RM 1,000/user = RM 10M AUM

Option A: Individual sub-accounts
- Too expensive (RM 1,000+ per account setup)

Option B: Omnibus account with internal ledger
- One master trust account
- VAYA maintains internal ledger of who owns what
- Much cheaper: ~0.1% p.a. on total AUM
- Cost: RM 10,000/year for RM 10M AUM
```

### Trustee Contact Information (Get Real Quotes)

**1. AmanahRaya Trustees Berhad** (Government-linked, largest)
- Website: https://www.artrustees.my
- Email: corporate@amanahraya.my
- Phone: +603-2055 5000
- Best for: Large volumes, established track record

**2. Pacific Trustees Berhad** (Escrow specialist)
- Website: https://www.pacifictrustees.com
- "No minimum size for escrow accounts"
- "Pricing upon application"
- Offices: KL, Labuan, Singapore

**3. Mega Trustee Berhad** (SME-friendly)
- Website: https://megatrustee.com.my
- Email: info@megatrustee.com.my
- Phone: +60 19-227 7786
- "Free consultation for escrow needs"

**4. PB Trustee Services Berhad** (Public Bank subsidiary)
- Website: https://www.pbtrustee.com.my
- Existing relationship with Public Bank customers

---

## 3. Do Trustees Have APIs?

### The Hard Truth: Most Don't (Yet)

**Current State:**
- Most Malaysian trustees operate on **manual/semi-automated** processes
- Email instructions, signed forms, bank transfers
- API connectivity is NOT standard

**What This Means for VAYA:**

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   IDEAL (Future):                        REALITY (Today):                  â”‚
â”‚                                                                            â”‚
â”‚   VAYA â”€â”€APIâ”€â”€â–º Trustee                  VAYA â”€â”€Emailâ”€â”€â–º Trustee           â”‚
â”‚         â—„â”€â”€APIâ”€â”€                                â—„â”€â”€Emailâ”€â”€                 â”‚
â”‚                                                                            â”‚
â”‚   Real-time, automated                   Manual processing                 â”‚
â”‚   Sub-second response                    Hours to days                     â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### Solutions:

**Option 1: Build Custom Integration with Trustee**

Negotiate with one trustee to build API integration:
- Development cost: RM 50,000-200,000
- Maintenance: RM 5,000-10,000/month
- Timeline: 3-6 months
- **Best for:** High volume, long-term commitment

**Option 2: Use PSP's Virtual Account Feature**

Some PSPs offer virtual account sub-ledgers:

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   iPay88 / Stripe                                                          â”‚
â”‚                                                                            â”‚
â”‚   â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”     â”‚
â”‚   â”‚  VAYA Master Merchant Account                                    â”‚     â”‚
â”‚   â”‚                                                                  â”‚     â”‚
â”‚   â”‚  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â” â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â” â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â” â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â” â”‚     â”‚
â”‚   â”‚  â”‚ User A      â”‚ â”‚ User B      â”‚ â”‚ User C      â”‚ â”‚ Pool #1    â”‚ â”‚     â”‚
â”‚   â”‚  â”‚ RM 500      â”‚ â”‚ RM 1,200    â”‚ â”‚ RM 300      â”‚ â”‚ RM 800,000 â”‚ â”‚     â”‚
â”‚   â”‚  â”‚ (Virtual)   â”‚ â”‚ (Virtual)   â”‚ â”‚ (Virtual)   â”‚ â”‚ (Virtual)  â”‚ â”‚     â”‚
â”‚   â”‚  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜ â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜ â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜ â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜ â”‚     â”‚
â”‚   â”‚                                                                  â”‚     â”‚
â”‚   â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜     â”‚
â”‚                                                                            â”‚
â”‚   API: âœ… Yes, real-time                                                   â”‚
â”‚   License: PSP is licensed, VAYA is merchant                               â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

**âš ï¸ BUT:** This may trigger e-money considerations if VAYA holds funds > 7 days.

**Option 3: Hybrid Approach (Recommended)**

```
For Pool Commitments (Short-term, <60 days):
â”œâ”€â”€ Use PSP's pending capture / virtual accounts
â”œâ”€â”€ API-driven, real-time
â””â”€â”€ Release directly to airline on pool activation

For Travel Fund (Long-term savings):
â”œâ”€â”€ Phase 1: User self-reports progress (no license needed)
â”œâ”€â”€ Phase 2: Partner with digital bank (e.g., GXBank, Boost)
â”‚            They hold funds, VAYA is UX layer
â””â”€â”€ Phase 3: Get own e-money license if volumes justify
```

---

## 4. How Easy to Put Money Into Trustee Account?

### Standard Process (Without API)

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   FUNDING ESCROW ACCOUNT - CURRENT PROCESS                                 â”‚
â”‚                                                                            â”‚
â”‚   Day 0: User joins pool                                                   â”‚
â”‚         â”œâ”€â”€ VAYA collects RM 200 via FPX/Card (instant)                    â”‚
â”‚         â””â”€â”€ Funds land in VAYA's PSP account                               â”‚
â”‚                                                                            â”‚
â”‚   Day 0-1: VAYA initiates transfer to trustee                              â”‚
â”‚         â”œâ”€â”€ Bank transfer to trustee's designated account                  â”‚
â”‚         â”œâ”€â”€ Reference: Pool ID + User ID                                   â”‚
â”‚         â””â”€â”€ Email confirmation to trustee                                  â”‚
â”‚                                                                            â”‚
â”‚   Day 1-2: Trustee acknowledges receipt                                    â”‚
â”‚         â”œâ”€â”€ Confirms funds received                                        â”‚
â”‚         â”œâ”€â”€ Updates internal ledger                                        â”‚
â”‚         â””â”€â”€ Issues receipt to VAYA                                         â”‚
â”‚                                                                            â”‚
â”‚   Timeline: 1-2 business days                                              â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### Optimized Process (With Integration)

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   OPTION: PSP Direct to Trustee                                            â”‚
â”‚                                                                            â”‚
â”‚   User pays RM 200 via FPX                                                 â”‚
â”‚         â”‚                                                                  â”‚
â”‚         â–¼                                                                  â”‚
â”‚   PSP (iPay88) routes directly to trustee's bank account                   â”‚
â”‚   (Instead of VAYA's account)                                              â”‚
â”‚         â”‚                                                                  â”‚
â”‚         â–¼                                                                  â”‚
â”‚   Trustee receives funds (T+0 or T+1)                                      â”‚
â”‚                                                                            â”‚
â”‚   VAYA sends: Transaction reference, user details, pool ID                 â”‚
â”‚   Trustee confirms: Ledger updated                                         â”‚
â”‚                                                                            â”‚
â”‚   Result: VAYA NEVER TOUCHES THE MONEY                                     â”‚
â”‚   License: âŒ None required                                                â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### Batch Processing for Efficiency

For high volumes, trustees accept batch instructions:

```
Daily batch at 5:00 PM:

VAYA sends to Trustee:
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚  File: VAYA_POOL_DEPOSITS_20260115.csv                       â”‚
â”‚                                                              â”‚
â”‚  Pool_ID     | User_ID | Amount  | Reference    | Status     â”‚
â”‚  TYO-MAR-001 | U12345  | 200.00  | FPX-ABC123   | New        â”‚
â”‚  TYO-MAR-001 | U12346  | 200.00  | FPX-ABC124   | New        â”‚
â”‚  TYO-MAR-001 | U12347  | 200.00  | FPX-ABC125   | New        â”‚
â”‚  ...         | ...     | ...     | ...          | ...        â”‚
â”‚  Total: 150 new deposits, RM 30,000                          â”‚
â”‚                                                              â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Trustee reconciles with bank statement
Trustee confirms receipt

This is how insurance companies and unit trusts operate today.
```

---

## 5. KYC Process in VAYA

### Who Does KYC?

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   ENTITY                  â”‚  KYC RESPONSIBILITY       â”‚  REGULATION         â”‚
â”‚   â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”‚
â”‚                           â”‚                           â”‚                     â”‚
â”‚   User â†’ VAYA             â”‚  Basic verification       â”‚  PDPA (data privacy)â”‚
â”‚   (Platform registration) â”‚  Email, phone, name       â”‚  No AML obligation  â”‚
â”‚                           â”‚                           â”‚  (not financial svc)â”‚
â”‚                           â”‚                           â”‚                     â”‚
â”‚   User â†’ PSP              â”‚  Payment verification     â”‚  BNM PSP guidelines â”‚
â”‚   (Payment processing)    â”‚  Card validation          â”‚  AML/CFT compliance â”‚
â”‚                           â”‚  FPX bank authentication  â”‚                     â”‚
â”‚                           â”‚                           â”‚                     â”‚
â”‚   User â†’ Trustee          â”‚  Full KYC if required     â”‚  Trust Companies    â”‚
â”‚   (Escrow accounts)       â”‚  IC verification          â”‚  Act 1949           â”‚
â”‚                           â”‚  Source of funds (large)  â”‚  AMLA 2001          â”‚
â”‚                           â”‚                           â”‚                     â”‚
â”‚   User â†’ Airline          â”‚  Passport/travel document â”‚  ICAO standards     â”‚
â”‚   (Booking)               â”‚  Name matching            â”‚                     â”‚
â”‚                           â”‚                           â”‚                     â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### VAYA's KYC Tiers

**Tier 1: Basic Account (No KYC)**
```
Requirements:
- Email verification
- Phone number (OTP)
- Name

Limits:
- Can search flights
- Can use Travel Fund (self-tracking)
- Can view pools
- Cannot transact

Cost: RM 0
```

**Tier 2: Verified Account (Lightweight KYC)**
```
Requirements:
- Tier 1 +
- MyKad/Passport scan (OCR verification)
- Selfie match (optional for enhanced)

Verification method:
- Partner with eKYC provider (e.g., Innov8tif, Jumio)
- Cost: RM 2-5 per verification

Limits:
- Can book flights up to RM 10,000
- Can join pools up to RM 5,000
- Can use Split Trip

This matches PSP requirements - they do their own KYC anyway.
```

**Tier 3: Premium Account (Full KYC)**
```
Requirements:
- Tier 2 +
- Address verification
- Source of funds declaration (for large transactions)

When required:
- Pool commitments > RM 5,000
- Monthly transactions > RM 50,000
- Trustee may require this for escrow

Done by: Trustee (not VAYA)
```

### Practical KYC Flow for Pool Commitment

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                            â”‚
â”‚   USER JOURNEY: JOIN POOL WITH RM 200                                      â”‚
â”‚                                                                            â”‚
â”‚   1. User clicks "Join Pool"                                               â”‚
â”‚      â””â”€â”€ Already logged in (Tier 1)                                        â”‚
â”‚                                                                            â”‚
â”‚   2. First-time pool user?                                                 â”‚
â”‚      â””â”€â”€ Yes: "Please verify your identity" (Tier 2)                       â”‚
â”‚          â”œâ”€â”€ Scan MyKad                                                    â”‚
â”‚          â”œâ”€â”€ Take selfie                                                   â”‚
â”‚          â””â”€â”€ Verified in 30 seconds                                        â”‚
â”‚                                                                            â”‚
â”‚   3. Payment                                                               â”‚
â”‚      â””â”€â”€ Select FPX â†’ Maybank                                              â”‚
â”‚          â””â”€â”€ Bank does its own authentication                              â”‚
â”‚                                                                            â”‚
â”‚   4. Funds to Trustee                                                      â”‚
â”‚      â””â”€â”€ Trustee receives: Name, IC number, amount                         â”‚
â”‚          â””â”€â”€ For small amounts (<RM 5,000): Simplified KYC                 â”‚
â”‚              (Trustee relies on bank's KYC + VAYA's verification)          â”‚
â”‚                                                                            â”‚
â”‚   5. Pool commitment recorded                                              â”‚
â”‚      â””â”€â”€ User's RM 200 held in escrow                                      â”‚
â”‚                                                                            â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜

Total KYC friction: ~30 seconds (one-time)
```

---

## 6. What Is Escrow For?

### Escrow Purposes in VayaPay

| Feature | Escrow Needed? | Purpose | Duration |
|---------|----------------|---------|----------|
| **Pool Commitment** | âœ… YES | Hold commitment until pool activates | 30-90 days |
| **Split Trip** | âŒ NO | PSP pending capture is sufficient | 24-48 hours |
| **Travel Fund** | âš ï¸ MAYBE | Depends on implementation | Months |
| **Instant Refund** | âŒ NO | VAYA's own capital | N/A |
| **VAYA Credits** | âŒ NO | Discounts, not stored value | N/A |

### Deep Dive: When Escrow Is Actually Needed

**POOL COMMITMENT - Escrow Required**
```
Why escrow:
- User commits RM 200 today
- Pool may not activate for 60 days
- If pool fails, user gets 100% refund
- Need neutral third party to hold funds

Without escrow:
- VAYA holds funds = potential e-money license
- Users may not trust VAYA with their money
- No legal protection for users if VAYA goes bankrupt

With escrow:
- Trustee is regulated, insured
- Funds segregated from VAYA's operational money
- Clear legal framework for release/refund
```

**SPLIT TRIP - No Escrow Needed**
```
Why no escrow:
- PSP holds pending capture (standard e-commerce)
- Duration: 24 hours max
- If all pay â†’ capture and book
- If any fail â†’ release all

This is how Shopee, Lazada work
PSP is already licensed
No additional license needed
```

**TRAVEL FUND - Depends on Structure**
```
Structure A: Self-tracking
- User saves in own bank account
- VAYA just tracks progress
- No escrow needed

Structure B: Auto-debit to holding
- Money leaves user's account
- If VAYA holds > 7 days = potential e-money
- Escrow recommended

Structure C: Direct to airline prepayment
- Some airlines offer fare lock / deposit
- VAYA facilitates, never touches money
- No escrow needed
```

### Summary: Escrow Decision Matrix

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                             â”‚
â”‚   DECISION: DO I NEED ESCROW?                                               â”‚
â”‚                                                                             â”‚
â”‚   Q1: Does VAYA hold user funds?                                            â”‚
â”‚       â”‚                                                                     â”‚
â”‚       â”œâ”€â”€ NO â†’ No escrow needed                                             â”‚
â”‚       â”‚        (PSP holds, airline holds, user's bank holds)                â”‚
â”‚       â”‚                                                                     â”‚
â”‚       â””â”€â”€ YES â†’ Q2: For how long?                                           â”‚
â”‚                    â”‚                                                        â”‚
â”‚                    â”œâ”€â”€ <7 days â†’ Gray area, probably OK                     â”‚
â”‚                    â”‚             (But escrow is safer)                      â”‚
â”‚                    â”‚                                                        â”‚
â”‚                    â””â”€â”€ >7 days â†’ Q3: Can user redeem anytime?               â”‚
â”‚                                      â”‚                                      â”‚
â”‚                                      â”œâ”€â”€ YES â†’ E-money license required     â”‚
â”‚                                      â”‚         OR use escrow                â”‚
â”‚                                      â”‚                                      â”‚
â”‚                                      â””â”€â”€ NO â†’ Escrow recommended            â”‚
â”‚                                               (Clear release conditions)    â”‚
â”‚                                                                             â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

---

## Recommended Architecture

Based on all considerations, here's the optimal structure:

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚                                                                             â”‚
â”‚                         VAYAPAY RECOMMENDED ARCHITECTURE                    â”‚
â”‚                                                                             â”‚
â”œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚                                                                             â”‚
â”‚   TRAVEL FUND (Phase 1):                                                    â”‚
â”‚   â””â”€â”€ User self-reports savings progress                                    â”‚
â”‚       â””â”€â”€ VAYA = goal tracking software only                                â”‚
â”‚           â””â”€â”€ License: NONE                                                 â”‚
â”‚                                                                             â”‚
â”‚   TRAVEL FUND (Phase 2 - Optional):                                         â”‚
â”‚   â””â”€â”€ Partner with digital bank (GXBank, Boost Bank)                        â”‚
â”‚       â””â”€â”€ They provide savings account, VAYA provides UX                    â”‚
â”‚           â””â”€â”€ License: Partner's license covers this                        â”‚
â”‚                                                                             â”‚
â”‚   POOL COMMITMENT:                                                          â”‚
â”‚   â””â”€â”€ Escrow via licensed trustee (AmanahRaya/Pacific)                      â”‚
â”‚       â””â”€â”€ Funds flow: User â†’ PSP â†’ Trustee                                  â”‚
â”‚           â””â”€â”€ License: NONE for VAYA (trustee is licensed)                  â”‚
â”‚                                                                             â”‚
â”‚   SPLIT TRIP:                                                               â”‚
â”‚   â””â”€â”€ PSP pending capture (standard e-commerce)                             â”‚
â”‚       â””â”€â”€ 24-hour window for all payments                                   â”‚
â”‚           â””â”€â”€ License: NONE (PSP is licensed)                               â”‚
â”‚                                                                             â”‚
â”‚   INSTANT REFUND:                                                           â”‚
â”‚   â””â”€â”€ VAYA's corporate capital via PSP instant payout                       â”‚
â”‚       â””â”€â”€ License: NONE (normal treasury function)                          â”‚
â”‚                                                                             â”‚
â”‚   PAYMENTS:                                                                 â”‚
â”‚   â””â”€â”€ All via licensed PSP (iPay88/Stripe)                                  â”‚
â”‚       â””â”€â”€ VAYA = merchant only                                              â”‚
â”‚           â””â”€â”€ License: NONE                                                 â”‚
â”‚                                                                             â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

---

## Action Items

1. **Travel Fund**: Start with self-reporting (Phase 1). No integration needed.

2. **Pool Escrow**: 
   - Contact Pacific Trustees for quote (escrow specialist)
   - Contact AmanahRaya for enterprise deal
   - Negotiate batch processing and API development

3. **KYC Provider**: 
   - Evaluate Innov8tif (Malaysian company)
   - Or use PSP's built-in KYC

4. **Open Banking** (Future):
   - Monitor Finverse Malaysia expansion
   - Contact Brankas for MY timeline

5. **PSP Selection**:
   - iPay88 for comprehensive MY coverage
   - Confirm pending capture capabilities
   - Confirm direct-to-trustee routing

---

## Cost Summary

| Component | Setup Cost | Monthly Cost | Per-Transaction |
|-----------|------------|--------------|-----------------|
| PSP (iPay88) | RM 0 | RM 0 | 1.5-2.5% |
| Trustee (Escrow) | RM 2,000-5,000 | RM 1,000-5,000 | RM 2-10 (negotiable) |
| KYC Provider | RM 0 | Based on usage | RM 2-5/verification |
| Open Banking | TBD | TBD | $0.30-1.00/call |
| Travel Fund (Self-track) | RM 0 | RM 0 | RM 0 |

**Phase 1 Total: RM 3,000-10,000 setup + RM 2,000-7,000/month**

This is very achievable for a startup!

# TinyMUSH Implementation TODO

**Last Updated**: 2025-10-06 (Phase 5 Week 3 Complete)

## Development Standards

**⚠️ CRITICAL: Zero Tolerance for Compiler Warnings**
- All warnings emitted by the Rust compiler must be fixed before committing
- All warnings in unit tests must be resolved
- Use `cargo check` and `cargo test` to verify clean builds
- This policy applies to all phases and contributions

This checklist tracks hands-on work for the TinyMUSH project. It bridges the high-level roadmap in `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md` and the detailed specification in `docs/development/MUD_MUSH_DESIGN.md` so we can see the next actionable steps at a glance.

- **Plan reference**: `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md`
- **Design reference**: `docs/development/MUD_MUSH_DESIGN.md`
- **Progress tracker**: `docs/development/PHASE5_PROGRESS.md`
- **Branch**: `tinymush`

## Legend
- [ ] TODO – not started
- [~] In progress
- [x] Done

---

## Phase 0 — Project Discipline & Games Menu Foundations
(Ref: Plan §Phase 0, Design §Admin & GM Tools → Menu UX)

- [x] Confirm project hygiene
  - [x] Reconcile existing unstaged files (`Cargo.lock`, archived `MUD_MUSH_PROPOSAL.md`)
  - [x] Document TinyMUSH-specific contribution rules in `CONTRIBUTING.md`
  - [x] Script UTF-8 byte-length validation (reuse prior tooling / `just` recipe)
  - [x] Capture 200-byte compliance checklist in repo
  - [x] Stabilize TinyHack minimap fog-of-war test (eliminate flaky failures)
- [x] Enumerated G)ames submenu scaffolding
  - [x] Introduce registry (slug + label) for game doors
  - [x] Render numbered submenu entries (e.g., `1) TinyHack`, `2) TinyMUSH`)
  - [x] Accept `G` + number (and slug alias) for selection
  - [x] Add per-door feature flags in `GamesConfig`
  - [x] Unit tests for menu rendering & selection routing
  - [x] Update help text / documentation for new menu behavior
- [x] Logging & metrics groundwork (commit 2d07085)
  - [x] Add entry/exit telemetry hooks keyed by game slug
  - [x] Document expected dashboards / log fields (`docs/development/game_telemetry.md`)

## Phase 1 — Core Data Models & Persistence
(Ref: Plan §Phase 1, Design §§Technical Implementation, Embedded Database Options)

- [x] Create `src/tmush/` module layout (`state`, `storage`, `types`, `errors`) (commit d91483e)
- [x] Define core structs (`PlayerState`, `RoomRecord`, `ObjectRecord`, etc.) (commit d91483e)
- [x] Implement Sled namespaces (`players:*`, `rooms:*`, `objects:*`, `mail:*`, `logs:*`) (commit d91483e)
- [x] Serialization via `bincode` with schema versioning (commit d91483e)
- [x] Migration helpers (seed default world rooms for this game) (commit d91483e)
- [x] Unit tests for save/load round trips using temp directories (commit d91483e)
- [x] Developer docs describing schema (`docs/development/tmush_schema.md`) (commit d91483e)

## Phase 2 — Command Parser & Session Plumbing
(Ref: Plan §Phase 2, Design §§Command Routing, Session Lifecycle)

- [x] Extend command parser for TinyMUSH verbs (look, move, say, etc.) (commit 97a797d)
- [x] Integrate parser with session state machine (`SessionState::TinyMush`) (commit 97a797d)
- [x] Node ID → session mapping with per-mode rate limiting (commit 97a797d)
- [x] Latency simulation harness / tests (commit 97a797d)
- [x] Moderation hooks & logging of rejected inputs (per design security section) (commit 97a797d)

## Phase 3 — Room Navigation & World State
(Ref: Plan §Phase 3, Design §§World Map, Room Capacity)

- [x] Implement `seed_world` migration to load Old Towne Mesh into Sled (commit d91483e)
- [x] Movement command handlers with real room-to-room navigation (commit ad8cc80)
  - [x] `LOOK` command with detailed room descriptions and exits
  - [x] Movement commands (`north`, `south`, `east`, `west`, etc.) with exit validation
  - [x] `WHERE` command showing current location details
  - [x] `MAP` command with area overview and current location marker
- [x] Player state persistence across movement (room tracking)
- [x] Basic integration testing with TinyMUSH command parsing and session flows
- [x] Room manager with LRU caching + instancing (commit 3a7e9c4)
- [x] Capacity enforcement tests (standard, shop, social room limits) (commit 3a7e9c4)
- [x] Basic integration tests for TinyMUSH functionality validation (commit 3a7e9c4)

## Phase 4 — Social & Communication Systems
(Ref: Plan §Phase 4, Design §§Social Features, Async Communication, Help System)

- [x] Implement `say`, `whisper`, `pose`, `emote`, `ooc` — commit a06bafe
  - [x] SAY <text> / ' <text> - speak aloud to room players
  - [x] WHISPER <player> <text> / WHIS <player> <text> - private messages  
  - [x] EMOTE <action> / : <action> - perform actions
  - [x] POSE <pose> / ; <pose> - strike poses
  - [x] OOC <text> - out of character communication
  - [x] Room occupancy detection and feedback
  - [x] Input validation and comprehensive help system
- [x] Town Stump bulletin board with pagination & persistence — commit 6cecd07
- [x] In-game mail storage (Sled-backed) with quotas and cleanup tasks — commit bd6e7f1
  - [x] MAIL [folder] - view inbox/sent mail folders
  - [x] SEND <player> <subject> <message> - send private mail
  - [x] RMAIL <id> - read specific mail message (marks as read)
  - [x] DMAIL <id> - delete mail message from folder
  - [x] Mail persistence with sender/recipient folders
  - [x] Quota enforcement (auto-cleanup of old read messages)
  - [x] Unread message tracking and status management
  - [x] Comprehensive test coverage (5 tests) - mail_system_basic.rs
- [x] Help/tutorial command integration (contextual responses) — commit a06bafe
  - [x] HELP command with main menu and topic-specific help
  - [x] Help topics: COMMANDS, MOVEMENT, SOCIAL, BOARD, MAIL
  - [x] Contextual help integrated into all command handlers
  - [x] Help methods: help_main, help_commands, help_movement, help_social, help_bulletin, help_mail
- [x] Tests guaranteeing all outbound messages < 200 bytes — NEW: tests/tinymush_message_size.rs
  - [x] Manual truncation in bulletin and mail handlers
  - [x] Fortune command validated (≤ 200 bytes)
  - [x] Comprehensive test suite for all TinyMUSH command outputs (help text, currency, errors)
  - [x] All 6 help methods condensed and validated (≤ 200 bytes each)

## Phase 5 — Economy, Inventory, Shops
(Ref: Plan §Phase 5, Design §§Enhanced Economy, Dual Currency Systems, Inventory Management)

### ✅ Currency System Foundation (Week 1 - COMPLETE)
- [x] Dual currency system architecture — commits afe6ebe, 33543d9
  - [x] `CurrencyAmount` enum (Decimal { minor_units: i64 } | MultiTier { base_units: i64 })
  - [x] Decimal currency support (name, symbol, decimal_places configurable)
  - [x] MultiTier currency support (tier names/symbols, conversion ratios)
  - [x] Unified base_value() accessor for both systems
  - [x] World-level currency type preserved in transactions
  - [x] Zero floating-point arithmetic (integer-only storage)

### ✅ Decimal Currency System (Modern/Sci-Fi) — COMPLETE
- [x] Integer minor unit storage (cents-equivalent) — commit afe6ebe
- [x] Configurable currency name (Credits, MiniBucks, Euros, etc.)
- [x] Configurable symbol ($, €, ¤, ₡, etc.)
- [x] Configurable decimal places (0-9, default 2)
- [x] Display formatting (e.g., "$10.50", "¤123.45")
- [x] Input parsing for decimal amounts ("10.50", "10", "$5.50")
- [x] Debug formatting for 200-byte message constraints

### ✅ Multi-Tier Currency System (Fantasy/Medieval) — COMPLETE
- [x] Base copper unit storage with tier ratios — commit afe6ebe
- [x] Configurable tier names (platinum/gold/silver/copper)
- [x] Configurable tier symbols (pp/gp/sp/cp)
- [x] Configurable conversion ratios (default: 1pp=1M cp, 1gp=10k cp, 1sp=100cp)
- [x] Multi-denomination display (e.g., "15gp 25sp 30cp")
- [x] Input parsing for multi-tier amounts ("5g 3s 7c", "537 copper")
- [x] Auto-conversion between tiers (base units stored)

### ✅ Currency Conversion & Migration — COMPLETE (Partial)
- [x] Bidirectional conversion functions (decimal ↔ multi-tier) — commit afe6ebe
- [x] Standard conversion ratio: 100 copper = 1 major decimal unit
- [x] Precision preservation during conversion
- [ ] Admin world migration command (future Phase 8)
- [ ] Batch conversion for player wallets, items, shops, banks (future)
- [ ] Conversion validation and rollback capability (future)
- [ ] Migration audit logging (future Phase 8)

### ✅ Transaction Engine — COMPLETE (Week 1)
- [x] CurrencyAmount operations: add(), subtract(), can_afford() — commit afe6ebe
- [x] Atomic operations with saturation arithmetic (no overflow)
- [x] Transaction reasons enum (SystemGrant, Purchase, Sale, Quest, Trade, etc.)
- [x] Transaction execution with error handling
- [x] Transaction audit log with timestamps and reasons — commit afe6ebe
- [x] Storage methods: transfer, grant, deduct, bank deposit/withdraw
- [x] Transaction history viewing (get_player_transactions)
- [x] Transaction rollback capability (rollback_transaction)
- [x] Prevent currency duplication (atomic two-phase updates)
- [x] System mismatch error handling (InsufficientFunds, InvalidCurrency)
- [x] 12 comprehensive tests (all passing)

### ✅ Inventory System — COMPLETE (Week 2)
- [x] InventoryConfig with capacity (100) and weight (1000) limits — commits ff19fc6, 716041e
- [x] ItemStack struct: object_id, quantity, added_at timestamp
- [x] Item metadata: currency_value (CurrencyAmount), weight, stackable flag
- [x] Inventory operations: add_item, remove_item, has_item with validation
- [x] Automatic item stacking for identical objects
- [x] Weight and capacity enforcement
- [x] Inventory display formatting (< 200 bytes): format_inventory_compact
- [x] Item examination: format_item_examination
- [x] Prevent item duplication (atomic transaction-based transfers)
- [x] Inventory persistence across sessions (PlayerRecord.inventory_stacks)
- [x] Storage integration: player_add_item, player_remove_item, transfer_item
- [x] Command stubs: GET, DROP, INVENTORY (I), EXAMINE (X) — commit c7d8b5f
- [x] 19 tests (10 unit + 9 integration, all passing)
- [ ] Item quality/condition system for value degradation (future feature)

### ✅ Shop & Vendor System — COMPLETE (Week 3)
- [x] ShopRecord with inventory HashMap<String, ShopItem> — commits a22e66a, 8868d8d
- [x] ShopConfig: max_unique_items (50), max_item_quantity (999)
- [x] ShopItem: object_id, quantity (Option<u32> for infinite/limited)
- [x] Dynamic pricing with configurable markup (1.2x) and markdown (0.7x)
- [x] Stock management: infinite vs limited stock, reduce/increase
- [x] Restock system: thresholds, intervals (24h default), automatic restocking
- [x] CurrencyAmount integration (shops preserve Decimal/MultiTier type)
- [x] Shop operations: calculate_buy_price, calculate_sell_price
- [x] Transactions: process_buy, process_sell with validation
- [x] Shop persistence: put_shop, get_shop, list_shop_ids — commit c2695d4
- [x] Location queries: get_shops_in_location for room-based shops
- [x] Shop commands: BUY, SELL, LIST/WARES/SHOP — commit 2cbd47d
- [x] Display formatting: format_shop_listing, format_shop_item_detail
- [x] 13 tests (8 unit + 5 integration, all passing)
- [ ] Reputation discounts (future feature)
- [ ] Vendor NPC dialog integration (future Phase 6)
- [ ] Vendor scripting for specific merchants (Bakery, General Store, etc.) (future)

### Banking System — Week 4 (IN PROGRESS)
- [x] Bank deposit/withdraw methods (storage.rs) — commit afe6ebe
- [x] Account balance tracking: pocket + banked_currency fields
- [x] Bank command handlers (BALANCE, DEPOSIT, WITHDRAW) — commit e8f2199
- [x] Bank transfer between players (BTRANSFER command) — commit e8f2199
- [x] BALANCE/BAL - shows pocket and bank currency balances
- [x] DEPOSIT/DEP <amount> - deposits currency from pocket to bank
- [x] WITHDRAW/WITH <amount> - withdraws currency from bank to pocket
- [x] BTRANSFER/BTRANS <player> <amount> - bank-to-bank transfers
- [ ] Bank integration tests (tests/banking_system.rs) — IN PROGRESS
- [ ] Vault storage for items (limited slots)
- [ ] Interest/fees configuration (optional, world-level)
- [ ] Bank NPC integration at specific locations

### Player-to-Player Economy — Week 4 TODO
- [ ] TradeSession struct for P2P trading state
- [ ] TRADE command to initiate with target player
- [ ] OFFER command to propose items/currency
- [ ] ACCEPT/REJECT commands for confirmation
- [ ] Secure two-phase commit (atomic completion)
- [ ] Trade timeout and cancellation handling
- [ ] Trade audit logging
- [ ] THISTORY command to view past trades

### ✅ Testing & Validation — COMPLETE (Week 1-3)
- [x] Unit tests for both currency systems (12 tests) — commit afe6ebe
- [x] Integration tests for currency operations (12 tests passing)
- [x] Transaction rollback tests (test_transaction_rollback) — commit afe6ebe
- [x] Inventory unit tests (10 tests) — commit ff19fc6
- [x] Inventory integration tests (9 tests) — commit ff19fc6
- [x] Shop unit tests (8 tests) — commit a22e66a
- [x] Shop storage tests (5 tests) — commit c2695d4
- [x] Anti-duplication: atomic transfers prevent currency/item duplication
- [x] Debug formatting for 200-byte message compliance (uses {:?})
- [x] Transaction audit log verified in tests
- [ ] Economy stress test (10k simulated transactions) — Week 4 TODO
- [ ] Performance profiling for high-volume transactions — Week 5 TODO

### 📊 Phase 5 Status: **Week 1-3 COMPLETE, Week 4 IN PROGRESS** (263 tests passing)
- ✅ **Week 1**: Currency foundation (12 tests) — COMPLETE (commits afe6ebe, 33543d9)
- ✅ **Week 2**: Inventory core (19 tests) — COMPLETE (commits ff19fc6, 716041e, c7d8b5f)
- ✅ **Week 3**: Shop system (13 tests) — COMPLETE (commits a22e66a, 8868d8d, c2695d4, 2cbd47d)
- 🔄 **Week 4**: Banking commands (4 commands) — IN PROGRESS (commit e8f2199)
- **Total: 263 tests passing** (89 unit + 174 integration)
- **Next**: Complete Week 4 - Player trading system and banking tests

---

## Phase 6 — Quest, Tutorial, Progression
(Ref: Plan §Phase 6, Design §§Tutorial, Quests, Achievements, New Player Experience)

- [ ] Script Gazebo → Mayor → City Hall tutorial flow
- [ ] Quest engine (objectives, progress, rewards) with data templates
- [ ] Achievement/title subsystem & announcement messaging
- [ ] Companion NPC system (player-bound NPCs that follow and assist)
  - [ ] Companion types: horses, dogs, cats, familiars, mercenaries
  - [ ] Player binding and loyalty mechanics
  - [ ] Auto-follow movement between rooms
  - [ ] Periodic behaviors (idle chatter, alerts, skill assists)
  - [ ] Combat assistance and healing capabilities
  - [ ] Companion inventory and equipment slots
  - [ ] Feed/care requirements and happiness system
- [ ] Scenario tests for full tutorial completion transcript

## Phase 7 — Housing, Building, Triggers
(Ref: Plan §Phase 7, Design §§Housing, MUSH Building System, Triggers)

- [ ] Player housing instancing (apartments, hotel rooms) with permissions & quotas
- [ ] Builder commands (`/dig`, `/describe`, `/link`, `/setflag`)
- [ ] Trigger engine (safe DSL) with abuse prevention
- [ ] Security review / tests for runaway builders or scripting loops

## Phase 8 — Admin/GM Tooling & Observability
(Ref: Plan §Phase 8, Design §§Admin Tools, Logging, Backup & Recovery, Mesh Resilience)

- [ ] Admin console commands (player monitor, teleport, event creation)
- [ ] Structured logging pipelines (action, security, trade)
- [ ] Automated backup routines with restore drills
- [ ] Reconnect autosave/resume flow per mesh resilience section
- [ ] Currency system migration commands (decimal ↔ multi-tier conversion)

## Phase 9 — Performance, Polish, Go-Live Prep
(Ref: Plan §Phase 9, Design §§Performance Considerations, Success Criteria)

- [ ] Load testing with simulated latency/packet loss profiles
- [ ] Profiling & optimization of hot paths (storage, parser, networking)
- [ ] Comprehensive documentation updates (player, admin, developer)
- [ ] Launch checklist, rollback plan, telemetry dashboard sign-off

---

### Notes
- Update this checklist alongside implementation progress so it stays authoritative.
- Cross-link PRs/issues next to items as they're tackled.
- When an item reaches `[x]`, note the commit hash or PR number for traceability.
- See `docs/development/PHASE5_PROGRESS.md` for detailed Phase 5 progress tracking.

# TinyMUSH Implementation TODO

## Development Standards

**⚠️ CRITICAL: Zero Tolerance for Compiler Warnings**
- All warnings emitted by the Rust compiler must be fixed before committing
- All warnings in unit tests must be resolved
- Use `cargo check` and `cargo test` to verify clean builds
- This policy applies to all phases and contributions

This checklist## Phase 2 — Command Parser & Session Plumbing
(Ref: Plan §Phase 2, Design §§Command Routing, Session Lifecycle)

- [x] Extend command parser for TinyMUSH verbs (look, move, say, etc.) — commit 97a797d
- [x] Integrate parser with session state machine (`SessionState::TinyMush`) — commit 97a797d
- [x] Node ID → session mapping with per-mode rate limiting — commit 97a797d
- [ ] Latency simulation harness / tests
- [ ] Moderation hooks & logging of rejected inputs (per design security section)hand- 
- [ ] Implement `seed_world` migration to load Old Towne Mesh world into Sled-on work for the TinyMUSH project. It bridges the high-level roadmap in `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md` and the detailed specification in `docs/development/MUD_MUSH_DESIGN.md` so we can see the next actionable steps at a glance.

- **Plan reference**: `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md`
- **Design reference**: `docs/development/MUD_MUSH_DESIGN.md`
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
- [x] In-game mail storage (Sled-backed) with quotas and cleanup tasks — commit [current]
  - [x] MAIL [folder] - view inbox/sent mail folders
  - [x] SEND <player> <subject> <message> - send private mail
  - [x] RMAIL <id> - read specific mail message (marks as read)
  - [x] DMAIL <id> - delete mail message from folder
  - [x] Mail persistence with sender/recipient folders
  - [x] Quota enforcement (auto-cleanup of old read messages)
  - [x] Unread message tracking and status management
  - [x] Comprehensive test coverage (5 tests) - mail_system_basic.rs
- [ ] Help/tutorial command integration (contextual responses)
- [ ] Tests guaranteeing all outbound messages < 200 bytes

## Phase 5 — Economy, Inventory, Shops
(Ref: Plan §Phase 5, Design §§Enhanced Economy, Dual Currency Systems, Inventory Management)

### Currency System Foundation
- [ ] Dual currency system architecture
  - [ ] `CurrencySystem` enum (Decimal | MultiTier)
  - [ ] `DecimalCurrency` config struct (name, symbol, minor_units_per_major, decimal_places)
  - [ ] `MultiTierCurrency` config struct (tier names/symbols, conversion ratios)
  - [ ] `CurrencyAmount` enum with unified base_value() accessor
  - [ ] World-level currency configuration in TOML
  - [ ] Zero floating-point arithmetic (integer-only storage)

### Decimal Currency System (Modern/Sci-Fi)
- [ ] Integer minor unit storage (cents-equivalent)
- [ ] Configurable currency name (Credits, MiniBucks, Euros, etc.)
- [ ] Configurable symbol ($, €, ¤, ₡, etc.)
- [ ] Configurable decimal places (default 2)
- [ ] Display formatting (e.g., "$10.50", "¤123.45")
- [ ] Input parsing for decimal amounts
- [ ] Validation for 200-byte message constraints

### Multi-Tier Currency System (Fantasy/Medieval)
- [ ] Base copper unit storage with tier ratios
- [ ] Configurable tier names (platinum/gold/silver/copper)
- [ ] Configurable tier symbols (pp/gp/sp/cp)
- [ ] Configurable conversion ratios (default: 1pp=1M cp, 1gp=10k cp, 1sp=100cp)
- [ ] Multi-denomination display (e.g., "15gp 25sp 30cp")
- [ ] Input parsing for multi-tier amounts
- [ ] Auto-conversion between tiers

### Currency Conversion & Migration
- [ ] Bidirectional conversion functions (decimal ↔ multi-tier)
- [ ] Standard conversion ratio: 100 copper = 1 major decimal unit
- [ ] Precision preservation during conversion
- [ ] Admin world migration command
- [ ] Batch conversion for player wallets, items, shops, banks
- [ ] Conversion validation and rollback capability
- [ ] Migration audit logging

### Transaction Engine
- [ ] Unified `Transaction` struct for both currency systems
- [ ] Atomic operations: add(), subtract(), can_afford()
- [ ] Transaction reasons enum (Purchase, Sale, Quest, Trade, etc.)
- [ ] Transaction execution with rollback on failure
- [ ] Transaction audit log with timestamps
- [ ] Admin transaction history viewing
- [ ] Admin transaction rollback capability
- [ ] Prevent currency duplication exploits
- [ ] System mismatch error handling

### Inventory System
- [ ] Inventory struct with capacity and weight limits
- [ ] Item metadata: value (CurrencyAmount), weight, stackable flag
- [ ] Item quality/condition system for value degradation
- [ ] Inventory management commands: GIVE, TAKE, DROP, INVENTORY
- [ ] Prevent item duplication exploits (transaction-based transfers)
- [ ] Inventory persistence across sessions

### Shop & Vendor System
- [ ] Shop configuration with inventory and pricing (both currency systems)
- [ ] Dynamic pricing with stock levels and reputation discounts
- [ ] Vendor NPC integration with dialog and purchase flow
- [ ] Buy/sell ratio configuration per vendor
- [ ] Vendor scripting for Bakery, General Store, Blacksmith, etc.
- [ ] Stock management and restock scheduling
- [ ] Infinite stock items (basic supplies)

### Banking System
- [ ] Bank deposit/withdraw with ledger entries
- [ ] Account balance tracking per player
- [ ] Vault storage for items (limited slots)
- [ ] Bank transfer between players
- [ ] Interest/fees configuration (optional)
- [ ] Audit logging for all bank transactions

### Player-to-Player Economy
- [ ] Secure two-phase trading system
- [ ] Trade offer and counter-offer mechanics
- [ ] Currency + item exchanges in single transaction
- [ ] Trade confirmation from both parties
- [ ] Trade rollback on disconnect/timeout

### Testing & Validation
- [ ] Unit tests for each currency system independently
- [ ] Integration tests for currency conversion workflows
- [ ] Transaction rollback tests (failure recovery)
- [ ] Economy stress test (10k simulated transactions)
- [ ] Anti-duplication tests for items and currency
- [ ] 200-byte validation for all currency displays
- [ ] Transaction audit log verification

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

## Phase 9 — Performance, Polish, Go-Live Prep
(Ref: Plan §Phase 9, Design §§Performance Considerations, Success Criteria)

- [ ] Load testing with simulated latency/packet loss profiles
- [ ] Profiling & optimization of hot paths (storage, parser, networking)
- [ ] Comprehensive documentation updates (player, admin, developer)
- [ ] Launch checklist, rollback plan, telemetry dashboard sign-off

---

### Notes
- Update this checklist alongside implementation progress so it stays authoritative.
- Cross-link PRs/issues next to items as they’re tackled.
- When an item reaches `[x]`, note the commit hash or PR number for traceability.

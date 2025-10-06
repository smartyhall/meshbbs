# TinyMUSH Implementation Plan

_Last updated: 2025-10-05_

This document converts the design captured in `MUD_MUSH_DESIGN.md` into a disciplined implementation roadmap for the MeshBBS TinyMUSH feature set. It lays out phased milestones, guardrails, and verification steps to keep development focused, measurable, and tightly coupled to the specification.

## Guiding Principles

- **Specification-driven**: Every deliverable must trace back to a section or requirement in `MUD_MUSH_DESIGN.md`.
- **Incremental & testable**: End each phase with running code, automated checks, and documentation updates.
- **Mesh-first**: Honor the 200-byte message constraint, asynchronous delivery, and high-latency mesh behaviors throughout.
- **Audit-ready**: Maintain changelogs, structured commits, and validation logs for each phase.

## Phase Overview

| Phase | Focus | Primary Deliverables | Acceptance Gates |
|-------|-------|----------------------|------------------|
| 0 | Environment & discipline setup | Tooling checklist, contribution workflow updates | Branch created, plan approved |
| 1 | Core data models & persistence | Rust structs, Sled integration, serialization tests | Unit tests passing, schema migration docs |
| 2 | Command parser & session plumbing | Command dispatcher, routing, session state | Parser tests, latency simulation harness |
| 3 | Room navigation & world state foundation | Room graph engine, movement commands, persistence | Movement integration tests, 200-byte validation |
| 4 | Social & communication systems | Chat, emotes, mail, bulletin boards | Functional tests, moderation hooks |
| 5 | Economy, inventory, and shops | Currency handling, inventory CRUD, vendor logic | Transaction rollback tests, anti-dupe checks |
| 6 | Quest, tutorial, and progression | Tutorial flow, quest actors, achievements | Narrative scripts tested, telemetry dashboards |
| 7 | Housing, MUSH building, triggers | Room creation, permissions, builder tooling | Security review, abuse prevention tests |
| 8 | Admin/GM tooling & observability | Admin consoles, logging, backups | Admin workflows verified, recovery drill |
| 9 | Performance, polish, and go-live prep | Load tests, docs, launch checklist | Performance SLAs, release sign-off |

## Phase 0 – Project Discipline (Week 0)

**Objectives**
- Confirm `tinymush` branch strategy, commit message conventions, and review process.
- Establish a running checklist for 200-byte compliance validation (reuse scripts from prior work).
- Document testing matrix: unit, integration (`cargo test --tests`), scenario replay, and byte-check automation.

**Tasks**
1. Lock in branch naming (`feature/tinymush-*`), commit format (`tinymush: scope - summary`).
2. Configure `just` or shell scripts for:
   - Running full test suites with profiling.
   - Executing UTF-8 byte-length validation for sample payloads.
3. Update `CONTRIBUTING.md` with TinyMUSH-specific development expectations.
4. Inventory existing changes (e.g., `Cargo.lock`, `MUD_MUSH_DESIGN.md`) and ensure rebasing plan.
5. Specify the **enumerated G)ames submenu** contract:
   - Render games as numbered entries (e.g., `1) TinyHack`, `2) TinyMUSH`).
   - Accept `G` + number (and optional slug aliases) for selection.
   - Document fallback behavior when games are toggled off/on or when additional titles appear.
   - Update design notes so TinyMUSH leverages the shared numbering system from day one.

**Exit Criteria**
- Branch `tinymush` pushed (or ready) with baseline scaffolding.
- Engineering discipline artifacts stored in `docs/development`.

## Phase 1 – Core Data Models & Persistence (Weeks 1-2)

**Spec References**: Sections _Technical Implementation_, _Embedded Database Options_, _Player State_.

**Objectives**
- Implement foundational data structures (`PlayerState`, `RoomState`, `SessionState`, `InventoryItem`).
- Integrate Sled-backed persistence with serialization (prefer `bincode` or `serde_cbor`).
- Establish migration story for existing MeshBBS users.

**Tasks**
1. Define Rust modules under `src/mud/` (e.g., `state`, `storage`, `types`).
2. Implement Sled tree namespaces (`players`, `rooms`, `mail`, `logs`).
3. Create persistence API with transactional helpers (`save_player`, `load_room_graph`).
4. Write unit tests mocking Sled with temp directories (`tempfile`).
5. Generate docs (`cargo doc`) to confirm public API completeness.

**Exit Criteria**
- `cargo test` (unit-level) passes locally and in CI.
- Example scripts demonstrate save/load round-trips with schema checksum.
- Data structures documented in `docs/development` addendum (link back to design).

## Phase 2 – Command Parser & Session Plumbing (Weeks 2-3)

**Spec References**: _Command Routing_, _Session Lifecycle_, _Security & Moderation_.

**Objectives**
- Build command dispatch layer translating mesh input to engine actions.
- Implement session state machine (login, character creation, active play).
- Enforce rate limits and validation for command inputs.

**Tasks**
1. Extend existing parser or create new DSL for TinyMUSH commands.
2. Associate node IDs with active sessions; integrate with auth roles.
3. Add instrumentation for command latency and failure metrics.
4. Unit tests for command parsing (happy paths, malformed, abuse cases).
5. Byte-length validator integrated into parser tests.

**Exit Criteria**
- Parser handles at least 30 core commands (movement, look, say, inventory).
- Session transitions verified with integration test capturing DM transcripts.
- Moderation hooks (logging, anti-spam) wired but stubbed for later phases.

## Phase 3 – Room Navigation & World State (Weeks 3-5)

**Spec References**: _World Map: Old Towne Mesh_, _Room Capacity System_, _Movement Commands_.

**Objectives**
- Implement room graph loader for `Old Towne Mesh` layout seeded directly into Sled.
- Enforce capacity rules and room flags (safe, shop, instanced, etc.).
- Deliver core commands: `LOOK`, `EXAMINE`, `GO`, `WHERE`, `MAP` (textual).

**Tasks**
1. Build a one-shot `seed_world` migration that writes canonical rooms into the `rooms:world:*` Sled trees (mirrors design IDs).
2. Build room manager with caching + LRU eviction as per design.
3. Implement instancing logic for Gazebo, apartments, hotel rooms.
4. Integrate city map ASCII output with 200-byte compliance.
5. Integration tests simulating movement across all 51 rooms.

**Exit Criteria**
- Movement across town works; transcripts match spec (Mayor greeting, etc.).
- Capacity enforcement tested (entering full room returns proper message).
- Performance benchmark: 100 concurrent sessions moving stays within latency budget.

## Phase 4 – Social & Communication Systems (Weeks 5-6)

**Spec References**: _Social Features_, _Async Communication_, _Help System_.

**Objectives**
- Implement say/whisper, emotes, poses, bulletin boards, mail.
- Build tutorial/help command integration (contextual hints, `/help command`).

**Tasks**
1. `say`, `pose`, `emote`, `whisper`, `ooc` message handlers.
2. Bulletin board service (Town Stump) with 200-byte posts, pagination.
3. In-game mail storage with Sled tree, quotas.
4. Help system data loader (YAML/JSON) referencing design content.
5. Tests validating message formatting, DM routing, duplicates.

**Exit Criteria**
- Social commands produce correct transcripts (<200 bytes).
- Help/tutorial flows accessible from City Hall tutorial rooms.
- Logging and moderation capture communication events.

## Phase 5 – Economy, Inventory, Shops (Weeks 6-8)

**Spec References**: _Enhanced Economy_, _Dual Currency Systems_, _Inventory Management_, _Shops & Vendors_.

**Objectives**
- Implement **flexible dual currency system** supporting both decimal and multi-tier currencies
- Wire inventory slots, item metadata, and transaction engine
- Build vendor interactions (Bakery, General Store, Blacksmith, etc.)
- Add transaction logging, rollback capability, and anti-duplication safeguards

**Currency System Design** (new requirement)

The economy must support **two distinct currency systems** to accommodate different world themes:

1. **Decimal Currency** (modern/sci-fi): Single currency with configurable name, symbol, and decimal places
   - Examples: Credits ($), MiniBucks (¤), Euros (€)
   - Storage: Integer minor units (like cents) to avoid floating-point issues
   - Display: Configurable decimal places (typically 2)

2. **Multi-Tier Currency** (fantasy/medieval): Multiple denominations with conversion ratios
   - Examples: Platinum/Gold/Silver/Copper with configurable names
   - Storage: Base copper units with configurable tier ratios
   - Display: Multiple denominations (e.g., "15gp 25sp 30cp")

**World builders choose ONE system per world via configuration.**

**Tasks**

**Currency Foundation (Week 6, Days 1-2):**
1. Create `CurrencySystem` enum supporting both Decimal and MultiTier variants
2. Implement `DecimalCurrency` config struct:
   - name, symbol, minor_units_per_major, decimal_places
3. Implement `MultiTierCurrency` config struct:
   - tier names/symbols, conversion ratios (configurable, default: 1pp=1M cp, 1gp=10k cp, 1sp=100cp)
4. Create `CurrencyAmount` enum with unified base_value() accessor
5. Add currency config section to world settings (TOML)

**Storage & Conversion (Week 6, Days 3-4):**
6. Implement storage strategy:
   - Decimal: Store as integer minor units (cents)
   - Multi-tier: Store as base copper units
   - Both use i64 for range and signed math
7. Build conversion functions between systems:
   - Standard ratio: 100 copper = 1 major decimal unit
   - Bidirectional conversion preserving precision
8. Create admin conversion command for world migration
9. Unit tests for all conversion scenarios

**Transaction Engine (Week 6, Day 5 - Week 7, Day 2):**
10. Unified `Transaction` struct with system-agnostic operations:
    - add(), subtract(), can_afford() work for both systems
    - System mismatch errors for incompatible operations
11. Transaction reasons enum (Purchase, Sale, Quest, Trade, etc.)
12. Atomic transaction execution with rollback on failure
13. Transaction audit log with timestamps and reason tracking
14. Admin transaction history viewing and rollback capability

**Display & Parsing (Week 7, Days 3-4):**
15. Currency display formatting for both systems:
    - Decimal: Symbol + formatted number (e.g., "$10.50")
    - Multi-tier: Abbreviated tiers (e.g., "15gp 25sp 30cp")
16. Input parsing for player currency commands:
    - Decimal: Accept "10.50", "10", decimals optional
    - Multi-tier: Accept "15gp 25sp", "15g 25s", tier combos
17. Validation ensuring all displayed amounts fit 200-byte limit

**Inventory System (Week 7, Day 5 - Week 8, Day 2):**
18. Inventory struct with capacity and weight limits
19. Item metadata: value (CurrencyAmount), weight, stackable flag
20. Inventory management commands: GIVE, TAKE, DROP, INVENTORY
21. Item quality/condition system for value degradation
22. Prevent item duplication exploits (transaction-based item transfers)

**Shop & Vendor System (Week 8, Days 3-5):**
23. Shop configuration with inventory, pricing, and buy/sell ratios
24. Dynamic pricing with stock levels and reputation discounts
25. Vendor NPC integration with dialog and purchase flow
26. Bank system: deposits, withdrawals, vault storage
27. Player-to-player trading with two-phase commit

**Testing & Validation:**
28. Unit tests for each currency system independently
29. Integration tests for currency conversion workflows
30. Transaction rollback tests (failure recovery)
31. Economy stress test (10k simulated transactions)
32. Anti-duplication tests for items and currency
33. 200-byte validation for all currency displays

**Exit Criteria**
- ✅ Both currency systems functional with world-level configuration
- ✅ Currency conversion working bidirectionally with precision preserved
- ✅ Buying/selling validated via automated scenario tests in both systems
- ✅ Transaction logs persisted and viewable/rollbackable by admins
- ✅ Inventory limits enforced, item transfers atomic
- ✅ Economy stress test (10k transactions) within tolerances
- ✅ Zero floating-point arithmetic in currency calculations
- ✅ All currency displays fit 200-byte message constraint
- ✅ Admin currency migration command tested and documented

## Phase 6 – Quest, Tutorial, Progression (Weeks 8-9)

**Spec References**: _Tutorial_, _Quests_, _Achievements_, _New Player Experience_.

**Objectives**
- Script interactive tutorial (Gazebo → Mayor → City Hall circuit).
- Implement quest engine (objectives, rewards, progress tracking).
- Integrate achievement/titles subsystem.

**Tasks**
1. Quest definition format (JSON/YAML) with branching support.
2. Command handlers for `QUEST`, `ACCEPT`, `COMPLETE`.
3. Achievement tracker (badge awarding, notifications).
4. Tutorial autopilot (guiding messages, gating progression).
5. Automated scenario: new player completes tutorial end-to-end.

**Exit Criteria**
- Tutorial flows produce transcripts exactly matching spec.
- Quest journal persists across reconnects; achievements saved.
- Analytics instrumentation (completion rates) reporting.

## Phase 7 – Housing, Building, Triggers (Weeks 9-11)

**Spec References**: _Housing_, _MUSH Building System_, _Triggers_.

**Objectives**
- Deliver apartment/hotel instancing with personalization.
- Implement builder permissions, `/dig`, `/link`, `/describe` commands.
- Provide trigger scripting (simple event-action system as per spec).

**Tasks**
1. Housing manager for instanced spaces with persistence and quotas.
2. Builder role enforcement (Architect/Creator permissions).
3. Trigger engine with safe sandbox (limited scripting DSL).
4. Abuse prevention: rate limits, resource quotas, moderation hooks.
5. Tests for housing decoration, trigger creation, security boundaries.

**Exit Criteria**
- Builders can create rooms/objects within quotas; admins can audit.
- Trigger engine handles sample scenarios ("feep" button) safely.
- Housing data backed up and restorable via admin tools.

## Phase 8 – Admin/GM Tooling & Observability (Weeks 11-12)

**Spec References**: _Admin & GM Tools_, _Logging_, _Backup & Recovery_, _Mesh Resilience_.

**Objectives**
- Implement admin dashboards/commands (player monitor, teleport, event control).
- Complete logging (action, security, trade) with retention policies.
- Build automated backup routines and recovery scripts.

**Tasks**
1. `/admin` command suite with RBAC enforcement.
2. Log appenders for structured output (JSON Lines or protobuf).
3. Scheduled tasks for backups (cron-like within BBS runtime).
4. Reconnect auto-save/resume logic and tests.
5. Disaster recovery drill documentation.

**Exit Criteria**
- Admins can monitor, intervene, and audit all key systems.
- Backups run on schedule; restore test succeeds.
- Reconnect flow validated under simulated mesh disruptions.

## Phase 9 – Performance, Polish, Launch Prep (Weeks 12-14)

**Spec References**: _Performance Considerations_, _Success Criteria_.

**Objectives**
- Optimize for target concurrency (100 players, 500 rooms cached).
- Finalize documentation, tutorials, release notes.
- Conduct playtest, address feedback, prepare production rollout.

**Tasks**
1. Load testing using scripted clients (simulate high latency & packet loss).
2. Profiling hotspots; optimize serialization, caching, command dispatch.
3. Security pass: fuzzing inputs, checking quotas, verifying logs.
4. Update docs (`README`, `docs/development`) with user/admin guides.
5. Draft release checklist (launch steps, rollback plan, marketing notes).

**Exit Criteria**
- Performance metrics meet or exceed SLAs (latency, CPU, memory).
- All tests (unit/integration/e2e) green in CI.
- Sign-off from stakeholders; release branch created.

## Cross-Cutting Safeguards

- **Traceability**: Maintain a table mapping each feature to commits/tests.
- **Documentation**: Update relevant doc pages every phase.
- **Testing Cadence**: Daily `cargo test`; weekly full scenario replay.
- **Code Review**: At least one peer review before merging to `tinymush` feature branches.
- **Risk Log**: Track technical and timeline risks in `docs/development/tinymush_risks.md` (to be created in Phase 0).

## Monitoring Progress

- Maintain Kanban board with cards reflecting phase tasks.
- Weekly sync notes summarizing achievements, blockers, next goals.
- Telemetry dashboard for early metrics (sessions, command volume, latency).

## Definition of Done (Project Level)

1. All phases completed with exit criteria met.
2. TinyMUSH feature integrated into mainline MeshBBS with documentation.
3. Administrators trained and comfortable with tooling.
4. Launch communication plan executed.
5. Post-launch monitoring active with rollback plan rehearsed.

---

This plan should be revisited at the end of each phase to adjust timelines, re-prioritize as needed, and ensure alignment with the evolving MeshBBS roadmap.

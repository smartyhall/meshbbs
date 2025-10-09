# TinyMUSH Implementation TODO

**Last Updated**: 2025-10-08 (Phase 6 Week 3 COMPLETE - Achievement & Title System)

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

## Phase 0 — Project Discipline & Games Menu Foundations ✅ COMPLETE
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

## Phase 1 — Core Data Models & Persistence ✅ COMPLETE
(Ref: Plan §Phase 1, Design §§Technical Implementation, Embedded Database Options)

- [x] Create `src/tmush/` module layout (`state`, `storage`, `types`, `errors`) (commit d91483e)
- [x] Define core structs (`PlayerState`, `RoomRecord`, `ObjectRecord`, etc.) (commit d91483e)
- [x] Implement Sled namespaces (`players:*`, `rooms:*`, `objects:*`, `mail:*`, `logs:*`) (commit d91483e)
- [x] Serialization via `bincode` with schema versioning (commit d91483e)
- [x] Migration helpers (seed default world rooms for this game) (commit d91483e)
- [x] Unit tests for save/load round trips using temp directories (commit d91483e)
- [x] Developer docs describing schema (`docs/development/tmush_schema.md`) (commit d91483e)

## Phase 2 — Command Parser & Session Plumbing ✅ COMPLETE
(Ref: Plan §Phase 2, Design §§Command Routing, Session Lifecycle)

- [x] Extend command parser for TinyMUSH verbs (look, move, say, etc.) (commit 97a797d)
- [x] Integrate parser with session state machine (`SessionState::TinyMush`) (commit 97a797d)
- [x] Node ID → session mapping with per-mode rate limiting (commit 97a797d)
- [x] Latency simulation harness / tests (commit 97a797d)
- [x] Moderation hooks & logging of rejected inputs (per design security section) (commit 97a797d)

## Phase 3 — Room Navigation & World State ✅ COMPLETE
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

## Phase 4 — Social & Communication Systems ✅ COMPLETE
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
  - [x] Tests guaranteeing all outbound messages < 200 bytes — commit 10968d4
  - [x] Manual truncation in bulletin and mail handlers
  - [x] Fortune command validated (≤ 200 bytes)
  - [x] Comprehensive test suite for all TinyMUSH command outputs (help text, currency, errors)
  - [x] All 6 help methods condensed and validated (≤ 200 bytes each)## Phase 5 — Economy, Inventory, Shops ✅ COMPLETE
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

### ✅ Currency Conversion & Migration — COMPLETE (Week 1)
- [x] Bidirectional conversion functions (decimal ↔ multi-tier) — commit afe6ebe
- [x] Standard conversion ratio: 100 copper = 1 major decimal unit
- [x] Precision preservation during conversion

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

### ✅ Banking System — COMPLETE (Week 4)
- [x] Bank deposit/withdraw methods (storage.rs) — commit afe6ebe
- [x] Account balance tracking: pocket + banked_currency fields
- [x] Bank command handlers (BALANCE, DEPOSIT, WITHDRAW) — commit e8f2199
- [x] Bank transfer between players (BTRANSFER command) — commit e8f2199
- [x] BALANCE/BAL - shows pocket and bank currency balances
- [x] DEPOSIT/DEP <amount> - deposits currency from pocket to bank
- [x] WITHDRAW/WITH <amount> - withdraws currency from bank to pocket
- [x] BTRANSFER/BTRANS <player> <amount> - bank-to-bank transfers

### ✅ Player-to-Player Trading — COMPLETE (Week 5)
- [x] TradeSession struct for P2P trading state — commit 8a240f6
- [x] Trade session storage and management (storage.rs) — commit 8a240f6
- [x] TRADE <player> command to initiate with target player — commit 59414e6
- [x] OFFER <item/amount> command to propose items/currency — commit 59414e6
- [x] ACCEPT command for confirmation (two-phase commit) — commit 59414e6
- [x] REJECT command to cancel/reject trade — commit 59414e6
- [x] Secure two-phase commit (atomic completion with validation) — commit 59414e6
- [x] Trade timeout and expiration handling (5-minute default) — commit 59414e6
- [x] THISTORY command to view past trades (last 10) — commit 59414e6

### ✅ Testing & Validation — COMPLETE (Weeks 1-5)
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
- [ ] Economy stress test (10k simulated transactions) — Future enhancement
- [ ] Performance profiling for high-volume transactions — Future enhancement

### 📊 Phase 5 Status: **COMPLETE** (263 tests passing)
- ✅ **Week 1**: Currency foundation (12 tests) — COMPLETE (commits afe6ebe, 33543d9)
- ✅ **Week 2**: Inventory core (19 tests) — COMPLETE (commits ff19fc6, 716041e, c7d8b5f)
- ✅ **Week 3**: Shop system (13 tests) — COMPLETE (commits a22e66a, 8868d8d, c2695d4, 2cbd47d)
- ✅ **Week 4**: Banking system (4 commands) — COMPLETE (commit e8f2199)
- ✅ **Week 5**: P2P trading (5 commands) — COMPLETE (commits 8a240f6, 59414e6)
- **Total: 263 tests passing** (89 unit + 174 integration)

---

## Phase 6 — Quest, Tutorial, Progression & Content Systems
(Ref: Plan §Phase 6, Design §§Tutorial, Quests, Achievements, New Player Experience, Companion NPCs)

### ✅ Tutorial System (Week 1) — COMPLETE
- [x] Tutorial data structures (TutorialState, TutorialStep enums) — commit 5a4e920
- [x] NPC storage methods (put_npc, get_npc, get_npcs_in_location) — commit bad31e9
- [x] Mayor's Office room with Mayor Thompson NPC — commit 839e311
- [x] Tutorial progression logic (9 functions, 10 unit tests) — commit 9d7d07f
  - [x] start_tutorial, advance_tutorial_step, skip_tutorial, restart_tutorial
  - [x] distribute_tutorial_rewards, get_tutorial_hint, format_tutorial_status
  - [x] should_auto_start_tutorial, can_advance_from_location
- [x] TUTORIAL command (show/SKIP/RESTART/START with aliases) — commit 1ae937a
- [x] TALK <npc> command with Mayor Thompson dialog — commit 1ae937a
- [x] Tutorial reward distribution (100cp/$10 + Town Map item) — commit 9d7d07f
- [x] Script Gazebo → Town Square → City Hall Lobby → Mayor's Office flow
- [x] Tutorial state tracking (progress, completion) with Sled persistence
- [x] Location-based progression validation (can_advance_from_location)
- [x] Comprehensive integration tests (8 tests) — commit faa40e4
  - [x] test_complete_tutorial_flow (full walkthrough)
  - [x] test_tutorial_skip_flow, test_tutorial_restart_flow
  - [x] test_npc_persistence_and_queries, test_tutorial_status_messages
  - [x] test_tutorial_rewards_decimal_currency, test_tutorial_cannot_double_reward
  - [x] test_location_based_progression_validation
- **Total: 281 tests passing** (89 unit + 192 integration)

### ✅ Quest Engine (Week 2) — COMPLETE
- [x] Quest data structures (QuestState, ObjectiveType, QuestObjective, QuestRewards, QuestRecord, PlayerQuest) — commit 1db20d2
- [x] Quest storage methods (put_quest, get_quest, list_quest_ids, get_quests_by_npc, delete_quest) — commit a3baf24
- [x] Quest progression logic (9 functions, 9 unit tests) — commit 863ad44
  - [x] can_accept_quest, accept_quest, update_quest_objective, complete_quest
  - [x] abandon_quest, get_available_quests, get_active_quests, get_completed_quests
  - [x] format_quest_status, format_quest_list
- [x] QUEST/ABANDON command handlers with subcommands — commit 554d6bb
  - [x] QUEST LIST - show available quests
  - [x] QUEST ACCEPT <id> - accept quest
  - [x] QUEST <id> - show status
  - [x] ABANDON <id> - abandon active quest
- [x] Starter quest content (3 quests: welcome_towne → market_exploration → network_explorer) — commit a1f2f7d
- [x] Quest seeding on initialization (seed_quests_if_needed) — commit a1f2f7d
- [x] Prerequisite chain support (quest dependencies)
- [x] Reward distribution (currency, XP, items, reputation)
- [x] Comprehensive integration tests (7 tests) — commit ad9aecd
  - [x] test_quest_lifecycle_complete_flow (full quest completion)
  - [x] test_quest_prerequisites_enforced (chain validation)
  - [x] test_quest_list_formatting_under_200_bytes
  - [x] test_quest_status_formatting_under_200_bytes
  - [x] test_quest_cannot_be_accepted_twice (duplicate prevention)
  - [x] test_quest_rewards_include_items
  - [x] test_abandoned_quest_can_be_retaken
- [x] Bug fix: complete_quest now distributes rewards before marking complete
- **Total: 298 tests passing** (90 unit + 208 integration)

### ✅ Achievement & Title System (Week 3) — COMPLETE
- [x] Achievement data structures (AchievementCategory, AchievementTrigger, AchievementRecord, PlayerAchievement) — commit e444654
  - [x] 7 categories: Combat, Exploration, Social, Economic, Crafting, Quest, Special
  - [x] 10 trigger types: KillCount, RoomVisits, FriendCount, QuestCompletion, CurrencyEarned, CraftCount, TradeCount, MessagesSent, VisitLocation, CompleteQuest
  - [x] Optional title awards, hidden achievement support
- [x] Achievement storage layer (put/get/list/filter/delete methods) — commit 9bb316a
- [x] Starter achievement content (15 achievements across all categories) — commit 9bb316a
  - [x] Combat: First Blood, Veteran, Legendary Warrior
  - [x] Exploration: Wanderer, Explorer, Cartographer
  - [x] Social: Friendly, Popular, Chatterbox
  - [x] Economic: Merchant, Wealthy
  - [x] Quest: Quest Beginner, Quest Veteran
  - [x] Special: Town Founder, Network Pioneer
- [x] Achievement tracking logic (6 functions, 8 unit tests) — commit 358e27f
  - [x] check_trigger - automatic progress from game events
  - [x] update_achievement_progress - manual progress updates
  - [x] award_achievement - immediate grant (bypass progress)
  - [x] get_earned_achievements - retrieval filtering
  - [x] get_available_achievements - visibility rules
  - [x] get_achievements_by_category - category filtering
- [x] ACHIEVEMENTS command with subcommands — commit 2141149
  - [x] ACHIEVEMENTS LIST (default) - all achievements with progress
  - [x] ACHIEVEMENTS EARNED - completed only
  - [x] ACHIEVEMENTS <CATEGORY> - filter by Combat/Exploration/etc
- [x] TITLE command with subcommands — commit 2141149
  - [x] TITLE LIST (default) - show unlocked titles
  - [x] TITLE EQUIP <name> - equip title
  - [x] TITLE UNEQUIP - remove title
- [x] Title system integration with PlayerRecord.equipped_title
- [x] Progress percentage display and completion markers
- [x] Hidden achievement visibility (hidden until earned)
- [x] Comprehensive integration tests (13 tests) — commit 2836066
  - [x] test_achievement_earning_flow
  - [x] test_incremental_progress
  - [x] test_hidden_achievements
  - [x] test_title_awarding
  - [x] test_title_equipping
  - [x] test_category_filtering
  - [x] test_social_achievements
  - [x] test_economic_achievements
  - [x] test_quest_achievements
  - [x] test_special_achievements
  - [x] test_achievement_command_output_size
  - [x] test_multiple_simultaneous_achievements
  - [x] test_achievement_persistence
- **Total: 318 tests passing** (98 unit + 220 integration)

### 🔄 Companion NPC System (Week 4-5) — IN PROGRESS
- [x] Companion data structures (CompanionType, CompanionBehavior, CompanionRecord) — commit 0962f24
  - [x] 6 companion types: Horse, Dog, Cat, Familiar, Mercenary, Construct
  - [x] 7 behavior types: AutoFollow, IdleChatter, AlertDanger, CombatAssist, Healing, ExtraStorage, SkillBoost
  - [x] State tracking: owner, room_id, loyalty (0-100), happiness (0-100), last_fed, behaviors, inventory, is_mounted
  - [x] Helper methods: needs_feeding(), feed(), pet(), apply_happiness_decay(), has_auto_follow(), storage_capacity()
  - [x] PlayerRecord integration: companions list, mounted_companion field
- [x] Companion storage layer (8 methods) — commit 4161bcc
  - [x] put_companion, get_companion, list_companion_ids
  - [x] get_companions_in_room, get_player_companions, get_wild_companions_in_room
  - [x] delete_companion, seed_companions_if_needed
- [x] Starter companion content (3 companions) — commit 4161bcc
  - [x] gentle_mare (Horse) at south_market
  - [x] loyal_hound (Dog) at town_square
  - [x] shadow_cat (Cat) at mesh_museum
- [x] Companion behavior logic module (Step 3) — commit b05f007
  - [x] Taming/bonding mechanics (tame_companion, release_companion)
  - [x] Auto-follow on room movement (auto_follow_companions, move_companion_to_room)
  - [x] Feed companion (feed_companion - happiness increase)
  - [x] Pet companion (pet_companion - loyalty increase)
  - [x] Mount/dismount horse mechanics (mount_companion, dismount_companion)
  - [x] Utility functions (find_companion_in_room, get_player_companions, format_companion_status, format_companion_list)
  - [x] 10 unit tests covering all interaction mechanics
- [x] COMPANION command implementation (Step 4) — commit 6b023dc
  - [x] COMPANION/COMP (list player's companions with loyalty/happiness bars)
  - [x] COMPANION <name> (view detailed companion status)
  - [x] COMPANION TAME <name> (claim wild companion in current room)
- [x] Additional companion commands (Step 5) — commit 6b023dc
  - [x] FEED <companion> - maintain happiness (shows gain +X)
  - [x] PET <companion> - increase loyalty (shows gain +X)
  - [x] MOUNT <companion> - mount horse (type checking)
  - [x] DISMOUNT - dismount from horse (returns name)
- [x] Extended companion commands (Step 5 continued) — commit b0a58c2
  - [x] TRAIN <companion> <skill> - skill development system with type-specific skills
  - [x] COMPANION STAY - toggle off auto-follow behavior
  - [x] COMPANION COME - summon companions to player location
  - [x] COMPANION INVENTORY/INV - view companion storage
  - [x] COMPANION RELEASE <name> - release companion back to wild
- [x] Help system documentation — commit f0fd98b
  - [x] HELP COMPANION topic with concise command reference
- [ ] Integration tests (Step 6)
  - [ ] Test taming wild companions in different rooms
  - [ ] Test feeding/petting mechanics and stat gains
  - [ ] Test mounting/dismounting horses  
  - [ ] Test auto-follow behavior on room movement
  - [ ] Test STAY/COME companion control
  - [ ] Test TRAIN skill system with loyalty requirements
  - [ ] Test RELEASE companion flow
  - [ ] Test command output stays under 200 bytes
  - [ ] Test companion persistence across sessions
- **Current: 328 tests passing** (98 unit + 220 integration + 10 companion)

**Companion System Summary:**
- **10 commands**: COMPANION (7 subcommands), FEED, PET, MOUNT, DISMOUNT, TRAIN
- **6 companion types** with type-specific skills and behaviors
- **Full lifecycle**: Tame → Feed/Pet → Train → Mount/Release
- **Auto-follow** system with STAY/COME control
- **Loyalty/happiness** mechanics with stat decay
- **Inventory storage** with type-specific capacity
- **Help integration**: HELP COMPANION topic

---

## Phase 7 — Housing, Building, World Creation
(Ref: Plan §Phase 7, Design §§Housing, MUSH Building System, Triggers)

### Player Housing (Week 1-2)
- [ ] Player housing instancing (apartments, hotel rooms)
- [ ] Housing permissions system (owner, guests, public)
- [ ] Housing quotas (room limits per player)
- [ ] Apartment/room templates with default descriptions
- [ ] Furniture and decoration objects
- [ ] Housing commands:
  - [ ] HOME - teleport to owned housing
  - [ ] RENT <room> - acquire housing
  - [ ] DESCRIBE HOME <text> - customize room description
  - [ ] INVITE <player> / UNINVITE <player> - guest management
  - [ ] LOCK / UNLOCK - access control

### Builder Commands (Week 3-4)
- [ ] Builder permission system (builder rank/flag)
- [ ] `/DIG <direction> <room_name>` - create new room
- [ ] `/DESCRIBE <target> <text>` - set descriptions
- [ ] `/LINK <direction> <destination>` - create exits
- [ ] `/UNLINK <direction>` - remove exits
- [ ] `/SETFLAG <target> <flag>` - modify object flags
- [ ] `/CREATE <object>` - create new objects
- [ ] `/DESTROY <object>` - delete objects (with safeguards)
- [ ] Builder undo/redo system
- [ ] Builder audit log for all creation/modification

### Trigger Engine (Week 5-6)
- [ ] Trigger DSL design (safe, sandboxed)
- [ ] Trigger types: OnEnter, OnLook, OnTake, OnDrop, OnUse, OnPoke
- [ ] Trigger execution engine
- [ ] Trigger variable substitution ($player, $object, etc.)
- [ ] Trigger actions: message, teleport, grant_item, spawn_mob
- [ ] Trigger condition evaluation (has_item, flag_set, etc.)
- [ ] Abuse prevention: execution limits, resource quotas
- [ ] Security review for trigger sandboxing
- [ ] Tests for runaway triggers and infinite loops

---

## Phase 8 — Admin/GM Tooling, Observability & World Management
(Ref: Plan §Phase 8, Design §§Admin Tools, Logging, Backup & Recovery, Mesh Resilience)

### Admin Console Commands (Week 1)
- [ ] Admin permission system (admin rank/flag)
- [ ] Player monitoring commands:
  - [ ] `/PLAYERS` - list all online players
  - [ ] `/WHERE <player>` - locate player
  - [ ] `/GOTO <player|room>` - teleport admin
  - [ ] `/SUMMON <player>` - teleport player to admin
  - [ ] `/BOOT <player>` - disconnect player
  - [ ] `/BAN <player>` - ban player access
- [ ] World event creation:
  - [ ] `/ANNOUNCE <message>` - broadcast to all players
  - [ ] `/EVENT <type> <params>` - trigger world events
  - [ ] `/SPAWN <mob> <location>` - create temporary NPCs

### Currency System Migration (Week 2)
- [ ] Admin migration command: `/CONVERT_CURRENCY <decimal|multitier>`
- [ ] Batch conversion for all player wallets
- [ ] Batch conversion for all item values
- [ ] Batch conversion for all shop inventories
- [ ] Batch conversion for all bank accounts
- [ ] Conversion validation and integrity checks
- [ ] Migration rollback capability
- [ ] Migration audit logging with before/after values
- [ ] Dry-run mode for testing conversion

### Logging & Observability (Week 3-4)
- [ ] Structured logging pipelines:
  - [ ] Action log (player commands, movement)
  - [ ] Security log (failed auth, abuse attempts)
  - [ ] Trade log (all economic transactions)
  - [ ] Admin log (admin commands, world modifications)
- [ ] Log retention policies (rotation, compression)
- [ ] Log query interface for admins
- [ ] Real-time monitoring dashboard (Grafana integration)
- [ ] Alert system for suspicious activity

### Backup & Recovery (Week 5)
- [ ] Automated backup routines (Sled database snapshots)
- [ ] Backup retention policies (daily, weekly, monthly)
- [ ] Backup verification tests
- [ ] Restore procedures and drills
- [ ] Point-in-time recovery capability
- [ ] Disaster recovery documentation

### Mesh Resilience (Week 6)
- [ ] Reconnect autosave/resume flow
- [ ] Session recovery after disconnection
- [ ] Graceful degradation under high latency
- [ ] Message queue persistence for offline players
- [ ] Automatic retry for failed operations
- [ ] Connection health monitoring

---

## Phase 9 — Performance, Polish, Go-Live Prep
(Ref: Plan §Phase 9, Design §§Performance Considerations, Success Criteria)

### Performance & Load Testing (Week 1-2)
- [ ] Load testing with simulated latency/packet loss profiles
- [ ] Concurrent player simulation (10, 50, 100+ players)
- [ ] Economy stress test (10k simulated transactions)
- [ ] Database performance profiling
- [ ] Network traffic analysis
- [ ] Bottleneck identification and optimization

### Optimization (Week 3-4)
- [ ] Profiling of hot paths (storage, parser, networking)
- [ ] Sled query optimization
- [ ] Cache tuning (room manager, object cache)
- [ ] Memory usage optimization
- [ ] Message serialization optimization
- [ ] Connection pool tuning

### Documentation (Week 5)
- [ ] Player documentation:
  - [ ] Getting started guide
  - [ ] Command reference (complete)
  - [ ] World map and locations
  - [ ] Economy system guide
  - [ ] Quest and achievement guide
  - [ ] Housing and building guide
- [ ] Admin documentation:
  - [ ] Admin command reference
  - [ ] World management guide
  - [ ] Troubleshooting guide
  - [ ] Backup and recovery procedures
- [ ] Developer documentation:
  - [ ] Architecture overview (updated)
  - [ ] API reference (complete)
  - [ ] Database schema (complete)
  - [ ] Extension guide (quests, triggers, NPCs)

### Go-Live Preparation (Week 6)
- [ ] Launch checklist completion
- [ ] Rollback plan documentation
- [ ] Emergency contact list
- [ ] Telemetry dashboard sign-off
- [ ] Beta testing with limited users
- [ ] Performance baseline establishment
- [ ] Security audit and penetration testing
- [ ] Final QA pass on all features

---

## Future Enhancements (Post-Launch)

### Economy Enhancements
- [ ] Item quality/condition system for value degradation
- [ ] Reputation discounts based on player standing
- [ ] Vendor NPC dialog integration
- [ ] Vendor scripting for specific merchants (Bakery, General Store, etc.)
- [ ] Bank vault storage for items (limited slots)
- [ ] Interest/fees configuration (optional, world-level)
- [ ] Bank NPC integration at specific locations
- [ ] Dynamic market prices based on supply/demand
- [ ] Auction house system
- [ ] Crafting system integration

### Social Features
- [ ] Player guilds/clans
- [ ] Guild chat channels
- [ ] Guild housing and shared resources
- [ ] Player reputation system
- [ ] Player-run shops and businesses
- [ ] In-game events and festivals

### Content Expansion
- [ ] Combat system (PvE and PvP)
- [ ] Magic system with spell casting
- [ ] Skills and leveling system
- [ ] Dungeon instances
- [ ] Boss encounters
- [ ] World events and dynamic content

### Technical Enhancements
- [ ] Web-based admin dashboard
- [ ] Metrics and analytics system
- [ ] A/B testing framework for features
- [ ] Localization support (multiple languages)
- [ ] Mobile app integration
- [ ] Voice chat integration for mesh networks

---

### Notes
- Update this checklist alongside implementation progress so it stays authoritative.
- Cross-link PRs/issues next to items as they're tackled.
- When an item reaches `[x]`, note the commit hash or PR number for traceability.
- See `docs/development/PHASE5_PROGRESS.md` for detailed Phase 5 progress tracking.
- Phase 5 is now COMPLETE with all 5 weeks finished and 263 tests passing.
- Next focus: Phase 6 Week 1 - Tutorial System implementation.

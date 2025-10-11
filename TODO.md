# TinyMUSH Implementation TODO

**Last Updated**: 2025-10-11 (Housing System 100% Complete!)

**Recent Achievement**: ✅ Housing System Fully Complete! All 387 tests passing:
- Background task integration for automated cleanup (daily checks)
- Notification system for abandonment warnings (30/60/80 days)
- Recurring housing payment system (monthly billing)
- @LISTABANDONED admin command fully implemented
- Automatic processing integrated into server housekeeping
- All infrastructure complete and tested

**Next Up**: Phase 9.5 - World Event Commands (Currency migration tools)

## Development Standards

**⚠️ CRITICAL: Zero Tolerance for Compiler Warnings**
- All warnings emitted by the Rust compiler must be fixed before committing
- All warnings in unit tests must be resolved
- Use `cargo check` and `cargo test` to verify clean builds
- This policy applies to all phases and contributions

**🚀 PERFORMANCE: Scale Target 500-1000 Users**
- ✅ All blocking database operations now async via spawn_blocking
- ✅ 27 async wrapper methods integrated into command processor
- ✅ Expected 5-10x performance improvement
- All database queries must be O(1) or O(log n) where possible
- Use secondary indexes for frequently accessed data
- Pagination required for list operations over 100 items
- Monitor and test at target scale during development

This checklist tracks hands-on work for the TinyMUSH project. It bridges the high-level roadmap in `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md` and the detailed specification in `docs/development/MUD_MUSH_DESIGN.md` so we can see the next actionable steps at a glance.

- **Plan reference**: `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md`
- **Design reference**: `docs/development/MUD_MUSH_DESIGN.md`
- **NPC Dialogue Design**: `docs/development/NPC_DIALOGUE_SYSTEM_DESIGN.md`
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
  - [x] All 6 help methods condensed and validated (≤ 200 bytes each)

## Phase 5 — Economy, Inventory, Shops ✅ COMPLETE
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
- [x] Tutorial auto-start for new players — commit b30650e
  - [x] Detects players with TutorialState::NotStarted on first command
  - [x] New players spawn at gazebo_landing (tutorial start location)
  - [x] Shows welcome message and tutorial instructions
  - [x] Seamlessly starts tutorial without manual TUTORIAL START
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

### ✅ Companion NPC System (Week 4-5) — COMPLETE
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
- [x] Integration tests (Step 6) — commits 8375fbc, f6cc1d5, 00767e0, 26d976a, c0375d7
  - [x] Test taming wild companions in different rooms
  - [x] Test feeding/petting mechanics and stat gains (loyalty/happiness)
  - [x] Test mounting/dismounting horses with state tracking
  - [x] Test STAY/COME companion control and movement
  - [x] Test TRAIN skill system with loyalty requirements
  - [x] Test RELEASE companion flow and state cleanup
  - [x] Test full lifecycle: discover → tame → care → move → release
  - [x] 6 integration tests covering end-to-end flows
- **Current: 343 tests passing** (98 unit + 235 integration + 10 companion unit)

**Companion System Summary:**
- **10 commands**: COMPANION (7 subcommands), FEED, PET, MOUNT, DISMOUNT, TRAIN
- **6 companion types** with type-specific skills and behaviors
- **Full lifecycle**: Tame → Feed/Pet → Train → Mount/Release
- **Auto-follow** system with STAY/COME control
- **Loyalty/happiness** mechanics with stat decay
- **Inventory storage** with type-specific capacity
- **Help integration**: HELP COMPANION topic

---

## Phase 6.5 — World Configuration & Customization ✅ COMPLETE
(Ref: Design principle - mutable strings should be database-stored, not hardcoded)

### World Configuration System (commits c3f0303, TBD)
- [x] WorldConfig data structure - **COMPLETE: 113 configurable fields for full i18n**
  - [x] 4 branding fields (welcome_message, motd, world_name, world_description)
  - [x] 7 help system templates (help_main through help_mail)
  - [x] 8 core error messages (err_say_what, err_emote_what, err_whisper_self, etc.)
  - [x] 5 core success messages (msg_deposit_success, msg_withdraw_success, etc.)
  - [x] **NEW: 7 validation & input error messages** (err_whisper_what, err_whisper_whom, err_pose_what, err_ooc_what, err_amount_positive, err_invalid_amount_format, err_transfer_self)
  - [x] **NEW: 16 empty state messages** (msg_empty_inventory, msg_no_companions, msg_no_quests, msg_no_achievements, etc.)
  - [x] **NEW: 7 shop error messages** (err_shop_no_sell, err_shop_doesnt_sell, err_shop_insufficient_funds, etc.)
  - [x] **NEW: 5 trading system messages** (err_trade_already_active, err_trade_partner_busy, msg_trade_accepted_waiting, etc.)
  - [x] **NEW: 2 movement messages** (err_movement_restricted, err_player_not_here)
  - [x] **NEW: 3 quest messages** (err_quest_cannot_accept, err_quest_not_found, msg_quest_abandoned)
  - [x] **NEW: 2 achievement messages** (err_achievement_unknown_category, msg_no_achievements_category)
  - [x] **NEW: 4 title messages** (err_title_not_unlocked, msg_title_equipped, msg_title_equipped_display, err_title_usage)
  - [x] **NEW: 4 companion messages** (msg_companion_tamed, err_companion_owned, err_companion_not_found, msg_companion_released)
  - [x] **NEW: 3 bulletin board messages** (err_board_location_required, err_board_post_location, err_board_read_location)
  - [x] **NEW: 3 NPC/tutorial messages** (err_no_npc_here, msg_tutorial_completed, msg_tutorial_not_started)
  - [x] **NEW: 13 technical/system messages** (err_player_load_failed, err_shop_save_failed, etc.)
- [x] TREE_CONFIG storage tree with get/put/update methods - **supports all 113 fields**
- [x] @SETCONFIG <field> <value> - update config (supports all 113 fields with categorized help)
- [x] @GETCONFIG [field] - view config (all or specific field)
- [x] Tutorial auto-start uses configurable welcome message
- [x] Help system migrated to load from WorldConfig instead of hardcoded functions
- [x] Error and success messages ready for migration to use configurable templates
- [x] Audit tracking (updated_by, updated_at timestamps)
- [x] Integration tests (9 tests covering all operations)

**World Config Benefits:**
- World creators can customize ALL user-facing text without source code edits
- **Complete Internationalization support**: All 113 user-facing strings database-backed
  - Create French, Spanish, German, Japanese, etc. worlds without code forks
  - Language packs can be distributed as WorldConfig JSON exports/imports
  - Error messages, help text, UI prompts, empty states, system messages all localizable
- Audit trail tracks who changed what and when
- Supports multi-world deployment with different configurations
- **Template variables** for dynamic content: {player}, {item}, {amount}, {quantity}, {error}, etc.
- Foundation for future customizable game text (achievements, quest text, NPC dialogue)

**Implementation Details:**
- `src/tmush/types.rs`: WorldConfig struct with **113 fields** and comprehensive documentation
- `src/tmush/storage.rs`: update_world_config_field() handles all 113 fields across 12 categories
- `src/tmush/commands.rs`: Command handlers ready to load world_config and use configurable strings
- Default implementation provides complete English language pack with sensible defaults
- Template variables (e.g., {amount}, {target}, {player}, {item}) support dynamic content in messages

**Future Extensibility:**
- All identified hardcoded strings now have configurable equivalents
- WorldConfig schema versioning enables smooth upgrades as new fields are added
- Export/import tools can enable community-created language packs and themed world configurations
- Next phase: Systematic migration of all command handlers to use WorldConfig fields

---

## Phase 7 — Housing, Building, World Creation
(Ref: Plan §Phase 7, Design §§Housing, MUSH Building System, Triggers)

### Player Housing (Week 1-2) — ✅ 100% COMPLETE!
- [x] Housing data structures (HousingPermissions, HousingTemplateRoom, HousingTemplate, HousingInstance) — types.rs
- [x] Housing storage trees (TREE_HOUSING_TEMPLATES, TREE_HOUSING_INSTANCES) — storage.rs
- [x] Housing CRUD methods (10 methods):
  - [x] get_housing_template, put_housing_template, list_housing_templates, delete_housing_template
  - [x] get_housing_instance, get_player_housing_instances, put_housing_instance, list_housing_instances, delete_housing_instance
  - [x] count_template_instances (for max_instances enforcement)
- [x] clone_housing_template method (creates rooms, preserves connectivity, maps template → instance IDs) — storage.rs
- [x] Housing abstraction system for multi-world support:
  - [x] RoomFlag::HousingOffice - marks locations providing housing services
  - [x] HousingTemplate.tags - filter templates by theme (["modern", "urban"], ["fantasy", "burrow"], etc.)
  - [x] HousingTemplate.category - grouping (apartment, house, burrow, treehouse, etc.)
  - [x] RoomRecord.housing_filter_tags - configure which templates each office shows
  - [x] HousingTemplate.matches_filter() - check if template matches office's tag filter
  - [x] WorldConfig housing messages (7 fields): err_housing_not_at_office, err_housing_no_templates, etc.
- [x] Housing template seeding (studio_apartment, basic_apartment, luxury_flat templates with tags)
- [x] Housing commands (all 9 commands complete):
  - [x] HOUSING - show player's owned housing status
  - [x] HOUSING LIST - show available templates catalog (location-restricted to HousingOffice rooms)
  - [x] RENT <template_id> - clone template to create player instance (COMPLETE - full validation, currency handling, location restrictions)
  - [x] HOME command (3-phase implementation) - ALL PHASES COMPLETE
  - [x] DESCRIBE <text> - customize current room description (COMPLETE)
  - [x] INVITE/UNINVITE <player> - guest management (COMPLETE)
  - [x] LOCK/UNLOCK - access control & object protection (7 phases COMPLETE)
  - [x] KICK <player> - remove guests from housing (COMPLETE)
  - [x] HISTORY <item> - view ownership audit trail (COMPLETE)
  - [x] RECLAIM - retrieve items from reclaim box (COMPLETE)
- [x] Housing lifecycle features:
  - [x] Reclaim box system (move_housing_to_reclaim_box helper)
  - [x] Abandonment tracking (inactive_since field)
  - [x] Cleanup system with check_and_cleanup_housing() (30/60/80/90 day thresholds)
  - [x] @LISTABANDONED command (admin visibility) - FULLY IMPLEMENTED
- [x] Background task integration for automated cleanup (scheduler/cron) - ✅ COMPLETE
  - [x] Integrated into server housekeeping loop (daily checks: 86400 seconds)
  - [x] housing_cleanup_last_check tracking
  - [x] Automatic processing via check_housing_cleanup()
- [x] Notification system for abandonment warnings (30/60/80 days) - ✅ COMPLETE
  - [x] WorldConfig notification messages (5 new fields added)
  - [x] Notification callback pattern for online player messaging
  - [x] 30-day warning: "Items will be moved to reclaim box"
  - [x] 60-day warning: "Housing marked for reclamation"
  - [x] 80-day final warning: "Reclaim box will be deleted at 90 days"
- [x] Housing cost deduction and payment system (recurring_cost implementation) - ✅ COMPLETE
  - [x] process_recurring_payments() function in housing_cleanup.rs
  - [x] Monthly billing cycle (every 30 days from last_payment)
  - [x] Automatic deduction from wallet first, then bank
  - [x] Payment failure handling (mark inactive, move to reclaim)
  - [x] Payment notifications (success/failure messages)
  - [x] Integrated into server housekeeping (daily checks)
  - [x] housing_payment_last_check tracking
- [x] Integration tests for housing lifecycle (10 tests passing)

**Housing System Status**: ✅ **100% COMPLETE!** All storage, commands, lifecycle management, notifications, and recurring payments fully implemented and tested. System is production-ready for alpha testing. 387 total tests passing!
- [x] Housing data structures (HousingPermissions, HousingTemplateRoom, HousingTemplate, HousingInstance) — types.rs
- [x] Housing storage trees (TREE_HOUSING_TEMPLATES, TREE_HOUSING_INSTANCES) — storage.rs
- [x] Housing CRUD methods (10 methods):
  - [x] get_housing_template, put_housing_template, list_housing_templates, delete_housing_template
  - [x] get_housing_instance, get_player_housing_instances, put_housing_instance, list_housing_instances, delete_housing_instance
  - [x] count_template_instances (for max_instances enforcement)
- [x] clone_housing_template method (creates rooms, preserves connectivity, maps template → instance IDs) — storage.rs
- [x] Housing abstraction system for multi-world support:
  - [x] RoomFlag::HousingOffice - marks locations providing housing services
  - [x] HousingTemplate.tags - filter templates by theme (["modern", "urban"], ["fantasy", "burrow"], etc.)
  - [x] HousingTemplate.category - grouping (apartment, house, burrow, treehouse, etc.)
  - [x] RoomRecord.housing_filter_tags - configure which templates each office shows
  - [x] HousingTemplate.matches_filter() - check if template matches office's tag filter
  - [x] WorldConfig housing messages (7 fields): err_housing_not_at_office, err_housing_no_templates, etc.
- [x] Housing template seeding (studio_apartment, basic_apartment, luxury_flat templates with tags)
- [ ] Housing commands:
  - [x] HOUSING - show player's owned housing status
  - [x] HOUSING LIST - show available templates catalog (location-restricted to HousingOffice rooms)
  - [x] RENT <template_id> - clone template to create player instance (COMPLETE - full validation, currency handling, location restrictions)
  - [ ] HOME command (3-phase implementation):
    - [x] **Phase 1 (MVP)**: Basic teleport to primary housing - COMPLETE
      - [x] Add PlayerRecord.primary_housing_id field
      - [x] Add PlayerRecord.last_teleport field for cooldown
      - [x] Add PlayerRecord.in_combat field
      - [x] Add RoomFlag::NoTeleportOut for teleport restrictions
      - [x] Add 6 WorldConfig teleport messages (err_teleport_in_combat, err_teleport_restricted, err_teleport_cooldown, err_no_housing, err_teleport_no_access, msg_teleport_success)
      - [x] HOME command: teleports to primary housing with validations
      - [x] Validation: Check in_combat flag → block with err_teleport_in_combat
      - [x] Validation: Check RoomFlag::NoTeleportOut → block with err_teleport_restricted
      - [x] Validation: Check cooldown (world_config.home_cooldown_seconds, default 300) → block with err_teleport_cooldown
      - [x] Validation: Check player has active housing → block with err_no_housing
      - [x] WorldConfig: home_cooldown_seconds field (default 300 = 5 minutes)
      - [x] Set last_teleport timestamp after successful teleport
    - [x] **Phase 2 (Polish)**: Multi-home management - COMPLETE
      - [x] HOME LIST - show all accessible housing instances with status
      - [x] Display format: [★ Primary] ID. Name (Category) - ACCESS_TYPE
      - [x] Show access types: OWNED, GUEST, GUILD, BUSINESS
      - [x] HOME <id> - teleport to specific housing instance by ID/number
      - [x] HOME SET <id> - designate a housing instance as primary home
      - [x] Update primary_housing_id when using HOME SET
      - [x] Auto-prompt if multiple housing and no primary set
      - [x] Number-based selection (HOME 1, HOME 2) or ID-based (HOME studio_apt_123)
    - [ ] **Phase 3 (Advanced)**: Extended access types — IN PROGRESS
      - [x] Guest access tracking (show instances where player is on guests list) - COMPLETE (commit 7f49cc3)
        - [x] get_guest_housing_instances() storage method
        - [x] HOME LIST shows owned + guest housing with unified numbering
        - [x] HOME <id> supports teleporting to guest housing
        - [x] Access type clearly marked: OWNED vs GUEST
      - [ ] Guild hall support (HousingInstance.access_type enum)
      - [ ] Business property support (owned businesses accessible via HOME)
      - [ ] Quest-based teleport restrictions (Quest.restrictions.allow_teleport)
      - [ ] Area-specific teleport rules (RoomFlag::QuestRestricted)
  - [x] DESCRIBE <text> - customize current room description (COMPLETE - commit 2130878)
    - [x] Check player is in a housing room they own or have guest access to
    - [x] Check HousingPermissions.can_edit_description flag
    - [x] Update RoomRecord.long_desc for current room
    - [x] Validate description length (500 char max)
    - [x] DESCRIBE (no args) - show current description and edit permissions
    - [x] Context-aware: works in current housing room only
    - [x] 5 WorldConfig messages for full i18n support
  - [x] INVITE <player> / UNINVITE <player> - guest management (COMPLETE - commit 42f0ca1)
    - [x] Add 7 WorldConfig fields for guest management messages
    - [x] Validate player owns housing and is in housing room
    - [x] Check target player exists
    - [x] Add/remove from HousingInstance.guests list
    - [x] Prevent duplicate invitations and invalid removals
  - [ ] LOCK / UNLOCK - access control & object protection
    - [x] **Phase 1: Data Model Changes** (COMPLETE - commit 7714f5b)
      - [x] Add ObjectRecord.owner field (Option<String>)
      - [x] Add ObjectRecord.locked field (bool, default false)
      - [x] Add ObjectRecord.ownership_history field (Vec<OwnershipTransfer>)
      - [x] Add RoomRecord.locked field (bool, default false)
      - [x] Add HousingInstance.reclaim_box field (Vec<String>)
      - [x] Add HousingInstance.inactive_since field (Option<DateTime<Utc>>)
      - [x] Create OwnershipTransfer struct (owner, timestamp, reason)
      - [x] Create OwnershipReason enum (Created, Purchased, Traded, etc.)
      - [x] Update all constructors and test helpers
      - [x] All 124 library tests passing
    - [x] **Phase 2: Room Access Control** (COMPLETE - commit a6a504a)
      - [x] LOCK command - lock current housing room (guests can't enter)
      - [x] UNLOCK command - unlock current housing room
      - [x] Check room.locked in movement validation (can_enter_room)
      - [x] Only owner and guests can enter locked rooms
      - [x] Command parsing for LOCK/UNLOCK
    - [x] **Phase 3: Guest Management** (COMPLETE - commit 1bfa0ae)
      - [x] KICK <player> - remove specific guest from current housing
      - [x] KICK ALL - remove all guests from current housing
      - [x] Teleport kicked players to town square
      - [x] Command parsing for KICK
      - [x] Owner-only validation and informative messages
    - [x] **Phase 4: Item Protection** (COMPLETE - commit db68d92)
      - [x] LOCK <item> - mark item as locked (guests can't take)
      - [x] UNLOCK <item> - remove lock from item
      - [x] Check item.locked ownership validation
      - [x] Show 🔒 indicator for locked items in inventory
      - [x] TODO added in TAKE command for future implementation
    - [x] **Phase 5: Ownership Tracking** (COMPLETE - commit 73ba7c7)
      - [x] Record ownership_history on item creation (new_player_owned)
      - [x] Record ownership_history on BUY (Purchased reason)
      - [x] Add reason codes enum (Created, Traded, Gifted, Dropped, PickedUp, etc.)
      - [x] HISTORY <item> command - view ownership audit trail (owner only)
      - [x] TODOs added for GIVE/TRADE/DROP/TAKE when implemented
      - [ ] Preserve history for 90 days after item deletion (storage layer TODO)
    - [x] **Phase 6: Reclaim Box System** (commit ca01a76)
      - [x] Move items to reclaim_box helper: move_housing_to_reclaim_box()
      - [x] Move people to town square on housing deletion (teleport loop)
      - [x] Return companions to owner's companion list (TODO in helper)
      - [x] RECLAIM command - view items in reclaim box (cross-housing display)
      - [x] RECLAIM <item> - retrieve item from reclaim box (with ownership tracking)
      - [x] WorldConfig messages for reclaim operations
      - [x] move_housing_to_reclaim_box() ready for future housing deletion command
      - [x] Ownership tracking integration (Reclaimed reason)
    - [x] **Phase 7: Abandonment/Cleanup** (commit 2b19752)
      - [x] housing_cleanup module with check_and_cleanup_housing()
      - [x] list_abandoned_housing() for admin visibility  
      - [x] @LISTABANDONED command (placeholder pending full Storage integration)
      - [x] Track inactive_since when owner doesn't login (checked against last_login)
      - [x] check_and_cleanup_housing() for periodic scans (30/60/80/90 day thresholds)
      - [x] Mark housing inactive at 30 days (sets inactive_since, active=false)
      - [x] Delete reclaim box items at 90 days (permanent deletion)
      - [x] AbandonedHousingInfo struct with status_message() helper
      - [x] CleanupConfig and CleanupStats for configuration and tracking
      - [ ] Background task integration (scheduler/cron for automated checks)
      - [ ] Notification system for warnings at 30/60/80 days
- [ ] Integration tests for housing lifecycle (template → rent → customize → guest access)
- [ ] Housing instance cleanup (inactive/abandoned housing reclamation)
- [ ] Housing cost deduction and payment system (recurring_cost for rental tracking)

**Housing System Architecture:**
- **Template-based**: World builders create HousingTemplate blueprints with multiple HousingTemplateRoom entries
- **Instancing**: Players rent/purchase → clone_housing_template() creates unique HousingInstance with actual RoomRecord copies
- **Connectivity**: Exit mappings preserved via room_mappings HashMap (template room ID → instance room ID)
- **Permissions**: HousingPermissions control what owners can customize (descriptions, objects, guests, building, flags, exits)
- **Limits**: max_instances per template (-1 = unlimited), cost/recurring_cost for economy integration
- **Multi-World Abstraction**: Tag-based filtering enables theme-appropriate housing across different worlds:
  - Modern city: apartments_lobby with ["modern", "urban"] tags → shows studio/apartment/flat
  - Fantasy world: tavern with ["fantasy", "medieval"] tags → shows cottage/manor/tavern_room
  - Furry world: burrow_warren with ["burrow", "underground"] tags → shows burrow variations
  - Same system works for: treehouses, underwater grottos, sky platforms, space stations, etc.
  - Location-restricted: Commands only work in rooms with RoomFlag::HousingOffice
- **Schema**: All housing data versioned (schema_version) for future extensibility

**Current Status: Storage + abstraction layer complete (124 tests passing), command handlers next**

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

## Phase 8 — Performance Optimization for Scale ✅ COMPLETE
(Target: 500-1000 concurrent users, Critical for alpha testing)

### Secondary Index Implementation (commit 51bf31e)
- [x] Add TREE_OBJECT_INDEX for O(1) object lookups by ID
  - [x] Maintain index in put_object() 
  - [x] Use index in get_object() with fallback scan for migration
  - [x] **Impact**: Object lookups O(n)→O(1), ~1000x faster at 10k objects
- [x] Add TREE_HOUSING_GUESTS for O(1) guest house lookups
  - [x] Maintain guest indexes in put_housing_instance()
  - [x] Clean up indexes in delete_housing_instance()
  - [x] Use index in get_guest_housing_instances()
  - [x] **Impact**: Guest searches O(n)→O(m), ~100x faster at 100 instances
- [x] Add TREE_PLAYER_TRADES for O(1) active trade lookups
  - [x] Maintain player trade indexes in put_trade_session()
  - [x] Clean up indexes in delete_trade_session()
  - [x] Use index in get_player_active_trade()
  - [x] **Impact**: Instant lookup vs full trades iteration
- [x] Add TREE_TEMPLATE_INSTANCES for O(1) template counting
  - [x] Maintain template indexes in put_housing_instance()
  - [x] Use index in count_template_instances()
  - [x] **Impact**: Template counting O(n)→O(k) where k=instances of template

### Index Maintenance & Migration (commit 51bf31e)
- [x] Add IndexStats struct for monitoring
- [x] Implement rebuild_all_indexes() for manual maintenance
- [x] Implement rebuild_object_index()
- [x] Implement rebuild_housing_guest_indexes()
- [x] Implement rebuild_template_instance_indexes()
- [x] Implement rebuild_player_trade_indexes()
- [x] Implement get_index_stats() for monitoring
- [x] Add automatic index rebuild in fallback scan paths (zero-downtime migration)

### Performance Validation (commit 51bf31e)
- [x] All 124 tests passing with new indexes
- [x] Fixed test fixtures for new object fields (locked, ownership_history)
- [x] Verified index maintenance during CRUD operations
- [x] Verified fallback scan paths auto-rebuild indexes

### Performance Characteristics
**Before Optimization:**
- Object lookup: O(n) where n = total player objects (could scan 10,000+ records)
- Guest housing: O(n) where n = total housing instances (deserializes all)
- Active trades: O(n) where n = total trade sessions (iterates all)
- Template counting: O(n) where n = total housing instances (scans all)

**After Optimization:**
- Object lookup: O(1) - single index hit + single read
- Guest housing: O(m) where m = houses where player is guest (typically < 10)
- Active trades: O(1) - instant player→session_id lookup
- Template counting: O(k) where k = instances of specific template

**Scale Impact:**
At 1000 users with 10 objects each (10,000 objects total):
- Object commands (TAKE/DROP/EXAMINE/LOCK/HISTORY): **~1000x faster**
- Guest house access: **~100x faster** 
- Trade operations: **Instant vs slow** at scale
- Housing rental (template check): **~100x faster**

### Async Integration (commit TBD - October 9, 2025) ✅ COMPLETE
- [x] Added 27 async wrapper methods to TinyMushStore using spawn_blocking
- [x] Integrated all async wrappers into command processor (26 call sites)
  - [x] Player operations: get_player_async, put_player_async (12 instances)
  - [x] Room operations: get_room_async, put_room_async (13 instances)
  - [x] Housing operations: list_housing_templates_async (1 instance)
  - [x] Mail operations: send_mail_async, get_mail_async, mark_mail_read_async, delete_mail_async (6 instances)
  - [x] Bulletin operations: post_bulletin_async (1 instance)
- [x] All 124 tests passing after integration
- [x] Zero compiler warnings maintained
- [x] **Impact**: Prevents async worker thread starvation, enables 500-1000+ concurrent users
- [x] **Expected improvement**: 5-10x throughput increase vs blocking operations

**Performance Characteristics:**
- **Before**: Blocking Sled ops on Tokio async workers → thread starvation → ~100-200 concurrent users
- **After**: Blocking ops on dedicated threadpool → no starvation → 500-1000+ concurrent users
- **Pattern**: `tokio::task::spawn_blocking(move || sync_operation())` + Arc-based Sled clones
- **Documentation**: `docs/development/ASYNC_INTEGRATION_COMPLETE.md`

---

## Phase 8.5 — Advanced NPC Dialogue System (Alpha Enhancement) ✅ COMPLETE!
(Ref: Design §NPC Dialogue System, `docs/development/NPC_DIALOGUE_SYSTEM_DESIGN.md`)

**Status**: ✅ COMPLETE & DEPLOYED  
**Priority**: High (UX improvement for Alpha)  
**Effort**: 15 hours actual (37-50 hours estimated) - 3x faster than planned!
**Completion Date**: 2025-10-09

### Phase 1: Multi-Topic Dialogue (1-2 hours) ⚡ QUICK WIN ✅ DEPLOYED
- [x] Update TALK command parser to accept optional topic parameter
- [x] Modify handle_talk_to_npc to look up `dialog[topic]`
- [x] Add LIST keyword: `TALK NPC LIST` shows available topics
- [x] Fallback to "greeting" if topic not found
- [x] Update NPC_SYSTEM.md documentation
- [~] Test with existing 5 NPCs (tests need debug - manual testing OK)

### Phase 2: Conversation State & History (4-6 hours) ✅ DEPLOYED
- [x] Define ConversationState struct (topics_discussed, flags, timestamps)
- [x] Add conversation_state:{player}:{npc} database keys
- [x] Track topics discussed per player per NPC
- [x] Add TALKED command to view conversation history
- [x] Expire old conversation state (30 days)
- [x] Unit tests for state persistence

### Phase 3: Branching Dialogue Trees (8-12 hours) ✅ DEPLOYED
- [x] Define DialogNode struct (text, choices, goto)
- [x] Define DialogChoice struct (label, goto, exit)
- [x] JSON format for dialog trees in database
- [x] Update TALK command to present numbered choices
- [x] Handle choice selection: `TALK NPC <number>`
- [x] Navigation: back, exit, goto nodes
- [x] Max tree depth: 10 levels (prevent infinite loops)
- [x] Integration tests for complex conversations

### Phase 4: Conditional Responses (6-8 hours) ✅ DEPLOYED
- [x] Define DialogCondition enum (quest, item, achievement, flag, currency, time)
- [x] Condition evaluation engine
- [x] Filter dialog nodes by conditions
- [x] Filter choices by conditions
- [x] Default/fallback dialog for unmet conditions
- [x] Unit tests for each condition type
- [x] Integration with quest system

### Phase 5: Dialog Actions (8-10 hours) ✅ DEPLOYED
- [x] Define DialogAction enum (give_quest, give_item, set_flag, etc.)
- [x] Action execution on dialog node entry
- [x] Quest integration (start/complete quests)
- [x] Inventory integration (give/take items)
- [x] Currency integration (rewards/costs)
- [x] Flag system for custom state
- [x] Teleport action (move player to room)
- [x] Achievement granting
- [x] System messages via dialogue
- [x] Action validation and error handling
- [x] 10 action types implemented: GiveItem, TakeItem, GiveCurrency, TakeCurrency, StartQuest, CompleteQuest, GrantAchievement, SetFlag, Teleport, SendMessage

### Phase 6: Admin Dialog Editor (10-12 hours) ✅ DEPLOYED
- [x] @DIALOG NPC LIST command - Show all dialogue topics
- [x] @DIALOG NPC VIEW TOPIC command - View dialogue JSON
- [x] @DIALOG NPC ADD TOPIC TEXT command - Add simple text dialogue
- [x] @DIALOG NPC EDIT TOPIC command - Edit dialogue trees with JSON
- [x] @DIALOG NPC DELETE TOPIC command - Remove dialogue topics
- [x] @DIALOG NPC TEST TOPIC command - Test conditions for current player
- [x] JSON validation and error messages
- [x] Pretty-print JSON output
- [x] Permission checking (admin command)
- [x] Help documentation integrated

### Documentation
- [x] Design specification: `docs/development/NPC_DIALOGUE_SYSTEM_DESIGN.md`
- [x] Phase 1 docs: `docs/development/PHASE1_MULTI_TOPIC_DIALOGUE.md`
- [x] Phase 2 docs: `docs/development/PHASE2_CONVERSATION_STATE.md`
- [x] Phase 3 docs: `docs/development/PHASE3_BRANCHING_DIALOGUE.md`
- [x] Phase 4 docs: `docs/development/PHASE4_CONDITIONAL_DIALOGUE.md`
- [x] Phase 5 docs: `docs/development/PHASE5_DIALOG_ACTIONS.md`
- [x] Phase 6 docs: `docs/development/PHASE6_ADMIN_DIALOG_EDITOR.md`
- [x] NPC System docs updated with all commands

### Content Creation & Automatic Seeding ✅ COMPLETE (2025-10-10)
- [x] Create comprehensive dialogue content for all 5 NPCs
- [x] Mayor Thompson: 9 dialogue nodes (tutorial + town info + quests)
- [x] City Clerk: 7 dialogue nodes (housing + admin help + quests)
- [x] Gate Guard: 8 dialogue nodes (security + wilderness warnings + advice)
- [x] Market Vendor: 13 dialogue nodes (trading + purchases + family history)
- [x] Museum Curator: 10 dialogue nodes (lore + history + exhibits)
- [x] Implement `seed_npc_dialogues_if_needed()` function
- [x] Integrate dialogue seeding into database initialization
- [x] Automatic population on first run (no manual setup required)
- [x] Idempotent seeding (won't overwrite custom changes)
- [x] Unit test verification
- [x] Documentation: `docs/development/NPC_DIALOGUE_SEEDING.md`
- [x] **Total**: 47 dialogue nodes across 5 NPCs, automatically seeded

**Achievement**: NPC dialogue content now automatically initializes with the database! No shell scripts, no manual admin commands needed. Dialogues are version-controlled in source code and deploy seamlessly.

---

## Phase 9 — Admin/GM Tooling, Observability & World Management
(Ref: Plan §Phase 8, Design §§Admin Tools, Logging, Backup & Recovery, Mesh Resilience)

### Phase 9.1 — Admin Permission System ✅ COMPLETE
- [x] Add `is_admin` and `admin_level` fields to PlayerRecord
- [x] Implement helper methods (is_admin, grant_admin, revoke_admin, admin_level)
- [x] Add storage methods (is_admin, require_admin, grant_admin, revoke_admin, list_admins)
- [x] Add PermissionDenied error type
- [x] **Automatic admin account seeding on database initialization**
- [x] Admin bootstrap with configurable username
- [x] Existing player promotion support
- [x] Comprehensive test suite (9 tests, all passing)
- [x] Documentation: `docs/development/TMUSH_ADMIN_PERMISSIONS.md`
- [x] Documentation: `docs/development/TMUSH_ADMIN_BOOTSTRAP.md`

**Status**: Admin system complete! Every fresh database automatically creates an admin account (default username: "admin") with sysop-level privileges. Idempotent seeding prevents duplicates, and existing players can be promoted. Ready for admin command handlers!

### Phase 9.2 — Admin Command Handlers ✅ COMPLETE
- [x] Admin status & privilege management commands:
  - [x] `@ADMIN` - Show admin status (level, available commands, total count)
  - [x] `@SETADMIN <player> <level>` - Grant admin privileges (level 0-3)
  - [x] `@REMOVEADMIN <player>` / `@REVOKEADMIN <player>` - Revoke admin privileges
  - [x] `@ADMINS` / `@ADMINLIST` - List all administrators (public command)
- [x] Permission checking (level 2+ required for grant/revoke)
- [x] Level validation (cannot grant higher than own level)
- [x] Self-protection (cannot revoke own admin privileges)
- [x] Username normalization (lowercase for storage compatibility)
- [x] Formatted Unicode output (🛡️, ✅, ❌, ⛔)
- [x] Comprehensive test suite (8 integration tests, all passing)

**Status**: Admin command handlers complete! Admins can now view status, grant/revoke privileges with proper permission checks, and list all administrators. Level-based access control (1=Moderator, 2=Admin, 3=Sysop) fully implemented and tested. Ready for player monitoring commands!

### Phase 9.3 — Player Monitoring Commands ✅ COMPLETE
- [x] Player monitoring commands:
  - [x] `/PLAYERS` / `/WHO` - list all players with location and admin status (level 1+)
  - [x] `WHERE [player]` - show own location OR locate specific player (admin level 1+)
  - [x] `/GOTO <player|room>` - teleport admin to player or room (level 1+)
  - [ ] `/SUMMON <player>` - teleport player to admin (future enhancement)
  - [ ] `/BOOT <player>` - disconnect player (future enhancement)
  - [ ] `/BAN <player>` - ban player access (requires persistence layer)
- [x] Smart WHERE command: dual-mode (self/admin) with optional argument
- [x] Permission checking (admin level 1+ for monitoring commands)
- [x] Location tracking via PlayerRecord.current_room
- [x] Teleportation with room preview and player listing
- [x] Unicode indicators (👥, 📍, ✈️, ⛔) for clear feedback
- [x] All 129 library tests passing
- [ ] Integration tests for player monitoring commands (next priority)

**Status**: Core player monitoring complete! Admins can now list players, locate them, and teleport to assist. Essential tools for alpha testing management are operational. ✅ Integration tests implemented and passing!

### Phase 9.4 — Player Monitoring Integration Tests ✅ COMPLETE
**Priority**: High - Verify player monitoring commands work correctly
**Effort**: 3 hours (actual)
**Completion Date**: 2025-10-11
**Commit**: 79114df

- [x] Test `/PLAYERS` command:
  - [x] Shows all registered players
  - [x] Displays admin levels correctly
  - [x] Shows current room location
  - [x] Denies access to non-admins
  - [x] Handles empty player list
  - [x] Handles 50+ player overflow correctly
- [x] Test `WHERE` command (dual-mode):
  - [x] Regular user: WHERE shows own location with occupancy
  - [x] Admin: WHERE shows own location
  - [x] Admin: WHERE <player> locates other player
  - [x] Shows player admin level if applicable
  - [x] Shows room name and location
  - [x] Denies WHERE <player> to non-admins
  - [x] Handles player-not-found gracefully
- [x] Test `/GOTO` command:
  - [x] Teleports admin to player's location
  - [x] Teleports admin to specific room
  - [x] Updates admin's current_room correctly
  - [x] Shows room description after teleport
  - [x] Lists players in destination room
  - [x] Denies access to non-admins
  - [x] Handles invalid player/room gracefully
  - [x] Works with both player names and room IDs
- [x] Test permission enforcement:
  - [x] Level 1 (Moderator) can use all monitoring commands
  - [x] Level 2 (Admin) can use all monitoring commands
  - [x] Level 3 (Sysop) can use all monitoring commands
  - [x] Non-admin gets clear permission denied messages
- [x] Test edge cases:
  - [x] Player in room without description
  - [x] Player in invalid/deleted room
  - [x] Multiple players in same room
  - [x] Teleport to player who just moved
- [x] Created test fixture file: `tests/player_monitoring.rs` (10 tests)
- [x] Fixed test isolation with temp directory configuration
- [x] Documentation in `docs/development/TMUSH_ADMIN_COMMANDS.md`

**Test Status**: 10/10 integration tests passing as part of 374 total tests.

**Infrastructure Improvements Completed**:
- Fixed temp directory configuration for TinyMUSH stores in tests
- Resolved Sled database locking conflicts (no more reopening same database)
- Created unique temp directories per test for proper isolation
- All tests now use isolated databases to prevent data contamination

### Phase 9.4.5 — Critical Infrastructure Fixes ✅ COMPLETE
**Priority**: URGENT - Blocking test development
**Effort**: 6 hours (actual)
**Completion Date**: 2025-10-11
**Commit**: 79114df

#### Problems Discovered and Fixed

**1. NPC Serialization Errors** ✅ FIXED
- **Problem**: `InvalidBoolEncoding(X)` errors when deserializing NPC records
- **Root Cause**: Bincode 1.3 incompatible with certain serde attributes:
  - `#[serde(skip_serializing_if = "Option::is_none")]` on DialogChoice.goto
  - `#[serde(tag = "type")]` on DialogCondition and DialogAction enums
- **Solution**: Removed incompatible serde attributes from `src/tmush/types.rs`
- **Impact**: NPCs with dialogue trees now serialize/deserialize correctly
- **Validation**: Created `tests/npc_serialization_test.rs` to prevent regression

**2. GameRegistry Pattern** ✅ IMPLEMENTED
- **Problem**: Multiple code paths opening same Sled store causing cache coherency issues
- **Solution**: Created `src/bbs/game_registry.rs` for centralized resource management
- **Pattern**: Single Arc<TinyMushStore> shared across all BBS code
- **Methods**: Builder pattern with `with_tinymush()`, `with_tinyhack()`, getter methods
- **Impact**: Prevents Sled database conflicts, enables proper test isolation

**3. Schema Migration System** ✅ IMPLEMENTED
- **Problem**: Data schema changes breaking existing databases
- **Solution**: Created `src/tmush/migration.rs` with Migratable trait
- **Components**: 
  - Migratable trait with `current_schema_version()`, `migrate()`, `needs_migration()`
  - `load_and_migrate<T>()` helper for automatic versioning
  - Implementations for PlayerRecord (v2), NpcRecord (v1), RoomRecord (v2), ObjectRecord (v2)
- **Usage**: `get_npcs_in_room()` now uses migration system
- **Documentation**: `docs/development/SCHEMA_MIGRATION.md`

**4. Test Navigation Issues** ✅ FIXED
- **Problem**: Tests assumed menu position (G2) and room locations
- **Solution A**: Changed all tests to use explicit "TINYMUSH" command (position-independent)
- **Solution B**: Fixed navigation paths (players start at gazebo_landing, must go N to town_square)
- **Files**: `tests/npc_multi_topic_dialogue.rs` (4 tests fixed)

**5. Test Database Isolation** ✅ FIXED
- **Problem**: Tests not configuring temp directory, used default "data/tinymush"
- **Solution**: Added `cfg.games.tinymush_db_path` to all relevant tests
- **Files**: 
  - `tests/player_monitoring.rs` (helper function updated)
  - `tests/npc_multi_topic_dialogue.rs` (all 4 tests)
- **Impact**: Each test gets isolated database, no cross-contamination

**6. Sled Database Locking** ✅ FIXED
- **Problem A**: `tests/tmush_admin_command_handlers.rs` reopening same database
- **Solution A**: Reuse existing store instance instead of reopening
- **Problem B**: `src/tmush/commands.rs` unit tests sharing temp directory
- **Solution B**: Unique temp dir per test: `create_test_store(test_name)`
- **Pattern**: `format!("tinymush_test_{}_{}", process_id, test_name)` with cleanup
- **Impact**: No more Sled locking errors, tests run in parallel safely

**Final Status**: 
- ✅ 374 tests passing (134 library + 240 integration)
- ✅ 0 test failures
- ✅ All infrastructure issues resolved
- ✅ Comprehensive documentation added
- ✅ Clean git commit (79114df) with 26 files changed



### Phase 9.4.5 — Game Door Test Infrastructure Overhaul (Week 2-3) **CRITICAL PATH**
**Priority**: URGENT - Blocking all game door integration tests
**Effort**: 1-2 weeks (comprehensive solution)
**Context**: Second time deferring integration tests due to async/persistence timing issues

#### Problem Domain Analysis

**Root Cause**: Architectural mismatch between production async design and test synchronization needs.

**The Issue**:
1. Game doors (TinyHack, TinyMUSH) use **lazy initialization**:
   - Entry command (G1/G2) only changes session state
   - Player records created on NEXT game command
   - Async persistence operations not immediately visible
   
2. Tests assume **synchronous behavior**:
   - route_test_text_direct() returns immediately
   - Subsequent test helpers expect records to exist
   - Race conditions cause "record not found" errors

3. **No coordination layer** between:
   - BBS layer (sessions, routing)
   - Game layer (player records, state)
   - Storage layer (persistence)
   - Test layer (verification)

**Affected Systems**:
- ❌ TinyMUSH integration tests (current blocker)
- ❌ TinyHack integration tests (previously deferred)
- ❌ Future game door tests (will hit same issues)

#### Multi-Step Implementation Plan

**Step 1: Define Test Helper Contracts** (2 hours)
- [ ] Document required test helper signatures
- [ ] Define coordination protocol between layers
- [ ] Specify async/await guarantees for tests
- [ ] Create trait `GameDoorTestSupport` for common interface
- [ ] Document which helpers are game-specific vs generic

**Step 2: Implement Generic BBS Test Helpers** (4 hours)
- [ ] `test_wait_for_session_state()` - poll until state change completes
  - [ ] Accepts: node_key, expected_state, timeout
  - [ ] Returns: Result<(), TimeoutError>
  - [ ] Used by: All game door tests
  - [ ] Test coverage: Unit tests for timeout, success, invalid state
  
- [ ] `test_ensure_logged_in()` - guarantee user logged in and session exists
  - [ ] Accepts: username, user_level
  - [ ] Returns: Result<String> (node_key)
  - [ ] Creates session if missing, logs in user
  - [ ] Test coverage: Already logged in, new user, existing user
  
- [ ] `test_verify_persistence()` - generic storage sync helper
  - [ ] Accepts: check_fn: Fn() -> Result<bool>, timeout, poll_interval
  - [ ] Returns: Result<(), TimeoutError>
  - [ ] Polls check_fn until true or timeout
  - [ ] Test coverage: Immediate success, delayed success, timeout

**Step 3: Implement TinyMUSH-Specific Test Helpers** (3 hours)
- [ ] `test_tmush_create_player()` - force synchronous player creation
  - [ ] Accepts: username, starting_room
  - [ ] Returns: Result<PlayerRecord>
  - [ ] Directly creates TinyMUSH player record
  - [ ] Bypasses lazy initialization
  - [ ] Test coverage: New player, duplicate player, invalid room
  
- [ ] `test_tmush_enter_and_init()` - complete TinyMUSH initialization
  - [ ] Accepts: node_key, username
  - [ ] Returns: Result<PlayerRecord>
  - [ ] Sends G2 command
  - [ ] Waits for session state change
  - [ ] Sends "look" to trigger get_or_create_player()
  - [ ] Polls until player record exists
  - [ ] Test coverage: Success path, timeout, invalid session
  
- [ ] `test_tmush_grant_admin_safe()` - retry-capable admin granting
  - [ ] Accepts: username, level, max_retries
  - [ ] Returns: Result<(), GrantError>
  - [ ] Waits for player record if not exists
  - [ ] Grants admin with retry logic
  - [ ] Test coverage: Immediate success, delayed success, timeout
  
- [ ] `test_tmush_player_exists()` - non-blocking existence check
  - [ ] Accepts: username
  - [ ] Returns: bool
  - [ ] Used for polling loops
  - [ ] No errors on missing player
  - [ ] Test coverage: Exists, doesn't exist, invalid store

**Step 4: Implement TinyHack-Specific Test Helpers** (2 hours)
- [ ] `test_tinyhack_create_game()` - force game state creation
  - [ ] Accepts: username, initial_position
  - [ ] Returns: Result<GameState>
  - [ ] Creates save file synchronously
  - [ ] Test coverage: New game, existing game, corrupted save
  
- [ ] `test_tinyhack_enter_and_init()` - complete TinyHack initialization
  - [ ] Accepts: node_key, username
  - [ ] Returns: Result<GameState>
  - [ ] Sends G1 command
  - [ ] Waits for session state change
  - [ ] Polls until save file exists
  - [ ] Test coverage: New player, returning player, timeout
  
- [ ] `test_tinyhack_verify_save()` - check save file persistence
  - [ ] Accepts: username, expected_fields
  - [ ] Returns: Result<GameState>
  - [ ] Polls until save file exists and is valid
  - [ ] Test coverage: Immediate, delayed, corrupted, timeout

**Step 5: Create Reusable Test Infrastructure Module** (3 hours)
- [ ] Create `tests/common/mod.rs` for shared helpers
- [ ] Create `tests/common/game_door.rs` - generic game door helpers
- [ ] Create `tests/common/tinymush.rs` - TinyMUSH-specific helpers
- [ ] Create `tests/common/tinyhack.rs` - TinyHack-specific helpers
- [ ] Create `tests/common/sync.rs` - async/await synchronization primitives
- [ ] Document usage patterns in `tests/common/README.md`
- [ ] Add examples for each helper function

**Step 6: Refactor Existing Tests** (4 hours)
- [ ] Update `tests/tinyhack_integration.rs` to use new helpers:
  - [ ] Replace manual session creation with test_ensure_logged_in()
  - [ ] Replace save file polling with test_tinyhack_verify_save()
  - [ ] Add test_tinyhack_enter_and_init() for initialization
  - [ ] Verify all tests still pass
  
- [ ] Update `tests/player_monitoring.rs` to use new helpers:
  - [ ] Replace manual player creation with test_tmush_create_player()
  - [ ] Replace grant_admin with test_tmush_grant_admin_safe()
  - [ ] Use test_tmush_enter_and_init() for setup
  - [ ] Add proper wait/polling for persistence
  - [ ] Verify all 12 tests pass

**Step 7: Integration Test Implementation** (3 hours)
- [ ] Implement all 12 player monitoring tests
- [ ] Test `/PLAYERS` command (2 tests)
- [ ] Test `WHERE` command (4 tests)
- [ ] Test `/GOTO` command (5 tests)
- [ ] Test permission enforcement (1 test covering all levels)
- [ ] Verify all tests pass reliably (5+ consecutive runs)

**Step 8: Documentation & Examples** (2 hours)
- [ ] Document test infrastructure in `docs/development/TESTING_GUIDE.md`
- [ ] Create example test file: `tests/examples/game_door_test_template.rs`
- [ ] Add troubleshooting section for common test issues
- [ ] Document timing parameters (timeouts, poll intervals)
- [ ] Add section on when to use which helper
- [ ] Document the generic GameDoorTestSupport trait

#### Validation Criteria

**Must Pass Before Completion**:
1. ✅ All new helpers have unit tests with >80% coverage
2. ✅ TinyMUSH integration tests: all 12 tests passing
3. ✅ TinyHack integration tests: all existing tests still passing
4. ✅ Run tests 10 times: 0 flakes or race conditions
5. ✅ No impact on production code performance or behavior
6. ✅ Documentation complete with working examples
7. ✅ All helpers follow consistent naming: test_<game>_<action>()
8. ✅ Zero compiler warnings in test code

**Acceptance Test**:
```bash
# Must pass 10 consecutive times without failures:
for i in {1..10}; do
  cargo test --test player_monitoring && \
  cargo test --test tinyhack_integration || exit 1
done
```

#### Design Principles

**1. Separation of Concerns**:
- BBS layer helpers (sessions, routing) - work for ALL games
- Game layer helpers (records, state) - game-specific
- Storage layer helpers (persistence) - generic retry/poll logic
- No cross-layer dependencies in helpers

**2. Composability**:
- Each helper does ONE thing well
- Helpers can be chained for complex scenarios
- No hidden side effects or global state

**3. Discoverability**:
- Clear naming: `test_<system>_<action>()`
- Comprehensive documentation
- Working examples for common patterns

**4. Reliability**:
- All helpers have timeouts (no infinite loops)
- Clear error messages on failure
- Retry logic with exponential backoff where appropriate

**5. Future-Proof**:
- Generic patterns work for new game doors
- Trait-based abstractions for common operations
- Easy to extend without breaking existing tests

#### Risk Mitigation

**Risk 1: Breaking Production Code**
- Mitigation: All helpers in `#[cfg(test)]` blocks only
- Mitigation: No changes to production async behavior
- Mitigation: Extensive testing of production code after changes

**Risk 2: Test Infrastructure Becomes Unmaintainable**
- Mitigation: Keep helpers simple and focused
- Mitigation: Document each helper's purpose clearly
- Mitigation: Regular refactoring as patterns emerge

**Risk 3: Timing Issues Persist**
- Mitigation: Configurable timeouts for different environments
- Mitigation: Poll intervals tuned to balance speed/reliability
- Mitigation: Clear error messages when timing out

**Risk 4: Game Door Divergence**
- Mitigation: GameDoorTestSupport trait enforces consistency
- Mitigation: Shared patterns in tests/common/game_door.rs
- Mitigation: Regular review of test patterns across games

#### Success Metrics

- ✅ 0 deferred integration test suites after completion
- ✅ <5 minutes to write new game door integration test
- ✅ 100% reliability on CI/CD (no flakes)
- ✅ New game doors can reuse 80%+ of test infrastructure
- ✅ Test execution time: <10 seconds for full suite

**Status**: Test infrastructure overhaul planned and scoped. This is CRITICAL PATH work that unblocks all future game door testing. Estimated 1-2 weeks for comprehensive solution. Implementation should begin before continuing with new feature development.

### Phase 9.5 — World Event Commands (Week 2)
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

## Phase 10 — Performance, Polish, Go-Live Prep
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

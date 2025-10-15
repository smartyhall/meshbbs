# Hardcoded Content Analysis & Data-Driven Migration Proposals

**Date:** October 14, 2025  
**Branch:** crafting_refactor  
**Status:** Analysis & Proposal Phase

## Executive Summary

After successfully migrating **Recipes** and **Quests** to data-driven systems, the following game content remains hardcoded in Rust source files. This document analyzes each system and proposes migration priorities.

---

## ✅ Already Data-Driven (Completed)

1. **Crafting Recipes** - `@RECIPE` command (completed)
2. **Quests** - `@QUEST` command (completed)

---

## 🔴 High Priority: Hardcoded Content Requiring Migration

### 1. **Achievements** (167 lines)
**Location:** `src/tmush/state.rs:625-792` (`seed_starter_achievements()`)  
**Current State:** 15 hardcoded achievements in Rust code  
**Storage:** Already in database (`achievements` tree)  
**Admin Commands:** None - no way to create/edit achievements in-world

#### Hardcoded Achievements:
- **Combat:** first_blood, veteran, legendary
- **Exploration:** wanderer, explorer, cartographer
- **Social:** friendly, popular, chatterbox
- **Economic:** merchant, wealthy
- **Quest:** quest_beginner, quest_veteran
- **Special:** town_founder, network_pioneer

#### Proposed Solution: `@ACHIEVEMENT` Command
```
@ACHIEVEMENT CREATE <id> <name>
@ACHIEVEMENT EDIT <id> DESCRIPTION <text>
@ACHIEVEMENT EDIT <id> CATEGORY <Combat|Exploration|Social|Economic|Quest|Special>
@ACHIEVEMENT EDIT <id> TRIGGER <type> <params>
  Types: KILLCOUNT, ROOMVISITS, FRIENDCOUNT, MESSAGESSENT, TRADECOUNT, 
         CURRENCYEARNED, QUESTCOMPLETION, VISITLOCATION, COMPLETEQUEST
@ACHIEVEMENT EDIT <id> TITLE <title_text>
@ACHIEVEMENT EDIT <id> HIDDEN <true|false>
@ACHIEVEMENT DELETE <id>
@ACHIEVEMENT LIST [category]
@ACHIEVEMENT SHOW <id>
```

#### Complexity: **Medium**
- Similar pattern to @QUEST (nested enums: AchievementCategory, AchievementTrigger)
- Trigger types have different parameters (count vs amount vs location)
- Boolean hidden flag
- Optional title rewards

#### Impact: **High**
- World builders can create custom achievement systems
- Easy to add game-specific milestones
- Player retention through custom goals

---

### 2. **NPCs & Dialogue** (141 lines)
**Location:** `src/tmush/state.rs:873-1014` (`seed_starter_npcs()`)  
**Current State:** 5 hardcoded NPCs with dialogue trees  
**Storage:** Already in database (`npcs` tree)  
**Admin Commands:** None - no way to create/edit NPCs in-world

#### Hardcoded NPCs:
1. **Mayor Thompson** (mayor_office) - Tutorial completion, quest giver
   - Dialogues: greeting, tutorial_complete, quest_welcome
2. **City Clerk** (city_hall_lobby) - Administrative help
   - Dialogues: greeting, help
   - Flag: TutorialNpc
3. **Gate Guard** (north_gate) - Security and directions
   - Dialogues: greeting, warning
   - Flag: Guard
4. **Mira the Vendor** (south_market) - Trading and commerce
   - Dialogues: greeting, wares, story
   - Flag: Vendor
5. **Dr. Reeves** (mesh_museum) - Museum curator, history
   - Dialogues: greeting, history, exhibit
   - Flag: TutorialNpc

#### Proposed Solution: `@NPC` Command
```
@NPC CREATE <id> <name>
@NPC EDIT <id> TITLE <title>
@NPC EDIT <id> DESCRIPTION <text>
@NPC EDIT <id> LOCATION <room_id>
@NPC EDIT <id> DIALOG <topic> <text>
@NPC EDIT <id> DIALOG REMOVE <topic>
@NPC EDIT <id> FLAG <Guard|Vendor|TutorialNpc|QuestGiver>
@NPC DELETE <id>
@NPC LIST [location]
@NPC SHOW <id>
@NPC TELEPORT <id> <room_id>  # Move NPC to different location
```

#### Additional Consideration: Dialogue Trees
NPCs already support multi-topic dialogues via `dialogue_tree` HashMap. The system is partially data-driven (dialogue content stored in DB), but NPC creation is hardcoded.

#### Complexity: **Medium-High**
- Multiple dialogue topics per NPC
- NPC flags (enum with multiple variants)
- Location management
- Integration with quest system (quest_giver_npc field in quests)

#### Impact: **Very High**
- Essential for world-building customization
- Enables custom storylines and lore
- Quest system depends on NPCs existing

---

### 3. **Companions** (76 lines)
**Location:** `src/tmush/state.rs:794-870` (`seed_starter_companions()`)  
**Current State:** 3 hardcoded companions (horse, dog, cat)  
**Storage:** Already in database (`companions` tree)  
**Admin Commands:** None - no way to create/edit companions in-world

#### Hardcoded Companions:
1. **Gentle Mare** (south_market) - Horse, rideable
2. **Loyal Hound** (town_square) - Dog
3. **Shadow Cat** (mesh_museum) - Cat

#### Proposed Solution: `@COMPANION` Command
```
@COMPANION CREATE <id> <name> <type>
  Types: Horse, Dog, Cat, Bird, Wolf, Bear, etc.
@COMPANION EDIT <id> DESCRIPTION <text>
@COMPANION EDIT <id> LOCATION <room_id>
@COMPANION EDIT <id> STATS HEALTH <value>
@COMPANION EDIT <id> STATS LOYALTY <value>
@COMPANION EDIT <id> STATS STRENGTH <value>
@COMPANION EDIT <id> RIDEABLE <true|false>  # Horses only
@COMPANION DELETE <id>
@COMPANION LIST [type]
@COMPANION SHOW <id>
@COMPANION TELEPORT <id> <room_id>
```

#### Complexity: **Low-Medium**
- Simple structure (CompanionRecord)
- CompanionType enum (extensible)
- Stats are optional/default
- Integration with mount system

#### Impact: **Medium**
- Adds variety to companion system
- Enables themed creatures
- Low priority compared to NPCs/Achievements

---

### 4. **World Rooms** (500+ lines)
**Location:** `src/tmush/state.rs:1-532` (`canonical_world_seed()`)  
**Current State:** 20+ rooms hardcoded in Rust  
**Storage:** Already in database (`rooms` tree)  
**Admin Commands:** Partial - `@DIG` exists but limited

#### Existing Room Commands:
- `@DIG <name>` - Creates basic room
- `@DESCRIBE <name>` - Sets description
- `@LINK <direction> <target>` - Creates exit

#### Hardcoded Rooms:
- **Core (2):** gazebo_landing, town_square (REQUIRED - system dependencies)
- **Government (3):** city_hall_lobby, mayor_office
- **Culture (1):** mesh_museum
- **Commerce (2):** north_gate, south_market
- **Wilderness (4):** pine_ridge_trail, repeater_tower, repeater_upper, forest_path
- **Social (2):** relay_tavern, west_residential
- **Nature (1):** ancient_grove
- **Underground (2):** maintenance_tunnels, workshop_district
- **Quest (8):** cipher_chamber, deep_caverns_entrance, sunken_chamber, hidden_vault, forgotten_ruins_entrance, ruins_dark_passage, etc.

#### Proposed Solution: Enhanced `@DIG` / `@ROOM` Commands
```
@ROOM CREATE <id> <name>  # Alias for @DIG
@ROOM EDIT <id> NAME <name>
@ROOM EDIT <id> DESCRIPTION <text>
@ROOM EDIT <id> LONG_DESC <text>  # Multi-line description
@ROOM EDIT <id> FLAG <Safe|Dark|Shop|QuestLocation|Moderated|Indoor>
@ROOM EDIT <id> CAPACITY <number>
@ROOM LINK <id> <direction> <target_id>
@ROOM UNLINK <id> <direction>
@ROOM DELETE <id>  # With safety checks
@ROOM LIST [flag]
@ROOM SHOW <id>
```

#### Complexity: **Low**
- Most functionality already exists in builder commands
- Need to enhance with flags and capacity editing
- Deletion needs safety checks (ensure no players trapped)

#### Impact: **Medium-Low**
- World builders already have basic room creation
- Enhancement would add convenience
- Core rooms (gazebo_landing, town_square) must remain in seed

---

### 5. **Game Objects/Items** (200+ lines)
**Location:** 
- `src/tmush/storage.rs:1122-1202` (`seed_example_trigger_objects()`, `seed_content_objects_if_needed()`)
- Various creation functions in `src/tmush/state.rs`

**Current State:** ~15-20 hardcoded objects  
**Storage:** Already in database (`objects` tree)  
**Admin Commands:** Partial - `@CREATE` exists

#### Hardcoded Objects:
**Example Trigger Objects (museum):**
- Healing potions, mystery boxes, teleport stones, quest clues, etc.

**Content Objects:**
- rumor_board (relay_tavern)
- diagnostic_panel (repeater_tower)
- northern_array (repeater_upper)
- carved_symbols_* (ancient_grove) - 4 trees
- crafting_bench (workshop_district)
- Materials: copper_wire, circuit_board, antenna_rod, crystal_shard, signal_capacitor
- torch (light source)

#### Existing Object Commands:
- `@CREATE <name>` - Creates object
- `@SET <object> = <flag>` - Sets flags
- Trigger system via builder wizard

#### Proposed Solution: Enhanced `@CREATE` / `@OBJECT` Commands
```
@OBJECT CREATE <id> <name>
@OBJECT EDIT <id> DESCRIPTION <text>
@OBJECT EDIT <id> LONG_DESC <text>
@OBJECT EDIT <id> FLAG <Takeable|Container|Wearable|Weapon|LightSource|etc>
@OBJECT EDIT <id> WEIGHT <number>
@OBJECT EDIT <id> VALUE <number>
@OBJECT EDIT <id> LOCATION <room_id>
@OBJECT EDIT <id> TRIGGER <type> <script>  # Existing trigger system
@OBJECT DELETE <id>
@OBJECT LIST [flag]
@OBJECT SHOW <id>
@OBJECT TELEPORT <id> <room_id>
```

#### Complexity: **Low-Medium**
- Builder commands already exist
- Need consolidated interface
- Trigger system already flexible

#### Impact: **Medium**
- Improves builder experience
- Objects are already somewhat manageable
- Enhancement, not critical need

---

## 🟡 Medium Priority: Partially Data-Driven

### 6. **Housing Templates** (Mentioned in seed functions)
**Location:** `src/tmush/storage.rs:2723` (`seed_housing_templates_if_needed()`)  
**Status:** Already has seeding infrastructure  
**Impact:** Low - Housing system is optional feature

### 7. **Default Recipes** (Already Done! ✅)
**Location:** `src/tmush/state.rs:837-870` (`seed_default_recipes()`)  
**Status:** Migrated to `@RECIPE` command  
**Note:** 2 default recipes still seeded (signal_booster, basic_antenna) for backwards compatibility

---

## 🟢 Low Priority: May Stay Hardcoded

### 8. **Tutorial State Machine**
**Location:** Various files in tutorial system  
**Reason to Keep Hardcoded:** Core game flow logic, not content  
**Recommendation:** Keep as-is

### 9. **Admin Seeding**
**Location:** `src/tmush/storage.rs:2651` (`seed_admin_if_needed()`)  
**Reason to Keep Hardcoded:** Security - admin creation should not be data-driven  
**Recommendation:** Keep as-is

### 10. **Core System Rooms**
- `gazebo_landing` (REQUIRED_LANDING_LOCATION_ID)
- `town_square` (REQUIRED_START_LOCATION_ID)

**Reason to Keep Hardcoded:** System dependencies, must exist  
**Recommendation:** Always seed these, allow customization of descriptions only

---

## Recommended Implementation Order

### Phase 1: Player Content Systems (Highest ROI)
1. **@ACHIEVEMENT** - Immediate gameplay impact, drives player engagement
2. **@NPC** - Essential for custom worlds, enables storytelling

### Phase 2: World Building Tools
3. **@COMPANION** - Adds variety, builds on NPC system patterns
4. **Enhanced @ROOM** - Quality of life for builders

### Phase 3: Polish & Enhancement
5. **Enhanced @OBJECT** - Consolidates existing commands
6. **Documentation** - Update docs to reflect all new admin commands

---

## Implementation Pattern (Based on @QUEST Success)

All new commands should follow the established pattern:

```rust
// 1. Add enum variant
TinyMushCommand::AchievementAdmin(String, Vec<String>)

// 2. Add parser with comprehensive help
"@ACHIEVEMENT" | "@ACH" => { /* full usage docs */ }

// 3. Add dispatch handler
TinyMushCommand::AchievementAdmin => handle_achievement_admin(...)

// 4. Implement CRUD handler with subcommands
fn handle_achievement_admin(
    store: &TinyMushStore,
    player_username: &str,
    args: Vec<String>,
) -> Result<String, TinyMushError> {
    // Admin level check
    // Subcommand routing: CREATE, EDIT, DELETE, LIST, SHOW
    // Field validation
    // Database operations
}

// 5. Add storage helper
impl TinyMushStore {
    pub fn achievement_exists(&self, id: &str) -> Result<bool> { ... }
}

// 6. Create integration tests
tests/achievement_management.rs
```

---

## Estimated Effort

| System | Lines of Code | Complexity | Test Coverage | Total Hours |
|--------|--------------|------------|---------------|-------------|
| @ACHIEVEMENT | ~400 | Medium | 5-6 tests | 6-8 hours |
| @NPC | ~500 | Medium-High | 6-8 tests | 8-12 hours |
| @COMPANION | ~300 | Low-Medium | 4-5 tests | 4-6 hours |
| Enhanced @ROOM | ~200 | Low | 3-4 tests | 3-4 hours |
| Enhanced @OBJECT | ~200 | Low-Medium | 3-4 tests | 3-4 hours |
| **TOTAL** | ~1600 | - | ~25 tests | **24-34 hours** |

---

## Benefits of Full Migration

1. **World Builder Empowerment**
   - Create custom game worlds without touching code
   - Rapid prototyping of quests, achievements, NPCs
   - Community-driven content expansion

2. **Operational Flexibility**
   - Fix typos/errors without recompile
   - Balance adjustments on live systems
   - Seasonal events and limited-time content

3. **Testing & Development**
   - Easier to test content changes
   - Rollback capability (restore from backup)
   - Version control for game content (via backup system)

4. **Consistency**
   - All game content managed the same way
   - Unified admin interface pattern
   - Predictable command structure

---

## Risks & Mitigations

### Risk 1: Complexity Creep
**Mitigation:** Follow proven @QUEST pattern, resist feature creep

### Risk 2: Breaking Changes
**Mitigation:** Keep seed functions as fallback, maintain backwards compatibility

### Risk 3: Admin Permission Abuse
**Mitigation:** All commands require admin level 2+, log all modifications

### Risk 4: Database Bloat
**Mitigation:** Already handled by existing backup/cleanup systems

---

## Conclusion

The quest and recipe systems demonstrate that data-driven content management is:
- ✅ **Feasible** - Pattern is proven and repeatable
- ✅ **Valuable** - Enables world customization without code changes
- ✅ **Maintainable** - Clear structure, well-tested

**Recommended Next Steps:**
1. Review this analysis and select systems to migrate
2. Prioritize based on world-building needs
3. Implement in phases (suggest starting with @ACHIEVEMENT)
4. Each system should follow 5-step pattern:
   - Enum → Parser → Dispatch → Handler → Tests

**Decision Point:** Which system should we implement first?
- Option A: @ACHIEVEMENT (player engagement, medium complexity)
- Option B: @NPC (world building essential, higher complexity)
- Option C: @COMPANION (low complexity, quick win)

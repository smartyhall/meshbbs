# Content Population Implementation Plan

## Overview
This document breaks down the comprehensive content design into implementable tasks for populating the expanded TinyMUSH world (15 rooms) with NPCs, quests, objects, and puzzles.

## Implementation Order & Rationale

### Phase 1: Foundation Objects
Implement simple interactive objects first to establish patterns and test examine/use mechanics without complex state management.

### Phase 2: NPCs & Dialogue
Add NPCs with dialogue trees - they serve as anchor points for quests and provide context for objects/puzzles.

### Phase 3: Quests
Implement quest structures that reference the NPCs and objects already in place.

### Phase 4: Puzzles & Complex Mechanics
Add puzzle systems that tie everything together and require complex state management.

---

## PHASE 1: FOUNDATION OBJECTS (Simple Interactive Items)

### 1.1 Rumor Board (The Relay Tavern)
**File**: `src/tmush/state.rs` - `canonical_world_seed()`
**Location**: `relay_tavern`
**Implementation**:
```rust
ObjectRecord {
    id: "rumor_board".to_string(),
    name: "Rumor Board".to_string(),
    description: "A cork board covered in handwritten notes, sketches of signal patterns, and cryptic messages about 'the Grove' and 'unusual readings'.".to_string(),
    room_id: "relay_tavern".to_string(),
    takeable: false,
    uses_remaining: None,
    trigger_type: Some(ObjectTrigger::OnLook),
    quest_item: false,
}
```
**Notes**: Read-only flavor object, no complex state

### 1.2 Northern Array (Repeater Tower)
**File**: `src/tmush/state.rs`
**Location**: `repeater_tower`
**Implementation**:
```rust
ObjectRecord {
    id: "northern_array".to_string(),
    name: "Northern Array".to_string(),
    description: "A directional antenna pointing north toward Pine Ridge. Examine closer to see signal strength indicators.".to_string(),
    room_id: "repeater_tower".to_string(),
    takeable: false,
    uses_remaining: None,
    trigger_type: Some(ObjectTrigger::OnLook),
    quest_item: false,
}
```

### 1.3 Carved Symbols (Ancient Grove)
**File**: `src/tmush/state.rs`
**Location**: `ancient_grove`
**Implementation**:
```rust
ObjectRecord {
    id: "carved_symbols_oak".to_string(),
    name: "Oak Symbols".to_string(),
    description: "Ancient carvings on the oak trunk: a circle with radiating lines (sun/signal symbol).".to_string(),
    room_id: "ancient_grove".to_string(),
    takeable: false,
    uses_remaining: None,
    trigger_type: Some(ObjectTrigger::OnLook),
    quest_item: false,
}
// Repeat for elm_tree, willow_tree, ash_tree
```

### 1.4 Crafting Bench (Workshop District)
**File**: `src/tmush/state.rs`
**Location**: `workshop_district`
**Implementation**:
```rust
ObjectRecord {
    id: "crafting_bench".to_string(),
    name: "Crafting Bench".to_string(),
    description: "A sturdy workbench with tools, wire spools, and component bins. Shows recipes for signal_booster (1 wire + 1 scrap) and basic_antenna (2 wire + 1 component).".to_string(),
    room_id: "workshop_district".to_string(),
    takeable: false,
    uses_remaining: None,
    trigger_type: Some(ObjectTrigger::OnLook),
    quest_item: false,
}
```

### 1.5 Quest Materials (Various Locations)
**File**: `src/tmush/state.rs`
**Locations**: Various
**Items**: wire_spool, scrap_metal, basic_component (takeable, spawn multiple)

---

## PHASE 2: NPCs & DIALOGUE TREES

### 2.1 Old Graybeard (Repeater Tower)
**File**: `src/tmush/npcs.rs` (or extend existing NPC system)
**Location**: `repeater_tower`
**Dialogue Tree**:
```rust
NpcRecord {
    id: "old_graybeard".to_string(),
    name: "Old Graybeard".to_string(),
    description: "An elderly technician with weathered hands and kind eyes...".to_string(),
    room_id: "repeater_tower".to_string(),
    dialogue_tree: DialogTree {
        root: "greeting".to_string(),
        nodes: hashmap! {
            "greeting" => DialogNode {
                text: "Ah, a visitor! Come to see the tower?...",
                choices: vec![
                    DialogChoice { text: "What is this place?", next: "about_tower" },
                    DialogChoice { text: "Can I help with anything?", next: "quest_offer" },
                    DialogChoice { text: "Just looking around.", next: "farewell" },
                ],
            },
            "about_tower" => DialogNode { ... },
            "quest_offer" => DialogNode { ... },
            // ... full tree
        },
    },
    quest_giver: Some("tower_diagnostics".to_string()),
    vendor_inventory: None,
}
```
**Integration**: Links to tower_diagnostics quest

### 2.2 Barkeep Mira (The Relay Tavern)
**File**: `src/tmush/npcs.rs`
**Location**: `relay_tavern`
**Role**: Social hub, rumor dispenser, no quests
**Dialogue**: 3-level tree (greeting → topics → details)

### 2.3 Old Elm (West Residential Lane)
**File**: `src/tmush/npcs.rs`
**Location**: `west_residential`
**Role**: Quest giver for grove_mystery
**Dialogue**: 4-level tree with quest acceptance flow

### 2.4 Tinker Brass (Workshop District)
**File**: `src/tmush/npcs.rs`
**Location**: `workshop_district`
**Role**: Quest giver for first_craft tutorial
**Dialogue**: 3-level tree with crafting tutorial

---

## PHASE 3: QUEST IMPLEMENTATION

### 3.1 Tower Diagnostics Quest
**File**: `src/tmush/state.rs` - `canonical_quest_seed()`
**ID**: `tower_diagnostics`
**Objectives**:
```rust
QuestRecord {
    id: "tower_diagnostics".to_string(),
    title: "Tower Diagnostics".to_string(),
    description: "Help Old Graybeard check the repeater systems...".to_string(),
    objectives: vec![
        QuestObjective::TalkToNpc { npc_id: "old_graybeard".to_string(), completed: false },
        QuestObjective::UseItem { item_id: "diagnostic_panel".to_string(), completed: false },
        QuestObjective::VisitLocation { location_id: "repeater_upper".to_string(), completed: false },
        QuestObjective::UseItem { item_id: "northern_array".to_string(), completed: false },
    ],
    prerequisites: vec![],
    rewards: QuestReward {
        experience: 100,
        currency: 50,
        items: vec!["signal_booster".to_string()],
        unlocks: Some(vec!["ancient_grove".to_string()]),
    },
    repeatable: false,
    completion_dialogue: "The tower's running smoothly now...".to_string(),
}
```

### 3.2 Grove Mystery Quest
**File**: `src/tmush/state.rs`
**ID**: `grove_mystery`
**Prerequisites**: `tower_diagnostics` completed
**Objectives**: Symbol observation sequence

### 3.3 Tunnel Salvage Quest
**File**: `src/tmush/state.rs`
**ID**: `tunnel_salvage`
**Objectives**: Dark navigation, item collection

### 3.4 First Craft Quest
**File**: `src/tmush/state.rs`
**ID**: `first_craft`
**Objectives**: Crafting tutorial

---

## PHASE 4: PUZZLE MECHANICS

### 4.1 Tower Climb Puzzle (UP/DOWN Navigation)
**File**: `src/tmush/commands.rs` - Extend UP/DOWN commands
**Mechanism**: Add `repeater_upper` pseudo-room accessible only from `repeater_tower`
**Requirements**: Quest objective tracking on navigation

### 4.2 Grove Symbol Sequence Puzzle
**File**: `src/tmush/commands.rs` - EXAMINE command extension
**Mechanism**: Track examination order in quest state
**Correct Sequence**: Oak → Elm → Willow → Ash

### 4.3 Dark Tunnel Navigation
**File**: `src/tmush/commands.rs` - Visibility system
**Mechanism**: Check for light_source in inventory
**Integration**: Junction boxes reveal when lit

### 4.4 Crafting System
**File**: `src/tmush/commands.rs` - New CRAFT command
**Mechanism**: Recipe checking, inventory consumption, item creation
**Recipes**: Defined in crafting_bench object

---

## DATA STRUCTURES NEEDED

### New/Extended Structures

1. **ObjectRecord Extension** (if needed):
   - Add `recipe` field for crafting items
   - Add `light_source` boolean flag

2. **QuestObjective Extension**:
   - May need `ExamineInOrder` variant for grove puzzle
   - May need `CraftItem` variant

3. **Room Flag Extension**:
   - Add `RequiresLight` flag for dark rooms

4. **Player Inventory Tracking**:
   - Ensure light source detection works
   - Crafting material counting

---

## TESTING STRATEGY

### Per-Phase Testing

**Phase 1**: Test each object individually
- Can examine objects
- Descriptions display correctly
- Takeable items can be picked up

**Phase 2**: Test NPC interactions
- Can talk to each NPC
- Dialogue choices work
- Quest offers appear correctly

**Phase 3**: Test quest flow
- Quest acceptance works
- Objectives update correctly
- Rewards are granted
- Prerequisites enforce properly

**Phase 4**: Test puzzle mechanics
- Tower climb navigation
- Symbol sequence detection
- Dark room visibility
- Crafting recipes work

### Integration Testing
- Complete full quest chains
- Test interconnected systems
- Verify no orphaned references

---

## FILES TO MODIFY

1. **src/tmush/state.rs**
   - Add objects to `canonical_world_seed()`
   - Add NPCs to world seed
   - Add quests to `canonical_quest_seed()`

2. **src/tmush/npcs.rs** (or create if doesn't exist)
   - Define NPC records with dialogue trees
   - Quest giver associations

3. **src/tmush/commands.rs**
   - Extend EXAMINE for puzzle mechanics
   - Extend UP/DOWN for tower climb
   - Add/extend CRAFT command
   - Dark room visibility logic

4. **src/tmush/storage.rs**
   - Quest state persistence
   - Object interaction state
   - Crafting system state

5. **tests/** (new test files)
   - `tests/tower_diagnostics_quest.rs`
   - `tests/grove_mystery_puzzle.rs`
   - `tests/crafting_system.rs`
   - `tests/dark_navigation.rs`

---

## IMPLEMENTATION ESTIMATES

- **Phase 1**: 2-3 hours (straightforward object creation)
- **Phase 2**: 4-5 hours (dialogue trees are verbose)
- **Phase 3**: 3-4 hours (quest wiring)
- **Phase 4**: 5-6 hours (complex mechanics)
- **Testing/Polish**: 2-3 hours

**Total**: ~16-21 hours of focused implementation

---

## ROLL-BACK STRATEGY

Each phase should be committed separately:
```bash
git add -p  # Stage changes incrementally
git commit -m "feat(content): Phase 1 - Foundation objects"
```

If issues arise, can revert individual phases without losing all work.

---

## SUCCESS CRITERIA

✅ All 15 rooms have at least one interactive element
✅ All 4 NPCs have working dialogue trees
✅ All 4 quests are completable end-to-end
✅ All 4 puzzles have clear mechanics and feedback
✅ No orphaned object/NPC/quest references
✅ All integration tests pass
✅ Player can discover content naturally through exploration

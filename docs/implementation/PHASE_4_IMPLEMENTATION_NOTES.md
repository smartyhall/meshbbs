# Phase 4 Implementation Notes

## Overview
Phase 4 involves implementing puzzle mechanics that require command system modifications. This document provides implementation guidance for the remaining features.

**Status as of 2025-10-14**:
- ✅ Phase 4.1: Tower Climb Puzzle (COMPLETE)
- ⏳ Phase 4.2: Symbol Sequence Puzzle (NOT STARTED - requires EXAMINE tracking)
- ⏳ Phase 4.3: Dark Tunnel Navigation (NOT STARTED - requires visibility system)
- ⏳ Phase 4.4: Crafting System (NOT STARTED - requires CRAFT command)

---

## ✅ PHASE 4.1: TOWER CLIMB PUZZLE (Complete)

### Implementation Summary
- **Room Added**: `repeater_upper` (16th room in world)
- **Object Added**: `diagnostic_panel` (interactive equipment)
- **Navigation**: UP from `repeater_tower` → `repeater_upper`, DOWN to return
- **Quest Integration**: VisitLocation objective tracks visit to upper platform

### How It Works
1. Player enters `repeater_tower`
2. Types `UP` to climb ladder
3. Arrives at `repeater_upper` platform
4. Quest objective completes when location is visited
5. Player types `DOWN` to descend

### No Additional Code Needed
The existing UP/DOWN command handlers in `src/tmush/commands.rs` already support vertical navigation. The room exits handle the puzzle automatically.

---

## ⏳ PHASE 4.2: SYMBOL SEQUENCE PUZZLE

### Current Status
**NOT IMPLEMENTED** - Requires EXAMINE command enhancement

### Design Specification

#### Puzzle Mechanic
Players must examine 4 tree carvings in the Ancient Grove in the correct sequence:
1. Oak (broadcast symbol)
2. Elm (frequency symbol)
3. Willow (network symbol)
4. Ash (convergence symbol)

#### Required Implementation

**File**: `src/tmush/commands.rs`

**Location**: Modify the EXAMINE command handler

**Pseudocode**:
```rust
// In EXAMINE command handler:
if object_id.starts_with("carved_symbols_") {
    // Check if player has grove_mystery quest active
    if let Some(quest) = player.get_active_quest("grove_mystery") {
        // Track examination in quest state
        let expected_sequence = ["oak", "elm", "willow", "ash"];
        let current_position = quest.get_sequence_progress();
        
        // Extract symbol type from object_id
        let symbol_type = object_id.strip_prefix("carved_symbols_").unwrap();
        
        if symbol_type == expected_sequence[current_position] {
            // Correct symbol! Advance sequence
            quest.increment_sequence_progress();
            send_to_player("The symbol resonates with understanding...");
            
            // Complete objective when sequence is done
            if current_position == 3 { // Last symbol
                complete_quest_objective("examine_sequence");
            }
        } else {
            // Wrong symbol - reset sequence
            quest.reset_sequence_progress();
            send_to_player("The symbols feel disconnected. Perhaps start from the beginning?");
        }
    }
}
```

#### Data Structure Changes

**Option 1**: Add sequence tracking to QuestProgress
```rust
// In src/tmush/types.rs
pub struct PlayerQuestProgress {
    pub quest_id: String,
    pub objectives: Vec<ObjectiveProgress>,
    pub started_at: DateTime<Utc>,
    // NEW: Track sequence state
    pub sequence_progress: u8, // 0-3 for 4-symbol sequence
}
```

**Option 2**: Use existing progress counters
```rust
// Use the ObjectiveProgress.count field
// to track which symbol in sequence (0-3)
// Reset to 0 if wrong symbol examined
```

#### Quest Objective Update

The `grove_mystery` quest currently has 4 separate "UseItem" objectives for each symbol. These would need to be:
- **Option A**: Replaced with a single "ExamineSequence" objective type
- **Option B**: Modified to check sequence order before completing

**Recommended**: Add new `ObjectiveType` variant:
```rust
pub enum ObjectiveType {
    // ... existing variants ...
    ExamineSequence { 
        object_ids: Vec<String>,  // Ordered list
        reset_on_error: bool,
    },
}
```

---

## ⏳ PHASE 4.3: DARK TUNNEL NAVIGATION

### Current Status
**NOT IMPLEMENTED** - Requires visibility system

### Design Specification

#### Puzzle Mechanic
The maintenance tunnels are dark. Players need a light source to see objects and read descriptions properly.

#### Required Implementation

**Files**: 
- `src/tmush/types.rs` - Add ObjectFlag::LightSource
- `src/tmush/commands.rs` - Modify LOOK command

**Step 1**: Add Light Source Flag
```rust
// In src/tmush/types.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ObjectFlag {
    // ... existing flags ...
    LightSource, // Emits light in dark rooms
}
```

**Step 2**: Create Torch/Lantern Object
```rust
// Add to create_content_objects()
let torch = ObjectRecord {
    id: "basic_torch".to_string(),
    name: "Torch".to_string(),
    description: "A simple wooden torch that provides light.".to_string(),
    // ... other fields ...
    flags: vec![ObjectFlag::LightSource],
    takeable: true,
};
```

**Step 3**: Modify LOOK Command
```rust
// In LOOK command handler:
fn handle_look(player: &Player, room: &Room, store: &Store) -> String {
    // Check if room is dark
    if room.flags.contains(&RoomFlag::Dark) {
        // Check if player has light source
        let has_light = player.inventory.iter().any(|item_id| {
            if let Ok(item) = store.get_object(item_id) {
                item.flags.contains(&ObjectFlag::LightSource)
            } else {
                false
            }
        });
        
        if !has_light {
            return "It's too dark to see anything. You need a light source.".to_string();
        }
    }
    
    // Normal room description
    format_room_description(room)
}
```

**Step 4**: Hide Objects in Dark
```rust
// In object listing logic:
if room.flags.contains(&RoomFlag::Dark) && !player_has_light() {
    // Show no objects or only light-emitting ones
    return vec![];
}
```

#### Quest Integration

The `tunnel_salvage` quest objectives should automatically work once visibility is implemented:
- Player needs torch to see wire spools and scrap metal
- CollectItem objectives trigger when items are taken
- No special quest code needed

#### Object Placement

Add torches to multiple locations for player access:
- Town Square (purchase from merchant)
- South Market (for sale)
- City Hall (free for explorers)
- North Gate (adventurer supply)

---

## ⏳ PHASE 4.4: CRAFTING SYSTEM

### Current Status
**NOT IMPLEMENTED** - Requires CRAFT command

### Design Specification

#### Puzzle Mechanic
Players learn to craft items by combining materials at the crafting bench in Workshop District.

#### Required Implementation

**File**: `src/tmush/commands.rs`

**Step 1**: Add CRAFT Command Variant
```rust
// In TinyMushCommand enum:
pub enum TinyMushCommand {
    // ... existing commands ...
    Craft { recipe_name: String },
}
```

**Step 2**: Add Command Parser
```rust
// In command parsing logic:
if cmd == "CRAFT" || cmd == "MAKE" || cmd == "BUILD" {
    if parts.len() < 2 {
        return Err("Usage: CRAFT <recipe_name>".into());
    }
    return Ok(TinyMushCommand::Craft {
        recipe_name: parts[1..].join(" ").to_lowercase(),
    });
}
```

**Step 3**: Define Recipes
```rust
// Recipe data structure:
pub struct CraftingRecipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<(String, u32)>, // (item_id, quantity)
    pub result: String, // result item_id
    pub requires_bench: bool,
}

// Recipe database:
fn get_crafting_recipes() -> Vec<CraftingRecipe> {
    vec![
        CraftingRecipe {
            id: "signal_booster".to_string(),
            name: "Signal Booster".to_string(),
            ingredients: vec![
                ("wire_spool".to_string(), 1),
                ("scrap_metal".to_string(), 1),
            ],
            result: "signal_booster_crafted".to_string(),
            requires_bench: true,
        },
        CraftingRecipe {
            id: "basic_antenna".to_string(),
            name: "Basic Antenna".to_string(),
            ingredients: vec![
                ("wire_spool".to_string(), 2),
                ("basic_component".to_string(), 1),
            ],
            result: "basic_antenna_crafted".to_string(),
            requires_bench: true,
        },
    ]
}
```

**Step 4**: Implement CRAFT Handler
```rust
async fn handle_craft(
    username: &str,
    recipe_name: &str,
    store: &TinyMushStore,
) -> Result<String, TinyMushError> {
    let mut player = store.get_player(username)?;
    let current_room = store.get_room(&player.current_room)?;
    
    // Step 1: Find recipe
    let recipes = get_crafting_recipes();
    let recipe = recipes.iter()
        .find(|r| r.name.to_lowercase() == recipe_name)
        .ok_or("Unknown recipe. Check the crafting bench for available recipes.")?;
    
    // Step 2: Check if at crafting bench (if required)
    if recipe.requires_bench {
        let has_bench = current_room.id == "workshop_district" || 
                       store.room_has_object(&current_room.id, "crafting_bench")?;
        if !has_bench {
            return Err("You need to be at a crafting bench to make that.".into());
        }
    }
    
    // Step 3: Check player has all ingredients
    for (item_id, required_qty) in &recipe.ingredients {
        let player_qty = player.inventory.iter()
            .filter(|inv_item| inv_item.starts_with(item_id))
            .count() as u32;
        
        if player_qty < *required_qty {
            return Err(format!(
                "You need {} {} but only have {}.",
                required_qty, item_id, player_qty
            ).into());
        }
    }
    
    // Step 4: Consume ingredients
    for (item_id, required_qty) in &recipe.ingredients {
        for _ in 0..*required_qty {
            // Find and remove one instance of this item
            if let Some(pos) = player.inventory.iter()
                .position(|inv_item| inv_item.starts_with(item_id)) 
            {
                player.inventory.remove(pos);
            }
        }
    }
    
    // Step 5: Create result item
    let crafted_item = ObjectRecord::new_player_owned(
        &recipe.result,
        &recipe.name,
        &format!("A {} you crafted yourself.", recipe.name),
        username,
        OwnershipReason::Crafted,
    );
    store.put_object(crafted_item)?;
    player.inventory.push(recipe.result.clone());
    
    // Step 6: Update quest objective (if applicable)
    // Check if player has first_craft quest and complete objective
    
    // Step 7: Save player
    store.put_player(player)?;
    
    Ok(format!(
        "✨ You successfully craft a {}! The components merge together perfectly.",
        recipe.name
    ))
}
```

#### Quest Integration

The `first_craft` quest should complete when:
1. Player has talked to Tinker Brass
2. Player has collected materials
3. Player is in Workshop District
4. Player executes: `CRAFT signal_booster`

Add quest completion check in the CRAFT handler:
```rust
// After successful craft:
if recipe.id == "signal_booster" {
    if let Ok(mut progress) = store.get_player_quest_progress(username, "first_craft") {
        // Complete the "first craft" objective
        complete_quest_objective(&mut progress, "craft_item");
        store.save_quest_progress(username, progress)?;
    }
}
```

---

## INTEGRATION CHECKLIST

### Prerequisites
Before the content goes live, ensure:

1. **Object Seeding**: Call `create_content_objects(now)` during world init
2. **NPC Seeding**: Call `create_content_npcs(now)` during NPC init  
3. **Quest Seeding**: Call `create_content_quests(now)` during quest init

### Testing Requirements

**Integration Tests Needed**:
1. `tests/tower_diagnostics_quest.rs`
   - Accept quest from Old Graybeard
   - Use diagnostic panel
   - Climb to repeater_upper (UP command)
   - Inspect northern array
   - Verify quest completion and rewards

2. `tests/grove_mystery_puzzle.rs`
   - Accept quest from Old Elm
   - Visit Ancient Grove
   - Examine symbols in correct order
   - Verify sequence tracking
   - Test reset on wrong symbol
   - Verify quest completion

3. `tests/crafting_system.rs`
   - Collect crafting materials
   - Visit workshop district
   - Craft signal booster
   - Verify inventory changes
   - Test invalid recipes
   - Test insufficient materials

4. `tests/dark_navigation.rs`
   - Enter maintenance tunnels without light
   - Verify limited visibility
   - Equip torch/lantern
   - Verify full visibility
   - Test object collection in dark

### Performance Considerations

- **Sequence Tracking**: Store in memory, persist with quest progress
- **Light Source Check**: Cache result per command to avoid repeated inventory scans
- **Recipe Lookup**: Consider caching recipe database
- **Crafting**: Validate atomically (all checks before any changes)

---

## FUTURE ENHANCEMENTS

### After Phase 4 Complete

1. **More Recipes**:
   - Advanced antenna (3 wire + 2 components)
   - Repair kit (2 scrap + 1 component)
   - Signal amplifier (1 signal_booster + 1 component)

2. **Dynamic Light Sources**:
   - Torches burn out after X minutes
   - Lanterns need oil refills
   - Glowsticks (disposable, limited duration)

3. **Complex Sequences**:
   - Musical note puzzles
   - Color pattern matching
   - Directional sequences (N,E,S,W)

4. **Crafting Progression**:
   - Skill levels (novice → expert)
   - Failure chances decrease with practice
   - Unlockable recipes from quest rewards

5. **Multiplayer Crafting**:
   - Co-op crafting (2+ players required)
   - Player-run workshops
   - Commissioned crafting requests

---

## SUMMARY

### Work Completed (Phase 4.1)
- ✅ Tower climb puzzle with vertical navigation
- ✅ `repeater_upper` room with atmospheric description
- ✅ `diagnostic_panel` interactive object
- ✅ UP/DOWN navigation working via existing commands

### Work Remaining
- ⏳ Symbol sequence tracking (Phase 4.2)
- ⏳ Visibility system for dark rooms (Phase 4.3)
- ⏳ CRAFT command implementation (Phase 4.4)
- ⏳ Integration tests for all mechanics

### Estimated Effort
- Phase 4.2: 2-3 hours (sequence tracking logic)
- Phase 4.3: 2-3 hours (visibility system + torch objects)
- Phase 4.4: 4-5 hours (CRAFT command + recipe system)
- Testing: 3-4 hours (4 integration test files)

**Total Remaining**: ~11-15 hours

### Recommendation
The foundation (Phases 1-3 + 4.1) is solid and provides immediate value:
- 16 rooms with rich descriptions
- 4 NPCs with 24 dialogue nodes
- 4 quests (one fully functional: tower_diagnostics)
- 13 interactive objects

Consider merging the current implementation and addressing Phases 4.2-4.4 in follow-up PRs. This allows:
- Earlier player feedback
- Incremental testing
- Iterative refinement
- Parallel development if needed


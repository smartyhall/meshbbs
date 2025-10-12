# NPC Dialogue Automatic Seeding

**Status**: ✅ Complete  
**Date**: October 10, 2025  
**Implementation**: `src/tmush/state.rs::seed_npc_dialogues_if_needed()`

## Overview

All NPC dialogue trees are now automatically populated during database initialization. No manual admin commands or shell scripts are required - dialogues are seeded alongside NPCs on first run.

## How It Works

1. **Database Initialization** (`src/tmush/storage.rs`)
   ```rust
   store.seed_npcs_if_needed()?;
   crate::tmush::state::seed_npc_dialogues_if_needed(&store)?;
   ```

2. **Dialogue Seeding** (`src/tmush/state.rs`)
   - Checks if each NPC's `dialog_tree` is empty
   - If empty, populates with complete branching dialogue
   - If populated, skips (preserves custom changes)
   - Returns count of NPCs updated

3. **Safe & Idempotent**
   - Only seeds on empty trees
   - Won't overwrite admin customizations
   - Can be called repeatedly without issue

## Seeded NPCs

### 1. Mayor Thompson (9 nodes)
**Location**: `mayor_office`  
**Role**: Tutorial and town information

**Dialogue Tree**:
- `greeting` → Main entry point (4 choices)
- `tutorial` → Tutorial information
- `tutorial_start` → How to begin
- `tutorial_commands` → Command reference
- `skip_tutorial` → Skip option
- `town_info` → Town overview
- `housing_info` → Housing details
- `quest_check` → Quest availability

**Topics**: Tutorial guidance, town features, housing, quests

---

### 2. City Clerk (7 nodes)
**Location**: `city_hall_lobby`  
**Role**: Administrative assistance

**Dialogue Tree**:
- `greeting` → Main entry point (4 choices)
- `housing_help` → Housing system overview
- `housing_cost` → Pricing information
- `quest_help` → Quest information
- `quest_locations` → Where to find quests
- `services` → City Hall services

**Topics**: Housing claims, administrative help, quest guidance

---

### 3. Gate Guard (8 nodes)
**Location**: `north_gate`  
**Role**: Security and wilderness warnings

**Dialogue Tree**:
- `greeting` → Main entry point (4 choices)
- `looking` → Wilderness overview
- `dangers` → Hazard warnings
- `adventure_warning` → Stay in town advice
- `equipment` → Preparation recommendations
- `market_direction` → Directions to market
- `advice` → Safety tips

**Topics**: Wilderness dangers, equipment needs, safety advice

---

### 4. Market Vendor (Mira) (13 nodes)
**Location**: `south_market`  
**Role**: Trading and commerce

**Dialogue Tree**:
- `greeting` → Main entry point (4 choices)
- `wares` → Inventory overview
- `best_item` → Featured item
- `buy_booster` → Purchase signal booster (placeholder)
- `story` → Family history
- `history_detail` → Grandmother's legacy
- `buying` → Purchase flow
- `prices` → Current items
- `purchase_knife` → Buy knife (placeholder)
- `valuable_items` → Rare finds
- `finding_valuables` → How to find
- `weapons` → Weapons available

**Topics**: Trading, family history, mesh hardware, purchases

**Note**: Transaction nodes (`purchase_knife`, `buy_booster`) are placeholders. Admins should use `@DIALOG EDIT` to add:
- `DialogCondition::HasCurrency` checks
- `DialogAction::TakeCurrency` + `DialogAction::GiveItem` actions

---

### 5. Museum Curator (Dr. Reeves) (10 nodes)
**Location**: `mesh_museum`  
**Role**: History and lore

**Dialogue Tree**:
- `greeting` → Main entry point (4 choices)
- `about_museum` → Museum purpose
- `founder` → Museum founders
- `history` → Network history overview
- `history_detail` → Winter Storm '19
- `relay_story` → Famous relay tale
- `exhibit` → Main exhibits
- `other_exhibits` → Additional displays
- `pioneers` → Network pioneers
- `heroes` → Notable individuals

**Topics**: Mesh network history, Winter Storm '19, pioneers, exhibits

---

## Total Dialogue Content

- **5 NPCs** with full dialogue trees
- **47 dialogue nodes** total
- **All branching paths** validated
- **All goto targets** exist
- **All exit paths** functional

## Benefits Over Manual Setup

✅ **Automatic** - No admin commands needed  
✅ **Idempotent** - Safe to run multiple times  
✅ **Version Controlled** - Content lives in source code  
✅ **Consistent** - Same dialogues on every fresh install  
✅ **Testable** - Unit tests verify seeding works  
✅ **Maintainable** - Easy to update and extend  

## Customization

Admins can still customize dialogues using `@DIALOG` commands:

```
@DIALOG <npc> VIEW <topic>           - View dialogue JSON
@DIALOG <npc> EDIT <topic> <json>    - Edit dialogue tree
@DIALOG <npc> ADD <topic> <text>     - Add simple text topic
@DIALOG <npc> DELETE <topic>         - Remove topic
@DIALOG <npc> TEST <topic>           - Test conditions
```

Custom changes are preserved - seeding only happens on empty trees.

## Testing

Run: `cargo test --lib mayor_dialogue`

Creates a fresh database and verifies dialogue seeding completes without errors.

## Future Enhancements

### Transaction Actions
Market Vendor's purchase nodes need proper actions:

```json
{
  "text": "Excellent choice! The knife is 15 credits.",
  "conditions": [
    {"type": "has_currency", "amount": 15}
  ],
  "actions": [
    {"type": "take_currency", "amount": 15},
    {"type": "give_item", "item_id": "basic_knife", "quantity": 1},
    {"type": "send_message", "text": "You received: Basic Knife"}
  ],
  "choices": [
    {"label": "Thank you!", "exit": true}
  ]
}
```

Use `@DIALOG market_vendor EDIT purchase_knife <json>` to add these.

### Quest Integration
Mayor Thompson's `quest_check` node could use:

```json
{
  "conditions": [
    {"type": "quest_status", "quest_id": "tutorial_quest", "status": "completed"}
  ],
  "actions": [
    {"type": "start_quest", "quest_id": "first_real_quest"}
  ]
}
```

### Achievement Tracking
Museum Curator could grant achievements:

```json
{
  "actions": [
    {"type": "grant_achievement", "achievement_id": "history_buff"}
  ]
}
```

## Related Documentation

- [Phase 3: Branching Dialogue](PHASE3_BRANCHING_DIALOGUE.md)
- [Phase 4: Conditional Dialogue](PHASE4_CONDITIONAL_DIALOGUE.md)
- [Phase 5: Dialog Actions](PHASE5_DIALOG_ACTIONS.md)
- [Phase 6: Admin Dialog Editor](PHASE6_ADMIN_DIALOG_EDITOR.md)
- [NPC Dialogue Content](../content/npc_dialogues.md)

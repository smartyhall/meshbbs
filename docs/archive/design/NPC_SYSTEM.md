# NPC System - Multi-Topic Dialogue

**Date**: October 9, 2025  
**Status**: ✅ Phase 1 Complete - Multi-Topic Dialogue Active

## Overview

The NPC system supports interactive dialogue with topic-based conversations. Players can talk to NPCs using simple commands and explore multiple conversation topics.

## Commands

### Basic Interaction
```
TALK <npc>              # Start conversation with greeting
TALK <npc> <topic>      # Discuss specific topic
TALK <npc> LIST         # Show available topics
```

### Examples
```
TALK MAYOR              # Greets you with default greeting
TALK MIRA WARES         # Ask about vendor's inventory
TALK MIRA STORY         # Learn her background
TALK GUARD WARNING      # Get safety warnings
TALK CLERK LIST         # See what clerk can discuss
```

## NPCs Deployed

### 1. Mayor Thompson (Tutorial Completion)
- **ID**: `mayor_thompson`
- **Location**: `mayor_office` (Mayor's Office)
- **Role**: Tutorial completion, quest giver
- **Flags**: None (special logic in TALK handler)
- **Dialog**:
  - `greeting`: General welcome
  - `tutorial_complete`: Tutorial completion message
  - `quest_welcome`: Quest system hint

### 2. City Clerk (Administrative Help)
- **ID**: `city_clerk`
- **Location**: `city_hall_lobby` (City Hall Lobby)
- **Role**: Tutorial helper, game mechanics info
- **Flags**: `TutorialNpc`
- **Dialog**:
  - `greeting`: Welcome to City Hall
  - `help`: Explanation of game systems

### 3. Gate Guard (Security & Exploration)
- **ID**: `gate_guard`
- **Location**: `north_gate` (North Gate)
- **Role**: Direction, warnings, lore
- **Flags**: `Guard`
- **Dialog**:
  - `greeting`: Welcome to gate
  - `warning`: Danger warnings for beyond the gate

### 4. Mira the Vendor (Commerce)
- **ID**: `market_vendor`
- **Location**: `south_market` (South Market)
- **Role**: Trading, shop system (future)
- **Flags**: `Vendor`
- **Dialog**:
  - `greeting`: Welcome to stall
  - `wares`: Current inventory status
  - `story`: Family history lore

### 5. Dr. Reeves (Museum Curator)
- **ID**: `museum_curator`
- **Location**: `mesh_museum` (Mesh Museum)
- **Role**: Lore, history, education
- **Flags**: `TutorialNpc`
- **Dialog**:
  - `greeting`: Museum welcome
  - `history`: Mesh network history
  - `exhibit`: Specific artifact stories

## @EDITNPC Command

### Syntax

```
@EDITNPC <npc_id> <field> <value>
```

### Fields

| Field | Description | Example |
|-------|-------------|---------|
| `dialog.<key>` | Edit/add dialog entry | `@EDITNPC mayor_thompson dialog.greeting Hello!` |
| `description` | Edit NPC description | `@EDITNPC city_clerk description A helpful clerk` |
| `room` | Move NPC to different room | `@EDITNPC gate_guard room town_square` |

### Examples

**Edit greeting:**
```
@EDITNPC mayor_thompson dialog.greeting Welcome to my office, citizen!
```

**Add new dialog:**
```
@EDITNPC market_vendor dialog.special Today's special: mesh antennas!
```

**Change description:**
```
@EDITNPC museum_curator description An elderly scholar with countless stories
```

**Move NPC:**
```
@EDITNPC city_clerk room town_square
```

### Validation

- **Dialog/Description**: 500 character max
- **Room**: Must exist in database
- **Permissions**: Currently open for alpha (TODO: add admin role check)

## TALK Command

### Basic Usage

```
TALK <npc_name>
```

The system matches by partial name or ID (case-insensitive):
- `TALK MAYOR` → finds Mayor Thompson
- `TALK CLERK` → finds City Clerk  
- `TALK MIRA` → finds Mira the Vendor
- `TALK GUARD` → finds Gate Guard
- `TALK REEVES` → finds Dr. Reeves

### Dialog System

**Current Implementation**: Simple key-based lookup
- Checks for special logic (e.g., Mayor tutorial completion)
- Falls back to "default" or "greeting" key
- Returns generic message if no dialog found

**Future Enhancement**: Conditional dialog trees based on:
- Player quest progress
- Inventory items
- Time of day
- Player reputation
- Previous conversations

## Testing Checklist

✅ **5 NPCs seeded** on database init  
✅ **@EDITNPC command** implemented  
✅ **TALK command** works with all NPCs  
✅ **Tutorial completion** via Mayor Thompson  
✅ **Dialog validation** (500 char limit)  
✅ **Room validation** (must exist)  
✅ **All 8 tutorial tests** passing  

## Alpha Testing Instructions

### To See NPCs in Action:

```bash
# 1. Restart meshbbs (delete DB to reseed)
rm -rf /opt/meshbbs/data/tinymush.db

# 2. Connect and explore
# Landing Gazebo -> no NPCs here
# Town Square -> no NPCs here  
# City Hall Lobby -> TALK CLERK
# Mayor's Office -> TALK MAYOR (completes tutorial)
# North Gate -> TALK GUARD
# South Market -> TALK MIRA
# Mesh Museum -> TALK REEVES
```

### Example Interactions:

**City Hall Lobby:**
```
> LOOK
=== City Hall Lobby ===
Clerks shuffle reports...

> TALK CLERK
City Clerk: 'Welcome to City Hall! I handle administrative matters. 
If you need help understanding how things work around here, just ask!'
```

**South Market:**
```
> TALK MIRA
Mira the Vendor: 'Welcome to my stall! I've got the finest mesh 
components and supplies in Old Towne. Looking for anything particular?'

> TALK VENDOR
Mira the Vendor: 'Welcome to my stall!...'
```

### Admin Commands:

**Change Mayor's greeting:**
```
@EDITNPC mayor_thompson dialog.greeting Greetings, citizen of the mesh!
```

**Add new dialog to vendor:**
```
@EDITNPC market_vendor dialog.prices Basic antenna: 50cp, Module: 200cp
```

**Move curator to town square (for testing):**
```
@EDITNPC museum_curator room town_square
```

## Technical Details

### Database Storage

- **Tree**: `npcs` (Sled key-value store)
- **Key Format**: `npc:{npc_id}`
- **Value**: Serialized `NpcRecord` (bincode)
- **Seeding**: `seed_npcs_if_needed()` on database init

### NPC Record Structure

```rust
pub struct NpcRecord {
    pub id: String,                        // Unique ID
    pub name: String,                      // Display name
    pub title: String,                     // Title/role
    pub description: String,               // Appearance
    pub room_id: String,                   // Current location
    pub dialog: HashMap<String, String>,   // Dialog entries
    pub flags: Vec<NpcFlag>,               // Behavior flags
    pub created_at: DateTime<Utc>,
    pub schema_version: u8,
}
```

### NPC Flags

```rust
pub enum NpcFlag {
    TutorialNpc,  // Helps new players
    QuestGiver,   // Offers quests (future)
    Vendor,       // Trading (future shop system)
    Guard,        // Security/protection
    Immortal,     // Cannot be killed (future combat)
}
```

## Future Enhancements

### Dialog Trees (High Priority)
- Branching conversations
- State-dependent responses
- Quest integration
- Item/inventory checks

### Shop Integration (Medium Priority)
- Vendor NPCs sell items
- Dynamic pricing
- Inventory management
- Buy/sell commands

### Quest System Integration (Medium Priority)
- Quest givers offer quests
- Track quest progress in dialog
- Rewards from NPCs

### Enhanced Admin Commands (Low Priority)
- `@LISTNPCS` - List all NPCs
- `@CREATENPC` - Create new NPC
- `@DELETENPC` - Remove NPC
- `@NPCDIALOG` - Manage dialog trees visually

## Known Limitations

1. **No Dialog Trees**: Currently single-response per key
2. **No Permissions**: @EDITNPC open to all (needs admin check)
3. **No Visual Editor**: Text-based editing only
4. **Static Locations**: NPCs don't move (except via @EDITNPC)
5. **No AI**: Pre-written responses only

## Files Changed

```
Modified:
  src/tmush/state.rs          - Added 4 NPCs to seed_starter_npcs()
  src/tmush/commands.rs       - Added @EDITNPC command
  
Created:
  docs/development/NPC_SYSTEM.md (this file)
```

## Success Metrics for Alpha

- [ ] Players successfully talk to all 5 NPCs
- [ ] Tutorial completion via Mayor Thompson works
- [ ] Admin successfully uses @EDITNPC to customize dialog
- [ ] No errors when moving NPCs between rooms
- [ ] Players report NPCs feel alive and helpful

---

**Status**: 🚀 Ready for alpha testing!

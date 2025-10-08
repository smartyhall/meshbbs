# Tutorial System Design - Phase 6 Week 1

**Status**: In Progress  
**Target**: Phase 6 Week 1 completion  
**Related**: MUD_MUSH_DESIGN.md §Tutorial, TINYMUSH_IMPLEMENTATION_PLAN.md §Phase 6

---

## Overview

The tutorial system guides new players through their first experience in Old Towne Mesh, teaching basic commands, navigation, and economy systems through a scripted quest flow.

## Tutorial Flow: Gazebo → Mayor → City Hall

### Stage 1: Welcome at Gazebo (gazebo_landing)
- **NPC**: Welcome Guide
- **Objective**: Learn basic commands (LOOK, HELP, directional movement)
- **Trigger**: Auto-start on first login (tutorial_state = NotStarted)
- **Completion**: Move north to Town Square

### Stage 2: Navigate to City Hall (town_square → city_hall_lobby)
- **Objective**: Navigate from Town Square to City Hall Lobby
- **Commands Taught**: North/South movement, WHERE command
- **Completion**: Enter city_hall_lobby

### Stage 3: Meet the Mayor (mayor_office)
- **NPC**: Mayor Thompson
- **Objective**: Speak with the Mayor using TALK command
- **Rewards**: 
  - Starter currency (100cp or $10.00 depending on world config)
  - Welcome item (Town Map)
- **Completion**: Receive welcome rewards

---

## Data Structures

### TutorialState Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TutorialState {
    /// Player has not started tutorial
    NotStarted,
    
    /// Player is in progress at a specific step
    InProgress { step: TutorialStep },
    
    /// Player has completed tutorial
    Completed { 
        completed_at: DateTime<Utc> 
    },
    
    /// Player manually skipped tutorial
    Skipped { 
        skipped_at: DateTime<Utc> 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TutorialStep {
    /// Step 1: Learn basics at gazebo
    WelcomeAtGazebo,
    
    /// Step 2: Navigate to City Hall
    NavigateToCityHall,
    
    /// Step 3: Meet the Mayor
    MeetTheMayor,
}
```

### PlayerRecord Changes

Add tutorial_state field to PlayerRecord:

```rust
pub struct PlayerRecord {
    // ... existing fields ...
    
    /// Tutorial progression state
    #[serde(default)]
    pub tutorial_state: TutorialState,
    
    pub schema_version: u8,
}
```

### NPC System (Simplified for Tutorial)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcRecord {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub room_id: String,
    pub dialog: HashMap<String, String>, // key -> response
    pub flags: Vec<NpcFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NpcFlag {
    TutorialNpc,
    QuestGiver,
    Vendor,
    Guard,
    Immortal,
}
```

---

## New Rooms Required

### 1. Mayor's Office (mayor_office)

```rust
RoomRecord::world(
    "mayor_office",
    "Mayor's Office",
    "A well-appointed office with oak desk and mesh maps on walls.",
    "Mayor Thompson sits behind a sturdy oak desk, reviewing network topology 
maps. Framed certificates line the walls alongside charts tracking mesh 
uptime metrics. A window overlooks the town square.",
)
.with_exit(Direction::South, "city_hall_lobby")
.with_flag(RoomFlag::Safe)
.with_flag(RoomFlag::Indoor)
.with_flag(RoomFlag::QuestLocation)
```

---

## Commands

### TUTORIAL Command

```
TUTORIAL               - Show current tutorial status
TUTORIAL SKIP          - Skip the tutorial (warns about missing rewards)
TUTORIAL RESTART       - Restart tutorial from beginning
```

### TALK Command (NPC Interaction)

```
TALK <npc>             - Initiate conversation with NPC
TALK MAYOR             - Talk to Mayor Thompson
TALK GUIDE             - Talk to Welcome Guide
```

Alias: GREET

---

## Tutorial Rewards

### Completion Rewards:
1. **Starter Currency**: 
   - MultiTier: 100 copper (1sp)
   - Decimal: $10.00 (1000 minor units)

2. **Welcome Item**: "Town Map"
   - Description: "Hand-drawn map of Old Towne Mesh"
   - Weight: 1
   - Value: 0 (quest item)
   - Flag: KeyItem

3. **Achievement**: "Welcome to Old Towne" (if achievement system exists)

---

## Implementation Steps

### Step 1: Tutorial Data Structures (types.rs)
- [ ] Add TutorialState enum
- [ ] Add TutorialStep enum  
- [ ] Add tutorial_state field to PlayerRecord
- [ ] Add NpcRecord struct
- [ ] Add NpcFlag enum
- [ ] Update Default impl for PlayerRecord

### Step 2: Storage Methods (storage.rs)
- [ ] Add NPC storage methods (put_npc, get_npc, list_npc_ids)
- [ ] Add get_npcs_in_room method
- [ ] Seed tutorial NPCs on first run

### Step 3: New Room (state.rs)
- [ ] Add mayor_office to canonical_world_seed
- [ ] Update OLD_TOWNE_WORLD_ROOM_IDS constant

### Step 4: Tutorial Logic (new file: tutorial.rs)
- [ ] Tutorial progression checker
- [ ] Auto-start logic on first login
- [ ] Step advancement logic
- [ ] Reward distribution

### Step 5: Commands (commands.rs)
- [ ] TUTORIAL command (status, skip, restart)
- [ ] TALK command (NPC interaction)
- [ ] Integrate tutorial hooks into movement commands

### Step 6: Tests (tests/tutorial_system.rs)
- [ ] Test tutorial state tracking
- [ ] Test NPC interactions
- [ ] Test tutorial completion flow
- [ ] Test rewards distribution
- [ ] Test skip functionality

---

## Success Criteria

- [ ] New players automatically start tutorial at gazebo_landing
- [ ] Tutorial guides players through Gazebo → City Hall → Mayor's Office
- [ ] NPCs respond to TALK command with contextual dialog
- [ ] Tutorial completion grants starter currency and welcome item
- [ ] TUTORIAL command shows current progress
- [ ] Players can skip tutorial (with warning)
- [ ] All tutorial messages under 200 bytes
- [ ] Zero compiler warnings
- [ ] All tests passing

---

## Future Enhancements (Not in Week 1)

- Interactive tutorial objectives UI
- Tutorial hints for stuck players
- Multiple tutorial paths
- Tutorial achievement tracking
- Tutorial NPC animations/behaviors
- Voice lines for NPCs (text-based descriptions)

---

## Notes

- Keep tutorial short (3-5 minutes)
- Make skip option prominent
- Don't gate essential commands behind tutorial
- Tutorial should teach, not restrict
- All dialog must fit in 200-byte Meshtastic limit

# MeshBBS v1.0.115-beta QA Test Script

**Testing Phase 4 & Phase 5 Content**

**Tester:** _________________  
**Date:** _________________  
**Version:** 1.0.115-beta  
**Environment:** _________________

---

## Pre-Test Setup

### 1. Start Fresh Server
```bash
# Build and run the server
cargo build --release
./target/release/meshbbs

# Or if installed:
systemctl restart meshbbs
```

### 2. Connect as Test User
```
# Use your Meshtastic client or test harness
# Create a fresh test character: "QATester"
```

### 3. Verify Version
```
Expected output: meshbbs 1.0.115
```

---

## Test Suite 1: Basic World Navigation

### Test 1.1: Tutorial Completion
**Objective:** Verify tutorial still works with new world

**Steps:**
1. Start as new character
2. Complete tutorial normally
3. Reach Town Square

**Expected Results:**
- [ ] Tutorial completes without errors
- [ ] Land in Town Square (not Landing Gazebo)
- [ ] Mayor Thompson dialogue accessible after tutorial

**Notes:**
_____________________________________________________________

---

### Test 1.2: Central Hub Navigation
**Objective:** Test all connections from Town Square

**From Town Square, test each direction:**

```
LOOK
GO NORTH  → City Hall Lobby
    LOOK  → Verify description
    GO SOUTH  → Return to Town Square

GO EAST  → Mesh Museum
    LOOK  → Verify description
    GO WEST  → Return to Town Square

GO WEST  → West Residential
    LOOK  → Verify description
    GO EAST  → Return to Town Square

GO SOUTH  → South Market
    LOOK  → Verify description
    GO NORTH  → Return to Town Square

GO DOWN  → Maintenance Tunnels (DARK)
    LOOK  → Should show darkness message
    GO UP  → Return to Town Square
```

**Checklist:**
- [ ] All 5 directions work from Town Square
- [ ] All return paths work correctly
- [ ] Room descriptions display properly
- [ ] Maintenance Tunnels shows darkness warning

**Notes:**
_____________________________________________________________

---

## Test Suite 2: Dark Navigation System

### Test 2.1: Dark Rooms Without Light
**Objective:** Verify dark room behavior without light source

**Steps:**
```
# Start at Town Square
GO DOWN  → Maintenance Tunnels
LOOK
```

**Expected Results:**
- [ ] See "=== Darkness ===" message
- [ ] Warning about needing light source
- [ ] Cannot see room objects
- [ ] Cannot see exits clearly
- [ ] CAN still move (soft requirement)

**Test Movement in Dark:**
```
GO DOWN  → Deep Caverns Entrance
LOOK  → Should still be dark
GO UP  → Return to Maintenance Tunnels
GO UP  → Return to Town Square
```

**Checklist:**
- [ ] Darkness message appears
- [ ] Movement still possible
- [ ] No crashes or errors

**Notes:**
_____________________________________________________________

---

### Test 2.2: Obtaining Light Source
**Objective:** Find or buy a light source

**Option A - Find Torch:**
```
GO SOUTH  → South Market
LOOK
# Check for torch or other light sources
TAKE TORCH
INVENTORY  → Verify torch in inventory
```

**Option B - Buy Lantern:**
```
GO SOUTH  → South Market
BUY LANTERN  → Costs 50 currency
INVENTORY  → Verify lantern in inventory
```

**Option C - Find in Workshop:**
```
GO SOUTH  → South Market
GO SOUTH  → Workshop District
LOOK
TAKE TORCH  (or TAKE GLOWSTICK)
```

**Checklist:**
- [ ] Can find or purchase light source
- [ ] Light source appears in inventory
- [ ] Item has correct name and description

**Notes:**
_____________________________________________________________

---

### Test 2.3: Dark Rooms With Light
**Objective:** Verify light source enables vision

**Prerequisites:** Have torch/lantern in inventory

**Steps:**
```
# From Town Square with light source
GO DOWN  → Maintenance Tunnels
LOOK
```

**Expected Results:**
- [ ] See full room name "Maintenance Tunnels"
- [ ] See complete room description
- [ ] See objects in room
- [ ] See all exits clearly
- [ ] NO darkness message

**Test Deep Caverns:**
```
GO DOWN  → Deep Caverns Entrance
LOOK  → Full description visible

GO DOWN  → Sunken Chamber
LOOK  → See flooded chamber description

GO EAST  → Hidden Vault
LOOK  → See vault with ancient equipment
```

**Checklist:**
- [ ] All 4 dark rooms show full descriptions with light
- [ ] Objects visible in each room
- [ ] No darkness warnings
- [ ] Navigation works normally

**Notes:**
_____________________________________________________________

---

### Test 2.4: Light Source Drop/Take
**Objective:** Verify light mechanics when dropping/picking up

**Steps:**
```
# In a dark room with light
LOOK  → Should see normally

DROP TORCH
LOOK  → Should see darkness

TAKE TORCH
LOOK  → Should see normally again
```

**Checklist:**
- [ ] Dropping light source triggers darkness
- [ ] Taking light source restores vision
- [ ] Transitions are immediate

**Notes:**
_____________________________________________________________

---

## Test Suite 3: World Map Completeness

### Test 3.1: North Zone Exploration
**Objective:** Visit all northern locations

**Route from Town Square:**
```
GO NORTH  → City Hall
    GO NORTH  → Mayor's Office
        LOOK  → Verify description
        EXAMINE objects
        GO SOUTH → Return to City Hall
    GO SOUTH  → Return to Town Square

# Different path to North Gate
GO EAST  → Mesh Museum
GO SOUTH  → North Gate
    LOOK
    GO NORTH  → Pine Ridge Trail
        LOOK
        GO NORTH  → Repeater Tower
            LOOK
            GO UP  → Repeater Upper Platform
                LOOK
                GO DOWN  → Return to Repeater
            GO SOUTH  → Return to Pine Ridge
        GO SOUTH  → Return to North Gate
```

**Checklist:**
- [ ] Mayor's Office accessible
- [ ] North Gate reachable
- [ ] Pine Ridge Trail found
- [ ] Repeater Tower accessible
- [ ] Repeater Upper Platform (UP direction) works

**Notes:**
_____________________________________________________________

---

### Test 3.2: East Zone Exploration
**Objective:** Visit Ancient Grove and quest locations

**Route from Town Square:**
```
GO EAST  → Mesh Museum
GO NORTH  → Forest Path
    LOOK
    GO EAST  → Ancient Grove
        LOOK  → Note peaceful atmosphere
        
        GO NORTH  → Cipher Chamber
            LOOK  → See four seasonal glyphs
            EXAMINE CIPHER_SPRING
            EXAMINE CIPHER_SUMMER
            EXAMINE CIPHER_AUTUMN
            EXAMINE CIPHER_WINTER
            GO SOUTH  → Return to Ancient Grove
            
        GO SOUTH  → Forgotten Ruins Entrance
            LOOK  → See four glyph pillars
            EXAMINE RUINS_GLYPH_ALPHA
            EXAMINE RUINS_GLYPH_BETA
            EXAMINE RUINS_GLYPH_GAMMA
            EXAMINE RUINS_GLYPH_DELTA
            
            # REQUIRES LIGHT SOURCE
            GO EAST  → Dark Passage
                LOOK  (with light)
                GO NORTH  → Artifact Chamber entrance
                    # May require key - don't try to enter yet
                    GO SOUTH  → Return to Dark Passage
                GO WEST  → Return to Ruins Entrance
            GO NORTH  → Return to Ancient Grove
```

**Checklist:**
- [ ] Forest Path accessible
- [ ] Ancient Grove found
- [ ] Cipher Chamber accessible (4 symbol glyphs present)
- [ ] Forgotten Ruins Entrance accessible (4 glyph pillars present)
- [ ] Dark Passage requires light source
- [ ] Artifact Chamber entrance found (locked)

**Notes:**
_____________________________________________________________

---

### Test 3.3: South Zone Exploration
**Objective:** Visit market and workshop areas

**Route from Town Square:**
```
GO SOUTH  → South Market
    LOOK  → See merchants, objects
    BUY TORCH  → Test purchasing
    
    GO SOUTH  → Workshop District
        LOOK  → See crafting area
        EXAMINE CRAFTING_BENCH
        # Try crafting if materials available
```

**Checklist:**
- [ ] South Market accessible
- [ ] Can purchase items
- [ ] Workshop District accessible
- [ ] Crafting bench present

**Notes:**
_____________________________________________________________

---

### Test 3.4: West Zone Exploration
**Objective:** Visit residential and social areas

**Route from Town Square:**
```
GO WEST  → West Residential
    LOOK
    TALK OLD ELM  → Quest giver
    
# Alternate route to Relay Tavern
GO NORTH  → City Hall
GO EAST  → Mesh Museum
GO SOUTH  → North Gate
GO WEST  → Relay Tavern
    LOOK  → Social hub
```

**Checklist:**
- [ ] West Residential accessible
- [ ] Old Elm NPC present
- [ ] Relay Tavern accessible
- [ ] Social atmosphere noted

**Notes:**
_____________________________________________________________

---

### Test 3.5: Underground Zone Exploration
**Objective:** Complete deep underground route (REQUIRES LIGHT)

**Prerequisites:** Have light source in inventory

**Route from Town Square:**
```
GO DOWN  → Maintenance Tunnels (DARK - need light)
    LOOK  (with light)
    TAKE objects if any
    
    GO DOWN  → Deep Caverns Entrance (DARK)
        LOOK  (with light)
        EXAMINE ROPE_COIL
        EXAMINE PICKAXE
        
        GO DOWN  → Sunken Chamber (DARK)
            LOOK  → See flooded chamber
            # Water on floor mentioned
            
            GO EAST  → Hidden Vault (DARK)
                LOOK  → See ancient artifacts
                EXAMINE SHELVES
                EXAMINE WORKBENCH
                TAKE CRYSTAL_OSCILLATOR (if present)
                TAKE POWER_CELL (if present)
```

**Checklist:**
- [ ] Maintenance Tunnels navigable with light
- [ ] Deep Caverns Entrance accessible
- [ ] Sunken Chamber shows flooding description
- [ ] Hidden Vault contains artifacts
- [ ] Objects can be examined and taken

**Notes:**
_____________________________________________________________

---

## Test Suite 4: Quest System

### Test 4.1: Quest Discovery
**Objective:** Find quest givers and view available quests

**Steps:**
```
# Check quest system works
QUEST LIST
QUEST ACTIVE

# Visit quest givers
GO WEST  → West Residential
TALK OLD ELM  → Should offer quests

GO SOUTH  → Workshop District
TALK TINKER BRASS  → Should offer crafting quests

# Check Repeater Tower
GO [navigate to Repeater Tower]
TALK OLD GRAYBEARD  → Should offer quests
```

**Checklist:**
- [ ] QUEST LIST command works
- [ ] Quest givers respond to TALK
- [ ] Quest dialogues appear
- [ ] Multiple quests available

**Notes:**
_____________________________________________________________

---

### Test 4.2: The Cipher Quest (Symbol Sequence)
**Objective:** Test Phase 4.2 symbol tracking

**Prerequisites:** Talk to Old Elm to accept quest

**Steps:**
```
# Accept quest
GO WEST  → West Residential
TALK OLD ELM
# Choose dialogue to accept "The Cipher"
QUEST ACCEPT THE_CIPHER

# Navigate to Cipher Chamber
GO EAST  → Town Square
GO EAST  → Mesh Museum
GO NORTH  → Forest Path
GO EAST  → Ancient Grove
GO NORTH  → Cipher Chamber

# Examine symbols IN CORRECT ORDER
EXAMINE CIPHER_SPRING  → 1st (Growth)
EXAMINE CIPHER_SUMMER  → 2nd (Strength)
EXAMINE CIPHER_AUTUMN  → 3rd (Change)
EXAMINE CIPHER_WINTER  → 4th (Rest)

# Check quest progress
QUEST STATUS THE_CIPHER

# Return to Old Elm
[navigate back to West Residential]
TALK OLD ELM
# Complete quest dialogue
```

**Expected Results:**
- [ ] Quest accepted successfully
- [ ] Symbol examination tracked
- [ ] Correct sequence recognized
- [ ] Quest completes
- [ ] Rewards received (10,000 currency, Decoder Lens, 250 XP)

**Test Wrong Order:**
```
# Try examining out of order
EXAMINE CIPHER_WINTER  (first - WRONG)
EXAMINE CIPHER_SPRING
# Should NOT complete
```

**Checklist:**
- [ ] Correct order completes quest
- [ ] Wrong order doesn't complete
- [ ] Can reset by talking to quest giver
- [ ] Rewards granted

**Notes:**
_____________________________________________________________

---

### Test 4.3: Into the Depths Quest (Dark Navigation)
**Objective:** Test Phase 4.3 light source requirement

**Prerequisites:** 
- Have light source
- Talk to Old Graybeard to accept quest

**Steps:**
```
# Accept quest
[Navigate to Repeater Tower]
TALK OLD GRAYBEARD
QUEST ACCEPT INTO_THE_DEPTHS

# Verify you have light source
INVENTORY  → Should show torch/lantern

# Navigate to Deep Caverns
GO [navigate to Town Square]
GO DOWN  → Maintenance Tunnels
GO DOWN  → Deep Caverns Entrance
GO DOWN  → Sunken Chamber
GO EAST  → Hidden Vault

# Quest should complete when reaching Hidden Vault
QUEST STATUS INTO_THE_DEPTHS

# Optional: Return to quest giver
[Navigate back to Repeater Tower]
TALK OLD GRAYBEARD
```

**Expected Results:**
- [ ] Quest accepted
- [ ] Light source detected by quest system
- [ ] Can navigate all dark rooms
- [ ] Quest completes at Hidden Vault
- [ ] Rewards received (12,000 currency, Ancient Relay Core, 300 XP)

**Test Without Light:**
```
# Try without light source
DROP TORCH
GO DOWN  → Should still work but see darkness
# Movement possible but not recommended
```

**Checklist:**
- [ ] Quest tracks light source
- [ ] Dark navigation functional
- [ ] Quest completion triggers
- [ ] Rewards granted

**Notes:**
_____________________________________________________________

---

### Test 4.4: Master Artisan Quest (Crafting Chain)
**Objective:** Test Phase 4.4 crafting system

**Prerequisites:** Talk to Tinker Brass

**Steps:**
```
# Accept quest
GO SOUTH  → Workshop District
TALK TINKER BRASS
QUEST ACCEPT MASTER_ARTISAN

# Part 1: Craft Basic Antenna
# Need: 2x Wire Spool, 1x Basic Component

# Gather materials
GO [search for materials in Maintenance Tunnels, etc.]
TAKE WIRE_SPOOL
TAKE WIRE_SPOOL
TAKE BASIC_COMPONENT

# Craft antenna
GO SOUTH  → Workshop District
CRAFT BASIC_ANTENNA
INVENTORY  → Verify antenna created

# Part 2: Craft Relay Module
# Need: 1x Wire Spool, 2x Scrap Metal, 1x Circuit Board

[Gather materials]
CRAFT RELAY_MODULE
INVENTORY  → Verify relay created

# Part 3: Craft Advanced Signal Array
# Need: 1x Basic Antenna, 1x Relay Module, 
#       1x Crystal Oscillator, 1x Power Cell

[Gather rare materials from Hidden Vault if needed]
CRAFT ADVANCED_SIGNAL_ARRAY
INVENTORY  → Verify signal array created

# Complete quest
TALK TINKER BRASS
```

**Expected Results:**
- [ ] Quest tracks crafting progress
- [ ] All three items craftable
- [ ] Materials consumed correctly
- [ ] Quest completes after all crafts
- [ ] Rewards received (15,000 currency, Master Crafter Badge, 400 XP)

**Checklist:**
- [ ] Can craft Basic Antenna
- [ ] Can craft Relay Module
- [ ] Can craft Advanced Signal Array
- [ ] Quest completion recognized
- [ ] Rewards granted

**Notes:**
_____________________________________________________________

---

### Test 4.5: The Lost Artifact Quest (Epic Combined)
**Objective:** Test combined Phase 4 mechanics

**Prerequisites:**
- Complete The Cipher quest
- Complete Into the Depths quest
- Complete Master Artisan quest
- Have light source
- Keep quest reward items (Ancient Relay Core, Decoder Lens)

**Steps:**
```
# Accept quest
TALK OLD ELM
QUEST ACCEPT THE_LOST_ARTIFACT

# Part 1: Decipher Ruins Entrance (Symbol Sequence)
[Navigate to Forgotten Ruins Entrance]

EXAMINE RUINS_GLYPH_ALPHA   → 1st (Transmit)
EXAMINE RUINS_GLYPH_BETA    → 2nd (Receive)
EXAMINE RUINS_GLYPH_GAMMA   → 3rd (Relay)
EXAMINE RUINS_GLYPH_DELTA   → 4th (Unity)

# Part 2: Navigate Dark Passage (Need Light)
GO EAST  → Dark Passage
LOOK  (should see with light)

# Part 3: Craft Artifact Chamber Key (Need quest rewards)
# Required: Ancient Relay Core, Decoder Lens, 
#           Crystal Oscillator, 2x Power Cell

INVENTORY  → Verify you have quest rewards
[Gather Crystal Oscillator and Power Cells from Hidden Vault]

GO [Workshop District]
CRAFT ARTIFACT_CHAMBER_KEY
INVENTORY  → Verify key created

# Part 4: Unlock and Enter Artifact Chamber
[Navigate back to Dark Passage]
GO NORTH  → Artifact Chamber entrance
USE ARTIFACT_CHAMBER_KEY
GO NORTH  → Artifact Chamber

LOOK  → See legendary device
TAKE ANCIENT_COMM_DEVICE
QUEST STATUS THE_LOST_ARTIFACT
```

**Expected Results:**
- [ ] Quest requires all 3 previous quests complete
- [ ] Symbol sequence at ruins works
- [ ] Dark passage navigable with light
- [ ] Artifact Chamber Key craftable
- [ ] Key unlocks chamber
- [ ] Quest completes
- [ ] HUGE rewards (50,000 currency, Legendary Artifact, 1,000 XP)

**Checklist:**
- [ ] All mechanics work together
- [ ] Quest progression logical
- [ ] Chamber key crafts correctly
- [ ] Unlock mechanism works
- [ ] Legendary artifact obtained
- [ ] Rewards granted

**Notes:**
_____________________________________________________________

---

## Test Suite 5: Phase 5 Reputation System

### Test 5.1: Reputation Tracking
**Objective:** Verify faction reputation system

**Steps:**
```
# Check starting reputation
# (Implementation may vary - check character stats)

# Complete quests and note reputation changes
# The Cipher should affect: Scholars Circle
# Into the Depths should affect: Explorers Guild
# Master Artisan should affect: Tinker's Union
# Lost Artifact should affect: Scholars Circle (major)

# Check reputation levels
# Expected progression: 
# Neutral (0) → Friendly (+10) → Honored (+25) → Revered (+50)
```

**Checklist:**
- [ ] Reputation tracked per faction
- [ ] Quest completion grants reputation
- [ ] Reputation levels progress
- [ ] Can query reputation status

**Notes:**
_____________________________________________________________

---

### Test 5.2: Reputation Effects
**Objective:** Test if reputation affects NPC interactions

**Steps:**
```
# With low reputation
TALK [NPC]  → Note dialogue options

# After gaining reputation (complete quests)
TALK [NPC]  → Check if dialogue changes

# Check for reputation-gated content
# (may not be fully implemented yet)
```

**Checklist:**
- [ ] Reputation values stored
- [ ] Values persist across sessions
- [ ] NPC dialogue aware of reputation
- [ ] Future content can check reputation

**Notes:**
_____________________________________________________________

---

## Test Suite 6: Performance and Stability

### Test 6.1: Room Capacity
**Objective:** Verify room capacity limits work

**Steps:**
```
# If possible, have multiple test clients connect
# Try to exceed room capacity (Town Square: 25)
# Most rooms have 3-20 capacity

# Expected: Warning or prevention when full
```

**Checklist:**
- [ ] Capacity limits enforced
- [ ] Appropriate messages shown
- [ ] No crashes when full

**Notes:**
_____________________________________________________________

---

### Test 6.2: Object Persistence
**Objective:** Verify objects persist correctly

**Steps:**
```
# Drop an item
DROP TORCH

# Disconnect and reconnect
# (or wait and revisit room)

# Check if item still there
LOOK  → Should see dropped torch
TAKE TORCH
```

**Checklist:**
- [ ] Dropped items persist
- [ ] Objects spawn in correct rooms
- [ ] Object state maintained

**Notes:**
_____________________________________________________________

---

### Test 6.3: Quest State Persistence
**Objective:** Verify quest progress saves

**Steps:**
```
# Start a quest (e.g., The Cipher)
QUEST ACCEPT THE_CIPHER

# Examine 2 symbols (partial progress)
EXAMINE CIPHER_SPRING
EXAMINE CIPHER_SUMMER

# Disconnect and reconnect

# Check quest progress
QUEST STATUS THE_CIPHER
# Should show partial progress (2/4 symbols)

# Continue quest
EXAMINE CIPHER_AUTUMN
EXAMINE CIPHER_WINTER
# Should complete
```

**Checklist:**
- [ ] Active quests persist
- [ ] Partial progress saves
- [ ] Can resume after disconnect
- [ ] Completed quests stay completed

**Notes:**
_____________________________________________________________

---

### Test 6.4: Long Session Stability
**Objective:** Test for memory leaks or crashes

**Steps:**
```
# Keep client connected for extended period
# Navigate extensively
# Execute many commands
# Check for:
# - Memory usage increase
# - Response time degradation
# - Crashes or disconnects
```

**Checklist:**
- [ ] No crashes after 30+ minutes
- [ ] Commands remain responsive
- [ ] No memory leaks observed
- [ ] Navigation smooth throughout

**Notes:**
_____________________________________________________________

---

## Test Suite 7: Edge Cases and Error Handling

### Test 7.1: Invalid Commands
**Objective:** Test error handling

**Steps:**
```
GO INVALID_DIRECTION
TAKE NONEXISTENT_OBJECT
CRAFT INVALID_ITEM
QUEST ACCEPT FAKE_QUEST
EXAMINE NOTHING
```

**Expected Results:**
- [ ] Appropriate error messages
- [ ] No crashes
- [ ] Helpful suggestions where possible

**Notes:**
_____________________________________________________________

---

### Test 7.2: Sequence Violations
**Objective:** Test quest prerequisite checking

**Steps:**
```
# Try The Lost Artifact without completing prerequisites
QUEST ACCEPT THE_LOST_ARTIFACT
# Should fail or warn about requirements

# Try crafting without materials
CRAFT ARTIFACT_CHAMBER_KEY
# Should fail with material error

# Try using key without crafting it
USE ARTIFACT_CHAMBER_KEY
# Should fail
```

**Checklist:**
- [ ] Prerequisites enforced
- [ ] Clear error messages
- [ ] Cannot skip required steps

**Notes:**
_____________________________________________________________

---

### Test 7.3: Boundary Conditions
**Objective:** Test limits and edge cases

**Steps:**
```
# Try examining symbols in wrong order multiple times
# Try entering dark room without light repeatedly
# Try crafting with insufficient materials
# Try taking items when inventory full
# Try entering rooms at capacity
```

**Checklist:**
- [ ] Boundary conditions handled
- [ ] No infinite loops
- [ ] Sensible error recovery

**Notes:**
_____________________________________________________________

---

## Test Suite 8: Documentation Verification

### Test 8.1: Quest Solution Guide Accuracy
**Objective:** Follow PHASE_4_QUEST_SOLUTIONS.md guide

**Steps:**
```
# Open docs/PHASE_4_QUEST_SOLUTIONS.md
# Follow exact steps for each quest
# Verify all commands work as documented
# Check navigation routes are accurate
# Confirm material lists correct
```

**Checklist:**
- [ ] All quest walkthroughs accurate
- [ ] Navigation routes correct
- [ ] Material lists complete
- [ ] Commands work as shown

**Notes:**
_____________________________________________________________

---

### Test 8.2: World Map Accuracy
**Objective:** Verify WORLD_MAP.md matches reality

**Steps:**
```
# Open docs/WORLD_MAP.md
# Verify room count (should be 23)
# Check all connections listed are correct
# Verify room flags (Dark, Safe, etc.)
# Test navigation paths shown
```

**Checklist:**
- [ ] Room count correct (23 total)
- [ ] All connections accurate
- [ ] Flags correctly listed
- [ ] Navigation paths work

**Notes:**
_____________________________________________________________

---

### Test 8.3: Dark Navigation Documentation
**Objective:** Verify DARK_NAVIGATION_SYSTEM.md accurate

**Steps:**
```
# Open docs/DARK_NAVIGATION_SYSTEM.md
# Verify technical details match implementation
# Check all 5 dark rooms listed
# Test behaviors described
```

**Checklist:**
- [ ] 5 dark rooms (Maintenance, Deep Caverns, Sunken, Hidden Vault, Dark Passage)
- [ ] Light source behavior correct
- [ ] Movement mechanics accurate

**Notes:**
_____________________________________________________________

---

## Test Results Summary

### Overall Statistics

- **Total Tests:** 80+
- **Passed:** _____
- **Failed:** _____
- **Skipped:** _____
- **Issues Found:** _____

### Critical Issues (Severity: High)

1. _______________________________________________
2. _______________________________________________
3. _______________________________________________

### Major Issues (Severity: Medium)

1. _______________________________________________
2. _______________________________________________
3. _______________________________________________

### Minor Issues (Severity: Low)

1. _______________________________________________
2. _______________________________________________
3. _______________________________________________

### Recommendations

_______________________________________________
_______________________________________________
_______________________________________________
_______________________________________________

### Sign-Off

**Tested By:** _________________  
**Date:** _________________  
**Version Approved:** YES / NO  
**Notes:** _________________________________

---

## Quick Reference Commands

### Navigation
```
LOOK                    - See current room
GO <direction>          - Move (NORTH, SOUTH, EAST, WEST, UP, DOWN)
MAP                     - View map if available
```

### Inventory
```
INVENTORY / INV         - See what you're carrying
TAKE <object>           - Pick up object
DROP <object>           - Drop object
EXAMINE <object>        - Look closely at object
USE <object>            - Use/activate object
```

### Quests
```
QUEST LIST              - See available quests
QUEST ACTIVE            - See your active quests
QUEST ACCEPT <id>       - Accept a quest
QUEST STATUS <id>       - Check quest progress
QUEST OBJECTIVES <id>   - View quest objectives
ABANDON <id>            - Abandon quest
```

### NPCs
```
TALK <npc>              - Initiate dialogue
```

### Crafting
```
CRAFT <item>            - Craft an item at workshop
```

### Social
```
SAY <message>           - Speak in current room
WHISPER <player> <msg>  - Private message
EMOTE <action>          - Perform emote
```

---

**End of QA Test Script**

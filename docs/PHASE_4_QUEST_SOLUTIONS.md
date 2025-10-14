# Phase 4 Quest Solution Guide

Complete walkthroughs for all Phase 4 quests in MeshBBS.

---

## Table of Contents

1. [The Cipher](#the-cipher) - Symbol Sequence Puzzle (Difficulty 4)
2. [Into the Depths](#into-the-depths) - Dark Navigation (Difficulty 4)
3. [Master Artisan](#master-artisan) - Crafting Chain (Difficulty 5)
4. [The Lost Artifact](#the-lost-artifact) - Epic Combined Quest (Difficulty 5)

---

## The Cipher

**Quest Giver:** Old Elm (west_residential)  
**Prerequisites:** Must complete "The Grove Mystery" quest first  
**Difficulty:** 4/5  
**Estimated Time:** 15-20 minutes  
**Rewards:**
- Decoder Lens (special item for future puzzles)
- 10,000 currency ($100 or 1000cp)
- 250 experience points

### Overview
An ancient message has been discovered, encoded in symbols scattered throughout the ancient ruins. You must examine the symbols in the correct sequence to unlock their meaning. The symbols follow the cycle of seasons.

### Solution Guide

#### Step 1: Accept the Quest
```
TALK OLD ELM
> Choose "Tell me about the cipher" or similar dialogue option
QUEST ACCEPT the_cipher
```

#### Step 2: Locate the Cipher Chamber
From your current location, navigate to the Cipher Chamber:
```
GO MUSEUM          (to Mesh Museum)
GO EAST            (to Forest Path)
GO EAST            (to Ancient Grove)
GO NORTH           (to Cipher Chamber)
```

#### Step 3: Examine Symbols in Correct Sequence

**IMPORTANT:** The symbols must be examined in seasonal order:
1. **Spring** (Growth - The First Signal)
2. **Summer** (Strength - The Broadcast Peak)
3. **Autumn** (Change - The Signal Adapts)
4. **Winter** (Rest - The Silent Network Awaits)

Commands:
```
EXAMINE CIPHER_SPRING    or    LOOK AT SPRING GLYPH
EXAMINE CIPHER_SUMMER    or    LOOK AT SUMMER GLYPH
EXAMINE CIPHER_AUTUMN    or    LOOK AT AUTUMN GLYPH
EXAMINE CIPHER_WINTER    or    LOOK AT WINTER GLYPH
```

**Note:** If you examine them in the wrong order, you'll need to clear your sequence and start over. Talk to Old Elm to reset.

#### Step 4: Return to Old Elm
```
GO SOUTH           (back to Ancient Grove)
GO WEST            (to Forest Path)
GO WEST            (to Mesh Museum)
GO WEST            (to West Residential)
TALK OLD ELM
> Discuss your discovery
```

#### Step 5: Complete Quest
The quest will automatically complete once you've examined all symbols in the correct sequence and reported back to Old Elm.

### Tips
- Write down the symbols as you see them mentioned in room descriptions
- The symbols represent the mesh network's lifecycle: broadcast, connection, adaptation, rest
- Spring → Summer → Autumn → Winter is the natural order

---

## Into the Depths

**Quest Giver:** Old Graybeard (repeater_tower)  
**Prerequisites:** Must complete "Tunnel Salvage" quest first  
**Difficulty:** 4/5  
**Estimated Time:** 20-25 minutes  
**Rewards:**
- Ancient Relay Core (rare crafting material)
- 12,000 currency ($120 or 1200cp)
- 300 experience points

### Overview
The Deep Caverns beneath Old Towne have never been fully explored. They're pitch black - you'll need a light source to navigate safely. Rumors speak of a hidden chamber containing pre-mesh artifacts.

### Solution Guide

#### Step 1: Accept the Quest
```
GO NORTH           (from Town Square to North Gate)
GO NORTH           (to Repeater Tower)
TALK OLD GRAYBEARD
> Choose dialogue about the Deep Caverns
QUEST ACCEPT into_the_depths
```

#### Step 2: Obtain a Light Source

You need an object with the `LightSource` flag. Three options:

**Option A: Find a Torch** (easiest)
- Available in various locations around Old Towne
- Check the Relay Tavern, Workshop District, or Maintenance Tunnels
```
LOOK              (to see objects in room)
TAKE TORCH
```

**Option B: Buy a Lantern** (best, but expensive)
- Purchase from merchants in South Market
- More reliable than torches
- Cost: 50 currency
```
GO MARKET
BUY LANTERN
```

**Option C: Find a Glowstick**
- Scattered in maintenance areas
- Lightweight alternative
- Cost: 5 currency if purchased
```
TAKE GLOWSTICK
```

#### Step 3: Navigate to Deep Caverns

From Town Square:
```
GO SOUTH           (to Maintenance Tunnels - already dark!)
GO DOWN            (to Deep Caverns Entrance)
```

**IMPORTANT:** You must have a light source in your inventory. If you enter without one, you won't be able to see anything and may get lost.

#### Step 4: Explore the Deep Caverns

```
GO DOWN            (from Deep Caverns Entrance to Sunken Chamber)
```

The Sunken Chamber is partially flooded - watch your step! With your light, you can see:
- Standing water covering the floor
- Stalactites hanging from ceiling
- Tool marks on walls
- Passage to the east

```
GO EAST            (to Hidden Vault)
```

#### Step 5: Discover the Hidden Vault

The Hidden Vault contains:
- Pre-mesh artifacts on metal shelves
- Ancient communication equipment
- Vacuum tubes, crystal sets, relay switches
- A workbench with tools and schematics

Look around and examine items:
```
LOOK
EXAMINE SHELVES
EXAMINE WORKBENCH
EXAMINE ANCIENT EQUIPMENT
```

#### Step 6: Return to Old Graybeard (Optional)

The quest completes automatically when you reach the Hidden Vault, but you can return to Graybeard for additional dialogue:

```
GO WEST            (back to Sunken Chamber)
GO UP              (to Deep Caverns Entrance)
GO UP              (to Maintenance Tunnels)
GO NORTH           (to Town Square)
GO NORTH           (to North Gate)
GO NORTH           (to Repeater Tower)
TALK OLD GRAYBEARD
```

### Tips
- **Never venture into dark areas without a light source!**
- Torches can burn out over time - carry a backup
- The LED Lantern is the best investment for explorers
- Map the caverns as you go - it's easy to get turned around
- The Sunken Chamber has standing water - be careful

### Light Source Comparison

| Item | Weight | Cost | Brightness | Duration |
|------|--------|------|------------|----------|
| Torch | 1 | 10 | Medium | Limited |
| LED Lantern | 2 | 50 | Bright | Long |
| Glowstick | 1 | 5 | Soft | Several hours |

---

## Master Artisan

**Quest Giver:** Tinker Brass (workshop_district)  
**Prerequisites:** Must complete "Your First Craft" quest first  
**Difficulty:** 5/5  
**Estimated Time:** 30-45 minutes  
**Rewards:**
- Master Crafter Badge (title unlock item)
- 15,000 currency ($150 or 1500cp)
- 400 experience points

### Overview
Tinker Brass believes you're ready for advanced crafting techniques. Prove your skill by crafting multiple items from scratch: an antenna, a relay module, and finally a complex signal array.

### Solution Guide

#### Step 1: Accept the Quest
```
GO SOUTH           (from Town Square to South Market)
GO SOUTH           (to Workshop District)
TALK TINKER BRASS
> Choose "I want to learn advanced crafting"
QUEST ACCEPT master_artisan
```

#### Step 2: Craft a Basic Antenna

**Materials Needed:**
- 2x Wire Spool
- 1x Basic Component

**Where to Find Materials:**
- **Wire Spools**: Maintenance Tunnels, Workshop District
- **Basic Components**: Maintenance Tunnels, scattered around town

**Gathering Materials:**
```
GO NORTH           (to South Market)
GO NORTH           (to Town Square)
GO SOUTH           (to Maintenance Tunnels)
LOOK
TAKE WIRE SPOOL
TAKE WIRE SPOOL    (take a second one)
TAKE BASIC COMPONENT
```

**Crafting the Antenna:**
```
GO EAST            (to Workshop District)
GO CRAFTING BENCH  (or USE CRAFTING BENCH if standing near it)
CRAFT BASIC ANTENNA
> System will check if you have materials
> Antenna will be created if successful
```

**Antenna Recipe:**
- Input: 2 Wire Spool + 1 Basic Component
- Output: Basic Antenna
- Weight: 3
- Value: ~100 currency

#### Step 3: Craft a Relay Module

**Materials Needed:**
- 1x Wire Spool
- 2x Scrap Metal
- 1x Circuit Board

**Where to Find Materials:**
- **Scrap Metal**: Maintenance Tunnels, Workshop District (common)
- **Circuit Board**: Workshop District, sometimes in Maintenance areas (uncommon)

**Gathering Materials:**
```
GO WEST            (to Maintenance Tunnels)
TAKE WIRE SPOOL
TAKE SCRAP METAL
TAKE SCRAP METAL
```

For Circuit Board - may need to search multiple locations:
```
LOOK               (check current room)
GO EAST            (to Workshop District)
LOOK
TAKE CIRCUIT BOARD
```

If you can't find a circuit board, check the South Market merchants:
```
GO NORTH           (to South Market)
BUY CIRCUIT BOARD  (may cost ~100 currency)
```

**Crafting the Relay Module:**
```
GO SOUTH           (to Workshop District)
CRAFT RELAY MODULE
```

**Relay Module Recipe:**
- Input: 1 Wire Spool + 2 Scrap Metal + 1 Circuit Board
- Output: Relay Module
- Weight: 5
- Value: ~250 currency

#### Step 4: Craft an Advanced Signal Array

**Materials Needed:**
- 1x Basic Antenna (you just crafted this!)
- 1x Relay Module (you just crafted this!)
- 1x Crystal Oscillator (RARE)
- 1x Power Cell

**Where to Find Rare Materials:**

**Crystal Oscillator** (rare, expensive):
- Hidden Vault (from Into the Depths quest)
- South Market (expensive - 200 currency)
- Workshop District (very rare spawn)

**Power Cell**:
- Maintenance Tunnels
- Workshop District
- South Market (75 currency)

**Gathering Materials:**
```
GO WEST            (to Maintenance Tunnels)
GO DOWN            (to Deep Caverns - if you've unlocked them)
GO DOWN            (to Sunken Chamber)
GO EAST            (to Hidden Vault)
LOOK
TAKE CRYSTAL OSCILLATOR
TAKE POWER CELL    (if available)
```

If materials aren't in the vault, purchase from market:
```
GO WEST            (back through caverns)
GO UP
GO UP
GO NORTH           (to Town Square)
GO SOUTH           (to South Market)
BUY CRYSTAL OSCILLATOR  (200 currency)
BUY POWER CELL          (75 currency)
```

**Crafting the Advanced Signal Array:**
```
GO SOUTH           (to Workshop District)
CRAFT SIGNAL ARRAY ADVANCED
> or CRAFT ADVANCED SIGNAL ARRAY
```

**Advanced Signal Array Recipe:**
- Input: 1 Basic Antenna + 1 Relay Module + 1 Crystal Oscillator + 1 Power Cell
- Output: Advanced Signal Array
- Weight: 10
- Value: ~1000 currency

#### Step 5: Complete the Quest
```
TALK TINKER BRASS
> Discuss your achievements
```

The quest will complete, granting you the Master Crafter Badge and allowing you to use the title "Master Crafter" in your profile.

### Tips
- **Start collecting materials early** - this quest requires a LOT of items
- **Budget approximately 400 currency** if you need to buy materials
- The Hidden Vault from "Into the Depths" quest has rare components
- Some materials respawn over time - check back if areas are empty
- You can sell crafted items if you need money for materials
- Keep your completed items safe - don't accidentally drop them!

### Material Checklist

| Material | Quantity | Common Location | Purchase Price |
|----------|----------|-----------------|----------------|
| Wire Spool | 3 total | Maintenance Tunnels | 10 each |
| Basic Component | 1 | Maintenance Tunnels | 25 |
| Scrap Metal | 2 | Maintenance Tunnels | 15 each |
| Circuit Board | 1 | Workshop District | 100 |
| Crystal Oscillator | 1 | Hidden Vault | 200 |
| Power Cell | 1 | Maintenance Tunnels | 75 |
| **TOTAL COST** | | | **~470 currency** |

---

## The Lost Artifact

**Quest Giver:** Old Elm (west_residential)  
**Prerequisites:** Must complete ALL three previous Phase 4 quests:
- The Cipher
- Into the Depths  
- Master Artisan
**Difficulty:** 5/5 (EPIC)  
**Estimated Time:** 45-60 minutes  
**Rewards:**
- Legendary Mesh Artifact (legendary item!)
- 50,000 currency ($500 or 5000cp) - HUGE reward!
- 1,000 experience points

### Overview
Legends tell of an ancient communication device hidden in the Forgotten Ruins. To reach it, you must decipher the entrance symbols, navigate dark passages with a light source, and craft a special key to unlock the artifact chamber. This is the ultimate test of your skills.

### Solution Guide

#### Step 1: Accept the Quest
```
GO WEST            (from Town Square to West Residential)
TALK OLD ELM
> Choose "Tell me about the Lost Artifact"
> Elm will explain the legendary device
QUEST ACCEPT the_lost_artifact
```

#### Step 2: Find the Forgotten Ruins

The ruins are hidden beyond the Ancient Grove:
```
GO EAST            (to Town Square)
GO EAST            (to Mesh Museum)
GO EAST            (to Forest Path)
GO EAST            (to Ancient Grove)
GO SOUTH           (to Forgotten Ruins Entrance)
```

The entrance is flanked by four stone pillars with glyphs.

#### Step 3: Decipher the Entrance Sequence

You must examine the four glyph pillars in the correct order. The sequence follows the communication process:

1. **TRANSMIT** (Alpha Glyph)
2. **RECEIVE** (Beta Glyph)
3. **RELAY** (Gamma Glyph)
4. **UNITY** (Delta Glyph)

```
EXAMINE RUINS_GLYPH_ALPHA     (upward arrow - TRANSMIT)
EXAMINE RUINS_GLYPH_BETA      (circle with waves - RECEIVE)
EXAMINE RUINS_GLYPH_GAMMA     (two circles connected - RELAY)
EXAMINE RUINS_GLYPH_DELTA     (interconnected circles - UNITY)
```

**IMPORTANT:** These must be examined in this exact order! If you mess up, talk to Old Elm to reset your sequence.

Clue: The glyphs tell the story of communication: first you transmit, then someone receives, then it's relayed through the network, achieving unity.

#### Step 4: Navigate the Dark Passage

**Before entering, ensure you have:**
- ✅ A light source (torch, lantern, or glowstick)
- ✅ The entrance sequence completed

Enter the ruins:
```
GO EAST            (from Forgotten Ruins Entrance to Dark Passage)
```

The passage is pitch black without your light. The walls show precision engineering that surpasses modern techniques. Symbols are carved at intervals as way markers.

```
GO NORTH           (to Artifact Chamber entrance)
```

**DO NOT PROCEED YET** - the chamber is locked!

#### Step 5: Craft the Artifact Chamber Key

The chamber requires a special key. You'll need to craft it.

**Materials Required:**
- 1x Ancient Relay Core (reward from "Into the Depths" quest)
- 1x Decoder Lens (reward from "The Cipher" quest)  
- 1x Crystal Oscillator
- 2x Power Cell

**Where to Get Materials:**

You should already have:
- Ancient Relay Core (from Into the Depths reward)
- Decoder Lens (from The Cipher reward)

You need to gather:
- 1x Crystal Oscillator
- 2x Power Cell

**Gathering Missing Materials:**

If you sold your quest rewards, you'll need to find or buy them again. Otherwise, gather the additional materials:

```
GO SOUTH           (back to Dark Passage)
GO WEST            (to Forgotten Ruins Entrance)
GO NORTH           (to Ancient Grove)
GO WEST            (to Forest Path)
GO WEST            (to Mesh Museum)
GO WEST            (to Town Square)
GO SOUTH           (to Maintenance Tunnels)
GO DOWN            (to Deep Caverns)
GO DOWN            (to Sunken Chamber)
GO EAST            (to Hidden Vault)
TAKE CRYSTAL OSCILLATOR
TAKE POWER CELL
TAKE POWER CELL
```

If items aren't available, purchase from market:
```
GO WEST, GO UP, GO UP, GO NORTH, GO SOUTH (navigate to South Market)
BUY CRYSTAL OSCILLATOR  (200 currency)
BUY POWER CELL          (75 currency)
BUY POWER CELL          (75 currency)
```

**Crafting the Chamber Key:**
```
GO SOUTH           (to Workshop District)
CRAFT ARTIFACT CHAMBER KEY
> or CRAFT CHAMBER KEY
```

**Artifact Chamber Key Recipe:**
- Input: 1 Ancient Relay Core + 1 Decoder Lens + 1 Crystal Oscillator + 2 Power Cell
- Output: Artifact Chamber Key
- This is a UNIQUE item - can only craft once!

#### Step 6: Unlock the Artifact Chamber

Return to the ruins with your key:
```
GO WEST            (to Maintenance Tunnels)
GO NORTH           (to Town Square)
GO EAST            (to Mesh Museum)
GO EAST            (to Forest Path)
GO EAST            (to Ancient Grove)
GO SOUTH           (to Forgotten Ruins Entrance)
GO EAST            (to Dark Passage)
GO NORTH           (to Artifact Chamber entrance - still locked)
USE ARTIFACT CHAMBER KEY
> The door will unlock!
GO NORTH           (enter the chamber)
```

#### Step 7: Claim the Ancient Communication Device

The Artifact Chamber contains:
- A raised platform with a sophisticated device under a crystal dome
- Indicator lights that still glow after centuries
- Wall inscriptions in multiple languages
- The legendary communication artifact

```
LOOK
EXAMINE DEVICE
EXAMINE PLATFORM
TAKE ANCIENT COMM DEVICE
```

**Congratulations!** You now possess the Legendary Mesh Artifact, one of the rarest items in the game!

#### Step 8: Return to Old Elm (Optional)
```
GO SOUTH           (back to Dark Passage)
GO WEST            (to Forgotten Ruins Entrance)
GO NORTH           (to Ancient Grove)
GO WEST            (to Forest Path)
GO WEST            (to Mesh Museum)
GO WEST            (to West Residential)
TALK OLD ELM
> Special dialogue about your achievement!
```

### Tips for The Lost Artifact
- **Do not attempt this quest until you've completed all three prerequisite quests**
- **Save your quest rewards** - don't sell the Ancient Relay Core or Decoder Lens!
- **Budget at least 500 currency** for purchasing materials if needed
- **Bring multiple light sources** - don't get stuck in the dark
- **Map your route** - the ruins are complex
- The chamber key is a one-time craft - don't lose it before using it!
- Take screenshots of the Artifact Chamber - it's beautiful!

### Complete Material Checklist

| Material | Source | Cost if Purchased |
|----------|--------|-------------------|
| Ancient Relay Core | Into the Depths reward | QUEST REWARD |
| Decoder Lens | The Cipher reward | QUEST REWARD |
| Crystal Oscillator | Hidden Vault or Market | 200 |
| Power Cell (x2) | Hidden Vault or Market | 75 each (150 total) |
| Light Source | Various | 10-50 |
| **TOTAL PURCHASE COST** | | **~360 currency** |

### Rewards Summary

The Legendary Mesh Artifact grants:
- Massive reputation boost with Scholars Circle faction
- Access to advanced lore and historical knowledge
- Possible future quest unlock
- Display piece for your housing
- 50,000 currency (can buy almost anything!)
- Bragging rights!

---

## General Quest Tips

### Quest Commands
```
QUEST LIST               - View all available quests
QUEST ACTIVE            - View quests you're currently doing
QUEST ACCEPT <id>       - Accept a quest
QUEST STATUS <id>       - Check progress on a quest
QUEST OBJECTIVES <id>   - View detailed objectives
ABANDON <id>            - Abandon a quest (can reaccept later)
```

### Inventory Management
```
INVENTORY               - View what you're carrying
DROP <item>             - Drop an item
TAKE <item>             - Pick up an item
EXAMINE <item>          - Get details about an item
USE <item>              - Use an item (for light sources, keys, etc.)
```

### Navigation
```
LOOK                    - See room description and objects
EXITS                   - Show available exits
MAP                     - View your map (if you have one)
GO <direction>          - Move in a direction (NORTH, SOUTH, EAST, WEST, UP, DOWN)
```

### Getting Help
```
HELP QUESTS             - General quest help
HELP CRAFTING           - Crafting system help
HELP NAVIGATION         - Movement and exploration help
TALK <npc>              - Get hints from quest givers
```

---

## Quest Sequence Recommendation

For the best experience, complete the quests in this order:

1. **Welcome to Old Towne** (starter quest)
2. **Market Exploration** (starter quest)
3. **Your First Craft** (learn crafting basics)
4. **Tunnel Salvage** (gather materials, unlock Deep Caverns)
5. **Tower Diagnostics** (meet Old Graybeard)
6. **The Grove Mystery** (prerequisite for The Cipher)
7. **The Cipher** ⭐ (Phase 4.2 - symbol puzzles)
8. **Into the Depths** ⭐ (Phase 4.3 - dark navigation)
9. **Master Artisan** ⭐ (Phase 4.4 - advanced crafting)
10. **The Lost Artifact** ⭐⭐⭐ (Epic finale - all mechanics)

**Total Phase 4 Completion Time:** 2-3 hours  
**Total Currency Earned:** 87,000+ (from all Phase 4 quests)  
**Total Experience:** 1,950 XP

---

## Troubleshooting

### "I'm stuck in a dark room!"
- If you're in a dark room without a light source, type `GO <opposite direction>` to backtrack
- Example: If you came from the north, type `GO SOUTH` to leave
- Always carry a backup light source when exploring!

### "I examined the symbols in the wrong order!"
- Talk to the quest giver (Old Elm) to reset your sequence
- You can also abandon and reaccept the quest

### "I can't find the materials I need!"
- Check multiple locations - items respawn in different areas
- Buy from merchants in South Market if desperate
- Some materials only appear after completing certain quests

### "I sold my quest reward items!"
- For The Lost Artifact, you NEED the Ancient Relay Core and Decoder Lens
- If you sold them, you may need to petition an admin or wait for a future system to recover them
- **Lesson learned:** Keep quest rewards until you're sure you don't need them!

### "The crafting command isn't working!"
- Make sure you're at a crafting bench (Workshop District)
- Check that you have ALL required materials in your inventory
- Verify the exact item names with `INVENTORY`
- Some recipes require specific component quality/types

---

## Achievements Related to Phase 4 Quests

Completing these quests may unlock achievements:

- **Cipher Breaker** - Complete The Cipher quest
- **Deep Explorer** - Complete Into the Depths quest
- **Master Craftsman** - Complete Master Artisan quest
- **Legendary Seeker** - Complete The Lost Artifact quest
- **Phase 4 Champion** - Complete all four Phase 4 quests

Check your achievements with: `ACHIEVEMENTS` or `ACHIEVEMENTS PHASE4`

---

**Good luck, adventurer! May your journey through Phase 4 be enlightening!** 🎮✨

---

*Last Updated: October 14, 2025*  
*MeshBBS Version: 1.0.110-beta+*  
*Quest Content: Phase 4 Complete*

# TinyMUSH Tutorial Walkthrough

## What You'll See When You Start

When you first connect to TinyMUSH and select it from the games menu, you'll see:

```
╔════════════════════════════════════╗
║   WELCOME TO OLD TOWNE MESH!     ║
╚════════════════════════════════════╝

You've arrived at the Landing Gazebo.
This tutorial will teach you the basics.

📋 GETTING STARTED:
  LOOK or L     - See your surroundings
  N/S/E/W       - Move (North/South/East/West)
  TUTORIAL      - Check your progress
  HELP          - View all commands

💡 TIP: Start by typing LOOK to explore!
```

## Step-by-Step Guide

### STEP 1: Landing Gazebo (WelcomeAtGazebo)

**Your Goal:** Learn basic commands and head north to Town Square

**Commands to try:**
```
LOOK          - See the room description
L             - Short version of LOOK
TUTORIAL      - Check your progress
WHERE         - See your current location
HELP          - View all available commands
```

**What you'll see:**
- Room description: "Script Gazebo" or "Landing Gazebo"
- Exits available (should show "Exit north to Town Square")
- Tutorial hint: "🎯 STEP 1: Try typing 'LOOK' or 'L' to see around you, then go NORTH."

**To advance:**
```
NORTH         - or just 'N'
```

---

### STEP 2: Town Square (NavigateToCityHall)

**Your Goal:** Navigate to City Hall

**Commands to try:**
```
LOOK          - See what's in Town Square
WHERE         - Confirm you're at Town Square
MAP           - View area map (if available)
TUTORIAL      - Check your progress
```

**What you'll see:**
- Tutorial hint: "🎯 STEP 2: Head to City Hall. Use 'WHERE' to check your location, then go NORTH."
- Exits showing direction to City Hall

**To advance:**
```
NORTH         - Head to City Hall Lobby
```

---

### STEP 3: City Hall & Mayor's Office (MeetTheMayor)

**Your Goal:** Find and talk to the Mayor

**Commands to try:**
```
LOOK          - See what's in the lobby
TUTORIAL      - Check your progress
NORTH         - Go to Mayor's Office
LOOK          - See the Mayor
```

**What you'll see:**
- Tutorial hint: "🎯 STEP 3: Find the Mayor's Office. Go NORTH from the lobby, then type 'TALK MAYOR'."
- Mayor Thompson NPC in the Mayor's Office

**To complete the tutorial:**
```
TALK MAYOR    - Complete the tutorial!
```

**Rewards:**
- 100 copper pieces (or $10.00 depending on currency system)
- Town Map item (added to your inventory)
- Full access to Old Towne Mesh

---

## Quick Command Reference

| Command | What It Does |
|---------|-------------|
| `LOOK` or `L` | Examine your surroundings |
| `N/S/E/W/U/D` | Move North/South/East/West/Up/Down |
| `WHERE` | See your current location |
| `MAP` | View area map |
| `INVENTORY` or `I` | Check your items |
| `TUTORIAL` | Check tutorial progress |
| `TUTORIAL SKIP` | Skip the tutorial |
| `HELP` | View all commands |
| `TALK <name>` | Talk to an NPC |
| `SAY <text>` | Speak aloud |
| `WHO` | See who's online |

---

## Skipping the Tutorial

If you're experienced with MUDs, you can skip the tutorial:

```
TUTORIAL SKIP
```

You can always restart it later with:

```
TUTORIAL RESTART
```

---

## Troubleshooting

**Q: I typed LOOK but nothing happened**
- Make sure you're using uppercase (LOOK not look)
- Check if there's a typo
- Try the short version: just `L`

**Q: I'm stuck, how do I see where to go?**
- Type `LOOK` to see available exits
- Type `WHERE` to see your current location
- Type `TUTORIAL` to see your current step and hint

**Q: I went the wrong direction**
- Most rooms have opposite exits (if you went NORTH, go SOUTH to return)
- Type `WHERE` to see where you are
- Type `LOOK` to see available exits

**Q: How do I know if I completed the tutorial?**
- Type `TUTORIAL` - it will say "Tutorial: Complete!"
- You'll receive rewards (currency + Town Map)
- You can explore freely anywhere

---

## After the Tutorial

Once you complete the tutorial, you have full access to:
- **Exploration**: All of Old Towne Mesh
- **Social**: Talk to other players with SAY, WHISPER, EMOTE
- **Communication**: Mail system and bulletin boards
- **Housing**: Build and customize your own spaces (if enabled)
- **Economy**: Use currency to buy items and services
- **Quests**: Talk to NPCs for adventures (coming soon)

Type `HELP` anytime to see all available commands!

---

## Command Aliases

Many commands have shortcuts:
- `L` = LOOK
- `I` = INVENTORY
- `N/S/E/W` = NORTH/SOUTH/EAST/WEST
- `U/D` = UP/DOWN
- `NE/NW/SE/SW` = Northeast/Northwest/Southeast/Southwest

You can use full words or abbreviations - whatever you prefer!

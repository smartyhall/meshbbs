# Object Trigger Editing Guide

## Overview

Object triggers allow admins to create interactive, event-driven objects without modifying code. Triggers execute scripts automatically when players interact with objects or when specific game events occur.

## Quick Start

```bash
# Create an object
@OBJECT CREATE singing_mushroom "Singing Mushroom"

# Set basic properties
@OBJECT EDIT singing_mushroom DESCRIPTION "A peculiar purple mushroom that hums softly."
@OBJECT EDIT singing_mushroom WEIGHT 1
@OBJECT EDIT singing_mushroom TAKEABLE true

# Add a trigger that fires when entering the room
@OBJECT EDIT singing_mushroom TRIGGER ONENTER message("🍄 The mushroom chimes a cheerful tune!")

# View the object (including triggers)
@OBJECT SHOW singing_mushroom
```

## Trigger Types

| Trigger | Fires When | Example Use Case |
|---------|-----------|------------------|
| `ONENTER` | Player enters room containing object | Ambient effects, room atmosphere |
| `ONLOOK` | Player examines object | Hidden details, quest clues |
| `ONTAKE` | Player picks up object | Collection messages, warnings |
| `ONDROP` | Player drops object | Environmental reactions |
| `ONUSE` | Player uses object | Consumables, tools, keys |
| `ONPOKE` | Player pokes/prods object | Interactive puzzles, secrets |
| `ONFOLLOW` | Player follows something | Companion behaviors |
| `ONIDLE` | Periodic/idle time | Ambient messages, timers |
| `ONCOMBAT` | During combat | Combat effects, reactions |
| `ONHEAL` | When healing occurs | Healing item effects |

## Trigger Script Commands

### Basic Commands

```bash
message("text")                    # Display message to player
heal(amount)                       # Heal player by amount
consume()                          # Destroy object after use
teleport("room_id")               # Move player to room
unlock_exit("direction")          # Unlock room exit
```

### Control Flow

```bash
random_chance(percent)             # 50% chance gate
has_quest("quest_id")             # Check if player has quest
```

### Chaining Commands

Use `&&` to chain multiple commands:

```bash
message("✨ Flash!") && heal(50) && consume()
```

## Common Patterns

### 1. Consumable Healing Potion

```bash
@OBJECT CREATE healing_potion "Healing Potion"
@OBJECT EDIT healing_potion DESCRIPTION "A glowing red potion."
@OBJECT EDIT healing_potion USABLE true
@OBJECT EDIT healing_potion TAKEABLE true
@OBJECT EDIT healing_potion TRIGGER ONUSE message("✨ The potion glows as you drink it!") && heal(50) && consume()
```

### 2. Ambient Room Object

```bash
@OBJECT CREATE crackling_fire "Crackling Fire"
@OBJECT EDIT crackling_fire DESCRIPTION "A warm fire crackles in the hearth."
@OBJECT EDIT crackling_fire TAKEABLE false
@OBJECT EDIT crackling_fire TRIGGER ONENTER message("🔥 The fire crackles warmly, casting dancing shadows.")
```

### 3. Quest Key

```bash
@OBJECT CREATE ancient_key "Ancient Key"
@OBJECT EDIT ancient_key DESCRIPTION "An ornate brass key covered in runes."
@OBJECT EDIT ancient_key TAKEABLE true
@OBJECT EDIT ancient_key FLAG KEYITEM
@OBJECT EDIT ancient_key TRIGGER ONLOOK message("The runes glow faintly. Perhaps it unlocks something nearby?")
@OBJECT EDIT ancient_key TRIGGER ONUSE unlock_exit("north") && message("🔓 The key turns with a satisfying click!")
```

### 4. Mystery Box with Random Reward

```bash
@OBJECT CREATE mystery_box "Mystery Box"
@OBJECT EDIT mystery_box DESCRIPTION "A wooden box with strange markings."
@OBJECT EDIT mystery_box TAKEABLE false
@OBJECT EDIT mystery_box TRIGGER ONPOKE random_chance(50) && message("✨ The box clicks open, revealing something inside!")
```

### 5. Teleportation Stone

```bash
@OBJECT CREATE teleport_stone "Teleport Stone"
@OBJECT EDIT teleport_stone DESCRIPTION "A smooth stone that pulses with energy."
@OBJECT EDIT teleport_stone USABLE true
@OBJECT EDIT teleport_stone TAKEABLE true
@OBJECT EDIT teleport_stone TRIGGER ONUSE message("✨ The stone flashes brilliantly!") && teleport("old_towne_square")
```

### 6. Hidden Quest Clue

```bash
@OBJECT CREATE tattered_note "Tattered Note"
@OBJECT EDIT tattered_note DESCRIPTION "An old, weathered piece of paper."
@OBJECT EDIT tattered_note TAKEABLE true
@OBJECT EDIT tattered_note FLAG QUESTITEM
@OBJECT EDIT tattered_note TRIGGER ONLOOK has_quest("ancient_mystery") && message("📜 The note mentions the old tower at midnight...")
```

## Managing Triggers

### View All Triggers on an Object

```bash
@OBJECT SHOW singing_mushroom
```

Output includes:
```
Triggers: (1)
  OnEnter: message("🍄 The mushroom chimes a cheerful tune!")
```

### Update a Trigger

Simply set it again with new script:

```bash
@OBJECT EDIT singing_mushroom TRIGGER ONENTER message("🍄 The mushroom SINGS loudly!")
```

### Remove a Trigger

```bash
@OBJECT EDIT singing_mushroom TRIGGER ONENTER REMOVE
```

### List All Objects with Triggers

```bash
@OBJECT LIST
```

Objects with triggers will show in the list. Use `@OBJECT SHOW <id>` to see details.

## Best Practices

### 1. Keep Scripts Simple

✅ Good:
```bash
message("The torch flickers.") && heal(10)
```

❌ Too Complex:
```bash
# Avoid overly long chained commands
message("A") && message("B") && message("C") && message("D") && heal(1) && heal(2)
```

### 2. Use Appropriate Trigger Types

- **ONENTER**: Ambient effects that enhance atmosphere
- **ONLOOK**: Hidden details discovered by examining
- **ONUSE**: Active player actions with consequences
- **ONTAKE**: Warnings or collection messages

### 3. Test Your Triggers

After creating triggers, test them as a regular player:

```bash
# As admin, create and configure
@OBJECT EDIT mushroom TRIGGER ONENTER message("Test!")

# Then test by entering the room
# The trigger should fire automatically
```

### 4. Document Special Objects

When creating complex objects with multiple triggers, consider adding comments to the description:

```bash
@OBJECT EDIT puzzle_box DESCRIPTION "A complex puzzle box. (Admin: Has ONPOKE for 50% random reward)"
```

### 5. Use Flags Appropriately

Combine triggers with object flags for better organization:

```bash
@OBJECT EDIT quest_item FLAG QUESTITEM
@OBJECT EDIT quest_item FLAG UNIQUE
@OBJECT EDIT quest_item TRIGGER ONLOOK message("This seems important!")
```

## Trigger Security

### Rate Limiting

Triggers are automatically rate-limited to prevent spam. Players can't trigger the same object repeatedly in quick succession.

### Sandboxing

Trigger scripts are sandboxed and cannot:
- Access unauthorized data
- Modify other players' data
- Execute system commands
- Break game balance (healing/damage have reasonable limits)

### Admin Visibility

All trigger executions are logged. Use server logs to debug trigger behavior:

```bash
# Check logs for trigger execution
tail -f /var/log/meshbbs/meshbbs.log | grep trigger
```

## Troubleshooting

### Trigger Doesn't Fire

1. Verify trigger is set: `@OBJECT SHOW object_id`
2. Check trigger type matches action (e.g., ONUSE requires `USE object`)
3. Verify object is in the right location (room, player inventory)
4. Check rate limiting - wait a few seconds and try again

### Script Syntax Errors

If trigger script has syntax errors, it will fail silently. Check:
- Proper quoting: `message("text")` not `message(text)`
- Command spelling: `heal(50)` not `health(50)`
- Chaining syntax: `cmd1 && cmd2` not `cmd1 & cmd2`

### Object Not Responding

1. Verify object exists: `@OBJECT SHOW object_id`
2. Check object properties (takeable, usable)
3. For ONUSE triggers, ensure `USABLE true`
4. For ONTAKE triggers, ensure `TAKEABLE true`

## Advanced Usage

### Conditional Triggers

Use quest checks for conditional behavior:

```bash
# Only shows message if player has quest
@OBJECT EDIT clue TRIGGER ONLOOK has_quest("mystery") && message("🔍 Aha! This is the clue you needed!")
```

### Random Events

Create variety with random_chance:

```bash
# 30% chance of special message
@OBJECT EDIT fountain TRIGGER ONLOOK random_chance(30) && message("💫 You spot a coin at the bottom!")
```

### Multi-Stage Objects

Combine triggers for multi-stage interactions:

```bash
@OBJECT EDIT statue TRIGGER ONLOOK message("The statue's eyes seem to follow you.")
@OBJECT EDIT statue TRIGGER ONPOKE message("The statue shifts slightly...")
@OBJECT EDIT statue TRIGGER ONUSE unlock_exit("secret") && message("🗿 The statue slides aside, revealing a passage!")
```

## Example World Building

### Creating an Interactive Museum

```bash
# 1. Healing fountain
@OBJECT CREATE healing_fountain "Healing Fountain"
@OBJECT EDIT healing_fountain TAKEABLE false
@OBJECT EDIT healing_fountain TRIGGER ONLOOK message("💧 The water sparkles with magical energy.")
@OBJECT EDIT healing_fountain TRIGGER ONUSE heal(25) && message("💫 The cool water refreshes you!")

# 2. Ancient artifact
@OBJECT CREATE glowing_orb "Glowing Orb"
@OBJECT EDIT glowing_orb TAKEABLE true
@OBJECT EDIT glowing_orb FLAG MAGICAL
@OBJECT EDIT glowing_orb TRIGGER ONENTER message("✨ An orb pulses with ethereal light.")
@OBJECT EDIT glowing_orb TRIGGER ONTAKE message("⚡ Power surges through you as you grasp the orb!")

# 3. Interactive painting
@OBJECT CREATE mysterious_painting "Mysterious Painting"
@OBJECT EDIT mysterious_painting TAKEABLE false
@OBJECT EDIT mysterious_painting TRIGGER ONLOOK message("🖼️  The painted eyes seem to watch your every move.")
@OBJECT EDIT mysterious_painting TRIGGER ONPOKE random_chance(20) && message("👁️  The painting shifts slightly, revealing hidden text!")
```

## Integration with Quests

Triggers work seamlessly with the quest system:

```bash
# Quest item that only reveals info when quest is active
@OBJECT CREATE ancient_map "Ancient Map"
@OBJECT EDIT ancient_map FLAG QUESTITEM
@OBJECT EDIT ancient_map TRIGGER ONLOOK has_quest("treasure_hunt") && message("🗺️  The X marks the old lighthouse!")

# Item that should be used at quest location
@OBJECT CREATE ritual_scroll "Ritual Scroll"
@OBJECT EDIT ritual_scroll USABLE true
@OBJECT EDIT ritual_scroll TRIGGER ONUSE message("📜 The scroll crumbles to dust as ancient words are spoken...") && consume()
```

## See Also

- [@OBJECT Command Reference](OBJECT_COMMANDS.md)
- [Quest System Documentation](../QUEST_SYSTEM.md)
- [Object Flags Reference](OBJECT_FLAGS.md)
- [Admin Command Overview](../ADMIN_COMMANDS.md)

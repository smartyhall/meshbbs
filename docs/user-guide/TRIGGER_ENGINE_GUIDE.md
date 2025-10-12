# TinyMUSH Trigger Engine - Player Guide

## What Are Triggers?

Triggers are simple scripts that make objects in the world interactive! They fire automatically when you interact with objects in specific ways.

**No programming knowledge required!** The trigger system is designed for creative storytelling, not coding.

---

## Trigger Types

Objects can respond to 6 different player actions:

| Trigger | When It Fires | Example Use |
|---------|---------------|-------------|
| **OnLook** | When you examine an object | Quest clues, hidden messages, descriptions |
| **OnUse** | When you use an object | Healing potions, tools, magical items |
| **OnTake** | When you pick up an object | Cursed items, quest triggers, warnings |
| **OnDrop** | When you drop an object | Puzzle mechanics, placement requirements |
| **OnPoke** | When you poke/prod an object | Interactive toys, mystery boxes, creatures |
| **OnEnter** | When you enter a room with the object | Ambient effects, greetings, atmosphere |

---

## Creating Simple Triggers (One-Liners)

Use the `/when` command for quick, simple triggers:

```
/when <trigger_type> <object> <action>
```

### Examples:

**Make a crystal say MEEP when poked:**
```
/when poke crystal say MEEP!
```

**Give health when using a potion:**
```
/when use potion give 50 health
```

**Display a message when examining a note:**
```
/when look note say The note reads: "Meet me at midnight!"
```

**Trigger when taking a cursed ring:**
```
/when take ring say The ring feels cold and heavy...
```

### Available Actions:
- `say <text>` - Display a message
- `give <amount> health` - Heal the player
- `give <item>` - Grant an item
- `teleport to <room>` - Move the player
- `unlock <direction>` - Unlock an exit
- `lock <direction>` - Lock an exit

---

## Creating Multi-Line Scripts

For more complex behavior, use `/script`:

```
/script <object> <trigger_type>
<your script lines>
/done
```

### Example: Healing Potion

```
/script potion use
Say You drink the healing potion!
Give 50 health
Say You feel much better.
/done
```

### Example: Quest-Aware Ancient Key

```
/script ancient_key look
If player has quest find_key:
  Say The key glows with recognition!
Otherwise:
  Say It's just an old key.
/done
```

### Example: Mystery Box with Random Outcomes

```
/script mystery_box poke
1 in 2 chance:
  Say The box springs open! You find treasure!
  Give 100 coins
Otherwise:
  Say The box rattles but stays locked.
/done
```

---

## Guided Creation (Wizard Mode)

Don't want to type scripts? Use the interactive wizard:

```
/wizard
```

The wizard will:
1. Show you objects you can modify
2. Let you pick a trigger type
3. Offer action templates
4. Guide you through customization
5. Generate the script automatically!

Perfect for beginners!

---

## Managing Triggers

### View Triggers on an Object
```
/show <object>
```
Shows all triggers attached to the object.

### Remove a Trigger
```
/remove <object> <trigger_type>
```
Deletes the specified trigger.

### Test a Trigger
```
/test <object> <trigger_type>
```
Runs the trigger without side effects (dry-run mode).

---

## Advanced: Natural Language Syntax

You can write triggers in friendly, readable language:

### Conditions:
- `If player has <item>:`
- `If player has quest <quest_id>:`
- `If room flag <flag>:`
- `If object flag <flag>:`
- `1 in <N> chance:` (random)

### Actions:
- `Say <text>` or `Say to room <text>`
- `Give player <amount> health`
- `Give player <item>`
- `Take <item> from player`
- `Remove <item>` (consume item)
- `Teleport player to <room>`
- `Unlock <direction>`
- `Lock <direction>`

### Logic:
- `and` - Both conditions must be true
- `or` - Either condition can be true
- `Otherwise:` - Else branch

### Example: Complete Quest Trigger
```
If player has quest dragon_slayer and player has dragon_scale:
  Say You present the dragon scale to the king!
  Give 1000 coins
  Say Quest complete: Dragon Slayer
Otherwise:
  Say The king awaits proof of your victory.
```

---

## Advanced: DSL Syntax (Power Users)

For maximum control, use the advanced DSL:

### Syntax:
```
function(args) && condition ? action : fallback
```

### Functions:
- `message("text")` - Display message
- `message_room("text")` - Broadcast to room
- `heal(amount)` - Heal player
- `grant_item("item_id")` - Give item
- `consume()` - Remove this object
- `teleport("room_id")` - Move player
- `unlock_exit("direction")` - Unlock exit
- `lock_exit("direction")` - Lock exit

### Conditions:
- `has_item("item_id")` - Check inventory
- `has_quest("quest_id")` - Check quest status
- `flag_set("flag")` - Check object flag
- `room_flag("flag")` - Check room flag
- `random_chance(50)` - 50% probability
- `current_room == "room_id"` - Location check

### Variables:
- `$player` - Player's display name
- `$object` - Object's name
- `$room` - Room's name
- `$player_name` - Player's username
- `$object_id` - Object's ID
- `$room_id` - Room's ID

### Example:
```
has_quest("treasure_hunt") && has_item("old_map") ? 
  message("The X marks the spot!") && teleport("treasure_room") :
  message("The map looks meaningless to you.")
```

---

## Tips & Best Practices

### 1. Start Simple
Begin with `/when` one-liners before moving to `/script`.

### 2. Test Your Triggers
Use `/test <object> <trigger>` to check behavior before players see it.

### 3. Keep Scripts Short
Triggers have limits:
- Max 512 characters per script
- Max 3 messages per trigger
- Max 10 actions per trigger
- 100ms execution timeout

### 4. Use Meaningful Names
Name objects clearly so players know what to interact with.

### 5. Provide Feedback
Always give the player a message so they know the trigger worked!

### 6. Think About Order
Triggers fire in this order:
1. OnLook (when examining)
2. OnTake (when picking up)
3. OnDrop (when dropping)
4. OnUse (when using)
5. OnPoke (when poking)
6. OnEnter (when entering room)

---

## Example Objects in the Museum

Visit the Mesh Museum to see working examples:

1. **Healing Potion** - OnUse with consume & heal
2. **Ancient Key** - OnLook with quest conditions
3. **Mystery Box** - OnPoke with random outcomes
4. **Tattered Note** - OnLook with hidden message
5. **Teleport Stone** - OnUse with teleportation
6. **Singing Mushroom** - OnEnter with ambience

Use `/show <object>` to see their trigger scripts!

---

## Getting Help

- **In-game**: Use `HELP TRIGGERS` for quick reference
- **Questions**: Ask in the Town Square (SAY command)
- **Bugs**: Contact admins with `/report` command
- **Ideas**: Share creative uses in the Community Board!

---

## Security & Rate Limiting

To prevent abuse, the trigger system has safeguards:

- **Rate Limit**: Objects can fire max 100 times per minute
- **Player Cooldown**: 1 second between triggering same object
- **Execution Timeout**: Scripts must complete in 100ms
- **Global Shutoff**: Admins can disable all triggers if needed

If you hit a rate limit, wait a moment and try again!

---

Happy scripting! Make the world come alive with interactive objects! 🎭✨

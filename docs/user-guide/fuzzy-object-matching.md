# Fuzzy Object Matching

## Overview

MeshBBS TinyMUSH now supports intelligent fuzzy matching for object commands. You no longer need to type the exact object name - partial matches work automatically!

## How It Works

### Single Match - Auto-Select

When only one object matches your search term, it's automatically selected:

```
You go east.

=== Mesh Museum ===
Objects here:
  📦 Healing Potion
  📦 Ancient Key

> take potion
You take the Healing Potion.

> drop key
You drop the Ancient Key.
```

### Multiple Matches - Disambiguation Menu

When multiple objects match, you'll get a numbered menu:

```
=== Treasure Room ===
Objects here:
  📦 Healing Potion
  📦 Mana Potion  
  📦 Strength Potion

> take potion
Did you mean:

1) Healing Potion
2) Mana Potion
3) Strength Potion

Enter the number of your choice:

> 2
You take the Mana Potion.
```

### No Matches - Helpful Error

If nothing matches, you'll get a clear message:

```
> take sword
There is no 'SWORD' here to take.
```

## Supported Commands

Fuzzy matching works with all major object interaction commands:

- **TAKE** / **GET** - Pick up objects from the room
- **DROP** - Drop objects from your inventory
- **USE** - Use items from your inventory
- **EXAMINE** / **X** - Examine objects in room or inventory

## Examples

### Taking Items

```
> take key          # Matches "Ancient Key"
> take healing      # Matches "Healing Potion"
> take potion       # Shows menu if multiple potions exist
```

### Using Items

```
> use torch         # Matches "Old Torch" in inventory
> use health        # Matches "Health Pack" in inventory
```

### Examining Items

```
> examine stone     # Matches "Moonstone" or shows menu if multiple
> x crystal         # Matches "Crystal Shard"
```

## Tips

1. **Be Specific**: The more specific your search term, the better
   - `take key` might match multiple keys
   - `take ancient key` will likely match exactly one

2. **Case Insensitive**: Capitalization doesn't matter
   - `TAKE POTION` = `take potion` = `Take Potion`

3. **Partial Matching**: Any substring of the object name works
   - "potion" matches "Healing **Potion**"
   - "heal" matches "**Heal**ing Potion"

4. **Disambiguation Numbers**: Just type the number when prompted
   - No need for "1)" or extra text, just `1`

5. **Exact Names Still Work**: The full object name always works
   - `take Healing Potion` will never trigger disambiguation

## Technical Details

### How Matching Works

The system performs case-insensitive substring matching on object **display names**:

1. Search term is converted to uppercase
2. Each object's name is converted to uppercase
3. Objects containing the search term are collected
4. Results are sorted by relevance (exact matches first)

### Disambiguation Session

When disambiguation is triggered:

1. A session is created storing:
   - Command type (take/drop/use/examine)
   - Search term you entered
   - List of matching objects
   - Context (room or inventory)

2. Your next numeric input selects from the list

3. Session is automatically cleared after selection

### Priority Order

When you examine an object, the system checks:

1. **Inventory first** - items you're carrying
2. **Room second** - items in the current location

This means if you have a "key" in inventory and there's a "key" in the room, `examine key` will show your inventory key first.

## Error Handling

### Invalid Selection

```
> take potion
Did you mean:
1) Healing Potion
2) Mana Potion

> 5
Invalid selection. Please choose a number between 1 and 2.
```

### Session Timeout

Disambiguation sessions are temporary. If you enter other commands after seeing a disambiguation menu, the session clears and you'll need to retry the command.

## Backward Compatibility

All existing exact-match commands continue to work exactly as before. This feature enhances the user experience without breaking any existing functionality.

## Future Enhancements

Potential improvements being considered:

- **Relevance Scoring**: Sort matches by how well they match
- **Recent Items**: Prioritize recently used objects
- **Context Awareness**: Consider quest objectives when disambiguating
- **Aliases**: Support common abbreviations (hp = healing potion)
- **Fuzzy NPC Matching**: Extend to TALK command for NPCs

## See Also

- [Object System](./objects.md)
- [Inventory Management](./inventory.md)
- [Command Reference](./commands.md)

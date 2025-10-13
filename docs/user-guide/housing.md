# Housing System Guide

The housing system in TinyMUSH allows players to purchase, customize, and manage their own private rooms within the game world.

## Table of Contents
- [Purchasing Housing](#purchasing-housing)
- [Managing Your Home](#managing-your-home)
- [Room Customization](#room-customization)
- [Guest Management](#guest-management)
- [Room Security](#room-security)
- [Item Protection](#item-protection)

## Purchasing Housing

### Finding Housing Offices

Housing is purchased at designated **Housing Office** locations throughout the world. Use the `MAP` command to explore and find housing offices in different areas.

### RENT Command

Once you're at a housing office, purchase housing using:

```
RENT <template_id>
```

**Example:**
```
> RENT suburban_cottage
```

**Requirements:**
- You must be at a Housing Office location
- You must have enough currency to cover the purchase price
- The housing template must be available
- You can only own one housing unit at a time (limitation may vary by server)

**Purchase Cost:**
- One-time payment (varies by housing type)
- No recurring fees or rent payments
- Price set by the housing template

**What Happens:**
- Currency is deducted from your balance (on-hand and bank combined)
- The housing becomes permanently yours
- You gain owner permissions for the room
- A personal instance of the housing is created for you

## Managing Your Home

### HOME Command

Teleport to your home or manage your housing:

```
HOME              # Teleport to your primary home
HOME LIST         # List all homes you own or have guest access to
HOME <number>     # Teleport to a specific home by number
HOME SET <number> # Set a different home as your primary
```

**Examples:**
```
> HOME              # Go to primary home

> HOME LIST         # Show all accessible housing
=== Your Housing ===

[★ Primary] 1. Suburban Cottage (Residential)
   Location: instance_suburban_cottage_alice | Access: OWNED

           2. Downtown Loft (Residential)  
   Owner: Bob | Access: GUEST

Use 'HOME <number>' to travel to a specific property.
Use 'HOME SET <number>' to change your primary home.

> HOME 2            # Go to Bob's loft (as guest)
> HOME SET 1        # Make cottage the primary home
```

### Multiple Homes

While the current implementation typically allows one owned home per player, you can:
- Have **guest access** to other players' homes
- Visit any home you're invited to
- Set which owned home is your "primary" (if multiple are allowed)

### Teleportation

The HOME command instantly teleports you to the housing location. There may be a cooldown period between uses (typically 5 minutes) to prevent abuse.

## Room Customization

### DESCRIBE Command

Customize your room's description:

```
DESCRIBE                   # View current room description
DESCRIBE <new description> # Set a new description
```

**Requirements:**
- Must be in your own housing (or have edit permission as guest)
- Maximum description length: 500 characters
- Description is visible to everyone who enters the room

**Examples:**
```
> DESCRIBE
Current description: "Empty room."

> DESCRIBE A cozy cottage with warm wooden floors and a crackling fireplace. Bookshelves line the walls, and a comfortable armchair sits by the window.
Room description updated successfully!

> LOOK
Suburban Cottage
A cozy cottage with warm wooden floors and a crackling fireplace. Bookshelves line the walls, and a comfortable armchair sits by the window.

Exits: [OUT]
```

**Permissions:**
- **Owners**: Can always edit descriptions in their own housing
- **Guests**: Can only edit if the housing template allows `can_edit_description` permission (typically disabled)

### Viewing Your Description

Use `LOOK` to see the current room description:

```
> LOOK
Suburban Cottage
A cozy cottage with wooden beams and a stone fireplace. Sunlight 
streams through lace curtains.

Exits: South (to Town Square)
Players here: You
Items here: Wooden Chair, Bookshelf
```

## Guest Management

### Inviting Guests

Grant access to your housing:

```
INVITE <player_name>    # Grant guest access
UNINVITE <player_name>  # Revoke guest access
```

**Requirements:**
- Must be **inside your own housing** when inviting/uninviting
- Target player must exist in the system
- Cannot invite someone who's already a guest

**Examples:**
```
> INVITE Bob
You have granted Bob guest access to your housing.

> INVITE Alice
Alice is already a guest here.

> UNINVITE Bob
You have revoked Bob's guest access.
```

### Guest Privileges

**What Guests Can Do:**
- Use `HOME LIST` to see homes they have guest access to
- Teleport to your home with `HOME <number>`
- Enter rooms even if locked (owner's discretion)

**What Guests Cannot Do:**
- Edit room descriptions (unless template explicitly allows)
- Invite other guests
- Take or move items (unless items are unlocked)

### Removing Unwanted Guests

If someone is currently in your housing and causing trouble:

```
KICK <player_name>    # Eject player from your housing
```

**KICK Command:**
- Immediately teleports the target player out of your housing
- Does **not** revoke guest access automatically
- Use `UNINVITE` afterward to prevent their return
- Target must currently be in your housing room

**Example:**
```
> KICK Bob
You have ejected Bob from your housing.

> UNINVITE Bob
You have revoked Bob's guest access.
```

## Room Security

### Locking Your Room

Restrict access to your housing room:

```
LOCK      # Lock the current room
UNLOCK    # Unlock the current room
```

**Requirements:**
- Must be inside your own housing (not guest access)
- Cannot lock rooms you don't own

**When Locked:**
- Only you and invited guests can enter
- Other players see "The door is locked" when attempting entry
- Room may still be visible in area lists

**When Unlocked:**
- Any player can enter your housing
- Items can still be locked individually for protection

**Examples:**
```
> LOCK
You lock the room. Only you and your guests can enter now.

> LOCK
This room is already locked.

> UNLOCK
You unlock the room. Anyone can now enter.
```

### Checking Lock Status

When you `LOOK` at a room, locked rooms display a [LOCKED] indicator:

```
> LOOK
Suburban Cottage [LOCKED]
A cozy cottage with warm wooden floors and a crackling fireplace.

Exits: [OUT]
```

## Item Protection

### Locking Items

Protect items in your inventory from being taken by others:

```
LOCK <item_name>      # Lock an item you own
UNLOCK <item_name>    # Unlock an item you own
```

**Requirements:**
- Item must be in your inventory
- You must be the owner of the item
- World items cannot be locked

**When Locked:**
- Item cannot be taken by other players
- Item can still be dropped or given away by you
- Locked status persists even if item is on the ground

**When Unlocked:**
- Other players can take the item if it's accessible
- Useful for sharing items with guests

**Examples:**
```
> LOCK sword
You lock Sword. It cannot be taken by others.

> LOCK sword
Sword is already locked.

> UNLOCK sword
You unlock Sword. Others can take it now.

> LOCK table
Table is a world item and cannot be locked.
```

### Ownership

**Item Ownership Types:**
1. **Player-owned**: Items you purchased, crafted, or were given
   - Can be locked/unlocked by owner
   - Shows owner in item description
   
2. **World items**: Fixed objects in the world
   - Cannot be locked
   - Cannot be taken from their location

**Viewing Ownership:**
When you `EXAMINE` an item, it shows the owner:

```
> EXAMINE sword
Steel Longsword
A finely crafted blade with intricate engravings.
Owner: Alice
Status: LOCKED
```

## Tips and Best Practices

### For New Players
- Check available housing at Housing Office locations
- Save enough currency for the full purchase price
- Lock valuable items to protect them
- Use `DESCRIBE` to personalize your space

### For Established Players
- Invite trusted friends with `INVITE`
- Use room locking when you have guests
- Organize items with LOCK/UNLOCK
- Use `HOME LIST` to manage multiple homes (if supported)

### Security Best Practices
- Lock your room when away or when guests are present
- Only invite trusted players to your guest list
- Lock items before giving guests access
- Use `KICK` to remove disruptive guests immediately

## Troubleshooting

### "You don't have permission to purchase housing"
- You may already own housing (one at a time limit)
- You may not have enough currency
- Make sure you're at a Housing Office location

### "You cannot access this room"
- The room is locked and you're not on the guest list
- Ask the owner to `INVITE` you

### "You can only lock rooms in your own housing"
- You're trying to lock a room you don't own
- Move to your own housing first
- Guests cannot lock rooms

### "You don't own [item]"
- You're trying to lock an item owned by someone else
- Only the item owner can lock/unlock it
- World items cannot be locked

## Related Commands

- `MAP` - View available rooms
- `LOOK` - Examine current room
- `GET` / `DROP` - Pick up or drop items
- `INVENTORY` - See what you're carrying
- `EXAMINE` - View item details

## See Also

- [Commands Reference](commands.md) - Full command list
- [Economy Guide](economy.md) - Gold and trading
- [Quest Guide](quests.md) - Earning gold through quests

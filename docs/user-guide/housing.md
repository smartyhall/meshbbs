# Housing System Guide

The housing system in TinyMUSH allows players to rent, customize, and manage their own private rooms within the game world.

## Table of Contents
- [Renting a Room](#renting-a-room)
- [Managing Your Home](#managing-your-home)
- [Room Customization](#room-customization)
- [Guest Management](#guest-management)
- [Room Security](#room-security)
- [Item Protection](#item-protection)
- [Reclaim Box](#reclaim-box)
- [Abandonment](#abandonment)

## Renting a Room

### RENT Command

Find available rooms using the `MAP` command, then rent a room using:

```
RENT <room_id>
```

**Example:**
```
> RENT suburban_cottage
```

**Requirements:**
- You must have enough gold to cover the rental fee
- The room must be available (not already rented)
- You can rent multiple rooms (multi-home support)

**Rental Costs:**
- Initial rental fee (varies by room)
- Recurring payments (monthly or weekly, depending on room)

**What Happens:**
- Your gold is deducted for the initial payment
- The room becomes your "home"
- You gain owner permissions for the room
- Recurring payment schedule begins

## Managing Your Home

### HOME Command

Teleport to your home or manage multiple homes:

```
HOME              # Teleport to your primary home
HOME <number>     # Teleport to a specific home (if you have multiple)
HOME LIST         # List all your rented homes
HOME SET <number> # Set a different home as your primary
```

**Examples:**
```
> HOME              # Go to primary home
> HOME LIST         # Show all homes
  1. Suburban Cottage (Primary) - 100g/month
  2. Downtown Loft - 150g/month
> HOME 2            # Go to home #2
> HOME SET 2        # Make home #2 the primary
```

### Recurring Payments

**Payment Schedule:**
- Payments are automatically deducted from your gold
- You'll receive notifications before payment is due
- Grace period: 7 days after missed payment

**Payment Notifications:**
- "Payment due in 3 days" - First warning
- "Payment due tomorrow" - Second warning
- "Payment overdue" - Grace period begins

**What Happens if You Can't Pay:**
- 7-day grace period after missed payment
- Your belongings are moved to the Reclaim Box
- The room becomes available for others to rent

## Room Customization

### DESCRIBE Command

Customize your room's description:

```
DESCRIBE <text>
```

**Example:**
```
> DESCRIBE A cozy cottage with wooden beams and a stone fireplace. 
  Sunlight streams through lace curtains.
```

**Limitations:**
- Maximum length: 500 characters
- Can only describe rooms you own
- Description is visible to all visitors

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

Allow other players to access your home:

```
INVITE <username>
```

**Example:**
```
> INVITE Alice
Alice has been added to your guest list.
```

**Guest Permissions:**
- Can enter your home
- Can see the room and items
- Cannot pick up locked items
- Cannot modify the room description

### Removing Guests

Revoke guest access:

```
UNINVITE <username>
```

**Example:**
```
> UNINVITE Bob
Bob has been removed from your guest list.
```

### Kicking Guests

Remove a guest who is currently in your home:

```
KICK <username>
```

**Example:**
```
> KICK Charlie
Charlie has been removed from your home.
```

**What Happens:**
- The player is immediately teleported to the Town Square
- They can still return if they're on your guest list
- Use `UNINVITE` to permanently revoke access

### Viewing Guest List

See who has access to your home:

```
HOME GUESTS
```

## Room Security

### Locking Your Room

Prevent unauthorized access:

```
LOCK
```

**When Locked:**
- Only you and invited guests can enter
- Other players see "The door is locked" if they try to enter
- Room still appears on the MAP

### Unlocking Your Room

Allow anyone to enter:

```
UNLOCK
```

**When Unlocked:**
- Any player can enter your home
- Items can still be protected individually
- Guests on your list have no special privileges

### Checking Lock Status

Use `LOOK` to see if a room is locked:

```
> LOOK
Suburban Cottage [LOCKED]
A cozy cottage with wooden beams...
```

## Item Protection

### Locking Items

Protect items from being picked up by guests:

```
LOCK <item_name>
```

**Example:**
```
> LOCK Treasure Chest
The Treasure Chest is now locked.
```

**Protected Items:**
- Guests cannot `GET` locked items
- Only the room owner can pick them up
- Locked items show [LOCKED] in the room description

### Unlocking Items

Allow items to be picked up:

```
UNLOCK <item_name>
```

**Example:**
```
> UNLOCK Wooden Chair
The Wooden Chair is now unlocked.
```

### Ownership Tracking

**How It Works:**
- Items you `DROP` in your home are automatically owned by you
- Items you bring into your home retain their original owner
- Only owned items can be locked

**Viewing Ownership:**
```
> EXAMINE Treasure Chest
Treasure Chest [LOCKED]
Owner: YourUsername
Description: A sturdy wooden chest with iron bands.
```

## Reclaim Box

### What is the Reclaim Box?

A secure storage area where your items go if:
- Your home is abandoned due to non-payment
- You need to reclaim items from a former home

### Location

The Reclaim Box is located in the **Town Hall**.

### Accessing Your Items

```
> GO Town Hall
> LOOK
Town Hall
...
Reclaim boxes available: Type RECLAIM to access yours.

> RECLAIM
Your Reclaim Box:
  1. Wooden Chair
  2. Bookshelf
  3. Treasure Chest
  4. Gold Coins (50)
  
> GET Wooden Chair
You retrieve the Wooden Chair from your reclaim box.
```

### Reclaim Box Rules

- Items are stored for **30 days** after abandonment
- After 30 days, unclaimed items are deleted
- No storage limit (all your items are saved)
- No fee to reclaim items

## Abandonment

### Voluntary Abandonment

Currently, there is no command to voluntarily abandon a home. To stop paying rent:
- Let the payment lapse
- Your items will go to the Reclaim Box after the grace period

### Automatic Abandonment

**Triggers:**
- Missed payment + 7 day grace period expires
- Player account inactive for 90 days

**What Happens:**
1. You receive a final notification
2. All items in the room are moved to your Reclaim Box
3. The room becomes available for rent
4. Your gold balance is unchanged

**Notification Timeline:**
- Day 0: Payment missed
- Day 3: "Payment overdue - 4 days remaining"
- Day 7: "Final notice - payment overdue"
- Day 8: Room abandoned, items moved to Reclaim Box

## Tips and Best Practices

### For New Players
- Start with an inexpensive room
- Keep enough gold in reserve for 2-3 months of rent
- Lock valuable items immediately
- Use `DESCRIBE` to make your space unique

### For Established Players
- Rent multiple homes for different purposes
- Create guest lists for friends and guild members
- Organize items by locking/unlocking as needed
- Check `HOME LIST` regularly to track payments

### Security Best Practices
- Always lock your home if you have valuable items
- Only invite trusted players to your guest list
- Lock items before inviting guests
- Use `KICK` if someone is disruptive

## Troubleshooting

### "You don't have permission to rent this room"
- The room may already be rented
- You may not have enough gold
- Check `MAP` for available rooms

### "You cannot access this room"
- The room is locked and you're not on the guest list
- Ask the owner to `INVITE` you

### "Payment failed - insufficient funds"
- Earn more gold before the grace period expires
- Sell items at shops or complete quests

### "Item not found"
- Make sure you're in the correct room
- Items may be in your Reclaim Box if the room was abandoned
- Check spelling of the item name

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

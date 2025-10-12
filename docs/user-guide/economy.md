# Economy & Trading Guide

TinyMUSH features a flexible currency system that can be configured as either modern decimal currency (like credits or dollars) or fantasy multi-tier currency (like gold/silver/copper). The game also includes shops, banking, and player-to-player trading systems.

## Table of Contents
- [Currency System](#currency-system)
- [Earning Money](#earning-money)
- [Shops](#shops)
- [Banking](#banking)
- [Player Trading](#player-trading)
- [Economy Tips](#economy-tips)

## Currency System

### Currency Modes

The server administrator configures which currency system the game uses. Your game will use one of these two modes:

#### Fantasy Multi-Tier Currency
Traditional fantasy-style coins with conversion between tiers:
- **Copper (c)** - Base currency for small purchases
- **Silver (s)** - Mid-tier (typically 10 copper = 1 silver)
- **Gold (g)** - High-tier (typically 100 copper = 1 gold)

**Example:** `5g 3s 7c` means 5 gold, 3 silver, 7 copper

#### Modern Decimal Currency
Modern-style currency with decimal subdivisions:
- **Credits/Dollars** - Single currency with cents
- **Display:** `¤12.34` or `$1.50`

**Example:** `¤100.00` means 100 credits

> **Note:** The examples in this guide assume fantasy multi-tier currency (gold/silver/copper), but the concepts apply to both systems. Your server's currency display will match its configuration.

### Checking Your Balance

```
INVENTORY     # Shows currency balance and items
```

**Example Output (Multi-Tier):**
```
Your Inventory:
Currency: 5g 3s 7c (507 copper total)

Carrying:
- Wooden Sword
- Health Potion x3
- Bread x2
```

**Example Output (Decimal):**
```
Your Inventory:
Currency: ¤507.00

Carrying:
- Wooden Sword
- Health Potion x3
- Bread x2
```

## Earning Money

### Quests

Complete quests to earn gold and platinum rewards.

```
QUEST          # View available quests
QUEST ACTIVE   # View your active quests
QUEST <id>     # View quest details
```

**Quest Rewards:**
- Easy quests: 50-100c (or ¤0.50-1.00 in decimal)
- Medium quests: 150-300c
- Hard quests: 400-800c
- Daily quests: Repeatable for consistent income

**See Also:** [Quest Guide](quests.md)

### Selling Items

Sell items you've found or crafted to shops.

```
LIST            # View shop inventory and prices
SELL <item>     # Sell an item to the shop
```

**Example:**
```
> SELL Wooden Sword
You sold Wooden Sword for 25c.
```

**Item Values (Multi-Tier Example):**
- Weapons: 20-500c depending on quality
- Armor: 30-600c depending on type
- Consumables: 5-50c
- Crafting materials: 10-100c

### Companion Activities

Train companions to earn money passively.

**See Also:** [Companion Guide](companions.md)

### Treasure Hunting

Explore the world to find chests and treasure rooms.

```
> LOOK
Hidden Grove
A secluded grove with ancient trees. In the corner, you notice 
something gleaming.

Items here: Treasure Chest [LOCKED]

> EXAMINE Treasure Chest
A locked treasure chest. You might be able to open it with a key.
```

**Treasure Sources:**
- Locked chests (requires lockpicking or keys)
- Hidden rooms (use MAP to explore)
- Monster drops (from combat encounters)

## Shops

### Finding Shops

Shops are located in various towns. Use `MAP` to find them:

```
MAP
```

**Common Shop Locations:**
- Town Square - General Store
- Market District - Armor Shop, Weapon Shop
- Harbor - Trading Post
- Residential Area - Crafting Supplies

### Shopping Commands

#### LIST - View Shop Inventory

```
LIST            # View all items for sale
LIST <category> # View items in a category
```

**Example:**
```
> LIST
General Store Inventory:
  1. Bread - 5g
  2. Health Potion - 15g
  3. Lockpick - 25g
  4. Rope - 10g
  5. Torch - 8g

Your gold: 450g
```

#### BUY - Purchase Items

```
BUY <item>        # Buy one item
BUY <item> <qty>  # Buy multiple items
```

**Examples:**
```
> BUY Health Potion
You purchased Health Potion for 15g.

> BUY Bread 5
You purchased 5x Bread for 25g.
```

**Purchase Rules:**
- Must have enough gold/platinum
- Shop must have the item in stock
- Some items have purchase limits

#### SELL - Sell Items

```
SELL <item>       # Sell one item
SELL <item> <qty> # Sell multiple items
```

**Examples:**
```
> SELL Wooden Sword
You sold Wooden Sword for 25g.

> SELL Iron Ore 10
You sold 10x Iron Ore for 150g.
```

**Selling Rules:**
- Shop must accept the item type
- Sell price is typically 50% of buy price
- Cursed or broken items may be refused

### Shop Types

#### General Store
- **Location:** Town Square
- **Sells:** Basic supplies, food, tools
- **Buys:** Most common items
- **Prices:** 5-50c per item

#### Weapon Shop
- **Location:** Market District
- **Sells:** Swords, axes, bows, ammunition
- **Buys:** Weapons and combat gear
- **Prices:** 20-500c

#### Armor Shop
- **Location:** Market District
- **Sells:** Helmets, chest armor, shields, boots
- **Buys:** Armor and protective gear
- **Prices:** 30-600c

#### Potion Shop
- **Location:** Market District
- **Sells:** Healing potions, mana potions, buff potions
- **Buys:** Herbs, potion ingredients
- **Prices:** 15-100c

#### Trading Post
- **Location:** Harbor
- **Sells:** Rare items, imports, special goods
- **Buys:** Almost anything
- **Prices:** Variable

#### Black Market (if discovered)
- **Location:** Hidden
- **Sells:** Rare weapons, forbidden items, powerful potions
- **Buys:** Stolen goods, rare materials
- **Prices:** Very high

## Banking

### Bank Locations

Banks are found in major towns:
- Town Square Bank
- Market District Credit Union
- Harbor First Bank

### Banking Commands

#### DEPOSIT - Store Money

```
DEPOSIT <amount>
```

**Examples (Multi-Tier):**
```
> DEPOSIT 500
You deposited 500c. New balance: 12g 7c

> DEPOSIT 5g 3s
You deposited 5g 3s (530c). New balance: 18g 7c
```

**Examples (Decimal):**
```
> DEPOSIT 50.00
You deposited ¤50.00. New balance: ¤125.07
```

**Deposit Benefits:**
- Money is safe from theft
- Earn 1% interest monthly
- Access from any bank location

#### WITHDRAW - Retrieve Money

```
WITHDRAW <amount>
```

**Examples (Multi-Tier):**
```
> WITHDRAW 300
You withdrew 300c (3g). New balance: 9g 7c
```

**Examples (Decimal):**
```
> WITHDRAW 75.00
You withdrew ¤75.00. New balance: ¤50.07
```

#### BALANCE - Check Bank Balance

```
BALANCE
```

**Example Output (Multi-Tier):**
```
Your Bank Account:
Banked: 9g 7c (Earning 1% interest)
On hand: 3g 0c

Total wealth: 12g 7c
```

**Example Output (Decimal):**
```
Your Bank Account:
Banked: ¤50.07 (Earning 1% interest)
On hand: ¤75.00

Total wealth: ¤125.07
```

### Interest Rates

- **Interest Rate:** 1% per month (30 real-world days)
- **Minimum Balance:** 100c (or ¤1.00) for interest
- **Compound Interest:** Yes
- **Maximum Interest:** 10c/month (or ¤0.10)

### Fees

- **Deposit:** Free
- **Withdrawal:** Free
- **Monthly Maintenance:** None
- **Overdraft:** Not available

## Player Trading

### Trade Commands

#### REQUEST - Initiate a Trade

```
REQUEST <username>
```

**Example:**
```
> REQUEST Alice
You sent a trade request to Alice.

[Alice must accept with: ACCEPT]
```

#### ACCEPT - Accept a Trade

```
ACCEPT
```

#### OFFER - Add Items/Money to Trade

```
OFFER <item>       # Offer an item
OFFER <amount>     # Offer money
```

**Examples (Multi-Tier):**
```
> OFFER Wooden Sword
You offered: Wooden Sword

> OFFER 100
You offered: 100c (1g)

> OFFER 5g 3s
You offered: 5g 3s
```

**Examples (Decimal):**
```
> OFFER 10.50
You offered: ¤10.50
```

#### REMOVE - Remove Items from Trade

```
REMOVE <item>
REMOVE <amount>
```

#### READY - Mark Yourself as Ready

```
READY
```

**Trade Flow:**
1. Both players offer items/money
2. Both players type `READY`
3. Trade executes automatically

#### CANCEL - Cancel the Trade

```
CANCEL
```

### Trade Session Example

```
[Player1] > REQUEST Player2
You sent a trade request to Player2.

[Player2] > ACCEPT
You accepted the trade request from Player1.

=== Trade Window ===
Player1 offers:
  - Iron Sword
  - 50c

Player2 offers:
  - Health Potion x5
  - 1g

Both parties ready: No
===================

[Player1] > OFFER Iron Sword
You offered: Iron Sword

[Player1] > OFFER 50
You offered: 50c

[Player1] > READY
You are ready. Waiting for Player2...

[Player2] > OFFER Health Potion 5
You offered: 5x Health Potion

[Player2] > OFFER 100
You offered: 100c (1g)

[Player2] > READY
You are ready.

=== Trade Complete! ===
You received:
  - Health Potion x5
  - 100c

You gave:
  - Iron Sword
  - 50c
```

### Trade Safety

**Safety Rules:**
- Both players must be in the same location
- Trade window shows exactly what's being exchanged
- Both players must confirm with `READY`
- Trade can be cancelled anytime before both are ready
- No partial trades - it's all or nothing

**Scam Prevention:**
- Always check the trade window carefully
- Verify item names and quantities
- Watch for item swaps (someone removing/adding at last second)
- Never trade for promises of "later" - all trades are immediate

**Disputes:**
- Trades are final once executed
- Moderators cannot reverse trades
- Report scammers to admins
- Keep screenshots/logs of trades for high-value items

## Economy Tips

### For New Players

**Starting Out:**
1. Complete starter quests for initial currency
2. Don't spend all your money - save for housing
3. Shop around - prices vary by location
4. Sell loot instead of hoarding

**Budget Management (Multi-Tier Example):**
- Keep 200-300c emergency fund
- Save for rent (100-150c/month typical)
- Budget 50c/week for potions and supplies

### For Intermediate Players

**Earning Strategies:**
1. **Quest Chains** - Complete related quests for bonus rewards
2. **Shop Arbitrage** - Buy low in one town, sell high in another
3. **Crafting** - Craft items to sell for profit
4. **Treasure Hunting** - Explore for chests and rare finds

**Investment:**
- Bank excess currency for interest
- Upgrade equipment gradually
- Rent additional homes for prestige

### For Advanced Players

**Wealth Building:**
1. **Companion Training** - Passive income
2. **High-Value Trades** - Trading rare items
3. **Rare Item Flipping** - Buy rare, sell rarer
4. **Quest Mastery** - Speed-run high-reward quests

**Economy Mastery:**
- Focus on hard quests for maximum rewards
- Trade valuable items at Trading Post
- Save currency for rare shop items

### Market Strategies

**Buy Low, Sell High:**
- General Store buys most items at 50% value
- Trading Post pays 75% value for rare items
- Timing matters - some shops restock weekly

**Item Values (Multi-Tier Example):**
| Item Type | Shop Buy | Shop Sell | Player Trade |
|-----------|----------|-----------|--------------|
| Basic Weapon | 50c | 25c | 30-40c |
| Rare Weapon | 5g | 2g 50c | 4-6g |
| Healing Potion | 15c | 7c | 10-12c |
| Crafting Material | 50c | 25c | 40-80c |

## Troubleshooting

### "Insufficient funds"
- Check `INVENTORY` for current balance
- Withdraw from bank if needed
- Sell items or complete quests

### "Shop does not accept that item"
- Try a different shop (Trading Post accepts most items)
- Item may be quest-related or cursed
- Check if item is locked to you

### "Trade request failed"
- Other player must be in same location
- Other player may have auto-decline enabled
- Try again or use `/WHISPER` to coordinate

### "Trade cancelled"
- Either player typed `CANCEL`
- Connection lost during trade
- Re-initiate trade if desired

## Related Commands

- `INVENTORY` - Check your money and items
- `EXAMINE` - View item details and value
- `QUEST` - Find money-earning quests
- `MAP` - Locate shops and banks

## See Also

- [Quest Guide](quests.md) - Earning through quests
- [Housing Guide](housing.md) - Spending on homes
- [Companion Guide](companions.md) - Passive income
- [Commands Reference](commands.md) - All commands

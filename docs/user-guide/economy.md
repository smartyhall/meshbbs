# Economy & Trading Guide

TinyMUSH features a dual-currency economy with gold and platinum, along with shops, banking, and player-to-player trading systems.

## Table of Contents
- [Currency System](#currency-system)
- [Earning Money](#earning-money)
- [Shops](#shops)
- [Banking](#banking)
- [Player Trading](#player-trading)
- [Economy Tips](#economy-tips)

## Currency System

### Dual Currency

**Gold (g)**
- Primary currency for everyday transactions
- Used for: shops, housing rent, item purchases
- Easier to earn through quests and activities

**Platinum (p)**
- Premium currency for rare items and services
- Used for: special shops, rare equipment, premium housing
- Harder to earn, more valuable

### Conversion

Currency conversion is not available in-game. Gold and platinum are separate economies with distinct uses.

### Checking Your Balance

```
INVENTORY     # Shows gold and platinum balance
```

**Example Output:**
```
Your Inventory:
Gold: 450g
Platinum: 12p

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
- Easy quests: 50-100g
- Medium quests: 150-300g
- Hard quests: 400-800g + platinum
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
You sold Wooden Sword for 25g.
```

**Item Values:**
- Weapons: 20-500g depending on quality
- Armor: 30-600g depending on type
- Consumables: 5-50g
- Crafting materials: 10-100g

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
- **Currency:** Gold

#### Weapon Shop
- **Location:** Market District
- **Sells:** Swords, axes, bows, ammunition
- **Buys:** Weapons and combat gear
- **Currency:** Gold, some platinum items

#### Armor Shop
- **Location:** Market District
- **Sells:** Helmets, chest armor, shields, boots
- **Buys:** Armor and protective gear
- **Currency:** Gold, some platinum items

#### Potion Shop
- **Location:** Market District
- **Sells:** Healing potions, mana potions, buff potions
- **Buys:** Herbs, potion ingredients
- **Currency:** Gold

#### Trading Post
- **Location:** Harbor
- **Sells:** Rare items, imports, special goods
- **Buys:** Almost anything
- **Currency:** Gold and platinum

#### Black Market (if discovered)
- **Location:** Hidden
- **Sells:** Rare weapons, forbidden items, powerful potions
- **Buys:** Stolen goods, rare materials
- **Currency:** Platinum only

## Banking

### Bank Locations

Banks are found in major towns:
- Town Square Bank
- Market District Credit Union
- Harbor First Bank

### Banking Commands

#### DEPOSIT - Store Money

```
DEPOSIT <amount> <currency>
```

**Examples:**
```
> DEPOSIT 500 gold
You deposited 500g. New balance: 1,200g

> DEPOSIT 10 platinum
You deposited 10p. New balance: 45p
```

**Deposit Benefits:**
- Money is safe from theft
- Earn 1% interest monthly
- Access from any bank location

#### WITHDRAW - Retrieve Money

```
WITHDRAW <amount> <currency>
```

**Examples:**
```
> WITHDRAW 300 gold
You withdrew 300g. New balance: 900g

> WITHDRAW 5 platinum
You withdrew 5p. New balance: 40p
```

#### BALANCE - Check Bank Balance

```
BALANCE
```

**Example Output:**
```
Your Bank Account:
Gold: 900g (Earning 1% interest)
Platinum: 40p (Earning 1% interest)

On hand:
Gold: 300g
Platinum: 5p

Total wealth: 1,200g, 45p
```

### Interest Rates

- **Interest Rate:** 1% per month (30 real-world days)
- **Minimum Balance:** 100g for interest
- **Compound Interest:** Yes
- **Maximum Interest:** 10g/month on gold, 1p/month on platinum

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
OFFER <item>              # Offer an item
OFFER <amount> <currency> # Offer money
```

**Examples:**
```
> OFFER Wooden Sword
You offered: Wooden Sword

> OFFER 100 gold
You offered: 100g

> OFFER 5 platinum
You offered: 5p
```

#### REMOVE - Remove Items from Trade

```
REMOVE <item>
REMOVE <amount> <currency>
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
  - 50g

Player2 offers:
  - Health Potion x5
  - 100g

Both parties ready: No
===================

[Player1] > OFFER Iron Sword
You offered: Iron Sword

[Player1] > OFFER 50 gold
You offered: 50g

[Player1] > READY
You are ready. Waiting for Player2...

[Player2] > OFFER Health Potion 5
You offered: 5x Health Potion

[Player2] > OFFER 100 gold
You offered: 100g

[Player2] > READY
You are ready.

=== Trade Complete! ===
You received:
  - Health Potion x5
  - 100g

You gave:
  - Iron Sword
  - 50g
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
1. Complete starter quests for initial gold
2. Don't spend all your money - save for housing
3. Shop around - prices vary by location
4. Sell loot instead of hoarding

**Budget Management:**
- Keep 200-300g emergency fund
- Save for rent (100-150g/month typical)
- Budget 50g/week for potions and supplies

### For Intermediate Players

**Earning Strategies:**
1. **Quest Chains** - Complete related quests for bonus rewards
2. **Shop Arbitrage** - Buy low in one town, sell high in another
3. **Crafting** - Craft items to sell for profit
4. **Treasure Hunting** - Explore for chests and rare finds

**Investment:**
- Bank excess gold for interest
- Upgrade equipment gradually
- Rent additional homes for prestige

### For Advanced Players

**Wealth Building:**
1. **Companion Training** - Passive income
2. **High-Value Trades** - Platinum trading
3. **Rare Item Flipping** - Buy rare, sell rarer
4. **Quest Mastery** - Speed-run high-reward quests

**Platinum Economy:**
- Focus on hard quests for platinum rewards
- Trade gold items for platinum items
- Save platinum for rare shop items

### Market Strategies

**Buy Low, Sell High:**
- General Store buys most items at 50% value
- Trading Post pays 75% value for rare items
- Timing matters - some shops restock weekly

**Item Values:**
| Item Type | Shop Buy | Shop Sell | Player Trade |
|-----------|----------|-----------|--------------|
| Basic Weapon | 50g | 25g | 30-40g |
| Rare Weapon | 500g | 250g | 400-600g |
| Healing Potion | 15g | 7g | 10-12g |
| Crafting Material | 50g | 25g | 40-80g |

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

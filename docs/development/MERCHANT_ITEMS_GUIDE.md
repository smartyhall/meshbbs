# Merchant Items and Dialog-Based Commerce

## Overview

The merchant system uses **data-driven items** combined with **dialog-based purchasing**. World builders can modify items, prices, and dialog flow without changing code.

## How It Works

### 1. Items are ObjectRecords

Merchant items are created in `src/tmush/state.rs` in the `create_example_trigger_objects()` function as standard ObjectRecords:

```rust
let basic_knife = ObjectRecord {
    id: "vendor_basic_knife".to_string(),
    name: "Basic Knife".to_string(),
    description: "A simple but sturdy knife...",
    currency_value: CurrencyAmount::decimal(15),  // Price!
    value: 15,
    // ... other fields
};
```

**Key Points:**
- `id` is used in DialogAction::GiveItem
- `currency_value` determines the price
- `name` and `description` are what players see
- All fields can be modified by builders using `@OBJECT EDIT` command

### 2. Dialog Actions Handle Transactions

The NPC dialog tree uses DialogActions to execute the purchase:

```rust
tree.insert("purchase_knife".to_string(), DialogNode::new(
    "Excellent choice! Here's your knife..."
)
.with_action(DialogAction::TakeCurrency { amount: 15 })
.with_action(DialogAction::GiveItem { 
    item_id: "vendor_basic_knife".to_string(), 
    quantity: 1 
})
```

**Actions Execute in Order:**
1. `TakeCurrency` - Deducts credits from player
2. `GiveItem` - Adds item to player inventory

### 3. Conditions Control Visibility

DialogConditions show/hide purchase options based on player's currency:

```rust
.with_choice(DialogChoice::new("I'll buy the knife (15 credits)")
    .goto("purchase_knife")
    .with_condition(DialogCondition::HasCurrency { amount: 15 }))
```

**Behavior:**
- Choice only appears if player has ≥15 credits
- Prevents "purchase failed" scenarios
- Gracefully handles poverty

## Current Merchant Items

### Mira's Market Stall (market_vendor NPC)

| Item ID | Name | Price | Description |
|---------|------|-------|-------------|
| `vendor_basic_knife` | Basic Knife | 15 credits | Utility tool for rope/tasks |
| `vendor_signal_booster` | Signal Booster | 50 credits | +50% mesh range for wilderness |

## Adding New Merchant Items

### Step 1: Create the Item Object

Add to `create_example_trigger_objects()` in `src/tmush/state.rs`:

```rust
let rope_bundle = ObjectRecord {
    id: "vendor_rope_bundle".to_string(),
    name: "Bundle of Rope".to_string(),
    description: "50 feet of strong hemp rope, coiled neatly.".to_string(),
    owner: ObjectOwner::World,
    created_at: now,
    weight: 5,
    currency_value: CurrencyAmount::decimal(10),  // 10 credits
    value: 10,
    takeable: true,
    usable: false,
    actions: std::collections::HashMap::new(),
    flags: vec![],
    locked: false,
    clone_depth: 0,
    clone_source_id: None,
    clone_count: 0,
    created_by: "world".to_string(),
    ownership_history: vec![],
    schema_version: OBJECT_SCHEMA_VERSION,
};
objects.push(rope_bundle);
```

### Step 2: Add to Merchant's Prices List

Update the `prices` dialog node:

```rust
tree.insert("prices".to_string(), DialogNode::new(
    "I've got a few items on hand:\n\n\
    • Basic Knife - 15 credits\n\
    • Signal Booster - 50 credits\n\
    • Rope Bundle - 10 credits (NEW!)\n\n\
    What would you like?"
)
.with_choice(DialogChoice::new("I'll buy the rope (10 credits)")
    .goto("purchase_rope")
    .with_condition(DialogCondition::HasCurrency { amount: 10 }))
// ... other choices
```

### Step 3: Create Purchase Dialog Node

```rust
tree.insert("purchase_rope".to_string(), DialogNode::new(
    "Great choice! This rope is strong and reliable. 10 credits, please!"
)
.with_action(DialogAction::TakeCurrency { amount: 10 })
.with_action(DialogAction::GiveItem { 
    item_id: "vendor_rope_bundle".to_string(), 
    quantity: 1 
})
.with_choice(DialogChoice::new("Thanks!").exit())
.with_choice(DialogChoice::new("What else do you have?").goto("prices")));
```

## Modifying Existing Items (In-Game)

World builders can modify merchant items using admin commands:

```
@OBJECT EDIT vendor_basic_knife
> name "Sharp Utility Knife"
> description "A professionally sharpened knife with a leather-wrapped handle."
> currency_value 20
> value 20
> save
```

**Important:** If you change the price via `@OBJECT EDIT`, also update the dialog:

```
@DIALOG EDIT museum_curator prices
> (Edit the text to show new price)
> save

@DIALOG EDIT museum_curator purchase_knife
> (Update the TakeCurrency action amount)
> save
```

## Modifying Dialog Trees (In-Game)

Builders can edit dialog without code changes:

```
@DIALOG EDIT market_vendor prices
> text "I've got great deals today! ..."
> add_choice "I'll buy the rare crystal (100 credits)" purchase_crystal
> save

@DIALOG CREATE market_vendor purchase_crystal
> text "Wow, you have excellent taste! Here's your rare crystal."
> add_action take_currency 100
> add_action give_item vendor_rare_crystal 1
> add_choice "Amazing!" exit
> save
```

## Best Practices

### 1. Consistent Naming
- Item IDs: `vendor_{item_name}` (e.g., `vendor_basic_knife`)
- Dialog nodes: `purchase_{item}` (e.g., `purchase_knife`)

### 2. Price in Two Places
- `ObjectRecord.currency_value` - The actual price stored in item
- `DialogAction::TakeCurrency { amount }` - What the dialog charges
- **Keep these synchronized!**

### 3. Condition Matches Price
```rust
.with_condition(DialogCondition::HasCurrency { amount: 15 })  // Must match
.with_action(DialogAction::TakeCurrency { amount: 15 })       // Must match
```

### 4. Use Descriptive Dialog Text
Good:
```
"The knife is 15 credits. It's sharp and reliable!"
```

Bad:
```
"That'll be X credits." // No price shown
```

### 5. Provide Exit Options
Always give players a way out:
```rust
.with_choice(DialogChoice::new("Just looking for now").exit())
.with_choice(DialogChoice::new("Maybe later").exit())
```

## Testing

### Test Purchase Flow
1. Create test player with enough credits
2. Talk to vendor: `talk market_vendor`
3. Navigate to prices: Choose "I'd like to buy something" → "What's available now?"
4. Verify choices appear if you have enough credits
5. Purchase item
6. Check inventory: `inventory`
7. Verify credits deducted: `credits`

### Test Insufficient Funds
1. Create test player with low credits (e.g., 5)
2. Talk to vendor
3. Navigate to prices
4. Verify expensive options are hidden (e.g., signal booster)
5. Verify "I need to earn more credits" option appears

### Test Item Properties
```
examine vendor_basic_knife
look vendor_signal_booster
```

## Future Enhancements

### Dynamic Pricing
Could add DialogCondition::HasFlag to show different prices for VIP players:

```rust
.with_choice(DialogChoice::new("I'll buy (VIP discount: 10 credits)")
    .goto("purchase_knife_vip")
    .with_condition(DialogCondition::HasFlag { 
        flag: "vip_member".to_string(), 
        value: true 
    }))
```

### Bulk Purchases
```rust
.with_action(DialogAction::TakeCurrency { amount: 40 })
.with_action(DialogAction::GiveItem { 
    item_id: "vendor_basic_knife".to_string(), 
    quantity: 3  // Buy 3 for price of 2.67 each
})
```

### Quest Requirements
```rust
.with_condition(DialogCondition::QuestStatus { 
    quest_id: "merchant_guild".to_string(), 
    status: "completed".to_string() 
})
```

## Summary

✅ **Items are data** - ObjectRecords modifiable with @OBJECT EDIT
✅ **Prices are data** - currency_value field sets the cost
✅ **Dialog is data** - Editable with @DIALOG EDIT commands
✅ **Conditions control access** - HasCurrency prevents broke purchases
✅ **Actions execute transactions** - TakeCurrency + GiveItem = sale
✅ **World builders have full control** - No code changes needed

This system gives maximum flexibility while maintaining clean separation between data (items) and behavior (dialog actions).

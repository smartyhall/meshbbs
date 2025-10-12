# TinyMUSH Currency System Design

**Decision Date:** 2025-10-06  
**Status:** Approved - Ready for Phase 5 Implementation

## Executive Summary

TinyMUSH will support **two distinct currency systems** to accommodate different world themes and builder preferences. World builders choose **one system per world** via configuration, with full conversion capability if they wish to migrate between systems.

## Currency Systems

### 1. Decimal Currency (Modern/Sci-Fi)

**Use Cases:** Contemporary, modern, sci-fi, space station, cyberpunk settings

**Features:**
- Single currency with decimal places (like real-world money)
- Fully configurable name, symbol, and precision
- Integer-only storage (no floating-point issues)

**Examples:**
- Credits: `¤1,234.56`
- MiniBucks: `$10.50`
- Euros: `€5,000.00`
- Galactic Credits: `₡999.99`

**Configuration:**
```toml
[tinymush.currency]
system = "decimal"

[tinymush.currency.decimal]
name = "Credit"
name_plural = "Credits"
symbol = "¤"
minor_units_per_major = 100  # Like cents in a dollar
decimal_places = 2
```

**Storage:** Integer minor units (e.g., 1,234.56 stored as 123,456 cents)

---

### 2. Multi-Tier Currency (Fantasy/Medieval)

**Use Cases:** Fantasy, medieval, traditional RPG, D&D-style settings

**Features:**
- Multiple denominations with conversion ratios
- Fully configurable tier names and symbols
- Classic fantasy feel with multiple coin types

**Examples:**
- Standard: `15gp 25sp 30cp`
- Custom: `5 platinum 12 gold 8 silver`
- Abbreviated: `2pp 50gp`

**Configuration:**
```toml
[tinymush.currency]
system = "multi_tier"

[tinymush.currency.multi_tier]
platinum_name = "platinum"
platinum_symbol = "pp"
gold_name = "gold"
gold_symbol = "gp"
silver_name = "silver"
silver_symbol = "sp"
copper_name = "copper"
copper_symbol = "cp"

# Conversion ratios (from copper - base unit)
platinum_ratio = 1000000  # 1pp = 1,000,000cp
gold_ratio = 10000        # 1gp = 10,000cp
silver_ratio = 100        # 1sp = 100cp
copper_ratio = 1          # 1cp = 1cp
```

**Storage:** Integer copper units (base denomination)

---

## Technical Architecture

### Storage Strategy

Both systems use **integer-only arithmetic** to avoid floating-point precision issues:

**Decimal Currency:**
```rust
pub struct DecimalAmount {
    minor_units: i64,  // Cents, centimes, etc.
}
```

**Multi-Tier Currency:**
```rust
pub struct MultiTierAmount {
    copper_value: i64,  // Base copper units
}
```

### Unified Interface

Both systems implement a common interface for transactions:

```rust
pub enum CurrencyAmount {
    Decimal(DecimalAmount),
    MultiTier(MultiTierAmount),
}

impl CurrencyAmount {
    fn base_value(&self) -> i64;
    fn add(&self, other: &Self) -> Result<Self>;
    fn subtract(&self, other: &Self) -> Result<Self>;
    fn can_afford(&self, cost: &Self) -> bool;
}
```

This allows the transaction engine to work seamlessly with either system.

---

## Currency Conversion

### Conversion Ratios

**Standard conversion:** 100 copper = 1 major decimal unit

This provides a clean mapping:
- Decimal: 100 cents = $1.00
- Multi-tier: 100 copper = base unit
- Ratio: 1:1 at the base storage level

### Conversion Examples

**Multi-Tier → Decimal:**
```
15gp 25sp 30cp = 152,530 copper
             → $1,525.30 (152,530 cents)

2pp 50gp 75sp = 2,507,500 copper
             → €25,075.00
```

**Decimal → Multi-Tier:**
```
$10.50 (1,050 cents) = 1,050 copper
                     → 10gp 50cp

€1,234.56 (123,456 cents) = 123,456 copper
                          → 12gp 34sp 56cp
```

### World Migration

World builders can convert an existing world from one system to another using admin commands:

```
ADMIN CURRENCY CONVERT [decimal|multi_tier]

This batch converts:
- All player wallets
- All item prices
- All shop inventories
- All bank accounts
- All transaction logs
```

The conversion is atomic and logged for audit purposes.

---

## Design Rationale

### Why Two Systems?

1. **Theme Appropriateness:** Decimal suits modern/sci-fi, multi-tier suits fantasy
2. **Player Expectations:** Players expect familiar currency in their chosen setting
3. **Immersion:** Currency system reinforces world theme
4. **Flexibility:** Builders choose what fits their narrative

### Why Not a Hybrid?

- Simplicity: One system per world is easier to understand and use
- Consistency: Players don't have to learn multiple systems in one world
- Performance: Simpler transaction logic
- UI/UX: Cleaner display without system mixing

### Why Integer-Only Storage?

- **Precision:** Avoids floating-point rounding errors
- **Reliability:** Financial calculations must be exact
- **Performance:** Integer math is faster
- **Storage:** Smaller footprint in database

---

## Implementation Checklist

See `TODO.md` Phase 5 for detailed implementation tasks.

**Key Milestones:**
1. ✅ Design approved (this document)
2. ⏳ Data structures defined
3. ⏳ Storage layer implemented
4. ⏳ Transaction engine built
5. ⏳ Display formatting complete
6. ⏳ Conversion tools working
7. ⏳ Testing comprehensive
8. ⏳ Documentation complete

---

## Examples in Practice

### Decimal Currency World (Sci-Fi Station)

```
=== WALLET ===
Balance: ¤1,234.56
(Credits)

→ BUY oxygen_tank
*Purchased oxygen tank*
Cost: ¤250.00
Balance: ¤1,234.56 → ¤984.56

=== SHOP: Station Supplies ===
1. Oxygen Tank    ¤250.00
2. Med Kit        ¤150.00
3. Space Suit     ¤1,200.00
4. Rations        ¤45.75

[B]uy [S]ell [E]xit
```

### Multi-Tier Currency World (Fantasy Kingdom)

```
=== WALLET ===
💎 2pp 🟡 47gp ⚪ 93sp 🟤 15cp

Total: 2,479,315 copper

→ BUY iron_sword
*Purchased iron sword*
Cost: 15gp
Balance: 47gp 93sp 15cp
       → 32gp 93sp 15cp

=== SHOP: Blacksmith ===
1. Dagger      5gp
2. Sword       15gp
3. Axe         20gp
4. Plate Mail  150gp

[B]uy [S]ell [E]xit
```

---

## Future Enhancements (Post-Phase 5)

Potential additions for later phases:

- **Exchange Rates:** Between worlds (if multi-world support added)
- **Tax System:** Configurable sales tax or tariffs
- **Currency Rarity:** Special coins with collector value
- **Counterfeiting:** Risk of fake currency in player trading
- **Regional Variants:** Different symbols/names per region

These are **not** in scope for Phase 5 but could be added later.

---

## References

- Design Spec: `docs/development/MUD_MUSH_DESIGN.md` § Enhanced Economy System
- Implementation Plan: `docs/development/TINYMUSH_IMPLEMENTATION_PLAN.md` § Phase 5
- TODO Tracking: `TODO.md` § Phase 5 — Economy, Inventory, Shops

## Approval

- [x] Design reviewed and approved
- [x] Documentation updated
- [x] Implementation plan enhanced
- [x] TODO checklist expanded
- [ ] Implementation started (next step)

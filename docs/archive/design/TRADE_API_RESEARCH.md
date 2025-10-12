# P2P Trading System - API Research Summary

**Date**: 2025-10-06  
**Purpose**: Document correct API usage before implementing trade commands

---

## ✅ Key Findings

### 1. **Return Types**
- All command handlers return: `Result<String>` (from `anyhow::Result`)
- TinyMushError converts to anyhow::Error automatically via `?` operator
- Use `Ok(String)` for success, `Err(TinyMushError::variant)` for errors

### 2. **Storage Methods**

#### `get_player(username: &str)`
```rust
pub fn get_player(&self, username: &str) -> Result<PlayerRecord, TinyMushError>
```
- Returns `Result<PlayerRecord>` - NOT `Option`!
- Use: `self.store().get_player(&username)?` - no `.ok_or()` needed
- Returns `TinyMushError::NotFound` if player doesn't exist

#### `get_player_transactions(username: &str, limit: usize)`
```rust
pub fn get_player_transactions(
    &self,
    username: &str,
    limit: usize,
) -> Result<Vec<CurrencyTransaction>, TinyMushError>
```
- Takes **2 parameters**: username AND limit
- Use: `self.store().get_player_transactions(&username, 10)?`

#### `log_transaction(transaction: &CurrencyTransaction)`
```rust
fn log_transaction(&self, transaction: &CurrencyTransaction) -> Result<(), TinyMushError>
```
- **PRIVATE METHOD** - cannot call directly!
- Alternative: Use `transfer_currency()` which logs automatically

#### `transfer_currency(from, to, amount, reason)`
```rust
pub fn transfer_currency(
    &self,
    from_username: &str,
    to_username: &str,
    amount: &CurrencyAmount,
    reason: TransactionReason,
) -> Result<CurrencyTransaction, TinyMushError>
```
- **Public method** that handles atomic transfer + logging
- Returns the transaction record
- Use this instead of manual log_transaction

### 3. **CurrencyTransaction Fields**
```rust
pub struct CurrencyTransaction {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub from: Option<String>,        // ✅ from, NOT from_player
    pub to: Option<String>,          // ✅ to, NOT to_player
    pub amount: CurrencyAmount,
    pub reason: TransactionReason,
    pub rolled_back: bool,
}
```

### 4. **Error Types**
```rust
#[derive(Debug, Error)]
pub enum TinyMushError {
    InsufficientFunds,              // ✅ Unit variant, no message!
    NotFound(String),               // Takes message
    InvalidCurrency(String),        // Takes message
    TransactionNotFound,            // Unit variant
    // ...
}
```

### 5. **Store Access Pattern**
```rust
// In TinyMushProcessor methods:
self.store()  // Returns &TinyMushStore - the accessor method
```

---

## 📋 Implementation Checklist

### For TRADE command:
- [x] Use `self.store().get_player(&username)?` - returns PlayerRecord or error
- [x] Use `self.store().get_player_active_trade(&username)?` - returns Option<TradeSession>
- [x] Use `self.store().put_trade_session(&session)?`

### For OFFER command:
- [x] Parse amount as i64 (base units)
- [x] Create CurrencyAmount matching player's currency type
- [x] Validate with `player.currency.can_afford(&amount)`

### For ACCEPT/Execute Trade:
- [x] Use `self.store().put_player(&player)?` to save players
- [x] Create CurrencyTransaction manually with proper field names:
  ```rust
  CurrencyTransaction {
      id: format!("trade_{}_{}", timestamp, uuid),
      timestamp: Utc::now(),
      from: Some(from_username.to_string()),  // ✅ from, not from_player
      to: Some(to_username.to_string()),      // ✅ to, not to_player
      amount: amount.clone(),
      reason: TransactionReason::Trade,
      rolled_back: false,
  }
  ```
- [x] Cannot call private `log_transaction` - need alternative approach
- [x] Option 1: Use `transfer_currency()` for currency swaps (handles logging)
- [x] Option 2: Manually update players and skip transaction logging for trades
- [ ] **DECISION NEEDED**: Which approach for Execute Trade?

### For THISTORY command:
- [x] Use `self.store().get_player_transactions(&username, 10)?`
- [x] Filter: `txns.iter().filter(|tx| matches!(tx.reason, TransactionReason::Trade))`
- [x] Access fields: `tx.from`, `tx.to`, `tx.amount`, `tx.timestamp`

### For REJECT command:
- [x] Simple: get session, delete it, return message

---

## 🎯 Recommended Implementation Order

1. **REJECT** - Simplest, just delete session (5 min)
2. **TRADE** - Initiate session with validation (10 min)
3. **THISTORY** - Read-only history display (10 min)
4. **OFFER** - Add to session (15 min)
5. **Execute Trade Helper** - Complex atomic operations (30 min)
   - **DECISION**: Use manual updates (no transaction logging) for simplicity
6. **ACCEPT** - Calls Execute Trade (5 min)

---

## ⚠️ Critical Notes

- `InsufficientFunds` is **unit variant**: `Err(TinyMushError::InsufficientFunds)`
- NO `.ok_or()` after `get_player()` - it already returns Result<PlayerRecord>
- Transaction logging is private - either use `transfer_currency()` or skip logging for P2P trades
- All handlers return `Result<String>` (anyhow), not `Result<String, TinyMushError>`
- Store access via `self.store()` method, not `self.store` field

---

## 📝 Next Steps

1. Implement REJECT command (test immediately)
2. Implement TRADE command (test immediately)
3. Implement THISTORY command (test immediately)
4. Implement OFFER command (test immediately)
5. Implement Execute Trade helper (test thoroughly)
6. Implement ACCEPT command (test immediately)
7. Update TODO.md to mark Phase 5 Week 5 complete
8. Commit with message: "feat(tmush): complete Phase 5 Week 5 P2P trading system"

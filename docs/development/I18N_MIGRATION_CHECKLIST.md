# WorldConfig Migration Checklist

**Status**: Struct Complete (113 fields) - Command Handler Migration in Progress  
**Date**: 2025-10-09

## Migration Progress

- **Total String Fields**: 113
- **Commands Migrated**: 5 (handle_help, handle_say, handle_emote, handle_whisper, handle_deposit, handle_trade)
- **Commands Remaining**: ~40+

## Command Handler Migration Status

### ✅ Completed (6 handlers)
- [x] `handle_help()` - Uses help_* templates
- [x] `handle_say()` - Uses err_say_what
- [x] `handle_emote()` - Uses err_emote_what  
- [x] `handle_whisper()` - Uses err_whisper_self
- [x] `handle_deposit()` - Uses msg_deposit_success
- [x] `handle_trade()` - Uses msg_trade_initiated

### ⏳ High Priority (Validation & Input - 7 handlers)
- [ ] `handle_whisper()` - Add err_whisper_what, err_whisper_whom
- [ ] `handle_pose()` - Use err_pose_what
- [ ] `handle_ooc()` - Use err_ooc_what
- [ ] `handle_deposit()` - Use err_amount_positive, err_invalid_amount_format
- [ ] `handle_withdraw()` - Use err_amount_positive, err_invalid_amount_format
- [ ] `handle_btransfer()` - Use err_transfer_self, err_amount_positive, err_invalid_amount_format

### ⏳ High Priority (Empty States - 10 handlers)
- [ ] `handle_inventory()` - Use msg_empty_inventory
- [ ] `handle_list()` - Use msg_no_shops_available
- [ ] `handle_sell()` - Use msg_no_shops_sell_to
- [ ] `handle_companion()` - Use msg_no_companions, msg_no_companions_tame_hint, msg_no_companions_follow
- [ ] `handle_quest()` - Use msg_no_active_quests
- [ ] `handle_achievement()` - Use msg_no_achievements, msg_no_achievements_earned
- [ ] `handle_title()` - Use msg_no_titles_unlocked, msg_no_title_equipped
- [ ] `handle_trade()` - Use msg_no_active_trade, msg_no_active_trade_hint
- [ ] `handle_trade_history()` - Use msg_no_trade_history
- [ ] `handle_who()` - Use msg_no_players_found

### ⏳ Medium Priority (Shop Operations - 6 handlers)
- [ ] `handle_buy()` - Use err_shop_no_sell, err_shop_doesnt_sell, err_shop_insufficient_funds
- [ ] `handle_sell()` - Use err_shop_no_buy, err_shop_wont_buy_price, err_item_not_owned, err_only_have_quantity

### ⏳ Medium Priority (Trading - 4 handlers)
- [ ] `handle_trade()` - Use err_trade_already_active, err_trade_partner_busy, err_trade_player_not_here
- [ ] `handle_offer()` - Use err_trade_insufficient_amount, msg_no_active_trade_hint
- [ ] `handle_accept()` - Use msg_trade_accepted_waiting, msg_no_active_trade
- [ ] `handle_reject()` - Use msg_no_active_trade

### ⏳ Medium Priority (Quest System - 3 handlers)
- [ ] `handle_quest()` - Use err_quest_cannot_accept, err_quest_not_found, msg_quest_abandoned

### ⏳ Medium Priority (Achievement & Title - 3 handlers)
- [ ] `handle_achievement()` - Use err_achievement_unknown_category, msg_no_achievements_category
- [ ] `handle_title()` - Use err_title_not_unlocked, msg_title_equipped, msg_title_equipped_display, err_title_usage

### ⏳ Medium Priority (Companion System - 3 handlers)
- [ ] `handle_companion()` - Use msg_companion_tamed, err_companion_owned, err_companion_not_found, msg_companion_released

### ⏳ Low Priority (Movement - 1 handler)
- [ ] `handle_move()` - Use err_movement_restricted, err_player_not_here

### ⏳ Low Priority (Bulletin Board - 3 handlers)
- [ ] `handle_board()` - Use err_board_location_required
- [ ] `handle_post()` - Use err_board_post_location
- [ ] `handle_read()` - Use err_board_read_location

### ⏳ Low Priority (NPC & Tutorial - 2 handlers)
- [ ] `handle_talk()` - Use err_no_npc_here, msg_tutorial_completed, msg_tutorial_not_started

### ⏳ Low Priority (Technical Errors - All handlers)
Replace format!("Error loading player: {}", e) patterns with:
- [ ] Use err_player_load_failed throughout
- [ ] Use err_shop_save_failed in buy/sell handlers
- [ ] Use err_player_save_failed in buy/sell handlers
- [ ] Use err_payment_failed in buy/sell handlers
- [ ] Use err_purchase_failed in buy handler
- [ ] Use err_sale_failed in sell handler
- [ ] Use err_tutorial_error in tutorial handler
- [ ] Use err_reward_error in tutorial handler
- [ ] Use err_quest_failed in quest handler
- [ ] Use err_shop_find_failed in shop handlers
- [ ] Use err_player_list_failed in who handler
- [ ] Use err_movement_failed in move handler
- [ ] Use err_movement_save_failed in move handler

## Migration Pattern

For each handler:

```rust
async fn handle_command(&mut self, session: &Session, args: String, config: &Config) -> Result<String> {
    // 1. Load world config at start
    let world_config = self.get_world_config().await?;
    
    // 2. Replace hardcoded strings with config fields
    if input.is_empty() {
        return Ok(world_config.err_input_required);  // Instead of: "Input required"
    }
    
    // 3. Use .replace() for template variables
    Ok(world_config.msg_success.replace("{player}", &player_name))
}
```

## Template Variables Reference

Common template variables used across fields:

- `{player}` - Player name
- `{target}` - Target player name
- `{item}` - Item name
- `{quantity}` - Quantity/count
- `{amount}` - Currency amount
- `{price}` - Item price
- `{error}` - Error message
- `{quest}` - Quest name/ID
- `{title}` - Title name
- `{name}` - Generic name (companion, NPC)
- `{loyalty}` - Companion loyalty level
- `{category}` - Achievement category
- `{direction}` - Movement direction
- `{display}` - Display name with title

## Testing Strategy

After each batch of migrations:

1. Run `cargo test --lib` - library unit tests
2. Run `cargo test --tests` - integration tests
3. Manual testing of affected commands
4. Verify template variable substitution works

## Next Actions

1. **Batch 1** (Validation & Input): Migrate 7 handlers for input validation errors
2. **Batch 2** (Empty States): Migrate 10 handlers for empty state messages
3. **Batch 3** (Shop & Trading): Migrate 10 handlers for shop/trading operations
4. **Batch 4** (Game Systems): Migrate 9 handlers for quest/achievement/companion systems
5. **Batch 5** (Technical): Update all error handling to use config fields

Estimated completion: 5 batches × 30 min = 2.5 hours of focused migration work.

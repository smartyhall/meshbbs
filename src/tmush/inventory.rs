/// Inventory management system for TinyMUSH
use super::types::{InventoryConfig, InventoryResult, ItemStack, ObjectRecord, PlayerRecord};

// ============================================================================
// Inventory Operations
// ============================================================================

/// Calculate total weight of all items in inventory
pub fn calculate_total_weight(
    stacks: &[ItemStack],
    get_item: impl Fn(&str) -> Option<ObjectRecord>,
) -> u32 {
    stacks
        .iter()
        .map(|stack| {
            if let Some(item) = get_item(&stack.object_id) {
                stack.total_weight(item.weight)
            } else {
                0
            }
        })
        .sum()
}

/// Check if player can add an item to inventory
pub fn can_add_item(
    player: &PlayerRecord,
    item: &ObjectRecord,
    quantity: u32,
    config: &InventoryConfig,
    get_item: impl Fn(&str) -> Option<ObjectRecord>,
) -> Result<(), String> {
    // Check if item is takeable
    if !item.takeable {
        return Err(format!("The {} cannot be taken.", item.name));
    }

    // Calculate current and new weight
    let current_weight = calculate_total_weight(&player.inventory_stacks, get_item);
    let additional_weight = item.weight as u32 * quantity;
    let new_weight = current_weight + additional_weight;

    if new_weight > config.max_weight {
        return Err(format!(
            "Too heavy! You can only carry {} more weight units.",
            config.max_weight.saturating_sub(current_weight)
        ));
    }

    // Check if we're adding a new stack (not stacking with existing)
    let existing_stack = player
        .inventory_stacks
        .iter()
        .find(|s| s.object_id == item.id);

    if existing_stack.is_none() && player.inventory_stacks.len() >= config.max_stacks as usize {
        return Err(format!(
            "Your inventory is full! Maximum {} item types.",
            config.max_stacks
        ));
    }

    Ok(())
}

/// Add an item to player's inventory
pub fn add_item_to_inventory(
    player: &mut PlayerRecord,
    item: &ObjectRecord,
    quantity: u32,
    config: &InventoryConfig,
) -> InventoryResult {
    if quantity == 0 {
        return InventoryResult::Failed {
            reason: "Cannot add zero items".to_string(),
        };
    }

    // Check if we can stack with existing item
    if config.allow_stacking {
        if let Some(stack) = player
            .inventory_stacks
            .iter_mut()
            .find(|s| s.object_id == item.id)
        {
            stack.quantity += quantity;
            return InventoryResult::Added {
                quantity,
                stacked: true,
            };
        }
    }

    // Create new stack
    player
        .inventory_stacks
        .push(ItemStack::new(item.id.clone(), quantity));

    InventoryResult::Added {
        quantity,
        stacked: false,
    }
}

/// Remove an item from player's inventory
pub fn remove_item_from_inventory(
    player: &mut PlayerRecord,
    object_id: &str,
    quantity: u32,
) -> InventoryResult {
    if quantity == 0 {
        return InventoryResult::Failed {
            reason: "Cannot remove zero items".to_string(),
        };
    }

    // Find the stack
    let stack_index = player
        .inventory_stacks
        .iter()
        .position(|s| s.object_id == object_id);

    if let Some(index) = stack_index {
        let stack = &mut player.inventory_stacks[index];

        if quantity >= stack.quantity {
            // Remove entire stack
            let removed_quantity = stack.quantity;
            player.inventory_stacks.remove(index);
            InventoryResult::Removed {
                quantity: removed_quantity,
            }
        } else {
            // Reduce stack quantity
            stack.quantity -= quantity;
            InventoryResult::Removed { quantity }
        }
    } else {
        InventoryResult::Failed {
            reason: "Item not in inventory".to_string(),
        }
    }
}

/// Check if player has at least a certain quantity of an item
pub fn has_item(player: &PlayerRecord, object_id: &str, quantity: u32) -> bool {
    player
        .inventory_stacks
        .iter()
        .find(|s| s.object_id == object_id)
        .map(|s| s.quantity >= quantity)
        .unwrap_or(false)
}

/// Get the quantity of an item in inventory
pub fn get_item_quantity(player: &PlayerRecord, object_id: &str) -> u32 {
    player
        .inventory_stacks
        .iter()
        .find(|s| s.object_id == object_id)
        .map(|s| s.quantity)
        .unwrap_or(0)
}

/// Format inventory for display (compact for Meshtastic 200-byte limit)
pub fn format_inventory_compact(
    player: &PlayerRecord,
    get_item: impl Fn(&str) -> Option<ObjectRecord>,
) -> Vec<String> {
    if player.inventory_stacks.is_empty() {
        return vec!["Empty".to_string()];
    }

    let mut lines = Vec::new();
    let total_weight = calculate_total_weight(&player.inventory_stacks, &get_item);

    for (idx, stack) in player.inventory_stacks.iter().enumerate() {
        if let Some(item) = get_item(&stack.object_id) {
            let qty_str = if stack.quantity > 1 {
                format!("{}x ", stack.quantity)
            } else {
                String::new()
            };
            let weight_str = if item.weight > 0 {
                format!(" ({}w)", stack.total_weight(item.weight))
            } else {
                String::new()
            };
            let lock_str = if item.locked {
                " 🔒"
            } else {
                ""
            };
            lines.push(format!("{}. {}{}{}{}", idx + 1, qty_str, item.name, weight_str, lock_str));
        }
    }

    lines.push(format!("Total: {} items, {}w", player.inventory_stacks.len(), total_weight));
    lines
}

/// Format detailed item examination
pub fn format_item_examination(item: &ObjectRecord, quantity: u32) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("Name: {}", item.name));
    
    if quantity > 1 {
        lines.push(format!("Quantity: {}", quantity));
    }
    
    lines.push(format!("Description: {}", item.description));
    
    if item.weight > 0 {
        lines.push(format!("Weight: {} units", item.weight));
    }
    
    // Show value if non-zero
    if !item.currency_value.is_zero_or_negative() {
        lines.push(format!("Value: {:?}", item.currency_value));
    }
    
    if item.takeable {
        lines.push("Can be taken".to_string());
    }
    
    if item.usable {
        lines.push("Can be used".to_string());
    }

    lines
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::types::{CurrencyAmount, ObjectOwner};
    use chrono::Utc;

    fn test_player() -> PlayerRecord {
        PlayerRecord::new("alice", "Alice", "room1")
    }

    fn test_item(id: &str, name: &str, weight: u8, takeable: bool) -> ObjectRecord {
        ObjectRecord {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("A {}", name),
            owner: ObjectOwner::World,
            created_at: Utc::now(),
            weight,
            currency_value: CurrencyAmount::default(),
            value: 0,
            takeable,
            usable: false,
            actions: std::collections::HashMap::new(),
            flags: Vec::new(),
            locked: false, // Test items unlocked
            ownership_history: vec![], // Test items have no history
            schema_version: 1,
        }
    }

    #[test]
    fn test_add_item_creates_new_stack() {
        let mut player = test_player();
        let item = test_item("sword1", "Iron Sword", 10, true);
        let config = InventoryConfig::default();

        let result = add_item_to_inventory(&mut player, &item, 1, &config);
        assert_eq!(
            result,
            InventoryResult::Added {
                quantity: 1,
                stacked: false
            }
        );
        assert_eq!(player.inventory_stacks.len(), 1);
        assert_eq!(player.inventory_stacks[0].object_id, "sword1");
        assert_eq!(player.inventory_stacks[0].quantity, 1);
    }

    #[test]
    fn test_add_item_stacks() {
        let mut player = test_player();
        let item = test_item("potion1", "Health Potion", 1, true);
        let config = InventoryConfig::default();

        // Add first batch
        add_item_to_inventory(&mut player, &item, 3, &config);
        assert_eq!(player.inventory_stacks.len(), 1);
        assert_eq!(player.inventory_stacks[0].quantity, 3);

        // Add second batch (should stack)
        let result = add_item_to_inventory(&mut player, &item, 2, &config);
        assert_eq!(
            result,
            InventoryResult::Added {
                quantity: 2,
                stacked: true
            }
        );
        assert_eq!(player.inventory_stacks.len(), 1);
        assert_eq!(player.inventory_stacks[0].quantity, 5);
    }

    #[test]
    fn test_remove_partial_stack() {
        let mut player = test_player();
        let item = test_item("arrow1", "Arrow", 1, true);
        let config = InventoryConfig::default();

        add_item_to_inventory(&mut player, &item, 10, &config);
        let result = remove_item_from_inventory(&mut player, "arrow1", 3);

        assert_eq!(result, InventoryResult::Removed { quantity: 3 });
        assert_eq!(player.inventory_stacks[0].quantity, 7);
    }

    #[test]
    fn test_remove_entire_stack() {
        let mut player = test_player();
        let item = test_item("gem1", "Ruby", 1, true);
        let config = InventoryConfig::default();

        add_item_to_inventory(&mut player, &item, 5, &config);
        let result = remove_item_from_inventory(&mut player, "gem1", 10);

        assert_eq!(result, InventoryResult::Removed { quantity: 5 });
        assert_eq!(player.inventory_stacks.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_item() {
        let mut player = test_player();
        let result = remove_item_from_inventory(&mut player, "nonexistent", 1);

        match result {
            InventoryResult::Failed { reason } => {
                assert_eq!(reason, "Item not in inventory");
            }
            _ => panic!("Expected Failed result"),
        }
    }

    #[test]
    fn test_weight_calculation() {
        let mut player = test_player();
        let heavy = test_item("anvil", "Anvil", 100, true);
        let light = test_item("feather", "Feather", 1, true);
        let config = InventoryConfig::default();

        add_item_to_inventory(&mut player, &heavy, 2, &config);
        add_item_to_inventory(&mut player, &light, 10, &config);

        let get_item = |id: &str| -> Option<ObjectRecord> {
            match id {
                "anvil" => Some(heavy.clone()),
                "feather" => Some(light.clone()),
                _ => None,
            }
        };

        let total = calculate_total_weight(&player.inventory_stacks, get_item);
        assert_eq!(total, 210); // 2*100 + 10*1
    }

    #[test]
    fn test_can_add_item_weight_limit() {
        let player = test_player();
        let heavy = test_item("boulder", "Boulder", 250, true);
        let config = InventoryConfig {
            max_weight: 1000,
            ..Default::default()
        };

        let get_item = |_: &str| -> Option<ObjectRecord> { None };

        // Adding 5 boulders (1250 weight) should fail
        let result = can_add_item(&player, &heavy, 5, &config, get_item);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Too heavy"));
    }

    #[test]
    fn test_can_add_item_not_takeable() {
        let player = test_player();
        let scenery = test_item("mountain", "Mountain", 0, false);
        let config = InventoryConfig::default();
        let get_item = |_: &str| -> Option<ObjectRecord> { None };

        let result = can_add_item(&player, &scenery, 1, &config, get_item);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be taken"));
    }

    #[test]
    fn test_has_item() {
        let mut player = test_player();
        let item = test_item("coin", "Gold Coin", 1, true);
        let config = InventoryConfig::default();

        add_item_to_inventory(&mut player, &item, 50, &config);

        assert!(has_item(&player, "coin", 50));
        assert!(has_item(&player, "coin", 25));
        assert!(!has_item(&player, "coin", 51));
        assert!(!has_item(&player, "nonexistent", 1));
    }

    #[test]
    fn test_get_item_quantity() {
        let mut player = test_player();
        let item = test_item("scroll", "Magic Scroll", 1, true);
        let config = InventoryConfig::default();

        assert_eq!(get_item_quantity(&player, "scroll"), 0);

        add_item_to_inventory(&mut player, &item, 7, &config);
        assert_eq!(get_item_quantity(&player, "scroll"), 7);
    }
}

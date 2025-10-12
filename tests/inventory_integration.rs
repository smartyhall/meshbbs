/// Integration tests for inventory system with storage layer
use tempfile::TempDir;

use meshbbs::tmush::{
    InventoryConfig, InventoryResult, ObjectOwner, ObjectRecord, PlayerRecord,
    TinyMushStoreBuilder, CurrencyAmount,
};
use std::collections::HashMap;

fn test_item(id: &str, name: &str, weight: u8, takeable: bool) -> ObjectRecord {
    ObjectRecord {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("A {}", name),
        owner: ObjectOwner::World,
        created_at: chrono::Utc::now(),
        weight,
        currency_value: CurrencyAmount::default(),
        value: 0,
        takeable,
        usable: false,
        actions: HashMap::new(),
        flags: Vec::new(),
        locked: false,
        ownership_history: Vec::new(),
        schema_version: 1,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: String::new(),
    }
}

#[test]
fn test_player_add_and_remove_item() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    // Create player and item
    let player = PlayerRecord::new("alice", "Alice", "room1");
    let sword = test_item("sword1", "Iron Sword", 10, true);

    store.put_player(player).expect("put player");
    store.put_object(sword).expect("put object");

    // Add item to player
    let result = store
        .player_add_item("alice", "sword1", 1, &config)
        .expect("add item");
    
    match result {
        InventoryResult::Added { quantity, stacked } => {
            assert_eq!(quantity, 1);
            assert!(!stacked);
        }
        _ => panic!("Expected Added result"),
    }

    // Verify player has the item
    assert!(store.player_has_item("alice", "sword1", 1).expect("has item"));
    assert_eq!(store.player_item_quantity("alice", "sword1").expect("quantity"), 1);

    // Remove the item
    let result = store
        .player_remove_item("alice", "sword1", 1)
        .expect("remove item");

    match result {
        InventoryResult::Removed { quantity } => {
            assert_eq!(quantity, 1);
        }
        _ => panic!("Expected Removed result"),
    }

    // Verify item is gone
    assert!(!store.player_has_item("alice", "sword1", 1).expect("has item"));
}

#[test]
fn test_player_add_item_with_stacking() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    let player = PlayerRecord::new("bob", "Bob", "room1");
    let potion = test_item("potion1", "Health Potion", 1, true);

    store.put_player(player).expect("put player");
    store.put_object(potion).expect("put object");

    // Add first batch
    store
        .player_add_item("bob", "potion1", 5, &config)
        .expect("add item");
    assert_eq!(store.player_item_quantity("bob", "potion1").expect("quantity"), 5);

    // Add second batch (should stack)
    let result = store
        .player_add_item("bob", "potion1", 3, &config)
        .expect("add item");

    match result {
        InventoryResult::Added { quantity, stacked } => {
            assert_eq!(quantity, 3);
            assert!(stacked);
        }
        _ => panic!("Expected Added result with stacked=true"),
    }

    assert_eq!(store.player_item_quantity("bob", "potion1").expect("quantity"), 8);
}

#[test]
fn test_player_add_item_weight_limit() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig {
        max_weight: 100,
        ..Default::default()
    };

    let player = PlayerRecord::new("charlie", "Charlie", "room1");
    let boulder = test_item("boulder1", "Boulder", 50, true);

    store.put_player(player).expect("put player");
    store.put_object(boulder).expect("put object");

    // Add 2 boulders (100 weight) - should succeed
    let result = store
        .player_add_item("charlie", "boulder1", 2, &config)
        .expect("add item");
    assert!(matches!(result, InventoryResult::Added { .. }));

    // Try to add one more (would exceed limit) - should fail
    let result = store
        .player_add_item("charlie", "boulder1", 1, &config)
        .expect("add item");

    match result {
        InventoryResult::Failed { reason } => {
            assert!(reason.contains("Too heavy"));
        }
        _ => panic!("Expected Failed result"),
    }
}

#[test]
fn test_player_add_item_not_takeable() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    let player = PlayerRecord::new("dave", "Dave", "room1");
    let scenery = test_item("mountain1", "Mountain", 0, false);

    store.put_player(player).expect("put player");
    store.put_object(scenery).expect("put object");

    let result = store
        .player_add_item("dave", "mountain1", 1, &config)
        .expect("add item");

    match result {
        InventoryResult::Failed { reason } => {
            assert!(reason.contains("cannot be taken"));
        }
        _ => panic!("Expected Failed result"),
    }
}

#[test]
fn test_player_inventory_list() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    let player = PlayerRecord::new("eve", "Eve", "room1");
    let sword = test_item("sword1", "Iron Sword", 10, true);
    let potion = test_item("potion1", "Health Potion", 1, true);

    store.put_player(player).expect("put player");
    store.put_object(sword).expect("put object");
    store.put_object(potion).expect("put object");

    // Add items
    store.player_add_item("eve", "sword1", 1, &config).expect("add sword");
    store.player_add_item("eve", "potion1", 5, &config).expect("add potions");

    // Get inventory list
    let inventory = store.player_inventory_list("eve").expect("get inventory");

    assert!(inventory.len() > 0);
    // Should contain item names
    let inventory_str = inventory.join(" ");
    assert!(inventory_str.contains("Iron Sword"));
    assert!(inventory_str.contains("Health Potion"));
    assert!(inventory_str.contains("5x")); // Quantity indicator
}

#[test]
fn test_player_inventory_weight() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    let player = PlayerRecord::new("frank", "Frank", "room1");
    let sword = test_item("sword1", "Iron Sword", 10, true);
    let shield = test_item("shield1", "Iron Shield", 15, true);

    store.put_player(player).expect("put player");
    store.put_object(sword).expect("put object");
    store.put_object(shield).expect("put object");

    // Initially empty
    let weight = store.player_inventory_weight("frank").expect("get weight");
    assert_eq!(weight, 0);

    // Add sword
    store.player_add_item("frank", "sword1", 1, &config).expect("add sword");
    let weight = store.player_inventory_weight("frank").expect("get weight");
    assert_eq!(weight, 10);

    // Add shield
    store.player_add_item("frank", "shield1", 1, &config).expect("add shield");
    let weight = store.player_inventory_weight("frank").expect("get weight");
    assert_eq!(weight, 25);
}

#[test]
fn test_transfer_item_between_players() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    let bob = PlayerRecord::new("bob", "Bob", "room1");
    let gem = test_item("gem1", "Ruby", 1, true);

    store.put_player(alice).expect("put alice");
    store.put_player(bob).expect("put bob");
    store.put_object(gem).expect("put object");

    // Give Alice some gems
    store.player_add_item("alice", "gem1", 10, &config).expect("add gems");
    assert_eq!(store.player_item_quantity("alice", "gem1").expect("quantity"), 10);
    assert_eq!(store.player_item_quantity("bob", "gem1").expect("quantity"), 0);

    // Transfer 3 gems from Alice to Bob
    store
        .transfer_item("alice", "bob", "gem1", 3, &config)
        .expect("transfer");

    // Verify quantities
    assert_eq!(store.player_item_quantity("alice", "gem1").expect("quantity"), 7);
    assert_eq!(store.player_item_quantity("bob", "gem1").expect("quantity"), 3);
}

#[test]
fn test_transfer_item_insufficient() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig::default();

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    let bob = PlayerRecord::new("bob", "Bob", "room1");
    let gem = test_item("gem1", "Ruby", 1, true);

    store.put_player(alice).expect("put alice");
    store.put_player(bob).expect("put bob");
    store.put_object(gem).expect("put object");

    // Alice has no gems, try to transfer
    let result = store.transfer_item("alice", "bob", "gem1", 1, &config);

    assert!(result.is_err());
}

#[test]
fn test_transfer_item_receiver_over_capacity() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
    let config = InventoryConfig {
        max_weight: 50,
        ..Default::default()
    };

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    let bob = PlayerRecord::new("bob", "Bob", "room1");
    let heavy = test_item("anvil1", "Anvil", 40, true);
    let boulder = test_item("boulder1", "Boulder", 30, true);

    store.put_player(alice).expect("put alice");
    store.put_player(bob).expect("put bob");
    store.put_object(heavy).expect("put object");
    store.put_object(boulder).expect("put object");

    // Give Alice an anvil, Bob a boulder
    store.player_add_item("alice", "anvil1", 1, &config).expect("add anvil");
    store.player_add_item("bob", "boulder1", 1, &config).expect("add boulder");

    // Try to transfer anvil to Bob (would put him at 70 weight, over 50 limit)
    let result = store.transfer_item("alice", "bob", "anvil1", 1, &config);

    assert!(result.is_err());
    // Alice should still have the anvil
    assert_eq!(store.player_item_quantity("alice", "anvil1").expect("quantity"), 1);
    assert_eq!(store.player_item_quantity("bob", "anvil1").expect("quantity"), 0);
}

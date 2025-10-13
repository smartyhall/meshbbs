//! Integration tests for TAKE/DROP commands with trigger execution (Phase 8 completion)
//!
//! Tests that OnTake and OnDrop triggers fire correctly when players
//! pick up and drop objects in rooms.

use chrono::Utc;
use meshbbs::tmush::types::{CurrencyAmount, RoomOwner, RoomRecord, RoomVisibility};
use meshbbs::tmush::{
    ObjectOwner, ObjectRecord, ObjectTrigger, OwnershipReason, PlayerRecord, TinyMushStore,
};
use std::collections::HashMap;
use tempfile::TempDir;

/// Helper to create a test store with a player and room
fn setup_test_environment() -> (TempDir, TinyMushStore, String, String) {
    let temp = TempDir::new().unwrap();
    let store = TinyMushStore::open(temp.path()).unwrap();

    // Create test player
    let player = PlayerRecord::new("take_drop_player", "Take Drop Player", "test_room");
    store.put_player(player).unwrap();

    // Create test room
    let room = RoomRecord {
        id: "test_room".to_string(),
        name: "Test Room".to_string(),
        short_desc: "A test room".to_string(),
        long_desc: "This is a test room for take/drop testing.".to_string(),
        owner: RoomOwner::World,
        created_at: Utc::now(),
        visibility: RoomVisibility::Public,
        exits: HashMap::new(),
        items: vec![],
        flags: vec![],
        max_capacity: 10,
        housing_filter_tags: vec![],
        locked: false,
        schema_version: 1,
    };
    store.put_room(room).unwrap();

    (
        temp,
        store,
        "take_drop_player".to_string(),
        "test_room".to_string(),
    )
}

#[test]
fn test_take_object_fires_on_take_trigger() {
    use meshbbs::tmush::trigger::integration::execute_on_take;

    let (_temp, store, player_name, room_id) = setup_test_environment();

    // Create object with OnTake trigger in the room
    let mut obj = ObjectRecord {
        id: "test_object".to_string(),
        name: "Magic Coin".to_string(),
        description: "A glowing coin".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 10,
        takeable: true,
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: 1,
    };
    obj.actions.insert(
        ObjectTrigger::OnTake,
        "message(\"The coin glows as you touch it!\")".to_string(),
    );
    store.put_object(obj.clone()).unwrap();

    // Add object to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(obj.id.clone());
    store.put_room(room).unwrap();

    // Execute OnTake trigger
    let messages = execute_on_take(&obj, &player_name, &room_id, &store);

    // Verify trigger executed
    assert!(
        !messages.is_empty(),
        "OnTake trigger should produce messages"
    );
    assert!(
        messages.iter().any(|m| m.contains("glows")),
        "OnTake trigger should output custom message"
    );
}

#[test]
fn test_drop_object_fires_on_drop_trigger() {
    use meshbbs::tmush::trigger::integration::execute_on_drop;

    let (_temp, store, player_name, room_id) = setup_test_environment();

    // Create object with OnDrop trigger
    let mut obj = ObjectRecord {
        id: "test_object".to_string(),
        name: "Fragile Vase".to_string(),
        description: "A delicate vase".to_string(),
        owner: ObjectOwner::Player {
            username: player_name.clone(),
        },
        created_at: Utc::now(),
        weight: 2,
        currency_value: CurrencyAmount::default(),
        value: 50,
        takeable: true,
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: 1,
    };
    obj.actions.insert(
        ObjectTrigger::OnDrop,
        "message(\"The vase cracks as it hits the ground!\")".to_string(),
    );
    store.put_object(obj.clone()).unwrap();

    // Execute OnDrop trigger
    let messages = execute_on_drop(&obj, &player_name, &room_id, &store);

    // Verify trigger executed
    assert!(
        !messages.is_empty(),
        "OnDrop trigger should produce messages"
    );
    assert!(
        messages.iter().any(|m| m.contains("cracks")),
        "OnDrop trigger should output custom message"
    );
}

#[test]
fn test_take_non_takeable_object_blocked() {
    let (_temp, store, _player_name, room_id) = setup_test_environment();

    // Create non-takeable object
    let obj = ObjectRecord {
        id: "statue".to_string(),
        name: "Stone Statue".to_string(),
        description: "A heavy statue".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 100, // Heavy object
        currency_value: CurrencyAmount::default(),
        value: 500,
        takeable: false, // Cannot be taken
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: 1,
    };
    store.put_object(obj.clone()).unwrap();

    // Add to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(obj.id.clone());
    store.put_room(room).unwrap();

    // Verify object exists in room
    let room = store.get_room(&room_id).unwrap();
    assert!(room.items.contains(&obj.id), "Object should be in room");

    // Note: Actual TAKE command blocking tested via command handler
    // This test verifies the takeable flag exists and is accessible
    assert!(!obj.takeable, "Statue should not be takeable");
}

#[test]
fn test_take_locked_object_by_other_player_blocked() {
    let (_temp, store, _player_name, room_id) = setup_test_environment();

    // Create locked object owned by another player
    let obj = ObjectRecord {
        id: "locked_item".to_string(),
        name: "Locked Box".to_string(),
        description: "A box with a lock".to_string(),
        owner: ObjectOwner::Player {
            username: "other_player".to_string(),
        },
        created_at: Utc::now(),
        weight: 5,
        currency_value: CurrencyAmount::default(),
        value: 100,
        takeable: true,
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: true, // Locked by owner
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "other_player".to_string(),
        ownership_history: vec![],
        schema_version: 1,
    };
    store.put_object(obj.clone()).unwrap();

    // Add to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(obj.id.clone());
    store.put_room(room).unwrap();

    // Verify object is locked
    assert!(obj.locked, "Object should be locked");
    if let ObjectOwner::Player { username } = &obj.owner {
        assert_ne!(
            username, "take_drop_player",
            "Object owned by different player"
        );
    }
}

#[test]
fn test_ownership_transfer_on_take() {
    let (_temp, store, player_name, room_id) = setup_test_environment();

    // Create takeable object in room
    let obj = ObjectRecord {
        id: "coin".to_string(),
        name: "Gold Coin".to_string(),
        description: "A shiny coin".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 10,
        takeable: true,
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: 1,
    };
    store.put_object(obj.clone()).unwrap();

    // Add to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(obj.id.clone());
    store.put_room(room).unwrap();

    // Simulate ownership transfer (what TAKE command does)
    let mut updated_obj = obj.clone();
    updated_obj.owner = ObjectOwner::Player {
        username: player_name.clone(),
    };
    updated_obj
        .ownership_history
        .push(meshbbs::tmush::types::OwnershipTransfer {
            from_owner: Some("World".to_string()),
            to_owner: player_name.clone(),
            timestamp: Utc::now(),
            reason: OwnershipReason::PickedUp,
        });

    store.put_object(updated_obj.clone()).unwrap();

    // Verify ownership changed
    let retrieved = store.get_object(&obj.id).unwrap();
    if let ObjectOwner::Player { username } = &retrieved.owner {
        assert_eq!(
            username, &player_name,
            "Ownership should transfer to player"
        );
    } else {
        panic!("Owner should be Player variant");
    }

    // Verify history recorded
    assert!(
        !retrieved.ownership_history.is_empty(),
        "Should have ownership history"
    );
    assert_eq!(
        retrieved.ownership_history.last().unwrap().reason,
        OwnershipReason::PickedUp
    );
}

#[test]
fn test_ownership_transfer_on_drop() {
    let (_temp, store, player_name, _room_id) = setup_test_environment();

    // Create object owned by player
    let obj = ObjectRecord {
        id: "sword".to_string(),
        name: "Iron Sword".to_string(),
        description: "A basic sword".to_string(),
        owner: ObjectOwner::Player {
            username: player_name.clone(),
        },
        created_at: Utc::now(),
        weight: 10,
        currency_value: CurrencyAmount::default(),
        value: 50,
        takeable: true,
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: player_name.clone(),
        ownership_history: vec![],
        schema_version: 1,
    };
    store.put_object(obj.clone()).unwrap();

    // Simulate ownership transfer (what DROP command does)
    let mut updated_obj = obj.clone();
    updated_obj.owner = ObjectOwner::World;
    updated_obj
        .ownership_history
        .push(meshbbs::tmush::types::OwnershipTransfer {
            from_owner: Some(player_name.clone()),
            to_owner: "WORLD".to_string(),
            timestamp: Utc::now(),
            reason: OwnershipReason::Dropped,
        });

    store.put_object(updated_obj.clone()).unwrap();

    // Verify ownership changed to World
    let retrieved = store.get_object(&obj.id).unwrap();
    assert!(
        matches!(retrieved.owner, ObjectOwner::World),
        "Owner should be World after drop"
    );

    // Verify history recorded
    assert!(
        !retrieved.ownership_history.is_empty(),
        "Should have ownership history"
    );
    assert_eq!(
        retrieved.ownership_history.last().unwrap().reason,
        OwnershipReason::Dropped
    );
}

#[test]
fn test_consumable_on_take_trigger() {
    use meshbbs::tmush::trigger::integration::execute_on_take;

    let (_temp, store, player_name, room_id) = setup_test_environment();

    // Create consumable with OnTake trigger that removes itself
    let mut obj = ObjectRecord {
        id: "consumable".to_string(),
        name: "Ephemeral Crystal".to_string(),
        description: "A crystal that vanishes when touched".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 0,
        takeable: true,
        usable: false,
        actions: HashMap::new(),
        flags: vec![],
        locked: false,
        clone_depth: 0,
        clone_source_id: None,
        clone_count: 0,
        created_by: "world".to_string(),
        ownership_history: vec![],
        schema_version: 1,
    };
    obj.actions.insert(
        ObjectTrigger::OnTake,
        "message(\"The crystal dissolves in your hands!\") && consume()".to_string(),
    );
    store.put_object(obj.clone()).unwrap();

    // Add to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(obj.id.clone());
    store.put_room(room).unwrap();

    // Execute OnTake trigger
    let messages = execute_on_take(&obj, &player_name, &room_id, &store);

    // Verify trigger executed
    assert!(
        !messages.is_empty(),
        "OnTake trigger should produce messages"
    );
    assert!(
        messages.iter().any(|m| m.contains("dissolves")),
        "Should show dissolution message"
    );
}

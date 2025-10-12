//! Integration tests for TinyMUSH trigger system behaviors (Phase 9)
//!
//! Tests the example trigger objects to ensure all trigger types work correctly
//! in realistic scenarios with full storage integration.

use meshbbs::tmush::{TinyMushStore, PlayerRecord, ObjectRecord, ObjectOwner, ObjectTrigger};
use meshbbs::tmush::trigger::{execute_on_look, execute_on_use, execute_on_poke, execute_room_on_enter};
use meshbbs::tmush::types::{RoomRecord, RoomOwner, RoomVisibility, CurrencyAmount};
use chrono::Utc;
use std::collections::HashMap;
use tempfile::TempDir;

/// Helper to create a test store with a player and room
fn setup_test_environment(test_name: &str) -> (TempDir, TinyMushStore, String, String) {
    let temp = TempDir::new().unwrap();
    let store = TinyMushStore::open(temp.path()).unwrap();
    
    // Create test player with unique name based on test
    let player_name = format!("player_{}", test_name);
    let player = PlayerRecord::new(&player_name, &player_name, "test_room");
    store.put_player(player).unwrap();
    
    // Create test room
    let room = RoomRecord {
        id: "test_room".to_string(),
        name: "Test Room".to_string(),
        short_desc: "A test room".to_string(),
        long_desc: "This is a test room for trigger testing.".to_string(),
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
    
    (temp, store, player_name, "test_room".to_string())
}

#[test]
fn test_healing_potion_use() {
    let (_temp, store, player_name, room_id) = setup_test_environment("healing_potion");
    
    // Create healing potion with OnUse trigger
    let mut potion = ObjectRecord {
        id: "test_potion".to_string(),
        name: "Healing Potion".to_string(),
        description: "A glowing red potion".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 50,
        takeable: true,
        usable: true,
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
    potion.actions.insert(
        ObjectTrigger::OnUse,
        "message(\"The potion glows brightly!\") && heal(50)".to_string()
    );
    store.put_object(potion.clone()).unwrap();
    
    // Execute OnUse trigger
    let messages = execute_on_use(&potion, &player_name, &room_id, &store);
    
    // Verify trigger executed and produced messages
    assert!(!messages.is_empty(), "OnUse trigger should produce messages");
    assert!(messages.iter().any(|m| m.contains("glows brightly")), 
            "Trigger should output custom message");
    assert!(messages.iter().any(|m| m.contains("Healed for 50 HP")), 
            "Trigger should output heal message");
}

#[test]
fn test_quest_clue_reveal() {
    let (_temp, store, player_name, room_id) = setup_test_environment("quest_clue_reveal");
    
    // Create quest clue with OnLook trigger
    let mut clue = ObjectRecord {
        id: "test_clue".to_string(),
        name: "Tattered Note".to_string(),
        description: "An old piece of parchment".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 5,
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
    clue.actions.insert(
        ObjectTrigger::OnLook,
        "message(\"The note reads: Meet me at midnight!\")".to_string()
    );
    store.put_object(clue.clone()).unwrap();
    
    // Execute OnLook trigger
    let messages = execute_on_look(&clue, &player_name, &room_id, &store);
    
    // Verify trigger revealed hidden message
    assert!(!messages.is_empty(), "OnLook trigger should produce messages");
    assert!(messages.iter().any(|m| m.contains("Meet me at midnight")), 
            "Trigger should reveal hidden message");
}

#[test]
fn test_mystery_box_poke() {
    let (_temp, store, player_name, room_id) = setup_test_environment("mystery_box");
    
    // Create mystery box with OnPoke trigger (deterministic for testing)
    let mut box_obj = ObjectRecord {
        id: "test_box".to_string(),
        name: "Mystery Box".to_string(),
        description: "A mysterious box".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 5,
        currency_value: CurrencyAmount::default(),
        value: 25,
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
    box_obj.actions.insert(
        ObjectTrigger::OnPoke,
        "message(\"The box rattles mysteriously!\")".to_string()
    );
    store.put_object(box_obj.clone()).unwrap();
    
    // Execute OnPoke trigger
    let messages = execute_on_poke(&box_obj, &player_name, &room_id, &store);
    
    // Verify trigger executed
    assert!(!messages.is_empty(), "OnPoke trigger should produce messages");
    assert!(messages.iter().any(|m| m.contains("rattles mysteriously")), 
            "Trigger should output poke response");
}

#[test]
fn test_teleport_stone_use() {
    let (_temp, store, player_name, room_id) = setup_test_environment("mystery_box");
    
    // Create destination room
    let destination = RoomRecord {
        id: "destination_room".to_string(),
        name: "Destination".to_string(),
        short_desc: "A destination".to_string(),
        long_desc: "You have arrived.".to_string(),
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
    store.put_room(destination).unwrap();
    
    // Create teleport stone
    let mut stone = ObjectRecord {
        id: "test_stone".to_string(),
        name: "Teleport Stone".to_string(),
        description: "A magical stone".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 2,
        currency_value: CurrencyAmount::default(),
        value: 500,
        takeable: true,
        usable: true,
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
    stone.actions.insert(
        ObjectTrigger::OnUse,
        "message(\"The stone flashes!\") && teleport(\"destination_room\")".to_string()
    );
    store.put_object(stone.clone()).unwrap();
    
    // Execute OnUse trigger
    let messages = execute_on_use(&stone, &player_name, &room_id, &store);
    
    // Verify trigger executed
    assert!(!messages.is_empty(), "OnUse trigger should produce messages");
    assert!(messages.iter().any(|m| m.contains("flashes")), 
            "Trigger should show teleport effect");
    
    // Verify player was teleported
    let player = store.get_player(&player_name).unwrap();
    assert_eq!(player.current_room, "destination_room", 
               "Player should be teleported to destination");
}

#[test]
fn test_singing_mushroom_on_enter() {
    let (_temp, store, player_name, room_id) = setup_test_environment("mystery_box");
    
    // Create singing mushroom with OnEnter trigger
    let mut mushroom = ObjectRecord {
        id: "test_mushroom".to_string(),
        name: "Singing Mushroom".to_string(),
        description: "A purple mushroom".to_string(),
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
    mushroom.actions.insert(
        ObjectTrigger::OnEnter,
        "message(\"The mushroom hums a cheerful tune!\")".to_string()
    );
    store.put_object(mushroom.clone()).unwrap();
    
    // Add mushroom to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(mushroom.id.clone());
    store.put_room(room).unwrap();
    
    // Execute room OnEnter triggers
    let messages = execute_room_on_enter(&player_name, &room_id, &store);
    
    // Verify ambient trigger fired
    assert!(!messages.is_empty(), "OnEnter trigger should produce messages");
    assert!(messages.iter().any(|m| m.contains("cheerful tune")), 
            "Trigger should output ambient message");
}

#[test]
fn test_multiple_objects_on_enter() {
    let (_temp, store, player_name, room_id) = setup_test_environment("mystery_box");
    
    // Create two objects with OnEnter triggers
    let mut obj1 = ObjectRecord {
        id: "test_obj1".to_string(),
        name: "Object 1".to_string(),
        description: "First object".to_string(),
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
    obj1.actions.insert(
        ObjectTrigger::OnEnter,
        "message(\"Object 1 greets you!\")".to_string()
    );
    store.put_object(obj1.clone()).unwrap();
    
    let mut obj2 = ObjectRecord {
        id: "test_obj2".to_string(),
        name: "Object 2".to_string(),
        description: "Second object".to_string(),
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
    obj2.actions.insert(
        ObjectTrigger::OnEnter,
        "message(\"Object 2 waves!\")".to_string()
    );
    store.put_object(obj2.clone()).unwrap();
    
    // Add both objects to room
    let mut room = store.get_room(&room_id).unwrap();
    room.items.push(obj1.id.clone());
    room.items.push(obj2.id.clone());
    store.put_room(room).unwrap();
    
    // Execute room OnEnter triggers
    let messages = execute_room_on_enter(&player_name, &room_id, &store);
    
    // Verify both triggers fired
    assert_eq!(messages.len(), 2, "Both OnEnter triggers should fire");
    assert!(messages.iter().any(|m| m.contains("Object 1 greets")), 
            "First trigger should fire");
    assert!(messages.iter().any(|m| m.contains("Object 2 waves")), 
            "Second trigger should fire");
}

#[test]
fn test_conditional_quest_trigger() {
    let (_temp, store, player_name, room_id) = setup_test_environment("mystery_box");
    
    // Create ancient key with quest-conditional trigger
    let mut key = ObjectRecord {
        id: "test_key".to_string(),
        name: "Ancient Key".to_string(),
        description: "A mysterious key".to_string(),
        owner: ObjectOwner::World,
        created_at: Utc::now(),
        weight: 1,
        currency_value: CurrencyAmount::default(),
        value: 100,
        takeable: true,
        usable: true,
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
    key.actions.insert(
        ObjectTrigger::OnLook,
        "has_quest(\"test_quest\") ? message(\"The key glows with recognition!\") : message(\"The key looks ordinary.\")".to_string()
    );
    store.put_object(key.clone()).unwrap();
    
    // Execute OnLook trigger without quest
    let messages = execute_on_look(&key, &player_name, &room_id, &store);
    
    // Verify conditional branch works (player has no quest)
    assert!(!messages.is_empty(), "OnLook trigger should produce messages");
    assert!(messages.iter().any(|m| m.contains("ordinary")), 
            "Without quest, should show ordinary message");
}

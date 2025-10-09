/// Integration tests for companion NPC system (Phase 6 Week 4)
/// Validates end-to-end companion taming, care, training, and command outputs.

use meshbbs::tmush::{
    companion::{find_companion_in_room, get_player_companions, tame_companion},
    storage::TinyMushStore,
    state::seed_starter_companions,
    types::{CompanionRecord, PlayerRecord},
};
use tempfile::tempdir;

fn setup_store() -> TinyMushStore {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();
    
    // Seed starter companions
    for companion in seed_starter_companions() {
        store.put_companion(companion).unwrap();
    }
    
    // Create test player
    let player = PlayerRecord::new("testuser", "Test User", "town_square");
    store.put_player(player).unwrap();
    
    store
}

#[test]
fn test_companion_taming_and_listing() {
    let store = setup_store();
    
    // Verify starter companion exists in town_square
    let companion = find_companion_in_room(&store, "town_square", "Loyal Hound")
        .unwrap()
        .expect("Loyal Hound should exist in town_square");
    
    assert_eq!(companion.id, "loyal_hound");
    assert_eq!(companion.name, "Loyal Hound");
    assert!(companion.owner.is_none(), "Should be wild initially");
    
    // Tame the companion
    tame_companion(&store, "testuser", &companion.id).unwrap();
    
    // Verify ownership
    let updated = store.get_companion(&companion.id).unwrap();
    assert_eq!(updated.owner, Some("testuser".to_string()));
    assert_eq!(updated.loyalty, 30, "Should start with loyalty 30");
    
    // Verify player's companion list
    let companions = get_player_companions(&store, "testuser").unwrap();
    assert_eq!(companions.len(), 1);
    assert_eq!(companions[0].name, "Loyal Hound");
    
    // Verify player record updated
    let player = store.get_player("testuser").unwrap();
    assert!(player.companions.contains(&companion.id));
}

#[test]
fn test_feed_and_pet_stat_gains() {
    let store = setup_store();
    
    // Setup: Tame a companion first
    let companion = find_companion_in_room(&store, "town_square", "Loyal Hound")
        .unwrap()
        .unwrap();
    tame_companion(&store, "testuser", &companion.id).unwrap();
    
    let initial = store.get_companion(&companion.id).unwrap();
    assert_eq!(initial.happiness, 100, "Starts with max happiness");
    assert_eq!(initial.loyalty, 30, "Starts with loyalty 30 after taming");
    
    // Test petting - should gain loyalty (starts at 30, <50)
    use meshbbs::tmush::companion::pet_companion;
    let gain = pet_companion(&store, "testuser", &companion.id).unwrap();
    assert_eq!(gain, 5, "Should gain 5 loyalty when <50");
    
    let after_pet = store.get_companion(&companion.id).unwrap();
    assert_eq!(after_pet.loyalty, 35);
    
    // Pet multiple times to test reduced gains at high loyalty
    for _ in 0..4 {
        pet_companion(&store, "testuser", &companion.id).unwrap();
    }
    
    let high_loyalty = store.get_companion(&companion.id).unwrap();
    assert!(high_loyalty.loyalty >= 50, "Should reach loyalty 50+");
    
    let small_gain = pet_companion(&store, "testuser", &companion.id).unwrap();
    assert_eq!(small_gain, 2, "Should gain 2 loyalty when >=50");
    
    // Test feeding - happiness at 100 should give smaller gain
    use meshbbs::tmush::companion::feed_companion;
    let happiness_gain = feed_companion(&store, "testuser", &companion.id).unwrap();
    assert_eq!(happiness_gain, 10, "Should gain 10 happiness when >=50");
    
    let after_feed = store.get_companion(&companion.id).unwrap();
    assert_eq!(after_feed.happiness, 100, "Happiness caps at 100");
}

#[test]
fn test_mount_and_dismount_horses() {
    let store = setup_store();
    
    // Setup: Tame the horse at south_market
    let horse = find_companion_in_room(&store, "south_market", "Gentle Mare")
        .unwrap()
        .unwrap();
    tame_companion(&store, "testuser", &horse.id).unwrap();
    
    use meshbbs::tmush::companion::{mount_companion, dismount_companion};
    use meshbbs::tmush::types::CompanionType;
    
    // Verify it's a horse
    let companion = store.get_companion(&horse.id).unwrap();
    assert_eq!(companion.companion_type, CompanionType::Horse);
    assert!(!companion.is_mounted, "Should not be mounted initially");
    
    // Mount the horse
    mount_companion(&store, "testuser", &horse.id).unwrap();
    
    let mounted = store.get_companion(&horse.id).unwrap();
    assert!(mounted.is_mounted, "Should be mounted");
    
    // Verify player record updated
    let player = store.get_player("testuser").unwrap();
    assert_eq!(player.mounted_companion, Some(horse.id.clone()));
    
    // Dismount
    let name = dismount_companion(&store, "testuser").unwrap();
    assert_eq!(name, "Gentle Mare");
    
    let dismounted = store.get_companion(&horse.id).unwrap();
    assert!(!dismounted.is_mounted, "Should not be mounted");
    
    // Verify player record cleared
    let player = store.get_player("testuser").unwrap();
    assert_eq!(player.mounted_companion, None);
}

#[test]
fn test_companion_stay_and_come_control() {
    let store = setup_store();
    
    // Setup: Tame dog with auto-follow
    let dog = find_companion_in_room(&store, "town_square", "Loyal Hound")
        .unwrap()
        .unwrap();
    tame_companion(&store, "testuser", &dog.id).unwrap();
    
    use meshbbs::tmush::companion::move_companion_to_room;
    use meshbbs::tmush::types::CompanionBehavior;
    
    let initial = store.get_companion(&dog.id).unwrap();
    assert!(initial.has_auto_follow(), "Dog should have auto-follow");
    assert_eq!(initial.room_id, "town_square");
    
    // Simulate STAY: remove auto-follow behavior
    let mut stayed = initial.clone();
    stayed.behaviors.retain(|b| !matches!(b, CompanionBehavior::AutoFollow));
    store.put_companion(stayed).unwrap();
    
    let after_stay = store.get_companion(&dog.id).unwrap();
    assert!(!after_stay.has_auto_follow(), "Auto-follow should be disabled");
    
    // Move companion to different room
    move_companion_to_room(&store, &dog.id, "south_market").unwrap();
    
    let moved = store.get_companion(&dog.id).unwrap();
    assert_eq!(moved.room_id, "south_market");
    
    // Simulate COME: move back to player's room
    move_companion_to_room(&store, &dog.id, "town_square").unwrap();
    
    let returned = store.get_companion(&dog.id).unwrap();
    assert_eq!(returned.room_id, "town_square");
}

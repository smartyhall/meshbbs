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

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

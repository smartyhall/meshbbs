//! Test TinyHack mini-map feature with fog of war

use meshbbs::bbs::tinyhack::{handle_turn, load_or_new_with_flag};
use tempfile::tempdir;

#[test]
fn minimap_shows_fog_of_war() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();
    
    // Create new game
    let (mut gs, _view, is_new) = load_or_new_with_flag(data_dir, "testuser");
    assert!(is_new, "Should be a new game");
    
    // Initial state: only starting position (0,0) should be visited
    assert_eq!(gs.visited.len(), 36, "Should have 36 rooms (6x6)");
    assert!(gs.visited[0], "Starting position should be visited");
    
    // Request mini-map with M command
    let (gs2, map_output) = handle_turn(gs, "M");
    
    // Verify map output
    println!("=== Initial Map ===\n{}", map_output);
    assert!(map_output.contains("@"), "Map should show player position");
    assert!(map_output.contains("#"), "Map should show unexplored fog");
    assert!(map_output.contains("@=You"), "Map should have legend");
    assert!(map_output.contains("#=Fog"), "Map should explain fog symbol");
    
    // Verify character count is reasonable for Meshtastic
    assert!(
        map_output.len() < 230,
        "Map should fit in Meshtastic message (was {} chars)",
        map_output.len()
    );
    
    // Move east and check map updates
    let (mut gs3, _) = handle_turn(gs2, "E");
    assert_eq!(gs3.player.x, 1, "Player should have moved east");
    assert!(gs3.visited[1], "New position should be visited");
    
    let (gs4, map_output2) = handle_turn(gs3, "M");
    println!("=== After moving East ===\n{}", map_output2);
    
    // Player should be at new position
    let lines: Vec<&str> = map_output2.lines().collect();
    let map_start = lines.iter().position(|l| l.contains("@") || l.contains("#")).unwrap();
    let first_map_line = lines[map_start];
    
    // First line should have @ in second position (moved from col 0 to col 1)
    assert!(
        first_map_line.starts_with(". @") || first_map_line.starts_with(".@"),
        "Player should be at position 1 in first row, got: {}",
        first_map_line
    );
    
    // Move south
    let (gs5, _) = handle_turn(gs4, "S");
    assert_eq!(gs5.player.y, 1, "Player should have moved south");
    let (_, map_output3) = handle_turn(gs5, "M");
    println!("=== After moving South ===\n{}", map_output3);
    
    // Verify map still fits in message limit
    assert!(
        map_output3.len() < 230,
        "Map should still fit after movement (was {} chars)",
        map_output3.len()
    );
}

#[test]
fn minimap_shows_room_types() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();
    
    // Create new game
    let (mut gs, _, _) = load_or_new_with_flag(data_dir, "testuser2");
    
    // Explore multiple rooms to reveal different types
    for _ in 0..10 {
        // Move around to explore
        let (new_gs, _) = handle_turn(gs, "E");
        gs = new_gs;
        if gs.player.x >= gs.w - 1 {
            let (new_gs, _) = handle_turn(gs, "S");
            gs = new_gs;
            if gs.player.y >= gs.h - 1 {
                break;
            }
        }
    }
    
    // Request map
    let (_, map_output) = handle_turn(gs, "M");
    println!("=== Explored Map ===\n{}", map_output);
    
    // Should show some explored rooms (dots or other symbols)
    let dot_count = map_output.matches('.').count();
    let fog_count = map_output.matches('#').count();
    
    assert!(dot_count > 0, "Should have some explored rooms");
    assert!(fog_count > 0, "Should still have unexplored areas");
    
    // Legend should be present
    assert!(map_output.contains("@=You"), "Should have legend");
    assert!(map_output.contains("M=Mon"), "Should explain monster symbol");
    assert!(map_output.contains("C=Chest"), "Should explain chest symbol");
}

#[test]
fn minimap_backward_compatibility() {
    // Test that old saves without visited vector still work
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();
    
    // Create a game
    let (gs, _, _) = load_or_new_with_flag(data_dir, "compat_test");
    
    // Simulate old save by clearing visited (would happen on deserialization)
    let mut gs_old = gs.clone();
    gs_old.visited = vec![]; // Simulate old save
    
    // Request map should not crash
    let (_, map_output) = handle_turn(gs_old, "M");
    
    // Should still render something reasonable
    assert!(map_output.contains("@"), "Should show player even with empty visited");
    assert!(map_output.len() > 50, "Should have substantial output");
}

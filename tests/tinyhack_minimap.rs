//! Test TinyHack mini-map feature with fog of war

use meshbbs::bbs::tinyhack::{handle_turn, load_or_new_with_flag, GameState, RoomKind};
use std::collections::VecDeque;

fn find_path_to_new_tile(gs: &GameState) -> Option<Vec<char>> {
    const DIRS: &[(char, isize, isize)] = &[('N', 0, -1), ('S', 0, 1), ('W', -1, 0), ('E', 1, 0)];
    let mut queue: VecDeque<((usize, usize), Vec<char>)> = VecDeque::new();
    let mut seen = vec![false; gs.w * gs.h];
    let start = (gs.player.x, gs.player.y);
    seen[gs.idx(start.0, start.1)] = true;
    queue.push_back((start, Vec::new()));

    while let Some(((x, y), path)) = queue.pop_front() {
        for (dir, dx, dy) in DIRS.iter() {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= gs.w as isize || ny >= gs.h as isize {
                continue;
            }
            let nxu = nx as usize;
            let nyu = ny as usize;
            let idx = gs.idx(nxu, nyu);
            if seen[idx] {
                continue;
            }
            if matches!(gs.room(nxu, nyu).kind, RoomKind::LockedDoor) {
                continue;
            }
            let mut next_path = path.clone();
            next_path.push(*dir);
            if (nxu, nyu) != start {
                return Some(next_path);
            }
            seen[idx] = true;
            queue.push_back(((nxu, nyu), next_path));
        }
    }

    None
}

fn unlock_first_blocker(gs: &mut GameState) {
    const DIRS: &[(isize, isize)] = &[(1, 0), (0, 1), (-1, 0), (0, -1)];
    let (x, y) = (gs.player.x as isize, gs.player.y as isize);
    for (dx, dy) in DIRS.iter() {
        let nx = x + dx;
        let ny = y + dy;
        if nx < 0 || ny < 0 || nx >= gs.w as isize || ny >= gs.h as isize {
            continue;
        }
        let nxu = nx as usize;
        let nyu = ny as usize;
        let idx = gs.idx(nxu, nyu);
        if matches!(gs.map[idx].kind, RoomKind::LockedDoor) {
            gs.map[idx].kind = RoomKind::Empty;
            gs.map[idx].used = true;
            return;
        }
    }
}
use tempfile::tempdir;

#[test]
fn minimap_shows_fog_of_war() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();

    // Create new game
    let (gs, _view, is_new) = load_or_new_with_flag(data_dir, "testuser");
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
    assert!(
        map_output.contains("#=Fog"),
        "Map should explain fog symbol"
    );

    // Verify character count is reasonable for Meshtastic
    assert!(
        map_output.len() < 230,
        "Map should fit in Meshtastic message (was {} chars)",
        map_output.len()
    );

    // Move at least one step, unlocking a nearby door if the starting room is boxed in
    let mut gs_for_step = gs2.clone();
    let mut path1 = find_path_to_new_tile(&gs_for_step);
    if path1.is_none() {
        unlock_first_blocker(&mut gs_for_step);
        path1 = find_path_to_new_tile(&gs_for_step);
    }
    let path1 = path1.expect("Should be able to find a path to another tile");

    let mut gs3 = gs_for_step;
    for dir in path1 {
        let cmd = dir.to_string();
        let (next_state, _) = handle_turn(gs3, &cmd);
        gs3 = next_state;
    }
    let new_idx = (gs3.player.y as usize * gs3.w as usize) + gs3.player.x as usize;
    assert!(
        gs3.visited.get(new_idx).copied().unwrap_or(false),
        "New position should be marked visited"
    );

    let (gs4, map_output2) = handle_turn(gs3.clone(), "M");
    println!("=== After first move ===\n{}", map_output2);

    // Player marker should align with current coordinates in the rendered grid
    let grid_lines: Vec<&str> = map_output2
        .lines()
        .filter(|line| line.contains('@') || line.contains('#') || line.contains('.'))
        .take(gs3.h as usize)
        .collect();
    let player_row = grid_lines
        .get(gs3.player.y as usize)
        .expect("Player row should exist in map output");
    let tokens: Vec<&str> = player_row.split_whitespace().collect();
    assert_eq!(
        tokens
            .get(gs3.player.x as usize)
            .copied()
            .unwrap_or_default(),
        "@",
        "Player marker should be at ({}, {}) in map row: {}",
        gs3.player.x,
        gs3.player.y,
        player_row
    );

    // Try to move again from the new position (optional fallback unlock if boxed in)
    let mut gs5 = gs4.clone();
    let mut path2 = find_path_to_new_tile(&gs5);
    if path2.is_none() {
        unlock_first_blocker(&mut gs5);
        path2 = find_path_to_new_tile(&gs5);
    }
    if let Some(path2) = path2 {
        for dir in path2 {
            let cmd = dir.to_string();
            let (next_state, _) = handle_turn(gs5, &cmd);
            gs5 = next_state;
        }
    }
    let (_, map_output3) = handle_turn(gs5, "M");
    println!("=== After second move ===\n{}", map_output3);

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
    assert!(
        map_output.contains("M=Mon"),
        "Should explain monster symbol"
    );
    assert!(
        map_output.contains("C=Chest"),
        "Should explain chest symbol"
    );
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
    assert!(
        map_output.contains("@"),
        "Should show player even with empty visited"
    );
    assert!(map_output.len() > 50, "Should have substantial output");
}

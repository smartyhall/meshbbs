//! Integration tests for TinyMUSH functionality validation.
//! Basic tests to ensure TinyMUSH module is properly integrated.

use meshbbs::bbs::session::Session;
use meshbbs::tmush::should_route_to_tinymush;

/// Test basic session routing logic
#[tokio::test] 
async fn test_tinymush_session_routing() {
    // Test session without game slug - should not route to TinyMUSH
    let session_no_game = Session::new("test_node".to_string(), "12345".to_string());
    assert!(!should_route_to_tinymush(&session_no_game));
    
    // Test session with TinyMUSH game slug - should route to TinyMUSH
    let mut session_tinymush = Session::new("test_node".to_string(), "12345".to_string());
    session_tinymush.current_game_slug = Some("tinymush".to_string());
    assert!(should_route_to_tinymush(&session_tinymush));
    
    // Test session with different game slug - should not route to TinyMUSH
    let mut session_other = Session::new("test_node".to_string(), "12345".to_string());
    session_other.current_game_slug = Some("tinyhack".to_string());
    assert!(!should_route_to_tinymush(&session_other));
}

/// Test that TinyMUSH constants are accessible
#[test]
fn test_tinymush_constants_available() {
    // Test that we can access TinyMUSH constants
    use meshbbs::tmush::state::{REQUIRED_LANDING_LOCATION_ID, REQUIRED_START_LOCATION_ID};
    
    assert_eq!(REQUIRED_LANDING_LOCATION_ID, "gazebo_landing");
    assert_eq!(REQUIRED_START_LOCATION_ID, "town_square");
}

/// Test that room manager is accessible and working
#[test]
fn test_room_manager_available() {
    use meshbbs::tmush::room_manager::RoomManager;
    use meshbbs::tmush::storage::TinyMushStore;
    use tempfile::TempDir;
    
    // Create temp directory and store for testing
    let temp_dir = TempDir::new().unwrap();
    let store = TinyMushStore::open(temp_dir.path().join("test.db")).unwrap();
    
    // Test that we can create a room manager
    let room_manager = RoomManager::new(store);
    
    // Verify cache stats are initialized properly
    let stats = room_manager.cache_stats();
    assert_eq!(stats.cache_size, 0);
    assert_eq!(stats.total_accesses, 0);
    assert_eq!(stats.total_rooms, 0);
    assert!(stats.max_cache_size > 0); // Should have some capacity
}
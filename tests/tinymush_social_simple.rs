//! Simple integration test for TinyMUSH social commands
//! Tests the basic functionality without database conflicts

use meshbbs::bbs::session::Session;
use meshbbs::tmush::commands::{TinyMushCommand, TinyMushProcessor};
use meshbbs::tmush::TinyMushStore;
use tempfile::TempDir;

#[tokio::test]
async fn test_social_command_parsing() {
    // Test that we can parse social commands correctly
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStore::open(dir.path().join("tinymush")).expect("store");
    let processor = TinyMushProcessor::new(store.clone());

    // Test SAY parsing - should preserve case
    let parsed = processor.parse_command("SAY hello world");
    match parsed {
        TinyMushCommand::Say(text) => assert_eq!(text, "hello world"),
        _ => panic!("Expected Say command"),
    }

    // Test EMOTE parsing - should preserve case
    let parsed = processor.parse_command("EMOTE waves");
    match parsed {
        TinyMushCommand::Emote(text) => assert_eq!(text, "waves"),
        _ => panic!("Expected Emote command"),
    }

    // Test WHISPER parsing - should preserve case
    let parsed = processor.parse_command("WHISPER alice hello");
    match parsed {
        TinyMushCommand::Whisper(target, message) => {
            assert_eq!(target, "alice");
            assert_eq!(message, "hello");
        }
        _ => panic!("Expected Whisper command"),
    }

    // Test shortcuts - should preserve case
    let parsed = processor.parse_command("' hello");
    match parsed {
        TinyMushCommand::Say(text) => assert_eq!(text, "hello"),
        _ => panic!("Expected Say command from ' shortcut"),
    }

    let parsed = processor.parse_command(": waves");
    match parsed {
        TinyMushCommand::Emote(text) => assert_eq!(text, "waves"),
        _ => panic!("Expected Emote command from : shortcut"),
    }
}

#[test]
fn test_session_routing() {
    use meshbbs::tmush::should_route_to_tinymush;

    // Test session with TinyMUSH game slug
    let mut session = Session::new("test".to_string(), "123".to_string());
    session.current_game_slug = Some("tinymush".to_string());
    assert!(should_route_to_tinymush(&session));

    // Test session without game slug
    let session_no_game = Session::new("test".to_string(), "123".to_string());
    assert!(!should_route_to_tinymush(&session_no_game));
}

//! Test suite for TinyMUSH Phase 4 social communication features.
//! Validates SAY, WHISPER, EMOTE, POSE, and OOC commands.

use meshbbs::bbs::session::Session;
use meshbbs::config::Config;
use meshbbs::storage::Storage;
use meshbbs::tmush::handle_tinymush_command;
use tempfile::TempDir;

async fn setup_test_environment(test_name: &str) -> (Config, Storage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.storage.data_dir = temp_dir.path().to_string_lossy().to_string();
    config.games.tinymush_enabled = true;
    // Use a unique path per test to avoid lock conflicts
    config.games.tinymush_db_path = Some(temp_dir.path().join(format!("tinymush_{}", test_name)).to_string_lossy().to_string());
    
    let storage = Storage::new(&config.storage.data_dir).await.unwrap();
    (config, storage, temp_dir)
}

async fn create_test_session(name: &str, node_id: &str) -> Session {
    let mut session = Session::new(name.to_string(), node_id.to_string());
    session.current_game_slug = Some("tinymush".to_string());
    session.login(name.to_string(), 1).await.unwrap();
    session
}

#[tokio::test]
async fn test_say_command() {
    let (config, mut storage, _temp_dir) = setup_test_environment("say").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test SAY with message
    let response = handle_tinymush_command(&mut session, "SAY Hello everyone!", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("You say: \"Hello everyone!\""));
    assert!(response.contains("No other players in room"));

    // Test SAY without message
    let response = handle_tinymush_command(&mut session, "SAY", &mut storage, &config)
        .await
        .unwrap();
    
    assert_eq!(response, "Say what?");

    // Test SAY shortcut (')
    let response = handle_tinymush_command(&mut session, "' Greetings!", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("You say: \"Greetings!\""));
}

#[tokio::test]
async fn test_whisper_command() {
    let (config, mut storage, _temp_dir) = setup_test_environment("whisper").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test WHISPER to non-existent player
    let response = handle_tinymush_command(&mut session, "WHISPER bob Hello", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("Player 'bob' not found in this room"));

    // Test WHISPER without message
    let response = handle_tinymush_command(&mut session, "WHISPER bob", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("Usage: WHISPER <player> <message>"));

    // Test WHISPER without target
    let response = handle_tinymush_command(&mut session, "WHISPER", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("Usage: WHISPER <player> <message>"));

    // Test whisper to self (should be prevented once we have room tracking)
    // This will be enhanced when we implement actual multi-player room tracking
}

#[tokio::test]
async fn test_emote_command() {
    let (config, mut storage, _temp_dir) = setup_test_environment("emote").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test EMOTE with action
    let response = handle_tinymush_command(&mut session, "EMOTE waves cheerfully", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("alice waves cheerfully"));
    assert!(response.contains("No other players in room"));

    // Test EMOTE without action
    let response = handle_tinymush_command(&mut session, "EMOTE", &mut storage, &config)
        .await
        .unwrap();
    
    assert_eq!(response, "Emote what?");

    // Test EMOTE shortcut (:)
    let response = handle_tinymush_command(&mut session, ": smiles broadly", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("alice smiles broadly"));
}

#[tokio::test]
async fn test_pose_command() {
    let (config, mut storage, _temp_dir) = setup_test_environment("pose").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test POSE with description
    let response = handle_tinymush_command(&mut session, "POSE is leaning against the wall", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("alice is leaning against the wall"));
    assert!(response.contains("No other players in room"));

    // Test POSE without description
    let response = handle_tinymush_command(&mut session, "POSE", &mut storage, &config)
        .await
        .unwrap();
    
    assert_eq!(response, "Strike what pose?");

    // Test POSE shortcut (;)
    let response = handle_tinymush_command(&mut session, "; stands proudly", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("alice stands proudly"));
}

#[tokio::test]
async fn test_ooc_command() {
    let (config, mut storage, _temp_dir) = setup_test_environment("ooc").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test OOC with message
    let response = handle_tinymush_command(&mut session, "OOC This is really cool!", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("[OOC] alice: This is really cool!"));
    assert!(response.contains("No other players in room"));

    // Test OOC without message
    let response = handle_tinymush_command(&mut session, "OOC", &mut storage, &config)
        .await
        .unwrap();
    
    assert_eq!(response, "Say what out of character?");
}

#[tokio::test]
async fn test_social_help() {
    let (config, mut storage, _temp_dir) = setup_test_environment("help").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test social help
    let response = handle_tinymush_command(&mut session, "HELP SOCIAL", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("=== SOCIAL ==="));
    assert!(response.contains("SAY <text> (') - speak aloud to room"));
    assert!(response.contains("WHISPER <player> <text> - private message"));
    assert!(response.contains("EMOTE <action> (:) - perform action"));
    assert!(response.contains("POSE <pose> (;) - strike a pose"));
    assert!(response.contains("OOC <text> - out of character chat"));
    assert!(response.contains("Examples:"));
}

#[tokio::test]
async fn test_command_parsing_variations() {
    let (config, mut storage, _temp_dir) = setup_test_environment("parsing").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test various command formats and capitalization
    let test_cases = vec![
        ("say hello", "You say: \"hello\""),
        ("SAY HELLO", "You say: \"HELLO\""),
        ("Say Hello", "You say: \"Hello\""),
        ("emote waves", "alice waves"),
        ("EMOTE WAVES", "alice WAVES"),
        ("pose sits", "alice sits"),
        ("POSE SITS", "alice SITS"),
        ("ooc testing", "[OOC] alice: testing"),
        ("OOC TESTING", "[OOC] alice: TESTING"),
    ];

    for (command, expected_content) in test_cases {
        let response = handle_tinymush_command(&mut session, command, &mut storage, &config)
            .await
            .unwrap();
        
        assert!(
            response.contains(expected_content),
            "Command '{}' should contain '{}', but got: {}",
            command, expected_content, response
        );
    }
}

#[tokio::test]
async fn test_whisper_abbreviation() {
    let (config, mut storage, _temp_dir) = setup_test_environment("whis").await;
    let mut session = create_test_session("alice", "node123").await;

    // Test WHIS abbreviation
    let response = handle_tinymush_command(&mut session, "WHIS bob hello", &mut storage, &config)
        .await
        .unwrap();
    
    assert!(response.contains("Player 'bob' not found in this room"));
}
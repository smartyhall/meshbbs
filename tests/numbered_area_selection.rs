//! Test numbered area selection functionality
mod common;
use meshbbs::config::Config;
use meshbbs::storage::Storage;

#[tokio::test]
async fn numbered_area_selection() {
    let cfg = Config::default();
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    let registry = common::empty_game_registry();
    let mut session =
        meshbbs::bbs::session::Session::new("test_numbered".into(), "node_test".into());

    // Enter main menu
    let _ = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "init", &mut storage, &cfg, &registry)
        .await
        .unwrap();

    // Login first (required after authentication fix)
    session.login("test_user".into(), 1).await.unwrap();

    // Enter message areas
    let areas_output = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "M", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    assert!(areas_output.contains("1. general") || areas_output.contains("1."));
    assert!(
        areas_output.contains("Type number to select area")
            || areas_output.contains("Type number to select topic")
    );

    // Test selecting area by number
    let area1_output = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "1", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    assert!(area1_output.contains("Recent messages in") || area1_output.contains("Messages in"));

    // Reset to message areas state
    session.state = meshbbs::bbs::session::SessionState::MessageTopics;

    // Test invalid area number
    let invalid_output = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "99", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    assert!(
        invalid_output.contains("Invalid area number")
            || invalid_output.contains("Invalid topic number")
    );

    // Test that old R command still works
    let r_output = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "R", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    assert!(r_output.contains("Recent messages in") || r_output.contains("Messages in"));
}

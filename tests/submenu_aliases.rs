use meshbbs::config::Config;
use meshbbs::storage::Storage;

#[tokio::test]
async fn message_area_aliases() {
    let cfg = Config::default();
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    let mut session = meshbbs::bbs::session::Session::new("s_ma".into(), "node_ma".into());
    session.login("tester".into(), 1).await.unwrap();
    session.state = meshbbs::bbs::session::SessionState::MainMenu;

    // Long-form should be rejected once logged in.
    let areas_full = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "MESSAGES", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        areas_full.starts_with("Invalid command"),
        "MESSAGES should be rejected in favor of short-form M; got: {areas_full}"
    );

    // Short form still works and transitions to topics menu.
    let areas_short = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "M", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        areas_short.contains("Topics") || areas_short.contains("Message Topics"),
        "M should enter topics view; got: {areas_short}"
    );

    // Simulate navigating to MessageTopics menu (e.g., listing topics then returning).
    session.state = meshbbs::bbs::session::SessionState::MessageTopics;

    // Long-form READ should now be rejected, while short-form R lists messages.
    let r_full = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "READ", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        r_full.contains("Commands: [R]ead"),
        "READ should be nudged toward R shortcut; got: {r_full}"
    );

    let r_short = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "R", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        r_short.contains("Messages in") || r_short.contains("Recent messages"),
        "R output should list messages; got: {r_short}"
    );
}

#[tokio::test]
async fn user_menu_aliases() {
    let cfg = Config::default();
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    let mut session = meshbbs::bbs::session::Session::new("s_um".into(), "node_um".into());
    session.login("tester2".into(), 1).await.unwrap();
    session.state = meshbbs::bbs::session::SessionState::MainMenu;

    // Long-form USER should now be rejected to reinforce concise inputs.
    let full = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "USER", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        full.starts_with("Invalid command"),
        "USER should be rejected in favor of P; got: {full}"
    );

    // Short form P opens preferences.
    let short = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "P", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        short.contains("Preferences"),
        "P should open the preferences menu; got: {short}"
    );
}

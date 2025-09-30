use meshbbs::bbs::server::BbsServer;
use meshbbs::config::Config;
use std::fs;
use std::path::Path;

// Basic TinyHack integration: enter game, take turns, and verify save persistence.
#[tokio::test]
async fn tinyhack_enter_play_persist() {
    // Setup config with TinyHack enabled and use a temp data dir
    let mut cfg = Config::default();
    cfg.games.tinyhack_enabled = true;
    // Use a temp data dir and keep it alive for the duration of the test
    let _td = tempfile::tempdir().unwrap();
    cfg.storage.data_dir = _td.path().to_string_lossy().to_string();
    cfg.bbs.max_users = 10;

    let mut server = BbsServer::new(cfg.clone()).await.unwrap();

    // Create a session and login as 'alice'
    let mut session = meshbbs::bbs::session::Session::new("nodeTH1".into(), "nodeTH1".into());
    session.login("alice".into(), 1).await.unwrap();
    let node_key = session.node_id.clone();
    server.test_insert_session(session);

    // Enter TinyHack via main menu
    server.route_test_text_direct(&node_key, "T").await.unwrap();
    let first_screen = server.test_messages().last().unwrap().1.clone();
    assert!(first_screen.starts_with("L"), "expected TinyHack compact status, got: {}", first_screen);

    // Issue a couple of turns: move east, rest
    server.route_test_text_direct(&node_key, "E").await.unwrap();
    let after_e = server.test_messages().last().unwrap().1.clone();
    assert!(after_e.contains("Your move?"), "screen should prompt for move: {}", after_e);

    server.route_test_text_direct(&node_key, "R").await.unwrap();
    let after_r = server.test_messages().last().unwrap().1.clone();
    assert!(after_r.contains("Your move?"), "screen should prompt for move: {}", after_r);

    // Validate save file written and contains JSON with gid
    // Since server.config is private, reconstruct expected path: cfg.storage.data_dir + "/tinyhack/alice.json"
    let save_path = Path::new(&cfg.storage.data_dir).join("tinyhack").join("alice.json");
    let content = fs::read_to_string(&save_path).expect("save exists");
    assert!(content.contains("\"gid\":"), "save should contain gid: {}", content);
    // Parse JSON gid and turn
    #[derive(serde::Deserialize)]
    struct SaveHead { gid: u32, turn: u32 }
    let head: SaveHead = serde_json::from_str(&content).expect("valid json");

    // Leave game then re-enter; gid should persist and saved turn should match re-entry state
    server.route_test_text_direct(&node_key, "B").await.unwrap();
    let back_to_menu = server.test_messages().last().unwrap().1.clone();
    assert!(back_to_menu.contains("Main Menu:"));
    server.route_test_text_direct(&node_key, "T").await.unwrap();
    let reentered = server.test_messages().last().unwrap().1.clone();
    assert!(reentered.starts_with("L"));
    // Re-read save file and ensure gid remains and turn didn’t regress
    let content2 = fs::read_to_string(&save_path).expect("save exists");
    let head2: SaveHead = serde_json::from_str(&content2).expect("valid json");
    assert_eq!(head2.gid, head.gid, "gid should persist across re-entry");
    assert!(head2.turn >= head.turn, "turn should not regress: {} -> {}", head.turn, head2.turn);
}

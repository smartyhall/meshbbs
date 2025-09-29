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
    assert!(first_screen.starts_with("TH g"), "expected TinyHack screen, got: {}", first_screen);
    // Parse gid and turn from header line: "TH g<gid> t<turn> ..."
    fn parse_gid_turn(s: &str) -> (u32, u32) {
        let line = s.lines().next().unwrap_or("");
        let mut gid: u32 = 0; let mut turn: u32 = 0;
        for tok in line.split_whitespace() {
            if let Some(num) = tok.strip_prefix("g") { gid = num.parse().unwrap_or(0); }
            if let Some(num) = tok.strip_prefix("t") { turn = num.parse().unwrap_or(0); }
        }
        (gid, turn)
    }
    let (gid0, t0) = parse_gid_turn(&first_screen);
    assert!(gid0 > 0 && t0 >= 1, "invalid gid/turn: {} {} in {}", gid0, t0, first_screen);

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

    // Leave game then re-enter; gid should persist and turn should have advanced
    server.route_test_text_direct(&node_key, "B").await.unwrap();
    let back_to_menu = server.test_messages().last().unwrap().1.clone();
    assert!(back_to_menu.contains("Main Menu:"));
    server.route_test_text_direct(&node_key, "T").await.unwrap();
    let reentered = server.test_messages().last().unwrap().1.clone();
    assert!(reentered.starts_with("TH g"));
    let (gid1, t1) = parse_gid_turn(&reentered);
    // Re-entered view should match the saved file
    assert_eq!(gid1, head.gid, "gid should match saved state");
    assert_eq!(t1, head.turn, "turn should match saved state on re-enter");
}

use meshbbs::config::Config;
mod common;
use meshbbs::storage::Storage;

// Validate that 'h' and 'H' and other case variants are accepted like HELP and produce same output as full HELP
#[tokio::test]
async fn help_single_letter_alias() {
    let cfg = Config::default();
    let mut session = meshbbs::bbs::session::Session::new("s_h".into(), "node_h".into());
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    let registry = common::empty_game_registry();
    // First command transitions from Connected -> MainMenu regardless of content, returning banner
    let _banner = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "ignored", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    // Now in MainMenu: capture baseline help output
    let base = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "H", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    for variant in ["H", "h", "?"] {
        let out = meshbbs::bbs::commands::CommandProcessor::new()
            .process(&mut session, variant, &mut storage, &cfg, &registry)
            .await
            .unwrap();
        assert_eq!(
            base, out,
            "Variant '{variant}' should produce same guest help output"
        );
    }

    for forbidden in ["help", "HeLp"] {
        let out = meshbbs::bbs::commands::CommandProcessor::new()
            .process(&mut session, forbidden, &mut storage, &cfg, &registry)
            .await
            .unwrap();
        // After authentication fix: unauthenticated users get "Authentication required" for unknown commands
        // instead of "Invalid command"
        assert!(
            out.starts_with("Invalid command") || out.contains("Authentication required"),
            "Long-form variant '{forbidden}' should be rejected or require auth. Got: {out}"
        );
    }

    // Login and compare again (different content set)
    session.login("tester".into(), 1).await.unwrap();
    let user_base = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "H", &mut storage, &cfg, &registry)
        .await
        .unwrap();
    for variant in ["H", "h", "?"] {
        let out = meshbbs::bbs::commands::CommandProcessor::new()
            .process(&mut session, variant, &mut storage, &cfg, &registry)
            .await
            .unwrap();
        assert_eq!(
            user_base, out,
            "Variant '{variant}' should produce same logged-in help output"
        );
    }

    for forbidden in ["help", "HELP"] {
        let out = meshbbs::bbs::commands::CommandProcessor::new()
            .process(&mut session, forbidden, &mut storage, &cfg, &registry)
            .await
            .unwrap();
        assert!(
            out.starts_with("Invalid command"),
            "Long-form variant '{forbidden}' should be rejected when logged in"
        );
    }
}

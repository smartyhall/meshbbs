use meshbbs::config::Config;
use meshbbs::storage::Storage;

#[tokio::test]
async fn main_menu_single_letter_aliases() {
    let cfg = Config::default();
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    let mut session = meshbbs::bbs::session::Session::new("s_mm".into(), "node_mm".into());
    // Transition to MainMenu
    let _banner = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "anything", &mut storage, &cfg)
        .await
        .unwrap();

    let help_base = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "H", &mut storage, &cfg)
        .await
        .unwrap();
    for variant in ["h", "?"] {
        let help_variant = meshbbs::bbs::commands::CommandProcessor::new()
            .process(&mut session, variant, &mut storage, &cfg)
            .await
            .unwrap();
        assert_eq!(
            help_base, help_variant,
            "Variant '{variant}' should mirror 'H'"
        );
    }

    let help_word = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "HELP", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        help_word.starts_with("Invalid command"),
        "Long-form HELP should be rejected now"
    );

    let messages_word = meshbbs::bbs::commands::CommandProcessor::new()
        .process(&mut session, "MESSAGES", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        messages_word.starts_with("Invalid command"),
        "Long-form MESSAGES should be rejected"
    );
}

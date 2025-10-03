use meshbbs::bbs::commands::CommandProcessor;
use meshbbs::bbs::session::{Session, SessionState};
use meshbbs::config::Config;
use meshbbs::storage::Storage;
use tempfile::tempdir;

fn config_for_temp_data_dir(temp_path: &std::path::Path) -> Config {
    let mut cfg = Config::default();
    cfg.storage.data_dir = temp_path.join("data").to_string_lossy().to_string();
    cfg.logging.file = None;
    cfg.logging.security_file = None;
    cfg
}

#[tokio::test]
async fn password_change_flow_requires_current_password_and_updates_storage() {
    let tmp = tempdir().unwrap();
    let cfg = config_for_temp_data_dir(tmp.path());
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    storage
        .register_user("alice", "OldPass123", Some("node-pass-change"))
        .await
        .unwrap();

    let mut session = Session::new("sess-change".into(), "node-pass-change".into());
    session.login("alice".into(), 1).await.unwrap();

    let processor = CommandProcessor::new();

    let menu = processor
        .process(&mut session, "P", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        menu.contains("[C]hange pass"),
        "menu should offer change option for users with passwords"
    );
    assert!(
        !menu.contains("[N]ew pass"),
        "menu should hide new password option when one already exists"
    );
    assert_eq!(session.state, SessionState::UserMenu);

    let prompt = processor
        .process(&mut session, "C", &mut storage, &cfg)
        .await
        .unwrap();
    assert_eq!(prompt, "Enter current password (or '.' to cancel):\n");
    assert_eq!(session.state, SessionState::UserChangePassCurrent);
    assert!(
        session.pending_input.is_none(),
        "pending input should be cleared when entering change flow"
    );

    let next_prompt = processor
        .process(&mut session, "OldPass123", &mut storage, &cfg)
        .await
        .unwrap();
    assert_eq!(
        next_prompt,
        "Enter new password (min 8 chars, or '.' to cancel):\n"
    );
    assert_eq!(session.state, SessionState::UserChangePassNew);
    assert_eq!(session.pending_input.as_deref(), Some("OldPass123"));

    let retry_same = processor
        .process(&mut session, "OldPass123", &mut storage, &cfg)
        .await
        .unwrap();
    assert_eq!(
        retry_same,
        "New password must differ from current. Try again or '.' to cancel:\n"
    );
    assert_eq!(session.state, SessionState::UserChangePassNew);

    let completion = processor
        .process(&mut session, "NewPass987", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        completion.starts_with("Password changed."),
        "should confirm password change"
    );
    assert!(
        completion.contains("[C]hange pass"),
        "preferences menu should be re-rendered after change"
    );
    assert!(
        !completion.contains("[N]ew pass"),
        "preferences menu should not offer new password option after change"
    );
    assert_eq!(session.state, SessionState::UserMenu);
    assert!(session.pending_input.is_none());

    let (_u_new, ok_new) = storage
        .verify_user_password("alice", "NewPass987")
        .await
        .unwrap();
    assert!(ok_new, "new password must verify successfully");
    let (_u_old, ok_old) = storage
        .verify_user_password("alice", "OldPass123")
        .await
        .unwrap();
    assert!(!ok_old, "old password must no longer verify");
}

#[tokio::test]
async fn passwordless_user_sets_password_and_menu_updates() {
    let tmp = tempdir().unwrap();
    let cfg = config_for_temp_data_dir(tmp.path());
    let mut storage = Storage::new(&cfg.storage.data_dir).await.unwrap();
    storage
        .create_or_update_user("legacy", "node-passless")
        .await
        .unwrap();

    let (_pre_user, pre_ok) = storage
        .verify_user_password("legacy", "StrongPass8")
        .await
        .unwrap();
    assert!(
        !pre_ok,
        "passwordless user should not validate any password yet"
    );

    let mut session = Session::new("sess-set".into(), "node-passless".into());
    session.login("legacy".into(), 1).await.unwrap();

    let processor = CommandProcessor::new();

    let menu = processor
        .process(&mut session, "P", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        menu.contains("[N]ew pass"),
        "menu should offer new password option when none is set"
    );
    assert!(
        !menu.contains("[C]hange pass"),
        "menu should hide change option until a password exists"
    );
    assert_eq!(session.state, SessionState::UserMenu);

    let prompt = processor
        .process(&mut session, "N", &mut storage, &cfg)
        .await
        .unwrap();
    assert_eq!(
        prompt,
        "Enter new password (min 8 chars, or '.' to cancel):\n"
    );
    assert_eq!(session.state, SessionState::UserSetPassNew);

    let short = processor
        .process(&mut session, "short", &mut storage, &cfg)
        .await
        .unwrap();
    assert_eq!(
        short,
        "Password too short (min 8). Try again or '.' to cancel:\n"
    );
    assert_eq!(session.state, SessionState::UserSetPassNew);

    let completion = processor
        .process(&mut session, "StrongPass8", &mut storage, &cfg)
        .await
        .unwrap();
    assert!(
        completion.starts_with("Password set."),
        "should confirm password creation"
    );
    assert!(
        completion.contains("[C]hange pass"),
        "preferences menu should now show change option"
    );
    assert!(
        !completion.contains("[N]ew pass"),
        "preferences menu should remove new option once a password exists"
    );
    assert_eq!(session.state, SessionState::UserMenu);
    assert!(session.pending_input.is_none());

    let (_u_after, ok_after) = storage
        .verify_user_password("legacy", "StrongPass8")
        .await
        .unwrap();
    assert!(ok_after, "new password should verify successfully");
}

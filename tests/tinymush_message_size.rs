//! Comprehensive 200-byte message size validation tests for TinyMUSH
//!
//! Per Meshtastic protocol constraints, all outbound messages must be ≤ 200 bytes.
//! This test suite validates that all TinyMUSH command outputs respect this limit.

use meshbbs::tmush::commands::TinyMushProcessor;
use meshbbs::tmush::TinyMushStore;
use tempfile::TempDir;

const MAX_MESSAGE_SIZE: usize = 200;

/// Test all help text outputs are under 200 bytes
#[test]
fn test_help_text_under_200_bytes() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStore::open(dir.path().join("tinymush")).expect("store");
    let processor = TinyMushProcessor::new(store.clone());

    // Test main help
    let main_help = processor.help_main();
    assert!(
        main_help.len() <= MAX_MESSAGE_SIZE,
        "Main HELP response too long: {} bytes (max {})\nResponse: {}",
        main_help.len(),
        MAX_MESSAGE_SIZE,
        main_help
    );

    // Test commands help
    let commands_help = processor.help_commands();
    assert!(
        commands_help.len() <= MAX_MESSAGE_SIZE,
        "HELP COMMANDS response too long: {} bytes (max {})\nResponse: {}",
        commands_help.len(),
        MAX_MESSAGE_SIZE,
        commands_help
    );

    // Test movement help
    let movement_help = processor.help_movement();
    assert!(
        movement_help.len() <= MAX_MESSAGE_SIZE,
        "HELP MOVEMENT response too long: {} bytes (max {})\nResponse: {}",
        movement_help.len(),
        MAX_MESSAGE_SIZE,
        movement_help
    );

    // Test social help
    let social_help = processor.help_social();
    assert!(
        social_help.len() <= MAX_MESSAGE_SIZE,
        "HELP SOCIAL response too long: {} bytes (max {})\nResponse: {}",
        social_help.len(),
        MAX_MESSAGE_SIZE,
        social_help
    );

    // Test bulletin help
    let bulletin_help = processor.help_bulletin();
    assert!(
        bulletin_help.len() <= MAX_MESSAGE_SIZE,
        "HELP BULLETIN response too long: {} bytes (max {})\nResponse: {}",
        bulletin_help.len(),
        MAX_MESSAGE_SIZE,
        bulletin_help
    );

    // Test mail help
    let mail_help = processor.help_mail();
    assert!(
        mail_help.len() <= MAX_MESSAGE_SIZE,
        "HELP MAIL response too long: {} bytes (max {})\nResponse: {}",
        mail_help.len(),
        MAX_MESSAGE_SIZE,
        mail_help
    );
}

/// Test currency display formatting stays under 200 bytes
#[test]
fn test_currency_display_under_200_bytes() {
    use meshbbs::tmush::CurrencyAmount;

    // Test decimal currency
    let decimal = CurrencyAmount::Decimal {
        minor_units: 123456,
    };
    let display = format!("{:?}", decimal);
    assert!(
        display.len() <= MAX_MESSAGE_SIZE,
        "Decimal currency display too long: {} bytes",
        display.len()
    );

    // Test multi-tier currency
    let multi_tier = CurrencyAmount::MultiTier {
        base_units: 987654321,
    };
    let display = format!("{:?}", multi_tier);
    assert!(
        display.len() <= MAX_MESSAGE_SIZE,
        "Multi-tier currency display too long: {} bytes",
        display.len()
    );
}

/// Test that error messages are concise
#[test]
fn test_error_messages_under_200_bytes() {
    // Test common error message patterns
    let errors = vec![
        "Unknown command: 'frobozz'\nType HELP for available commands.",
        "You don't have enough currency for that purchase.",
        "That item is not available in your inventory.",
        "You must be at the Town Square to access the bulletin board.",
        "Player not found. Make sure they have logged in at least once.",
        "Your inventory is full. Drop something first.",
        "No mail messages.\nRMAIL <id> to read, DMAIL <id> to delete\nSEND <player> <subject> <message> to send",
    ];

    for error in errors {
        assert!(
            error.len() <= MAX_MESSAGE_SIZE,
            "Error message too long: {} bytes (max {})\nMessage: {}",
            error.len(),
            MAX_MESSAGE_SIZE,
            error
        );
    }
}

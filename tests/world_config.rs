// Integration test for world configuration system

use meshbbs::tmush::storage::TinyMushStore;
use tempfile::tempdir;

#[test]
fn test_get_default_world_config() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Get default config
    let config = store.get_world_config().unwrap();

    assert_eq!(config.world_name, "Old Towne Mesh");
    assert!(config.welcome_message.contains("Welcome to Old Towne Mesh"));
    assert!(config.motd.contains("Welcome to Old Towne Mesh"));
}

#[test]
fn test_set_config_world_name() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Set world name
    store
        .update_world_config_field("world_name", "My Custom World", "test_admin")
        .unwrap();

    // Verify it persisted
    let config = store.get_world_config().unwrap();
    assert_eq!(config.world_name, "My Custom World");
    assert_eq!(config.updated_by, "test_admin");
}

#[test]
fn test_set_config_welcome_message() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Set custom welcome message
    let custom_welcome = "Welcome to My Amazing MUD!\nEnjoy your stay!";
    store
        .update_world_config_field("welcome_message", custom_welcome, "test_admin")
        .unwrap();

    // Verify it persisted
    let config = store.get_world_config().unwrap();
    assert!(config.welcome_message.contains("Welcome to My Amazing MUD"));
    assert!(config.welcome_message.contains("Enjoy your stay!"));
}

#[test]
fn test_set_config_motd() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Set MOTD
    store
        .update_world_config_field("motd", "Server maintenance tonight at 10pm!", "test_admin")
        .unwrap();

    // Verify it persisted
    let config = store.get_world_config().unwrap();
    assert!(config.motd.contains("Server maintenance tonight"));
}

#[test]
fn test_set_config_world_description() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Set world description
    store
        .update_world_config_field("world_description", "A persistent test world", "tester")
        .unwrap();

    // Verify it persisted
    let config = store.get_world_config().unwrap();
    assert_eq!(config.world_description, "A persistent test world");
    assert_eq!(config.updated_by, "tester");
}

#[test]
fn test_set_invalid_config_field() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Try to set invalid field
    let result = store.update_world_config_field("invalid_field", "some_value", "test");

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown config field"));
}

#[test]
fn test_config_persists_across_reopens() {
    let dir = tempdir().unwrap();

    // First store sets config
    {
        let store1 = TinyMushStore::open(dir.path()).unwrap();
        store1
            .update_world_config_field("world_description", "A persistent test world", "tester")
            .unwrap();
    }

    // Second store reads config
    {
        let store2 = TinyMushStore::open(dir.path()).unwrap();
        let config = store2.get_world_config().unwrap();
        assert_eq!(config.world_description, "A persistent test world");
        assert_eq!(config.updated_by, "tester");
    }
}

#[test]
fn test_config_all_fields_can_be_updated() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Update all fields
    store
        .update_world_config_field("world_name", "Test World", "admin1")
        .unwrap();
    store
        .update_world_config_field("world_description", "Test Description", "admin2")
        .unwrap();
    store
        .update_world_config_field("motd", "Test MOTD", "admin3")
        .unwrap();
    store
        .update_world_config_field("welcome_message", "Test Welcome", "admin4")
        .unwrap();

    // Verify all updates
    let config = store.get_world_config().unwrap();
    assert_eq!(config.world_name, "Test World");
    assert_eq!(config.world_description, "Test Description");
    assert_eq!(config.motd, "Test MOTD");
    assert_eq!(config.welcome_message, "Test Welcome");
    assert_eq!(config.updated_by, "admin4"); // Last update wins
}

#[test]
fn test_put_whole_config() {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Get default config
    let mut config = store.get_world_config().unwrap();

    // Modify it
    config.world_name = "Completely New World".to_string();
    config.welcome_message = "Brand new welcome!".to_string();
    config.updated_by = "super_admin".to_string();

    // Save it
    store.put_world_config(&config).unwrap();

    // Verify it persisted
    let loaded = store.get_world_config().unwrap();
    assert_eq!(loaded.world_name, "Completely New World");
    assert_eq!(loaded.welcome_message, "Brand new welcome!");
    assert_eq!(loaded.updated_by, "super_admin");
}

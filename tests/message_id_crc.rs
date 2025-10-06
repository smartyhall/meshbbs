//! Test message ID and CRC-16 functionality

use meshbbs::storage::Storage;
use tempfile::tempdir;

#[tokio::test]
async fn test_message_id_and_crc_generation() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();

    let mut storage = Storage::new(data_dir).await.unwrap();
    storage
        .register_user("alice", "password", None)
        .await
        .unwrap();
    storage
        .create_topic("test", "Test Topic", "", 1, 1, "alice")
        .await
        .unwrap();
    storage
        .register_user("testuser", "password", None)
        .await
        .unwrap();

    let _msg_id = storage
        .store_message("test", "testuser", "Test message content")
        .await
        .unwrap();

    let messages = storage.get_messages("test", 10).await.unwrap();
    assert_eq!(messages.len(), 1);

    let message = &messages[0];

    assert!(message.message_id.is_some(), "message_id should be present");
    let message_id = message.message_id.as_ref().unwrap();
    assert_eq!(message_id.len(), 12, "message_id should be 12 hex chars");
    assert!(
        message_id.chars().all(|c| c.is_ascii_hexdigit()),
        "message_id should be valid hex"
    );

    assert!(message.crc16.is_some(), "crc16 should be present");
    let crc = message.crc16.unwrap();
    assert!(crc > 0, "crc16 should be non-zero for non-empty content");
}

#[tokio::test]
async fn test_message_id_uniqueness() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();

    let mut storage = Storage::new(data_dir).await.unwrap();
    storage
        .register_user("alice", "password", None)
        .await
        .unwrap();
    storage
        .create_topic("test", "Test Topic", "", 1, 1, "alice")
        .await
        .unwrap();
    storage
        .register_user("testuser", "password", None)
        .await
        .unwrap();

    for i in 0..10 {
        storage
            .store_message("test", "testuser", &format!("Message {}", i))
            .await
            .unwrap();
    }

    let messages = storage.get_messages("test", 20).await.unwrap();
    assert_eq!(messages.len(), 10);

    let mut ids = std::collections::HashSet::new();
    for msg in &messages {
        if let Some(msg_id) = &msg.message_id {
            ids.insert(msg_id.clone());
        }
    }

    assert_eq!(ids.len(), 10, "All message_ids should be unique");
}

#[tokio::test]
async fn test_crc_different_for_different_content() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();

    let mut storage = Storage::new(data_dir).await.unwrap();
    storage
        .register_user("alice", "password", None)
        .await
        .unwrap();
    storage
        .create_topic("test", "Test Topic", "", 1, 1, "alice")
        .await
        .unwrap();
    storage
        .register_user("testuser", "password", None)
        .await
        .unwrap();

    storage
        .store_message("test", "testuser", "First message")
        .await
        .unwrap();
    storage
        .store_message("test", "testuser", "Second message")
        .await
        .unwrap();

    let messages = storage.get_messages("test", 10).await.unwrap();
    assert_eq!(messages.len(), 2);

    let crc1 = messages[0].crc16.unwrap();
    let crc2 = messages[1].crc16.unwrap();

    assert_ne!(
        crc1, crc2,
        "Different content should produce different CRCs"
    );
}

#[tokio::test]
async fn test_backward_compatibility_with_old_messages() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();

    let mut storage = Storage::new(data_dir).await.unwrap();
    storage
        .register_user("alice", "password", None)
        .await
        .unwrap();
    storage
        .create_topic("test", "Test Topic", "", 1, 1, "alice")
        .await
        .unwrap();

    let messages_dir = std::path::Path::new(data_dir).join("messages").join("test");
    tokio::fs::create_dir_all(&messages_dir).await.unwrap();

    let old_message = r#"{
        "id": "old-message-123",
        "topic": "test",
        "author": "olduser",
        "content": "Old message without new fields",
        "timestamp": "2025-01-01T00:00:00Z",
        "replies": [],
        "pinned": false
    }"#;

    let msg_path = messages_dir.join("old-message-123.json");
    tokio::fs::write(&msg_path, old_message).await.unwrap();

    let messages = storage.get_messages("test", 10).await.unwrap();
    assert_eq!(messages.len(), 1);

    assert!(
        messages[0].message_id.is_none(),
        "Old message should have no message_id"
    );
    assert!(
        messages[0].crc16.is_none(),
        "Old message should have no crc16"
    );
}

#[tokio::test]
async fn test_message_id_format() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().to_str().unwrap();

    let mut storage = Storage::new(data_dir).await.unwrap();
    storage
        .register_user("alice", "password", None)
        .await
        .unwrap();
    storage
        .create_topic("test", "Test Topic", "", 1, 1, "alice")
        .await
        .unwrap();
    storage
        .register_user("testuser", "password", None)
        .await
        .unwrap();

    storage
        .store_message("test", "testuser", "Test")
        .await
        .unwrap();

    let messages = storage.get_messages("test", 1).await.unwrap();
    let message_id = messages[0].message_id.as_ref().unwrap();

    assert_eq!(message_id.len(), 12);

    let timestamp_hex = &message_id[0..8];
    let timestamp = u32::from_str_radix(timestamp_hex, 16).unwrap();

    let year_2020_timestamp = 1577836800u32;
    assert!(
        timestamp > year_2020_timestamp,
        "Timestamp should be after 2020"
    );
}

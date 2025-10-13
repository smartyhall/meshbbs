//! Basic mail system functionality tests for TinyMUSH
//!
//! Tests core mail functionality including sending, reading, deleting,
//! and folder management using the TinyMushStore directly.

use meshbbs::tmush::types::{MailMessage, MailStatus};
use meshbbs::tmush::TinyMushStoreBuilder;
use tempfile::TempDir;

#[test]
fn mail_message_round_trip() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Create a mail message
    let message = MailMessage::new(
        "alice",
        "bob",
        "Test Subject",
        "This is a test mail message body",
    );

    // Send the message
    let message_id = store.send_mail(message.clone()).expect("send mail");
    assert!(message_id > 0);

    // Retrieve from bob's inbox
    let retrieved_message = store
        .get_mail("inbox", "bob", message_id)
        .expect("get mail");
    assert_eq!(retrieved_message.sender, message.sender);
    assert_eq!(retrieved_message.recipient, message.recipient);
    assert_eq!(retrieved_message.subject, message.subject);
    assert_eq!(retrieved_message.body, message.body);
    assert_eq!(retrieved_message.status, MailStatus::Unread);

    // Retrieve from alice's sent folder
    let sent_message = store
        .get_mail("sent", "alice", message_id)
        .expect("get sent mail");
    assert_eq!(sent_message.sender, message.sender);
    assert_eq!(sent_message.recipient, message.recipient);
}

#[test]
fn mail_list_and_pagination() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Send multiple messages
    for i in 1..=5 {
        let message = MailMessage::new(
            "alice",
            "bob",
            &format!("Subject {}", i),
            &format!("Message body {}", i),
        );
        store.send_mail(message).expect("send mail");
    }

    // List all messages in bob's inbox
    let inbox_messages = store.list_mail("inbox", "bob", 0, 10).expect("list inbox");
    assert_eq!(inbox_messages.len(), 5);

    // Check that messages are sorted by date (newest first)
    for i in 0..4 {
        assert!(inbox_messages[i].sent_at >= inbox_messages[i + 1].sent_at);
    }

    // Test pagination
    let first_page = store.list_mail("inbox", "bob", 0, 3).expect("first page");
    let second_page = store.list_mail("inbox", "bob", 3, 3).expect("second page");

    assert_eq!(first_page.len(), 3);
    assert_eq!(second_page.len(), 2);

    // List alice's sent folder
    let sent_messages = store.list_mail("sent", "alice", 0, 10).expect("list sent");
    assert_eq!(sent_messages.len(), 5);
}

#[test]
fn mail_mark_read_and_delete() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Send a message
    let message = MailMessage::new("alice", "bob", "Test", "Test message");
    let message_id = store.send_mail(message).expect("send mail");

    // Verify it's unread
    let unread_count = store.count_unread_mail("bob").expect("count unread");
    assert_eq!(unread_count, 1);

    // Mark as read
    store
        .mark_mail_read("inbox", "bob", message_id)
        .expect("mark read");

    // Verify it's now read
    let unread_count = store
        .count_unread_mail("bob")
        .expect("count unread after read");
    assert_eq!(unread_count, 0);

    let read_message = store
        .get_mail("inbox", "bob", message_id)
        .expect("get read message");
    assert_eq!(read_message.status, MailStatus::Read);

    // Delete the message
    store
        .delete_mail("inbox", "bob", message_id)
        .expect("delete mail");

    // Verify it's gone
    let result = store.get_mail("inbox", "bob", message_id);
    assert!(result.is_err());

    // Verify count is correct
    let total_count = store
        .count_mail("inbox", "bob")
        .expect("count after delete");
    assert_eq!(total_count, 0);
}

#[test]
fn mail_quota_enforcement() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Send many messages to exceed quota and collect their IDs
    let mut message_ids = Vec::new();
    for i in 1..=10 {
        let message = MailMessage::new(
            "alice",
            "bob",
            &format!("Message {}", i),
            &format!("Body {}", i),
        );
        let message_id = store.send_mail(message).expect("send mail");
        message_ids.push(message_id);
    }

    // Mark first 5 as read
    for &message_id in &message_ids[0..5] {
        store
            .mark_mail_read("inbox", "bob", message_id)
            .expect("mark read");
    }

    // Enforce quota of 7 messages
    let removed = store.enforce_mail_quota("bob", 7).expect("enforce quota");
    assert_eq!(removed, 3); // Should remove 3 oldest read messages

    // Verify final count
    let final_count = store.count_mail("inbox", "bob").expect("final count");
    assert_eq!(final_count, 7);

    // Verify unread messages are preserved
    let unread_count = store.count_unread_mail("bob").expect("unread count");
    assert_eq!(unread_count, 5); // All unread messages should remain
}

#[test]
fn mail_cleanup_old_messages() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Send and immediately mark messages as read
    for i in 1..=5 {
        let message = MailMessage::new(
            "alice",
            "bob",
            &format!("Old Message {}", i),
            &format!("Old body {}", i),
        );
        let message_id = store.send_mail(message).expect("send mail");
        store
            .mark_mail_read("inbox", "bob", message_id)
            .expect("mark read");
    }

    // Send some unread messages
    for i in 1..=3 {
        let message = MailMessage::new(
            "alice",
            "bob",
            &format!("New Message {}", i),
            &format!("New body {}", i),
        );
        store.send_mail(message).expect("send mail");
    }

    let initial_count = store.count_mail("inbox", "bob").expect("initial count");
    assert_eq!(initial_count, 8);

    // Clean up messages older than 0 days (should remove read messages)
    let removed = store.cleanup_old_mail("bob", 0).expect("cleanup");
    assert_eq!(removed, 5); // Should remove all 5 read messages

    // Verify unread messages remain
    let final_count = store.count_mail("inbox", "bob").expect("final count");
    assert_eq!(final_count, 3);

    let unread_count = store.count_unread_mail("bob").expect("unread count");
    assert_eq!(unread_count, 3);
}

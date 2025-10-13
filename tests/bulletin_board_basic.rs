//! Basic bulletin board functionality tests

use meshbbs::tmush::types::{BulletinBoard, BulletinMessage};
use meshbbs::tmush::TinyMushStoreBuilder;
use tempfile::TempDir;

#[test]
fn bulletin_board_round_trip() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Create a bulletin board
    let board = BulletinBoard::new(
        "test_board",
        "Test Board",
        "A test bulletin board",
        "test_room",
    );

    store.put_bulletin_board(board.clone()).expect("put board");

    // Retrieve the board
    let retrieved_board = store.get_bulletin_board("test_board").expect("get board");
    assert_eq!(retrieved_board.id, board.id);
    assert_eq!(retrieved_board.name, board.name);
    assert_eq!(retrieved_board.description, board.description);
}

#[test]
fn bulletin_message_posting_and_retrieval() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Create a bulletin board first
    let board = BulletinBoard::new("test_board", "Test Board", "Test", "test_room");
    store.put_bulletin_board(board).expect("put board");

    // Post a message
    let message = BulletinMessage::new(
        "test_user",
        "Test Subject",
        "This is a test message body",
        "test_board",
    );

    let message_id = store.post_bulletin(message.clone()).expect("post message");
    assert!(message_id > 0);

    // Retrieve the message
    let retrieved_message = store
        .get_bulletin("test_board", message_id)
        .expect("get message");
    assert_eq!(retrieved_message.author, message.author);
    assert_eq!(retrieved_message.subject, message.subject);
    assert_eq!(retrieved_message.body, message.body);
    assert_eq!(retrieved_message.board_id, message.board_id);
    assert_eq!(retrieved_message.id, message_id);
}

#[test]
fn bulletin_message_listing() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Create a bulletin board
    let board = BulletinBoard::new("test_board", "Test Board", "Test", "test_room");
    store.put_bulletin_board(board).expect("put board");

    // Post multiple messages
    for i in 1..=5 {
        let message = BulletinMessage::new(
            &format!("user_{}", i),
            &format!("Subject {}", i),
            &format!("Message body {}", i),
            "test_board",
        );
        store.post_bulletin(message).expect("post message");
    }

    // List messages
    let messages = store
        .list_bulletins("test_board", 0, 10)
        .expect("list messages");
    assert_eq!(messages.len(), 5);

    // Check pagination
    let first_page = store
        .list_bulletins("test_board", 0, 3)
        .expect("first page");
    assert_eq!(first_page.len(), 3);

    let second_page = store
        .list_bulletins("test_board", 3, 3)
        .expect("second page");
    assert_eq!(second_page.len(), 2);
}

#[test]
fn bulletin_cleanup() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path())
        .without_world_seed()
        .open()
        .expect("store");

    // Create a bulletin board
    let board = BulletinBoard::new("test_board", "Test Board", "Test", "test_room");
    store.put_bulletin_board(board).expect("put board");

    // Post 10 messages
    for i in 1..=10 {
        let message = BulletinMessage::new(
            &format!("user_{}", i),
            &format!("Subject {}", i),
            &format!("Message body {}", i),
            "test_board",
        );
        store.post_bulletin(message).expect("post message");
    }

    // Verify we have 10 messages
    let count_before = store.count_bulletins("test_board").expect("count before");
    assert_eq!(count_before, 10);

    // Clean up to keep only 5 messages
    let removed = store.cleanup_bulletins("test_board", 5).expect("cleanup");
    assert_eq!(removed, 5);

    // Verify we now have 5 messages
    let count_after = store.count_bulletins("test_board").expect("count after");
    assert_eq!(count_after, 5);
}

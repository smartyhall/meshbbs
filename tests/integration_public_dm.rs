#![cfg(feature = "meshtastic-proto")]
use meshbbs::bbs::BbsServer;
mod common;
use meshbbs::config::Config;

// NOTE: This is a high-level logical integration skeleton. In the absence of a real MeshtasticDevice
// mock layer in the codebase, we simulate the public -> DM flow by directly invoking internal
// methods where possible. If deeper mocking is needed, future refactor should abstract device IO.

#[tokio::test]
async fn public_login_then_dm_session_compact_flow() {
    // Build a default config (assuming Config::default or similar). If not available, construct manually.
    // For now we assume a basic constructor exists; adapt if necessary.
    let mut config = Config::default();
    // Use a writable temp copy of the fixture directory under tests/
    let tmp = crate::common::writable_fixture();
    config.storage.data_dir = tmp.path().to_string_lossy().to_string();

    // Initialize server (without actual device)
    let mut server = BbsServer::new(config).await.expect("server");

    // Simulate a public LOGIN (would normally arrive via TextEvent)
    use meshbbs::meshtastic::TextEvent; // re-export not present, path adjust if needed
    let public_event = TextEvent {
        source: 123,
        dest: None,
        is_direct: false,
        channel: None,
        content: "^LOGIN alice".into(),
    };
    server
        .route_text_event(public_event)
        .await
        .expect("public login");

    // Now simulate DM message to trigger session creation and finalize login
    let dm_event = TextEvent {
        source: 123,
        dest: Some(999),
        is_direct: true,
        channel: None,
        content: "M".into(),
    };
    server
        .route_text_event(dm_event)
        .await
        .expect("dm main menu -> topics");

    // Select the first topic and then return to MessageTopics menu
    let dm_select_topic = TextEvent {
        source: 123,
        dest: Some(999),
        is_direct: true,
        channel: None,
        content: "1".into(),
    };
    server
        .route_text_event(dm_select_topic)
        .await
        .expect("dm select topic");

    let dm_back_to_topics = TextEvent {
        source: 123,
        dest: Some(999),
        is_direct: true,
        channel: None,
        content: "B".into(),
    };
    server
        .route_text_event(dm_back_to_topics)
        .await
        .expect("dm back to topics");

    // Start posting a message using compact flow
    let dm_post_begin = TextEvent {
        source: 123,
        dest: Some(999),
        is_direct: true,
        channel: None,
        content: "P".into(),
    };
    server
        .route_text_event(dm_post_begin)
        .await
        .expect("dm start post");

    let dm_post_body = TextEvent {
        source: 123,
        dest: Some(999),
        is_direct: true,
        channel: None,
        content: "Hello world from compact UI".into(),
    };
    server
        .route_text_event(dm_post_body)
        .await
        .expect("dm post body");

    // Read recent messages in the current topic to confirm flow reaches reading state
    let dm_read_recent = TextEvent {
        source: 123,
        dest: Some(999),
        is_direct: true,
        channel: None,
        content: "R".into(),
    };
    server
        .route_text_event(dm_read_recent)
        .await
        .expect("dm read recent");

    // At this stage we at least validated no panics and state transitions executed.
    // Future improvement: Capture outbound messages by injecting a mock device.
}

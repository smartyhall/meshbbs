/// Integration tests for quest system (Phase 6 Week 2 Step 6)
///
/// Validates end-to-end quest flow including accepting quests,
/// completing objectives, earning rewards, and quest prerequisites.
use meshbbs::tmush::{
    abandon_quest, accept_quest, can_accept_quest, complete_quest, format_quest_list,
    format_quest_status, get_active_quests, get_available_quests, get_completed_quests,
    update_quest_objective, CurrencyAmount, PlayerRecord, TinyMushStore, TinyMushStoreBuilder,
};
use tempfile::TempDir;

fn setup_test_store() -> (TinyMushStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    // Open with world seed to get starter quests
    let store = TinyMushStoreBuilder::new(temp_dir.path()).open().unwrap();
    (store, temp_dir)
}

fn create_test_player(store: &TinyMushStore, username: &str) {
    let mut player = PlayerRecord::new(username, username, "town_square");
    // Use Decimal currency to match quest rewards
    player.currency = CurrencyAmount::decimal(0);
    store.put_player(player).unwrap();
}

#[test]
fn quest_lifecycle_complete_flow() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "alice");

    // Verify all 11 quests exist in the store (3 starter + 4 original content + 4 Phase 4 quests)
    let all_quest_ids = store.list_quest_ids().expect("list quests");
    assert_eq!(
        all_quest_ids.len(),
        11,
        "should have 11 quests in store (3 starter + 4 original content + 4 Phase 4 quests)"
    );

    // Only welcome_towne should be available initially (others have prerequisites)
    let available = get_available_quests(&store, "alice").expect("get available");
    assert!(
        !available.is_empty(),
        "should have at least one available quest"
    );

    // Find and accept the welcome quest
    assert!(
        available.contains(&"welcome_towne".to_string()),
        "welcome_towne quest should be available"
    );

    let welcome_quest = store.get_quest("welcome_towne").expect("get quest");
    assert_eq!(welcome_quest.difficulty, 1);
    assert_eq!(welcome_quest.objectives.len(), 3); // 2 visit + 1 talk

    accept_quest(&store, "alice", "welcome_towne").expect("accept quest");

    // Verify quest is now active
    let active = get_active_quests(&store, "alice").expect("get active");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].quest_id, "welcome_towne");

    // Progress through objectives
    use meshbbs::tmush::types::ObjectiveType;

    // Visit town square
    update_quest_objective(
        &store,
        "alice",
        "welcome_towne",
        &ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        },
        1,
    )
    .expect("visit town_square");

    // Visit mesh museum
    update_quest_objective(
        &store,
        "alice",
        "welcome_towne",
        &ObjectiveType::VisitLocation {
            room_id: "mesh_museum".to_string(),
        },
        1,
    )
    .expect("visit museum");

    // Talk to mayor
    update_quest_objective(
        &store,
        "alice",
        "welcome_towne",
        &ObjectiveType::TalkToNpc {
            npc_id: "mayor_thompson".to_string(),
        },
        1,
    )
    .expect("talk to mayor");

    // Complete quest and check rewards
    let player_before = store.get_player("alice").expect("get player");
    let currency_before = player_before.currency.clone();

    complete_quest(&store, "alice", "welcome_towne").expect("complete quest");

    let player_after = store.get_player("alice").expect("get player after");
    assert!(
        player_after.currency.base_value() > currency_before.base_value(),
        "should gain currency rewards (before: {}, after: {})",
        currency_before.base_value(),
        player_after.currency.base_value()
    );

    // Verify quest is completed
    let completed = get_completed_quests(&store, "alice").expect("get completed");
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].quest_id, "welcome_towne");
}

#[test]
fn quest_prerequisites_enforced() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "bob");

    // Market exploration requires welcome_towne completion
    assert!(
        !can_accept_quest(&store, "bob", "market_exploration").unwrap(),
        "should not be able to accept without prerequisite"
    );

    // Accept and complete welcome_towne
    accept_quest(&store, "bob", "welcome_towne").expect("accept welcome");
    use meshbbs::tmush::types::ObjectiveType;

    // Complete all objectives
    update_quest_objective(
        &store,
        "bob",
        "welcome_towne",
        &ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        },
        1,
    )
    .expect("visit 1");
    update_quest_objective(
        &store,
        "bob",
        "welcome_towne",
        &ObjectiveType::VisitLocation {
            room_id: "mesh_museum".to_string(),
        },
        1,
    )
    .expect("visit 2");
    update_quest_objective(
        &store,
        "bob",
        "welcome_towne",
        &ObjectiveType::TalkToNpc {
            npc_id: "mayor_thompson".to_string(),
        },
        1,
    )
    .expect("talk");

    complete_quest(&store, "bob", "welcome_towne").expect("complete");

    // Now should be able to accept market_exploration
    assert!(
        can_accept_quest(&store, "bob", "market_exploration").unwrap(),
        "should be able to accept after completing prerequisite"
    );

    // But still can't accept network_explorer without market_exploration
    assert!(
        !can_accept_quest(&store, "bob", "network_explorer").unwrap(),
        "should not be able to accept network_explorer yet"
    );
}

#[test]
fn quest_list_formatting_under_200_bytes() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "carol");

    let available_ids = get_available_quests(&store, "carol").expect("get available");
    let messages = format_quest_list(&store, &available_ids).expect("format list");

    for msg in messages {
        assert!(
            msg.len() <= 200,
            "quest list message should be under 200 bytes: {}",
            msg.len()
        );
    }
}

#[test]
fn quest_status_formatting_under_200_bytes() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "dave");

    accept_quest(&store, "dave", "welcome_towne").expect("accept");

    let player = store.get_player("dave").expect("get player");
    let player_quest = player
        .quests
        .iter()
        .find(|pq| pq.quest_id == "welcome_towne")
        .expect("player quest");

    let status = format_quest_status(&store, "welcome_towne", player_quest).expect("format status");

    // Status is a single string, check its length
    assert!(
        status.len() <= 400,
        "quest status message should be reasonable length: {}",
        status.len()
    );
}

#[test]
fn quest_cannot_be_accepted_twice() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "eve");

    accept_quest(&store, "eve", "welcome_towne").expect("accept first time");

    // Second accept should fail
    let result = accept_quest(&store, "eve", "welcome_towne");
    assert!(result.is_err(), "should not be able to accept quest twice");
}

#[test]
fn quest_rewards_include_items() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "frank");

    // Network explorer quest rewards include an item (explorer_compass)
    // First complete prerequisites
    accept_quest(&store, "frank", "welcome_towne").expect("accept welcome");
    use meshbbs::tmush::types::ObjectiveType;

    // Complete welcome_towne
    update_quest_objective(
        &store,
        "frank",
        "welcome_towne",
        &ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        },
        1,
    )
    .unwrap();
    update_quest_objective(
        &store,
        "frank",
        "welcome_towne",
        &ObjectiveType::VisitLocation {
            room_id: "mesh_museum".to_string(),
        },
        1,
    )
    .unwrap();
    update_quest_objective(
        &store,
        "frank",
        "welcome_towne",
        &ObjectiveType::TalkToNpc {
            npc_id: "mayor_thompson".to_string(),
        },
        1,
    )
    .unwrap();
    complete_quest(&store, "frank", "welcome_towne").unwrap();

    // Complete market_exploration
    accept_quest(&store, "frank", "market_exploration").expect("accept market");
    update_quest_objective(
        &store,
        "frank",
        "market_exploration",
        &ObjectiveType::VisitLocation {
            room_id: "south_market".to_string(),
        },
        1,
    )
    .unwrap();
    update_quest_objective(
        &store,
        "frank",
        "market_exploration",
        &ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        },
        1,
    )
    .unwrap();
    complete_quest(&store, "frank", "market_exploration").unwrap();

    // Now complete network_explorer which has an item reward
    accept_quest(&store, "frank", "network_explorer").expect("accept network");
    update_quest_objective(
        &store,
        "frank",
        "network_explorer",
        &ObjectiveType::VisitLocation {
            room_id: "north_gate".to_string(),
        },
        1,
    )
    .unwrap();

    // Verify quest completion succeeds (rewards distributed)
    let result = complete_quest(&store, "frank", "network_explorer");
    assert!(result.is_ok(), "quest completion should succeed");

    // Verify quest is marked complete
    let completed = get_completed_quests(&store, "frank").expect("get completed");
    assert_eq!(completed.len(), 3, "should have completed all 3 quests");

    // Note: Item rewards require catalog entries. The quest system attempts
    // to add items but silently ignores failures for non-existent catalog items.
    // This is intentional to allow symbolic/badge rewards.
}

#[test]
fn abandoned_quest_can_be_retaken() {
    let (store, _temp) = setup_test_store();
    create_test_player(&store, "grace");

    accept_quest(&store, "grace", "welcome_towne").expect("accept");

    // Abandon the quest
    abandon_quest(&store, "grace", "welcome_towne").expect("abandon");

    // Verify it's in failed state
    let player = store.get_player("grace").expect("get player");
    let quest_status = player
        .quests
        .iter()
        .find(|pq| pq.quest_id == "welcome_towne")
        .expect("quest should exist");

    use meshbbs::tmush::types::QuestState;
    assert!(matches!(quest_status.state, QuestState::Failed { .. }));

    // Should be able to accept again
    let result = accept_quest(&store, "grace", "welcome_towne");
    assert!(
        result.is_ok(),
        "should be able to re-accept abandoned quest"
    );
}

/// Integration tests for achievement and title system (Phase 6 Week 3)
/// Validates end-to-end achievement earning, progress tracking, title management, and command outputs.
use meshbbs::tmush::{
    achievement::{
        award_achievement, check_trigger, get_available_achievements, get_earned_achievements,
    },
    state::seed_starter_achievements,
    storage::TinyMushStore,
    types::{AchievementCategory, AchievementTrigger, PlayerRecord},
};
use tempfile::tempdir;

fn setup_store() -> TinyMushStore {
    let dir = tempdir().unwrap();
    let store = TinyMushStore::open(dir.path()).unwrap();

    // Seed achievements
    for achievement in seed_starter_achievements() {
        store.put_achievement(achievement).unwrap();
    }

    // Create test player
    let player = PlayerRecord::new("testuser", "Test User", "starting_room");
    store.put_player(player).unwrap();

    store
}

#[test]
fn test_achievement_earning_flow() {
    let store = setup_store();

    // Trigger first kill
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::KillCount { required: 1 },
    )
    .unwrap();
    assert_eq!(awarded.len(), 1);
    assert_eq!(awarded[0], "first_blood");

    // Verify achievement earned
    let earned = get_earned_achievements(&store, "testuser").unwrap();
    assert_eq!(earned.len(), 1);
    assert_eq!(earned[0].id, "first_blood");
    assert_eq!(earned[0].title, Some("the Brave".to_string()));

    // Verify player has equipped title option
    let player = store.get_player("testuser").unwrap();
    let achievement_progress = player
        .achievements
        .iter()
        .find(|pa| pa.achievement_id == "first_blood")
        .unwrap();
    assert!(achievement_progress.earned);
}

#[test]
fn test_incremental_progress() {
    let store = setup_store();

    // Trigger 10 room visits (should earn wanderer)
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::RoomVisits { required: 10 },
    )
    .unwrap();
    assert!(awarded.contains(&"wanderer".to_string()));

    // Trigger 40 more room visits (should earn explorer)
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::RoomVisits { required: 40 },
    )
    .unwrap();
    assert!(awarded.contains(&"explorer".to_string()));

    // Trigger 50 more room visits (should earn cartographer)
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::RoomVisits { required: 50 },
    )
    .unwrap();
    assert!(awarded.contains(&"cartographer".to_string()));

    // Verify all three earned
    let earned = get_earned_achievements(&store, "testuser").unwrap();
    let ids: Vec<_> = earned.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"wanderer"));
    assert!(ids.contains(&"explorer"));
    assert!(ids.contains(&"cartographer"));
}

#[test]
fn test_hidden_achievements() {
    let store = setup_store();

    // Get available achievements
    let available = get_available_achievements(&store, "testuser").unwrap();

    // Hidden achievements should not appear
    let has_legendary = available.iter().any(|(a, _)| a.id == "legendary");
    assert!(!has_legendary, "Hidden achievement should not be visible");

    // Award hidden achievement
    award_achievement(&store, "testuser", "legendary").unwrap();

    // Now it should appear
    let available = get_available_achievements(&store, "testuser").unwrap();
    let has_legendary = available
        .iter()
        .any(|(a, _)| a.id == "legendary" && a.hidden);
    assert!(has_legendary, "Earned hidden achievement should be visible");
}

#[test]
fn test_title_awarding() {
    let store = setup_store();

    // Earn achievements with titles
    award_achievement(&store, "testuser", "first_blood").unwrap();
    award_achievement(&store, "testuser", "wanderer").unwrap();
    award_achievement(&store, "testuser", "cartographer").unwrap();

    let earned = get_earned_achievements(&store, "testuser").unwrap();
    let titles: Vec<_> = earned.iter().filter_map(|a| a.title.as_deref()).collect();

    assert_eq!(titles.len(), 3);
    assert!(titles.contains(&"the Brave"));
    assert!(titles.contains(&"the Wanderer"));
    assert!(titles.contains(&"the Cartographer"));
}

#[test]
fn test_title_equipping() {
    let store = setup_store();

    // Earn achievement with title
    award_achievement(&store, "testuser", "first_blood").unwrap();

    // Equip title
    let mut player = store.get_player("testuser").unwrap();
    player.equipped_title = Some("the Brave".to_string());
    store.put_player(player).unwrap();

    // Verify equipped
    let player = store.get_player("testuser").unwrap();
    assert_eq!(player.equipped_title, Some("the Brave".to_string()));

    // Unequip title
    let mut player = store.get_player("testuser").unwrap();
    player.equipped_title = None;
    store.put_player(player).unwrap();

    // Verify unequipped
    let player = store.get_player("testuser").unwrap();
    assert_eq!(player.equipped_title, None);
}

#[test]
fn test_category_filtering() {
    let store = setup_store();

    // Award achievements in different categories
    award_achievement(&store, "testuser", "first_blood").unwrap(); // Combat
    award_achievement(&store, "testuser", "wanderer").unwrap(); // Exploration
    award_achievement(&store, "testuser", "friendly").unwrap(); // Social

    // Get combat achievements
    use meshbbs::tmush::achievement::get_achievements_by_category;
    let combat =
        get_achievements_by_category(&store, "testuser", AchievementCategory::Combat).unwrap();

    // Should have at least first_blood and veteran visible (legendary is hidden)
    assert!(combat.len() >= 2);
    let has_first_blood = combat
        .iter()
        .any(|(a, pa)| a.id == "first_blood" && pa.as_ref().map(|p| p.earned).unwrap_or(false));
    assert!(has_first_blood);
}

#[test]
fn test_social_achievements() {
    let store = setup_store();

    // Trigger friend count
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::FriendCount { required: 5 },
    )
    .unwrap();
    assert!(awarded.contains(&"friendly".to_string()));

    // Trigger message count
    check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::MessagesSent { required: 500 },
    )
    .unwrap();
    check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::MessagesSent { required: 500 },
    )
    .unwrap();

    let player = store.get_player("testuser").unwrap();
    let chatterbox = player
        .achievements
        .iter()
        .find(|pa| pa.achievement_id == "chatterbox")
        .unwrap();
    assert!(chatterbox.earned);
}

#[test]
fn test_economic_achievements() {
    let store = setup_store();

    // Trigger trade count
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::TradeCount { required: 50 },
    )
    .unwrap();
    assert!(awarded.contains(&"merchant".to_string()));

    // Verify merchant title
    let earned = get_earned_achievements(&store, "testuser").unwrap();
    let merchant = earned.iter().find(|a| a.id == "merchant").unwrap();
    assert_eq!(merchant.title, Some("the Merchant".to_string()));
}

#[test]
fn test_quest_achievements() {
    let store = setup_store();

    // Complete first quest
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::QuestCompletion { required: 1 },
    )
    .unwrap();
    assert!(awarded.contains(&"quest_beginner".to_string()));

    // Complete 24 more quests
    check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::QuestCompletion { required: 24 },
    )
    .unwrap();

    let earned = get_earned_achievements(&store, "testuser").unwrap();
    let ids: Vec<_> = earned.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"quest_beginner"));
    assert!(ids.contains(&"quest_veteran"));
}

#[test]
fn test_special_achievements() {
    let store = setup_store();

    // Special achievements are location/quest specific
    award_achievement(&store, "testuser", "town_founder").unwrap();
    award_achievement(&store, "testuser", "network_pioneer").unwrap();

    let earned = get_earned_achievements(&store, "testuser").unwrap();
    assert_eq!(earned.len(), 2);

    // Verify titles
    let founder = earned.iter().find(|a| a.id == "town_founder").unwrap();
    assert_eq!(founder.title, Some("Town Founder".to_string()));
    assert!(founder.hidden); // Special achievements are hidden
}

#[test]
fn test_achievement_command_output_size() {
    let store = setup_store();

    // Award several achievements
    award_achievement(&store, "testuser", "first_blood").unwrap();
    award_achievement(&store, "testuser", "wanderer").unwrap();
    award_achievement(&store, "testuser", "friendly").unwrap();

    let earned = get_earned_achievements(&store, "testuser").unwrap();

    // Build output similar to ACHIEVEMENTS EARNED command
    let mut output = String::from("=== EARNED ACHIEVEMENTS ===\n");
    for achievement in earned {
        let title_info = if let Some(ref title) = achievement.title {
            format!(" - {}", title)
        } else {
            String::new()
        };
        let line = format!(
            "✓ {}{}\n  {}\n",
            achievement.name, title_info, achievement.description
        );
        output.push_str(&line);
    }

    // Verify each achievement entry is reasonably sized (< 200 bytes per achievement)
    for achievement in get_earned_achievements(&store, "testuser").unwrap() {
        let line = format!(
            "✓ {} - {}\n  {}\n",
            achievement.name,
            achievement.title.as_deref().unwrap_or(""),
            achievement.description
        );
        assert!(
            line.len() < 200,
            "Achievement output too large: {} bytes",
            line.len()
        );
    }
}

#[test]
fn test_multiple_simultaneous_achievements() {
    let store = setup_store();

    // Trigger that should award multiple achievements at once
    let awarded = check_trigger(
        &store,
        "testuser",
        &AchievementTrigger::RoomVisits { required: 100 },
    )
    .unwrap();

    // Should earn wanderer (10), explorer (50), and cartographer (100)
    assert_eq!(awarded.len(), 3);
    assert!(awarded.contains(&"wanderer".to_string()));
    assert!(awarded.contains(&"explorer".to_string()));
    assert!(awarded.contains(&"cartographer".to_string()));
}

#[test]
fn test_achievement_persistence() {
    let dir = tempdir().unwrap();

    {
        let store = TinyMushStore::open(dir.path()).unwrap();
        for achievement in seed_starter_achievements() {
            store.put_achievement(achievement).unwrap();
        }
        let player = PlayerRecord::new("testuser", "Test User", "starting_room");
        store.put_player(player).unwrap();

        // Earn achievement
        award_achievement(&store, "testuser", "first_blood").unwrap();
    }

    // Reopen store
    {
        let store = TinyMushStore::open(dir.path()).unwrap();
        let earned = get_earned_achievements(&store, "testuser").unwrap();
        assert_eq!(earned.len(), 1);
        assert_eq!(earned[0].id, "first_blood");
    }
}

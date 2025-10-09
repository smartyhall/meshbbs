/// Quest progression and management logic for Phase 6 Week 2
///
/// This module provides functions for quest lifecycle management including
/// accepting quests, tracking objective progress, and completing quests.

use crate::tmush::errors::TinyMushError;
use crate::tmush::storage::TinyMushStore;
use crate::tmush::types::{InventoryConfig, ObjectiveType, PlayerQuest, TransactionReason};

/// Check if player can accept a quest (prerequisites met)
pub fn can_accept_quest(
    store: &TinyMushStore,
    username: &str,
    quest_id: &str,
) -> Result<bool, TinyMushError> {
    let quest = store.get_quest(quest_id)?;
    let player = store.get_player(username)?;

    // Check if already active or completed
    for player_quest in &player.quests {
        if player_quest.quest_id == quest_id {
            if player_quest.is_active() {
                return Ok(false); // Already active
            }
            if player_quest.is_complete() {
                return Ok(false); // Already completed
            }
        }
    }

    // Check prerequisites
    for prereq_id in &quest.prerequisites {
        let has_completed = player.quests.iter().any(|pq| {
            pq.quest_id == *prereq_id && pq.is_complete()
        });
        if !has_completed {
            return Ok(false); // Missing prerequisite
        }
    }

    Ok(true)
}

/// Accept a quest and add it to player's active quests
pub fn accept_quest(
    store: &TinyMushStore,
    username: &str,
    quest_id: &str,
) -> Result<(), TinyMushError> {
    if !can_accept_quest(store, username, quest_id)? {
        return Err(TinyMushError::InvalidCurrency(
            "Cannot accept quest (prerequisites not met or already accepted)".to_string(),
        ));
    }

    let quest = store.get_quest(quest_id)?;
    let mut player = store.get_player(username)?;

    // Create player quest with fresh objectives
    let player_quest = PlayerQuest::new(quest_id, quest.objectives.clone());
    player.quests.push(player_quest);
    
    store.put_player(player)?;
    Ok(())
}

/// Update quest objective progress
pub fn update_quest_objective(
    store: &TinyMushStore,
    username: &str,
    quest_id: &str,
    objective_type: &ObjectiveType,
    progress: u32,
) -> Result<bool, TinyMushError> {
    let mut player = store.get_player(username)?;
    
    // Find the quest
    let quest_pos = player.quests.iter().position(|pq| {
        pq.quest_id == quest_id && pq.is_active()
    });

    if let Some(pos) = quest_pos {
        let quest = &mut player.quests[pos];
        
        // Find matching objective
        for objective in &mut quest.objectives {
            if objective.objective_type == *objective_type {
                objective.increment_progress(progress);
            }
        }

        // Check if all objectives complete
        let all_complete = quest.all_objectives_complete();
        store.put_player(player)?;
        return Ok(all_complete);
    }

    Ok(false)
}

/// Mark quest as complete and distribute rewards
pub fn complete_quest(
    store: &TinyMushStore,
    username: &str,
    quest_id: &str,
) -> Result<(), TinyMushError> {
    let quest = store.get_quest(quest_id)?;
    let mut player = store.get_player(username)?;

    // Find the quest and verify all objectives complete
    let quest_pos = player.quests.iter().position(|pq| {
        pq.quest_id == quest_id && pq.is_active() && pq.all_objectives_complete()
    });

    if let Some(pos) = quest_pos {
        // Distribute rewards BEFORE marking complete and saving
        // (grant_currency and player_add_item will load player fresh from DB)
        if let Some(ref currency) = quest.rewards.currency {
            store.grant_currency(username, currency, TransactionReason::QuestReward)?;
        }

        // Grant reward items
        let config = InventoryConfig::default();
        for item_id in &quest.rewards.items {
            // Note: player_add_item may fail if item doesn't exist in catalog,
            // but we don't want to fail the entire quest completion.
            // Items may be symbolic rewards or badges that don't need catalog entries.
            let _ = store.player_add_item(username, item_id, 1, &config);
        }

        // Reload player after rewards, mark quest complete, and save
        player = store.get_player(username)?;
        player.quests[pos].mark_complete();
        store.put_player(player)?;

        Ok(())
    } else {
        Err(TinyMushError::NotFound(format!(
            "Active quest not found or objectives not complete: {}",
            quest_id
        )))
    }
}

/// Abandon a quest (remove from active quests)
pub fn abandon_quest(
    store: &TinyMushStore,
    username: &str,
    quest_id: &str,
) -> Result<(), TinyMushError> {
    let mut player = store.get_player(username)?;

    let quest_pos = player.quests.iter().position(|pq| {
        pq.quest_id == quest_id && pq.is_active()
    });

    if let Some(pos) = quest_pos {
        player.quests[pos].mark_failed();
        store.put_player(player)?;
        Ok(())
    } else {
        Err(TinyMushError::NotFound(format!(
            "Active quest not found: {}",
            quest_id
        )))
    }
}

/// Get list of available quests for player (prerequisites met, not yet accepted)
pub fn get_available_quests(
    store: &TinyMushStore,
    username: &str,
) -> Result<Vec<String>, TinyMushError> {
    let all_quest_ids = store.list_quest_ids()?;
    let mut available = Vec::new();

    for quest_id in all_quest_ids {
        if can_accept_quest(store, username, &quest_id)? {
            available.push(quest_id);
        }
    }

    Ok(available)
}

/// Get player's active quests
pub fn get_active_quests(
    store: &TinyMushStore,
    username: &str,
) -> Result<Vec<PlayerQuest>, TinyMushError> {
    let player = store.get_player(username)?;
    Ok(player.quests.into_iter().filter(|pq| pq.is_active()).collect())
}

/// Get player's completed quests
pub fn get_completed_quests(
    store: &TinyMushStore,
    username: &str,
) -> Result<Vec<PlayerQuest>, TinyMushError> {
    let player = store.get_player(username)?;
    Ok(player.quests.into_iter().filter(|pq| pq.is_complete()).collect())
}

/// Format quest status message for display (<200 bytes)
pub fn format_quest_status(
    store: &TinyMushStore,
    quest_id: &str,
    player_quest: &PlayerQuest,
) -> Result<String, TinyMushError> {
    let quest = store.get_quest(quest_id)?;
    
    let mut output = format!("=== {} ===\n", quest.name);
    output.push_str(&format!("Difficulty: {}/5\n", quest.difficulty));
    
    for obj in &player_quest.objectives {
        let status = if obj.is_complete() { "✓" } else { " " };
        output.push_str(&format!(
            "[{}] {} [{}/{}]\n",
            status, obj.description, obj.progress, obj.required
        ));
    }

    // Truncate if needed
    if output.len() > 190 {
        output.truncate(187);
        output.push_str("...");
    }

    Ok(output)
}

/// Format quest list for display (<200 bytes per message)
pub fn format_quest_list(
    store: &TinyMushStore,
    quest_ids: &[String],
) -> Result<Vec<String>, TinyMushError> {
    let mut messages = Vec::new();
    let mut current = String::from("=== AVAILABLE QUESTS ===\n");

    for (idx, quest_id) in quest_ids.iter().enumerate() {
        if let Ok(quest) = store.get_quest(quest_id) {
            let line = format!("{}. {} (Lv{})\n", idx + 1, quest.name, quest.difficulty);
            
            if current.len() + line.len() > 190 {
                // Start new message
                messages.push(current.clone());
                current = line;
            } else {
                current.push_str(&line);
            }
        }
    }

    if !current.is_empty() && current != "=== AVAILABLE QUESTS ===\n" {
        messages.push(current);
    }

    if messages.is_empty() {
        messages.push("No quests available.".to_string());
    }

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::storage::TinyMushStoreBuilder;
    use crate::tmush::types::{CurrencyAmount, ObjectiveType, QuestObjective, QuestRecord};
    use tempfile::TempDir;

    fn setup_test_store() -> (TempDir, TinyMushStore) {
        let dir = TempDir::new().expect("tempdir");
        let store = TinyMushStoreBuilder::new(dir.path())
            .without_world_seed()
            .open()
            .expect("store");
        (dir, store)
    }

    fn create_test_player(store: &TinyMushStore, username: &str) {
        let player = crate::tmush::types::PlayerRecord::new(username, username, "gazebo");
        store.put_player(player).expect("put player");
    }

    fn create_test_quest(store: &TinyMushStore, quest_id: &str, npc_id: &str) -> QuestRecord {
        let quest = QuestRecord::new(quest_id, "Test Quest", "A test quest", npc_id, 1)
            .with_objective(QuestObjective::new(
                "Visit town square",
                ObjectiveType::VisitLocation {
                    room_id: "town_square".to_string(),
                },
                1,
            ))
            .with_reward_currency(CurrencyAmount::Decimal { minor_units: 500 })
            .with_reward_experience(100);
        store.put_quest(quest.clone()).expect("put quest");
        quest
    }

    #[test]
    fn test_accept_quest() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        assert!(can_accept_quest(&store, "alice", "quest1").unwrap());
        accept_quest(&store, "alice", "quest1").unwrap();

        let player = store.get_player("alice").unwrap();
        assert_eq!(player.quests.len(), 1);
        assert!(player.quests[0].is_active());
    }

    #[test]
    fn test_cannot_accept_twice() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        accept_quest(&store, "alice", "quest1").unwrap();
        assert!(!can_accept_quest(&store, "alice", "quest1").unwrap());
        assert!(accept_quest(&store, "alice", "quest1").is_err());
    }

    #[test]
    fn test_update_quest_objective() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        accept_quest(&store, "alice", "quest1").unwrap();

        let objective_type = ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        };
        let all_complete = update_quest_objective(&store, "alice", "quest1", &objective_type, 1).unwrap();
        assert!(all_complete);

        let player = store.get_player("alice").unwrap();
        assert_eq!(player.quests[0].objectives[0].progress, 1);
    }

    #[test]
    fn test_complete_quest() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        accept_quest(&store, "alice", "quest1").unwrap();

        let objective_type = ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        };
        update_quest_objective(&store, "alice", "quest1", &objective_type, 1).unwrap();

        complete_quest(&store, "alice", "quest1").unwrap();

        let player = store.get_player("alice").unwrap();
        assert!(player.quests[0].is_complete());
        assert_eq!(player.currency.base_value(), 500); // Reward granted
    }

    #[test]
    fn test_abandon_quest() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        accept_quest(&store, "alice", "quest1").unwrap();
        abandon_quest(&store, "alice", "quest1").unwrap();

        let player = store.get_player("alice").unwrap();
        assert!(matches!(player.quests[0].state, crate::tmush::types::QuestState::Failed { .. }));
    }

    #[test]
    fn test_quest_prerequisites() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        let quest2 = QuestRecord::new("quest2", "Quest 2", "Second quest", "npc1", 2)
            .with_prerequisite("quest1")
            .with_objective(QuestObjective::new(
                "Do something",
                ObjectiveType::VisitLocation {
                    room_id: "anywhere".to_string(),
                },
                1,
            ));
        store.put_quest(quest2).expect("put quest2");

        // Cannot accept quest2 without completing quest1
        assert!(!can_accept_quest(&store, "alice", "quest2").unwrap());

        // Accept and complete quest1
        accept_quest(&store, "alice", "quest1").unwrap();
        let objective_type = ObjectiveType::VisitLocation {
            room_id: "town_square".to_string(),
        };
        update_quest_objective(&store, "alice", "quest1", &objective_type, 1).unwrap();
        complete_quest(&store, "alice", "quest1").unwrap();

        // Now can accept quest2
        assert!(can_accept_quest(&store, "alice", "quest2").unwrap());
    }

    #[test]
    fn test_get_available_quests() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");
        create_test_quest(&store, "quest2", "npc1");

        let available = get_available_quests(&store, "alice").unwrap();
        assert_eq!(available.len(), 2);

        // Accept one
        accept_quest(&store, "alice", "quest1").unwrap();

        let available = get_available_quests(&store, "alice").unwrap();
        assert_eq!(available.len(), 1); // Only quest2 left
    }

    #[test]
    fn test_format_quest_status() {
        let (_dir, store) = setup_test_store();
        create_test_player(&store, "alice");
        create_test_quest(&store, "quest1", "npc1");

        accept_quest(&store, "alice", "quest1").unwrap();
        let player = store.get_player("alice").unwrap();
        let player_quest = &player.quests[0];

        let status = format_quest_status(&store, "quest1", player_quest).unwrap();
        assert!(status.contains("Test Quest"));
        assert!(status.contains("Difficulty: 1/5"));
        assert!(status.contains("[0/1]"));
        assert!(status.len() < 200);
    }

    #[test]
    fn test_format_quest_list() {
        let (_dir, store) = setup_test_store();
        create_test_quest(&store, "quest1", "npc1");
        create_test_quest(&store, "quest2", "npc1");

        let quest_ids = vec!["quest1".to_string(), "quest2".to_string()];
        let messages = format_quest_list(&store, &quest_ids).unwrap();
        
        assert!(!messages.is_empty());
        assert!(messages[0].contains("Test Quest"));
        assert!(messages[0].len() < 200);
    }
}

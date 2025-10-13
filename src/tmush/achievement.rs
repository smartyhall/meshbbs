/// Achievement tracking and progression logic for Phase 6 Week 3
///
/// This module provides functions for achievement tracking including
/// checking triggers, updating progress, and awarding achievements.
use crate::tmush::errors::TinyMushError;
use crate::tmush::storage::TinyMushStore;
use crate::tmush::types::{AchievementCategory, AchievementRecord, AchievementTrigger, PlayerAchievement};

/// Check if a trigger event should update achievement progress
pub fn check_trigger(
    store: &TinyMushStore,
    username: &str,
    trigger: &AchievementTrigger,
) -> Result<Vec<String>, TinyMushError> {
    let mut player = store.get_player(username)?;
    let mut awarded = Vec::new();

    // Get all achievements
    let achievement_ids = store.list_achievement_ids()?;
    
    for achievement_id in achievement_ids {
        let achievement = store.get_achievement(&achievement_id)?;
        
        // Check if trigger matches
        if !triggers_match(&achievement.trigger, trigger) {
            continue;
        }

        // Find or create player achievement progress
        let player_achievement = player.achievements.iter_mut()
            .find(|pa| pa.achievement_id == achievement_id);

        match player_achievement {
            Some(pa) if pa.earned => {
                // Already earned, skip
                continue;
            }
            Some(pa) => {
                // Update progress
                let progress_increment = extract_progress_value(trigger);
                pa.increment(progress_increment);
                
                // Check if earned
                if is_achievement_earned(&achievement.trigger, pa.progress) {
                    pa.mark_earned();
                    awarded.push(achievement_id.clone());
                }
            }
            None => {
                // Create new progress
                let progress_increment = extract_progress_value(trigger);
                let mut new_achievement = PlayerAchievement::new(&achievement_id);
                new_achievement.increment(progress_increment);
                
                // Check if earned immediately (e.g., single event achievements)
                if is_achievement_earned(&achievement.trigger, new_achievement.progress) {
                    new_achievement.mark_earned();
                    awarded.push(achievement_id.clone());
                }
                
                player.achievements.push(new_achievement);
            }
        }
    }

    store.put_player(player)?;
    Ok(awarded)
}

/// Update achievement progress directly (for manual/scripted triggers)
pub fn update_achievement_progress(
    store: &TinyMushStore,
    username: &str,
    achievement_id: &str,
    progress: u32,
) -> Result<bool, TinyMushError> {
    let achievement = store.get_achievement(achievement_id)?;
    let mut player = store.get_player(username)?;

    // Find or create player achievement
    let player_achievement = player.achievements.iter_mut()
        .find(|pa| pa.achievement_id == achievement_id);

    let earned = match player_achievement {
        Some(pa) if pa.earned => {
            return Ok(false); // Already earned
        }
        Some(pa) => {
            pa.increment(progress);
            let earned = is_achievement_earned(&achievement.trigger, pa.progress);
            if earned {
                pa.mark_earned();
            }
            earned
        }
        None => {
            let mut new_achievement = PlayerAchievement::new(achievement_id);
            new_achievement.increment(progress);
            let earned = is_achievement_earned(&achievement.trigger, new_achievement.progress);
            if earned {
                new_achievement.mark_earned();
            }
            player.achievements.push(new_achievement);
            earned
        }
    };

    store.put_player(player)?;
    Ok(earned)
}

/// Award an achievement immediately (bypasses progress tracking)
pub fn award_achievement(
    store: &TinyMushStore,
    username: &str,
    achievement_id: &str,
) -> Result<(), TinyMushError> {
    let _achievement = store.get_achievement(achievement_id)?; // Verify exists
    let mut player = store.get_player(username)?;

    // Check if already earned
    let already_earned = player.achievements.iter()
        .any(|pa| pa.achievement_id == achievement_id && pa.earned);

    if already_earned {
        return Err(TinyMushError::InvalidCurrency(
            "Achievement already earned".to_string(),
        ));
    }

    // Find or create player achievement
    if let Some(pa) = player.achievements.iter_mut()
        .find(|pa| pa.achievement_id == achievement_id) {
        pa.mark_earned();
    } else {
        let mut new_achievement = PlayerAchievement::new(achievement_id);
        new_achievement.mark_earned();
        player.achievements.push(new_achievement);
    }

    store.put_player(player)?;
    Ok(())
}

/// Get all earned achievements for a player
pub fn get_earned_achievements(
    store: &TinyMushStore,
    username: &str,
) -> Result<Vec<AchievementRecord>, TinyMushError> {
    let player = store.get_player(username)?;
    let mut earned = Vec::new();

    for player_achievement in &player.achievements {
        if player_achievement.earned {
            if let Ok(achievement) = store.get_achievement(&player_achievement.achievement_id) {
                earned.push(achievement);
            }
        }
    }

    Ok(earned)
}

/// Get all available (non-hidden or earned) achievements with player progress
pub fn get_available_achievements(
    store: &TinyMushStore,
    username: &str,
) -> Result<Vec<(AchievementRecord, Option<PlayerAchievement>)>, TinyMushError> {
    let player = store.get_player(username)?;
    let achievement_ids = store.list_achievement_ids()?;
    let mut available = Vec::new();

    for achievement_id in achievement_ids {
        let achievement = store.get_achievement(&achievement_id)?;
        
        // Skip hidden achievements unless earned
        let player_achievement = player.achievements.iter()
            .find(|pa| pa.achievement_id == achievement_id);
        
        if achievement.hidden {
            if let Some(pa) = player_achievement {
                if pa.earned {
                    available.push((achievement, Some(pa.clone())));
                }
            }
            continue;
        }

        available.push((achievement, player_achievement.cloned()));
    }

    Ok(available)
}

/// Get achievements by category
pub fn get_achievements_by_category(
    store: &TinyMushStore,
    username: &str,
    category: AchievementCategory,
) -> Result<Vec<(AchievementRecord, Option<PlayerAchievement>)>, TinyMushError> {
    let player = store.get_player(username)?;
    let achievements = store.get_achievements_by_category(&category)?;
    let mut result = Vec::new();

    for achievement in achievements {
        let player_achievement = player.achievements.iter()
            .find(|pa| pa.achievement_id == achievement.id)
            .cloned();
        
        // Skip hidden unless earned
        if achievement.hidden {
            if let Some(ref pa) = player_achievement {
                if pa.earned {
                    result.push((achievement, player_achievement));
                }
            }
            continue;
        }

        result.push((achievement, player_achievement));
    }

    Ok(result)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if two triggers match (same type)
fn triggers_match(template: &AchievementTrigger, event: &AchievementTrigger) -> bool {
    use AchievementTrigger::*;
    
    matches!(
        (template, event),
        (KillCount { .. }, KillCount { .. }) |
        (RoomVisits { .. }, RoomVisits { .. }) |
        (FriendCount { .. }, FriendCount { .. }) |
        (QuestCompletion { .. }, QuestCompletion { .. }) |
        (CurrencyEarned { .. }, CurrencyEarned { .. }) |
        (CraftCount { .. }, CraftCount { .. }) |
        (TradeCount { .. }, TradeCount { .. }) |
        (MessagesSent { .. }, MessagesSent { .. }) |
        (VisitLocation { .. }, VisitLocation { .. }) |
        (CompleteQuest { .. }, CompleteQuest { .. })
    )
}

/// Extract progress value from trigger event
fn extract_progress_value(trigger: &AchievementTrigger) -> u32 {
    use AchievementTrigger::*;
    
    match trigger {
        KillCount { required } | RoomVisits { required } | FriendCount { required } | 
        QuestCompletion { required } | CraftCount { required } | 
        TradeCount { required } | MessagesSent { required } => *required,
        CurrencyEarned { amount } => (*amount).max(0) as u32,
        VisitLocation { .. } | CompleteQuest { .. } => 1,
    }
}

/// Check if achievement is earned based on trigger requirement
fn is_achievement_earned(trigger: &AchievementTrigger, progress: u32) -> bool {
    use AchievementTrigger::*;
    
    match trigger {
        KillCount { required } | RoomVisits { required } | FriendCount { required } | 
        QuestCompletion { required } | CraftCount { required } | 
        TradeCount { required } | MessagesSent { required } => progress >= *required,
        CurrencyEarned { amount } => progress as i64 >= *amount,
        VisitLocation { .. } | CompleteQuest { .. } => progress >= 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::state::seed_starter_achievements;
    use tempfile::tempdir;

    fn setup_test_store() -> TinyMushStore {
        let dir = tempdir().unwrap();
        let store = TinyMushStore::open(dir.path()).unwrap();
        
        // Seed achievements
        for achievement in seed_starter_achievements() {
            store.put_achievement(achievement).unwrap();
        }
        
        // Create test player
        let player = crate::tmush::types::PlayerRecord::new("testuser", "Test User", "starting_room");
        store.put_player(player).unwrap();
        
        store
    }

    #[test]
    fn test_check_trigger_kill_count() {
        let store = setup_test_store();
        
        // Trigger first kill
        let awarded = check_trigger(&store, "testuser", &AchievementTrigger::KillCount { required: 1 }).unwrap();
        assert_eq!(awarded.len(), 1);
        assert_eq!(awarded[0], "first_blood");
        
        // Verify player achievement
        let player = store.get_player("testuser").unwrap();
        let pa = player.achievements.iter()
            .find(|pa| pa.achievement_id == "first_blood")
            .unwrap();
        assert!(pa.earned);
        assert_eq!(pa.progress, 1);
    }

    #[test]
    fn test_check_trigger_incremental() {
        let store = setup_test_store();
        
        // Trigger 5 kills (not enough for veteran)
        check_trigger(&store, "testuser", &AchievementTrigger::KillCount { required: 5 }).unwrap();
        
        let player = store.get_player("testuser").unwrap();
        let pa = player.achievements.iter()
            .find(|pa| pa.achievement_id == "veteran")
            .unwrap();
        assert!(!pa.earned);
        assert_eq!(pa.progress, 5);
        
        // Trigger 95 more kills (should earn veteran)
        let awarded = check_trigger(&store, "testuser", &AchievementTrigger::KillCount { required: 95 }).unwrap();
        assert!(awarded.contains(&"veteran".to_string()));
        
        let player = store.get_player("testuser").unwrap();
        let pa = player.achievements.iter()
            .find(|pa| pa.achievement_id == "veteran")
            .unwrap();
        assert!(pa.earned);
        assert_eq!(pa.progress, 100);
    }

    #[test]
    fn test_award_achievement_direct() {
        let store = setup_test_store();
        
        award_achievement(&store, "testuser", "town_founder").unwrap();
        
        let player = store.get_player("testuser").unwrap();
        let pa = player.achievements.iter()
            .find(|pa| pa.achievement_id == "town_founder")
            .unwrap();
        assert!(pa.earned);
    }

    #[test]
    fn test_get_earned_achievements() {
        let store = setup_test_store();
        
        award_achievement(&store, "testuser", "first_blood").unwrap();
        award_achievement(&store, "testuser", "wanderer").unwrap();
        
        let earned = get_earned_achievements(&store, "testuser").unwrap();
        assert_eq!(earned.len(), 2);
        
        let ids: Vec<_> = earned.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"first_blood"));
        assert!(ids.contains(&"wanderer"));
    }

    #[test]
    fn test_get_available_achievements_filters_hidden() {
        let store = setup_test_store();
        
        let available = get_available_achievements(&store, "testuser").unwrap();
        
        // Hidden achievements should not appear unless earned
        let has_legendary = available.iter()
            .any(|(a, _)| a.id == "legendary");
        assert!(!has_legendary);
        
        // Award hidden achievement
        award_achievement(&store, "testuser", "legendary").unwrap();
        
        let available = get_available_achievements(&store, "testuser").unwrap();
        let has_legendary = available.iter()
            .any(|(a, _)| a.id == "legendary");
        assert!(has_legendary);
    }

    #[test]
    fn test_get_achievements_by_category() {
        let store = setup_test_store();
        
        let combat = get_achievements_by_category(&store, "testuser", AchievementCategory::Combat).unwrap();
        assert!(combat.len() >= 2); // first_blood, veteran (legendary is hidden)
        
        let exploration = get_achievements_by_category(&store, "testuser", AchievementCategory::Exploration).unwrap();
        assert_eq!(exploration.len(), 3); // wanderer, explorer, cartographer
    }

    #[test]
    fn test_update_achievement_progress_direct() {
        let store = setup_test_store();
        
        // Update progress to 50 room visits
        let earned = update_achievement_progress(&store, "testuser", "explorer", 50).unwrap();
        assert!(earned);
        
        let player = store.get_player("testuser").unwrap();
        let pa = player.achievements.iter()
            .find(|pa| pa.achievement_id == "explorer")
            .unwrap();
        assert!(pa.earned);
        assert_eq!(pa.progress, 50);
    }
}

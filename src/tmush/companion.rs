/// Companion behavior and interaction logic for Phase 6 Week 4
///
/// This module provides functions for companion management including
/// taming, feeding, bonding, mounting, and auto-follow mechanics.

use crate::tmush::errors::TinyMushError;
use crate::tmush::storage::TinyMushStore;
use crate::tmush::types::{CompanionRecord, CompanionType};

/// Tame/claim a wild companion, adding it to player's companion list
pub fn tame_companion(
    store: &TinyMushStore,
    username: &str,
    companion_id: &str,
) -> Result<(), TinyMushError> {
    let mut companion = store.get_companion(companion_id)?;
    
    // Check if already owned
    if companion.owner.is_some() {
        return Err(TinyMushError::InvalidCurrency(
            "This companion already has an owner.".to_string(),
        ));
    }
    
    // Claim ownership
    companion.owner = Some(username.to_string());
    companion.loyalty = 30; // Start with modest loyalty
    store.put_companion(companion)?;
    
    // Add to player's companion list
    let mut player = store.get_player(username)?;
    if !player.companions.contains(&companion_id.to_string()) {
        player.companions.push(companion_id.to_string());
        player.touch();
        store.put_player(player)?;
    }
    
    Ok(())
}

/// Release a companion back to the wild
pub fn release_companion(
    store: &TinyMushStore,
    username: &str,
    companion_id: &str,
) -> Result<(), TinyMushError> {
    let mut companion = store.get_companion(companion_id)?;
    
    // Verify ownership
    if companion.owner.as_deref() != Some(username) {
        return Err(TinyMushError::InvalidCurrency(
            "You don't own this companion.".to_string(),
        ));
    }
    
    // Remove ownership
    companion.owner = None;
    companion.loyalty = 50; // Reset to neutral
    companion.is_mounted = false; // Unmount if mounted
    store.put_companion(companion)?;
    
    // Remove from player's companion list
    let mut player = store.get_player(username)?;
    player.companions.retain(|id| id != companion_id);
    if player.mounted_companion.as_deref() == Some(companion_id) {
        player.mounted_companion = None;
    }
    player.touch();
    store.put_player(player)?;
    
    Ok(())
}

/// Feed a companion, increasing happiness
pub fn feed_companion(
    store: &TinyMushStore,
    username: &str,
    companion_id: &str,
) -> Result<u32, TinyMushError> {
    let mut companion = store.get_companion(companion_id)?;
    
    // Verify ownership
    if companion.owner.as_deref() != Some(username) {
        return Err(TinyMushError::InvalidCurrency(
            "You can only feed your own companions.".to_string(),
        ));
    }
    
    let happiness_gain = companion.feed();
    store.put_companion(companion)?;
    
    Ok(happiness_gain)
}

/// Pet/interact with companion, increasing loyalty
pub fn pet_companion(
    store: &TinyMushStore,
    username: &str,
    companion_id: &str,
) -> Result<u32, TinyMushError> {
    let mut companion = store.get_companion(companion_id)?;
    
    // Verify ownership
    if companion.owner.as_deref() != Some(username) {
        return Err(TinyMushError::InvalidCurrency(
            "You can only pet your own companions.".to_string(),
        ));
    }
    
    let loyalty_gain = companion.pet();
    store.put_companion(companion)?;
    
    Ok(loyalty_gain)
}

/// Mount a companion (horses only)
pub fn mount_companion(
    store: &TinyMushStore,
    username: &str,
    companion_id: &str,
) -> Result<(), TinyMushError> {
    let mut companion = store.get_companion(companion_id)?;
    
    // Verify ownership
    if companion.owner.as_deref() != Some(username) {
        return Err(TinyMushError::InvalidCurrency(
            "You can only mount your own companions.".to_string(),
        ));
    }
    
    // Check if mountable
    if companion.companion_type != CompanionType::Horse {
        return Err(TinyMushError::InvalidCurrency(
            "Only horses can be mounted.".to_string(),
        ));
    }
    
    // Check if already mounted
    if companion.is_mounted {
        return Err(TinyMushError::InvalidCurrency(
            "This companion is already mounted.".to_string(),
        ));
    }
    
    // Mount
    companion.is_mounted = true;
    store.put_companion(companion)?;
    
    // Update player state
    let mut player = store.get_player(username)?;
    player.mounted_companion = Some(companion_id.to_string());
    player.touch();
    store.put_player(player)?;
    
    Ok(())
}

/// Dismount from current companion
pub fn dismount_companion(
    store: &TinyMushStore,
    username: &str,
) -> Result<String, TinyMushError> {
    let mut player = store.get_player(username)?;
    
    let companion_id = player
        .mounted_companion
        .take()
        .ok_or_else(|| TinyMushError::InvalidCurrency("You are not mounted.".to_string()))?;
    
    // Dismount the companion
    let mut companion = store.get_companion(&companion_id)?;
    companion.is_mounted = false;
    store.put_companion(companion.clone())?;
    
    player.touch();
    store.put_player(player)?;
    
    Ok(companion.name)
}

/// Move companion to a new room (auto-follow mechanic)
pub fn move_companion_to_room(
    store: &TinyMushStore,
    companion_id: &str,
    new_room_id: &str,
) -> Result<(), TinyMushError> {
    let mut companion = store.get_companion(companion_id)?;
    companion.room_id = new_room_id.to_string();
    store.put_companion(companion)?;
    Ok(())
}

/// Auto-follow: Move all player's companions with auto-follow behavior
pub fn auto_follow_companions(
    store: &TinyMushStore,
    username: &str,
    new_room_id: &str,
) -> Result<Vec<String>, TinyMushError> {
    let player = store.get_player(username)?;
    let mut followed = Vec::new();
    
    for companion_id in &player.companions {
        if let Ok(companion) = store.get_companion(companion_id) {
            // Only follow if has auto-follow behavior and not in same room
            if companion.has_auto_follow() && companion.room_id != new_room_id {
                move_companion_to_room(store, companion_id, new_room_id)?;
                followed.push(companion.name);
            }
        }
    }
    
    Ok(followed)
}

/// Get all companions owned by player
pub fn get_player_companions(
    store: &TinyMushStore,
    username: &str,
) -> Result<Vec<CompanionRecord>, TinyMushError> {
    store.get_player_companions(username)
}

/// Get companion by name in current room
pub fn find_companion_in_room(
    store: &TinyMushStore,
    room_id: &str,
    name: &str,
) -> Result<Option<CompanionRecord>, TinyMushError> {
    let companions = store.get_companions_in_room(room_id)?;
    let name_lower = name.to_lowercase();
    
    Ok(companions
        .into_iter()
        .find(|c| c.name.to_lowercase().contains(&name_lower)))
}

/// Format companion status for display
pub fn format_companion_status(companion: &CompanionRecord) -> String {
    let type_name = format!("{:?}", companion.companion_type);
    let owner_info = companion
        .owner
        .as_ref()
        .map(|o| format!("Owner: {}", o))
        .unwrap_or_else(|| "Wild".to_string());
    
    let mount_status = if companion.is_mounted {
        " [MOUNTED]"
    } else {
        ""
    };
    
    let mut output = format!(
        "=== {} ===\n{} ({}){}\n{}\n\n",
        companion.name, type_name, owner_info, mount_status, companion.description
    );
    
    if companion.owner.is_some() {
        output.push_str(&format!("Loyalty: {}/100\n", companion.loyalty));
        output.push_str(&format!("Happiness: {}/100\n", companion.happiness));
        
        if companion.needs_feeding() {
            output.push_str("⚠️  Needs feeding!\n");
        }
        
        // Show storage capacity if applicable
        let capacity = companion.storage_capacity();
        if capacity > 0 {
            output.push_str(&format!(
                "\nStorage: {}/{} items\n",
                companion.inventory.len(),
                capacity
            ));
        }
    }
    
    output
}

/// Format list of player's companions
pub fn format_companion_list(companions: &[CompanionRecord]) -> String {
    if companions.is_empty() {
        return "You don't have any companions.\nUse COMPANION TAME <name> to claim a wild companion.".to_string();
    }
    
    let mut output = String::from("=== YOUR COMPANIONS ===\n");
    for (idx, companion) in companions.iter().enumerate() {
        let type_name = format!("{:?}", companion.companion_type);
        let loyalty_bar = "█".repeat((companion.loyalty / 10) as usize);
        let happiness_bar = "█".repeat((companion.happiness / 10) as usize);
        let mount_flag = if companion.is_mounted { " [MOUNTED]" } else { "" };
        
        output.push_str(&format!(
            "{}. {}{} ({})\n   Loyalty: {}/100 {}\n   Happiness: {}/100 {}\n",
            idx + 1,
            companion.name,
            mount_flag,
            type_name,
            companion.loyalty,
            loyalty_bar,
            companion.happiness,
            happiness_bar
        ));
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::state::seed_starter_companions;
    use crate::tmush::types::{CompanionType, PlayerRecord};
    use tempfile::tempdir;

    fn setup_test_store() -> TinyMushStore {
        let dir = tempdir().unwrap();
        let store = TinyMushStore::open(dir.path()).unwrap();
        
        // Seed companions
        for companion in seed_starter_companions() {
            store.put_companion(companion).unwrap();
        }
        
        // Create test player
        let player = PlayerRecord::new("testuser", "Test User", "town_square");
        store.put_player(player).unwrap();
        
        store
    }

    #[test]
    fn test_tame_wild_companion() {
        let store = setup_test_store();
        
        // Tame the loyal hound
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        
        // Verify ownership
        let companion = store.get_companion("loyal_hound").unwrap();
        assert_eq!(companion.owner, Some("testuser".to_string()));
        assert_eq!(companion.loyalty, 30); // Initial loyalty
        
        // Verify player's companion list
        let player = store.get_player("testuser").unwrap();
        assert!(player.companions.contains(&"loyal_hound".to_string()));
    }

    #[test]
    fn test_cannot_tame_owned_companion() {
        let store = setup_test_store();
        
        // First tame
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        
        // Try to tame again
        let result = tame_companion(&store, "other_user", "loyal_hound");
        assert!(result.is_err());
    }

    #[test]
    fn test_release_companion() {
        let store = setup_test_store();
        
        // Tame then release
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        release_companion(&store, "testuser", "loyal_hound").unwrap();
        
        // Verify no longer owned
        let companion = store.get_companion("loyal_hound").unwrap();
        assert_eq!(companion.owner, None);
        
        // Verify removed from player list
        let player = store.get_player("testuser").unwrap();
        assert!(!player.companions.contains(&"loyal_hound".to_string()));
    }

    #[test]
    fn test_feed_companion() {
        let store = setup_test_store();
        
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        
        // Initial feed
        let gain = feed_companion(&store, "testuser", "loyal_hound").unwrap();
        assert!(gain > 0);
        
        // Check happiness increased
        let companion = store.get_companion("loyal_hound").unwrap();
        assert!(companion.happiness > 50);
    }

    #[test]
    fn test_pet_companion() {
        let store = setup_test_store();
        
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        
        let gain = pet_companion(&store, "testuser", "loyal_hound").unwrap();
        assert!(gain > 0);
        
        // Check loyalty increased
        let companion = store.get_companion("loyal_hound").unwrap();
        assert!(companion.loyalty > 30);
    }

    #[test]
    fn test_mount_horse() {
        let store = setup_test_store();
        
        tame_companion(&store, "testuser", "gentle_mare").unwrap();
        mount_companion(&store, "testuser", "gentle_mare").unwrap();
        
        // Verify mounted
        let companion = store.get_companion("gentle_mare").unwrap();
        assert!(companion.is_mounted);
        
        // Verify player state
        let player = store.get_player("testuser").unwrap();
        assert_eq!(player.mounted_companion, Some("gentle_mare".to_string()));
    }

    #[test]
    fn test_cannot_mount_non_horse() {
        let store = setup_test_store();
        
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        let result = mount_companion(&store, "testuser", "loyal_hound");
        assert!(result.is_err());
    }

    #[test]
    fn test_dismount() {
        let store = setup_test_store();
        
        tame_companion(&store, "testuser", "gentle_mare").unwrap();
        mount_companion(&store, "testuser", "gentle_mare").unwrap();
        
        let name = dismount_companion(&store, "testuser").unwrap();
        assert_eq!(name, "Gentle Mare");
        
        // Verify dismounted
        let companion = store.get_companion("gentle_mare").unwrap();
        assert!(!companion.is_mounted);
        
        let player = store.get_player("testuser").unwrap();
        assert_eq!(player.mounted_companion, None);
    }

    #[test]
    fn test_auto_follow() {
        let store = setup_test_store();
        
        // Tame a dog (has auto-follow)
        tame_companion(&store, "testuser", "loyal_hound").unwrap();
        
        // Move to different room
        auto_follow_companions(&store, "testuser", "city_hall_lobby").unwrap();
        
        // Verify companion followed
        let companion = store.get_companion("loyal_hound").unwrap();
        assert_eq!(companion.room_id, "city_hall_lobby");
    }

    #[test]
    fn test_find_companion_in_room() {
        let store = setup_test_store();
        
        let companion = find_companion_in_room(&store, "town_square", "loyal").unwrap();
        assert!(companion.is_some());
        assert_eq!(companion.unwrap().id, "loyal_hound");
    }
}

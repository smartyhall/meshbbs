//! Tutorial progression logic for TinyMUSH
//! 
//! This module handles:
//! - Auto-start tutorial on first login
//! - Step advancement with validation
//! - Reward distribution on completion
//! - Tutorial state tracking

use chrono::Utc;

use crate::tmush::{
    storage::TinyMushStore,
    types::{
        CurrencyAmount, InventoryConfig, ObjectRecord, PlayerRecord, 
        TutorialState, TutorialStep, TransactionReason, ObjectOwner,
    },
    errors::TinyMushError,
};

/// Check if player should auto-start tutorial
pub fn should_auto_start_tutorial(player: &PlayerRecord) -> bool {
    matches!(player.tutorial_state, TutorialState::NotStarted)
}

/// Start the tutorial for a player
pub fn start_tutorial(
    store: &TinyMushStore,
    username: &str,
) -> Result<TutorialState, TinyMushError> {
    let mut player = store.get_player(username)?;
    
    if !matches!(player.tutorial_state, TutorialState::NotStarted) {
        return Err(TinyMushError::InvalidCurrency(
            "Tutorial already started or completed".to_string()
        ));
    }
    
    player.tutorial_state = TutorialState::InProgress {
        step: TutorialStep::WelcomeAtGazebo,
    };
    
    let state = player.tutorial_state.clone();
    store.put_player(player)?;
    Ok(state)
}

/// Advance to the next tutorial step
pub fn advance_tutorial_step(
    store: &TinyMushStore,
    username: &str,
    expected_current_step: TutorialStep,
) -> Result<TutorialState, TinyMushError> {
    let mut player = store.get_player(username)?;
    
    // Validate current state
    match &player.tutorial_state {
        TutorialState::InProgress { step } => {
            if step != &expected_current_step {
                return Err(TinyMushError::InvalidCurrency(format!(
                    "Expected step {:?}, but player is at {:?}",
                    expected_current_step, step
                )));
            }
        }
        _ => {
            return Err(TinyMushError::InvalidCurrency(
                "Tutorial not in progress".to_string()
            ));
        }
    }
    
    // Advance to next step or complete
    player.tutorial_state = match expected_current_step {
        TutorialStep::WelcomeAtGazebo => TutorialState::InProgress {
            step: TutorialStep::NavigateToCityHall,
        },
        TutorialStep::NavigateToCityHall => TutorialState::InProgress {
            step: TutorialStep::MeetTheMayor,
        },
        TutorialStep::MeetTheMayor => {
            // Final step - complete tutorial
            TutorialState::Completed {
                completed_at: Utc::now(),
            }
        }
    };
    
    let state = player.tutorial_state.clone();
    store.put_player(player)?;
    Ok(state)
}

/// Check if player can advance from current step based on their location
pub fn can_advance_from_location(
    step: &TutorialStep,
    current_room: &str,
) -> bool {
    match step {
        TutorialStep::WelcomeAtGazebo => {
            // Can advance when they leave gazebo
            current_room != "gazebo_landing"
        }
        TutorialStep::NavigateToCityHall => {
            // Can advance when they reach city hall lobby
            current_room == "city_hall_lobby"
        }
        TutorialStep::MeetTheMayor => {
            // Can advance when they reach mayor's office
            current_room == "mayor_office"
        }
    }
}

/// Skip the tutorial (player opts out)
pub fn skip_tutorial(
    store: &TinyMushStore,
    username: &str,
) -> Result<TutorialState, TinyMushError> {
    let mut player = store.get_player(username)?;
    
    if matches!(player.tutorial_state, TutorialState::Completed { .. }) {
        return Err(TinyMushError::InvalidCurrency(
            "Tutorial already completed".to_string()
        ));
    }
    
    player.tutorial_state = TutorialState::Skipped {
        skipped_at: Utc::now(),
    };
    
    let state = player.tutorial_state.clone();
    store.put_player(player)?;
    Ok(state)
}

/// Restart the tutorial from the beginning
pub fn restart_tutorial(
    store: &TinyMushStore,
    username: &str,
) -> Result<TutorialState, TinyMushError> {
    let mut player = store.get_player(username)?;
    
    player.tutorial_state = TutorialState::InProgress {
        step: TutorialStep::WelcomeAtGazebo,
    };
    
    let state = player.tutorial_state.clone();
    store.put_player(player)?;
    Ok(state)
}

/// Distribute tutorial completion rewards
pub fn distribute_tutorial_rewards(
    store: &TinyMushStore,
    username: &str,
    currency_system: &CurrencyAmount, // World's currency system
) -> Result<(), TinyMushError> {
    // Verify tutorial is complete
    let player = store.get_player(username)?;
    
    if !matches!(player.tutorial_state, TutorialState::Completed { .. }) {
        return Err(TinyMushError::InvalidCurrency(
            "Tutorial not completed".to_string()
        ));
    }
    
    // 1. Grant starter currency
    let starter_amount = match currency_system {
        CurrencyAmount::Decimal { .. } => {
            // $10.00 = 1000 minor units
            CurrencyAmount::Decimal { minor_units: 1000 }
        }
        CurrencyAmount::MultiTier { .. } => {
            // 100 copper = 1 silver
            CurrencyAmount::MultiTier { base_units: 100 }
        }
    };
    
    store.grant_currency(
        username,
        &starter_amount,
        TransactionReason::QuestReward,
    )?;
    
    // 2. Grant welcome item (Town Map)
    let town_map = ObjectRecord {
        id: "town_map_001".to_string(),
        name: "Town Map".to_string(),
        description: "A detailed hand-drawn map showing the main locations of Old Towne Mesh, including the Town Square, City Hall, shops, and important landmarks. The parchment smells faintly of ink.".to_string(),
        owner: ObjectOwner::World,
        currency_value: CurrencyAmount::default(), // Quest item, no value
        value: 0, // Legacy field
        weight: 1,
        takeable: true,
        usable: false,
        actions: Default::default(),
        flags: vec![],
        created_at: Utc::now(),
        schema_version: 1,
    };
    
    // Save the object if it doesn't exist
    let existing = store.get_object(&town_map.id);
    if existing.is_err() || existing.unwrap().id != town_map.id {
        store.put_object(town_map.clone())?;
    }
    
    // Add to player's inventory
    let config = InventoryConfig::default();
    store.player_add_item(username, &town_map.id, 1, &config)?;
    
    Ok(())
}

/// Get helpful hint message for current tutorial step
pub fn get_tutorial_hint(step: &TutorialStep) -> &'static str {
    match step {
        TutorialStep::WelcomeAtGazebo => {
            "Try LOOK to examine your surroundings, then move NORTH to Town Square."
        }
        TutorialStep::NavigateToCityHall => {
            "Navigate to City Hall. Use WHERE to check location, NORTH to move."
        }
        TutorialStep::MeetTheMayor => {
            "Go NORTH from City Hall Lobby. Use TALK MAYOR to complete tutorial."
        }
    }
}

/// Format tutorial status message (under 200 bytes)
pub fn format_tutorial_status(state: &TutorialState) -> String {
    match state {
        TutorialState::NotStarted => {
            "Tutorial: Not started. Use TUTORIAL to begin.".to_string()
        }
        TutorialState::InProgress { step } => {
            let hint = get_tutorial_hint(step);
            format!("Tutorial: {:?}. {}", step, hint)
        }
        TutorialState::Completed { .. } => {
            "Tutorial: Complete! Welcome to Old Towne Mesh.".to_string()
        }
        TutorialState::Skipped { .. } => {
            "Tutorial: Skipped. Use TUTORIAL RESTART to try it.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::storage::TinyMushStoreBuilder;
    use tempfile::TempDir;
    
    fn setup_test_store() -> (TinyMushStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = TinyMushStoreBuilder::new(temp_dir.path())
            .open()
            .unwrap();
        (store, temp_dir)
    }
    
    fn create_test_player(store: &TinyMushStore, username: &str) {
        let player = PlayerRecord::new(username, username, "gazebo_landing");
        store.put_player(player).unwrap();
    }
    
    #[test]
    fn test_should_auto_start_tutorial() {
        let player = PlayerRecord::new("alice", "alice", "gazebo_landing");
        assert!(should_auto_start_tutorial(&player));
        
        let (store, _temp) = setup_test_store();
        create_test_player(&store, "bob");
        start_tutorial(&store, "bob").unwrap();
        let player_in_progress = store.get_player("bob").unwrap();
        assert!(!should_auto_start_tutorial(&player_in_progress));
    }
    
    #[test]
    fn test_start_tutorial() {
        let (store, _temp) = setup_test_store();
        create_test_player(&store, "alice");
        
        let state = start_tutorial(&store, "alice").unwrap();
        assert!(matches!(
            state,
            TutorialState::InProgress { step: TutorialStep::WelcomeAtGazebo }
        ));
        
        // Verify persistence
        let player = store.get_player("alice").unwrap();
        assert!(matches!(
            player.tutorial_state,
            TutorialState::InProgress { step: TutorialStep::WelcomeAtGazebo }
        ));
    }
    
    #[test]
    fn test_advance_tutorial_step() {
        let (store, _temp) = setup_test_store();
        create_test_player(&store, "alice");
        start_tutorial(&store, "alice").unwrap();
        
        // Advance from WelcomeAtGazebo to NavigateToCityHall
        let state = advance_tutorial_step(
            &store,
            "alice",
            TutorialStep::WelcomeAtGazebo,
        ).unwrap();
        assert!(matches!(
            state,
            TutorialState::InProgress { step: TutorialStep::NavigateToCityHall }
        ));
        
        // Advance from NavigateToCityHall to MeetTheMayor
        let state = advance_tutorial_step(
            &store,
            "alice",
            TutorialStep::NavigateToCityHall,
        ).unwrap();
        assert!(matches!(
            state,
            TutorialState::InProgress { step: TutorialStep::MeetTheMayor }
        ));
        
        // Advance from MeetTheMayor to Completed
        let state = advance_tutorial_step(
            &store,
            "alice",
            TutorialStep::MeetTheMayor,
        ).unwrap();
        assert!(matches!(state, TutorialState::Completed { .. }));
    }
    
    #[test]
    fn test_can_advance_from_location() {
        assert!(!can_advance_from_location(
            &TutorialStep::WelcomeAtGazebo,
            "gazebo_landing"
        ));
        assert!(can_advance_from_location(
            &TutorialStep::WelcomeAtGazebo,
            "town_square"
        ));
        
        assert!(!can_advance_from_location(
            &TutorialStep::NavigateToCityHall,
            "town_square"
        ));
        assert!(can_advance_from_location(
            &TutorialStep::NavigateToCityHall,
            "city_hall_lobby"
        ));
        
        assert!(!can_advance_from_location(
            &TutorialStep::MeetTheMayor,
            "city_hall_lobby"
        ));
        assert!(can_advance_from_location(
            &TutorialStep::MeetTheMayor,
            "mayor_office"
        ));
    }
    
    #[test]
    fn test_skip_tutorial() {
        let (store, _temp) = setup_test_store();
        create_test_player(&store, "alice");
        start_tutorial(&store, "alice").unwrap();
        
        let state = skip_tutorial(&store, "alice").unwrap();
        assert!(matches!(state, TutorialState::Skipped { .. }));
        
        // Verify persistence
        let player = store.get_player("alice").unwrap();
        assert!(matches!(player.tutorial_state, TutorialState::Skipped { .. }));
    }
    
    #[test]
    fn test_restart_tutorial() {
        let (store, _temp) = setup_test_store();
        create_test_player(&store, "alice");
        start_tutorial(&store, "alice").unwrap();
        advance_tutorial_step(&store, "alice", TutorialStep::WelcomeAtGazebo).unwrap();
        
        let state = restart_tutorial(&store, "alice").unwrap();
        assert!(matches!(
            state,
            TutorialState::InProgress { step: TutorialStep::WelcomeAtGazebo }
        ));
    }
    
    #[test]
    fn test_distribute_tutorial_rewards() {
        let (store, _temp) = setup_test_store();
        
        // Create player with MultiTier currency system
        let mut player = PlayerRecord::new("alice", "alice", "gazebo_landing");
        player.currency = CurrencyAmount::MultiTier { base_units: 0 };
        store.put_player(player).unwrap();
        
        start_tutorial(&store, "alice").unwrap();
        
        // Complete tutorial
        advance_tutorial_step(&store, "alice", TutorialStep::WelcomeAtGazebo).unwrap();
        advance_tutorial_step(&store, "alice", TutorialStep::NavigateToCityHall).unwrap();
        advance_tutorial_step(&store, "alice", TutorialStep::MeetTheMayor).unwrap();
        
        // Distribute rewards (MultiTier currency)
        let currency_system = CurrencyAmount::MultiTier { base_units: 0 };
        distribute_tutorial_rewards(&store, "alice", &currency_system).unwrap();
        
        // Check currency granted
        let player = store.get_player("alice").unwrap();
        assert_eq!(player.currency.base_value(), 100); // 100 copper
        
        // Check item granted
        assert_eq!(player.inventory_stacks.len(), 1);
        assert_eq!(player.inventory_stacks[0].object_id, "town_map_001");
    }
    
    #[test]
    fn test_format_tutorial_status() {
        let status = format_tutorial_status(&TutorialState::NotStarted);
        assert!(status.len() < 200);
        assert!(status.contains("Not started"));
        
        let status = format_tutorial_status(&TutorialState::InProgress {
            step: TutorialStep::WelcomeAtGazebo,
        });
        assert!(status.len() < 200);
        
        let status = format_tutorial_status(&TutorialState::Completed {
            completed_at: Utc::now(),
        });
        assert!(status.len() < 200);
        assert!(status.contains("Complete"));
    }
}

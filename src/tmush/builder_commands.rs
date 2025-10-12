//! Builder Commands for TinyMUSH Trigger System
//!
//! Provides user-facing commands for creating and managing object triggers.
//! These commands make the trigger system accessible without requiring
//! technical knowledge of IDs or DSL syntax.
//!
//! ## Commands
//! - `/when <trigger> <object> <action>` - Simple one-liner trigger
//! - `/script <object> <trigger>` - Multi-line trigger with /done
//! - `/wizard <object> <trigger>` - Guided menu-based creation
//! - `/show <object>` - View all triggers on object
//! - `/remove <object> <trigger>` - Delete trigger
//! - `/test <object> <trigger>` - Dry-run trigger execution

use crate::tmush::{
    errors::TinyMushError,
    resolver::{resolve_object_name, ResolutionContext, ResolveResult, format_disambiguation_prompt},
    storage::TinyMushStore,
    trigger::parser::parse_script,
    types::{ObjectRecord, ObjectTrigger},
};

/// Handle `/when` command - simple one-liner trigger creation
///
/// Syntax: `/when <trigger> <object> <action>`
///
/// Examples:
/// - `/when examine rusty_key Say "Ancient key from the ruins!"`
/// - `/when use healing_potion Give player 50 health`
/// - `/when take gold_coin Say "Cha-ching!"`
///
/// ## Arguments
/// - `context`: Resolution context (player state)
/// - `trigger`: Trigger type ("examine", "use", "take", "drop", "enter", "exit")
/// - `object_name`: Object to attach trigger to (resolved by name)
/// - `action`: Natural language or DSL action script
/// - `store`: Storage reference
///
/// ## Returns
/// Success message with object name and trigger details, or error message
pub fn handle_when_command(
    context: &ResolutionContext,
    trigger: &str,
    object_name: &str,
    action: &str,
    store: &TinyMushStore,
) -> Result<String, TinyMushError> {
    // Parse trigger type
    let trigger_type = parse_trigger_type(trigger)?;
    
    // Resolve object name
    let object_id = match resolve_object_name(context, object_name, store)? {
        ResolveResult::Found(id) => id,
        ResolveResult::Ambiguous(matches) => {
            return Ok(format!(
                "Ambiguous object name:\n\n{}",
                format_disambiguation_prompt(&matches)
            ));
        }
        ResolveResult::NotFound => {
            return Err(TinyMushError::NotFound(format!(
                "Object '{}' not found in your inventory or current room.",
                object_name
            )));
        }
    };
    
    // Get object to verify ownership
    let mut object = store.get_object(&object_id)?;
    
    // Check permissions (must be owner or admin)
    if !can_modify_object(&object, &context.username) {
        return Err(TinyMushError::NotFound(format!(
            "You don't have permission to modify '{}'.",
            object.name
        )));
    }
    
    // Parse action script (auto-detects natural language vs DSL)
    let _ast = parse_script(action).map_err(|e| {
        TinyMushError::NotFound(format!("Script parse error: {}", e))
    })?;
    
    // Store the original script text for now
    // Phase 6 will add proper trigger storage with compiled AST
    object.actions.insert(
        trigger_type.clone(),
        action.to_string(),
    );
    
    // Save updated object
    store.put_object(object.clone())?;
    
    // Format success message
    Ok(format!(
        "✓ Added {:?} trigger to '{}'\n\nScript: {}\n\nTry it: /use {}",
        trigger_type,
        object.name,
        action,
        object.name
    ))
}

/// Parse trigger type from user input
///
/// Accepts common aliases:
/// - "examine", "look", "inspect" → ObjectTrigger::OnLook
/// - "use", "activate" → ObjectTrigger::OnUse
/// - "take", "get", "pickup" → ObjectTrigger::OnTake
/// - "drop", "put" → ObjectTrigger::OnDrop
fn parse_trigger_type(trigger: &str) -> Result<ObjectTrigger, TinyMushError> {
    let trigger_lower = trigger.to_lowercase();
    
    match trigger_lower.as_str() {
        "examine" | "look" | "inspect" => Ok(ObjectTrigger::OnLook),
        "use" | "activate" => Ok(ObjectTrigger::OnUse),
        "take" | "get" | "pickup" => Ok(ObjectTrigger::OnTake),
        "drop" | "put" => Ok(ObjectTrigger::OnDrop),
        "enter" => Ok(ObjectTrigger::OnEnter),
        _ => Err(TinyMushError::NotFound(format!(
            "Unknown trigger type '{}'. Valid types: examine, use, take, drop, enter",
            trigger
        ))),
    }
}

/// Check if user can modify an object
///
/// Requirements:
/// - User must be the owner of the object, OR
/// - User must be an admin (future: check admin_level)
fn can_modify_object(object: &ObjectRecord, username: &str) -> bool {
    // Check if user is owner
    match &object.owner {
        crate::tmush::types::ObjectOwner::Player { username: owner_name } => {
            owner_name == username
        }
        crate::tmush::types::ObjectOwner::World => {
            // World objects can only be modified by admins (future enhancement)
            false
        }
    }
}

/// Multi-line script builder state
///
/// Tracks the state of an in-progress multi-line script creation.
/// The user enters `/script <object> <trigger>`, then types multiple
/// lines of script, and finally types `/done` to complete.
#[derive(Debug, Clone)]
pub struct ScriptBuilder {
    /// Object being modified
    pub object_id: String,
    /// Object name (for display)
    pub object_name: String,
    /// Trigger type
    pub trigger_type: ObjectTrigger,
    /// Accumulated script lines
    pub lines: Vec<String>,
}

impl ScriptBuilder {
    pub fn new(object_id: String, object_name: String, trigger_type: ObjectTrigger) -> Self {
        Self {
            object_id,
            object_name,
            trigger_type,
            lines: Vec::new(),
        }
    }
    
    /// Add a line to the script
    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
    }
    
    /// Get the complete script as a single string
    pub fn get_script(&self) -> String {
        self.lines.join("\n")
    }
    
    /// Get line count
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// Handle `/script` command - multi-line trigger creation
///
/// Syntax: `/script <object> <trigger>`
///
/// Enters multi-line mode where each message becomes a script line.
/// User types `/done` to finish and save the trigger.
///
/// Example:
/// ```text
/// > /script healing_potion use
/// Editing trigger for 'Healing Potion'. Type your script, then /done to finish:
///
/// > If player has quest wounded_soldier:
/// >   Say "This will help the wounded soldier!"
/// >   Give player 50 health
/// > Otherwise:
/// >   Say "You drink the potion and feel refreshed."
/// >   Give player 25 health
/// > /done
/// ✓ Added OnUse trigger to 'Healing Potion' (4 lines)
/// ```
///
/// ## Arguments
/// - `context`: Resolution context (player state)
/// - `object_name`: Object to attach trigger to (resolved by name)
/// - `trigger`: Trigger type ("examine", "use", "take", "drop")
/// - `store`: Storage reference
///
/// ## Returns
/// ScriptBuilder instance for session state tracking, or error message
pub fn handle_script_command(
    context: &ResolutionContext,
    object_name: &str,
    trigger: &str,
    store: &TinyMushStore,
) -> Result<ScriptBuilder, TinyMushError> {
    // Parse trigger type
    let trigger_type = parse_trigger_type(trigger)?;
    
    // Resolve object name
    let object_id = match resolve_object_name(context, object_name, store)? {
        ResolveResult::Found(id) => id,
        ResolveResult::Ambiguous(matches) => {
            return Err(TinyMushError::NotFound(format!(
                "Ambiguous object name:\n\n{}",
                format_disambiguation_prompt(&matches)
            )));
        }
        ResolveResult::NotFound => {
            return Err(TinyMushError::NotFound(format!(
                "Object '{}' not found in your inventory or current room.",
                object_name
            )));
        }
    };
    
    // Get object to verify ownership and get name
    let object = store.get_object(&object_id)?;
    
    // Check permissions
    if !can_modify_object(&object, &context.username) {
        return Err(TinyMushError::NotFound(format!(
            "You don't have permission to modify '{}'.",
            object.name
        )));
    }
    
    // Create script builder
    Ok(ScriptBuilder::new(
        object_id,
        object.name.clone(),
        trigger_type,
    ))
}

/// Handle `/done` command - finalize multi-line script
///
/// Validates and saves the accumulated script from ScriptBuilder.
///
/// ## Arguments
/// - `builder`: The script builder with accumulated lines
/// - `store`: Storage reference
///
/// ## Returns
/// Success message or error
pub fn handle_done_command(
    builder: &ScriptBuilder,
    store: &TinyMushStore,
) -> Result<String, TinyMushError> {
    // Get the complete script
    let script = builder.get_script();
    
    if script.trim().is_empty() {
        return Err(TinyMushError::NotFound(
            "Script is empty. Add at least one line before /done.".to_string()
        ));
    }
    
    // Parse and validate script
    let _ast = parse_script(&script).map_err(|e| {
        TinyMushError::NotFound(format!(
            "Script parse error on line {}:\n{}\n\nUse /cancel to abort.",
            builder.line_count(),
            e
        ))
    })?;
    
    // Get object
    let mut object = store.get_object(&builder.object_id)?;
    
    // Update trigger
    object.actions.insert(
        builder.trigger_type.clone(),
        script.clone(),
    );
    
    // Save
    store.put_object(object)?;
    
    // Success message
    Ok(format!(
        "✓ Added {:?} trigger to '{}' ({} lines)\n\nScript:\n{}\n",
        builder.trigger_type,
        builder.object_name,
        builder.line_count(),
        script
    ))
}

/// Handle `/cancel` command - abort multi-line script creation
///
/// Cancels the current script builder session.
pub fn handle_cancel_command() -> String {
    "Script creation cancelled.".to_string()
}

/// Wizard state for guided trigger creation
///
/// Walks the user through creating a trigger step-by-step with menus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardState {
    /// Step 1: Choose object
    ChooseObject,
    /// Step 2: Choose trigger type
    ChooseTrigger { object_id: String, object_name: String },
    /// Step 3: Choose action template
    ChooseAction { object_id: String, object_name: String, trigger_type: ObjectTrigger },
    /// Step 4: Customize action (if needed)
    CustomizeAction { 
        object_id: String, 
        object_name: String, 
        trigger_type: ObjectTrigger,
        action_template: String,
    },
}

/// Wizard session tracker
#[derive(Debug, Clone)]
pub struct WizardSession {
    pub state: WizardState,
    /// Available objects (for Step 1)
    pub available_objects: Vec<(String, String)>, // (id, name) pairs
}

impl WizardSession {
    pub fn new(available_objects: Vec<(String, String)>) -> Self {
        Self {
            state: WizardState::ChooseObject,
            available_objects,
        }
    }
}

/// Handle `/wizard` command - start guided trigger creation
///
/// Provides a step-by-step menu interface for creating triggers:
/// 1. Choose object from list
/// 2. Choose trigger type (examine, use, take, drop)
/// 3. Choose action template (message, heal, teleport, etc.)
/// 4. Customize parameters (optional)
///
/// ## Arguments
/// - `context`: Resolution context (player state)
/// - `store`: Storage reference
///
/// ## Returns
/// WizardSession with initial menu prompt, or error
pub fn handle_wizard_command(
    context: &ResolutionContext,
    store: &TinyMushStore,
) -> Result<(WizardSession, String), TinyMushError> {
    // Get player's inventory
    let player = store.get_player(&context.username)?;
    
    // Get current room items
    let room = store.get_room(&context.current_room)?;
    
    // Build list of available objects
    let mut available_objects = Vec::new();
    
    // Add inventory items
    for obj_id in &player.inventory {
        if let Ok(obj) = store.get_object(obj_id) {
            // Only include objects the player owns
            if can_modify_object(&obj, &context.username) {
                available_objects.push((obj.id.clone(), obj.name.clone()));
            }
        }
    }
    
    // Add room items (that player owns)
    for obj_id in &room.items {
        if let Ok(obj) = store.get_object(obj_id) {
            if can_modify_object(&obj, &context.username) {
                available_objects.push((obj.id.clone(), obj.name.clone()));
            }
        }
    }
    
    if available_objects.is_empty() {
        return Err(TinyMushError::NotFound(
            "No objects available to modify. Create an object first with /create.".to_string()
        ));
    }
    
    // Create wizard session
    let session = WizardSession::new(available_objects.clone());
    
    // Format object selection menu
    let mut prompt = String::from("🧙 Trigger Wizard - Step 1: Choose Object\n\n");
    prompt.push_str("Select an object to add a trigger to:\n\n");
    
    for (idx, (_id, name)) in available_objects.iter().enumerate() {
        prompt.push_str(&format!("  {}. {}\n", idx + 1, name));
    }
    
    prompt.push_str("\nReply with the number, or /cancel to abort.");
    
    Ok((session, prompt))
}

/// Handle wizard step progression
///
/// Processes user input for the current wizard step and advances to next step.
///
/// ## Arguments
/// - `session`: Current wizard session
/// - `input`: User's input (number selection or custom text)
/// - `store`: Storage reference
///
/// ## Returns
/// Updated wizard session and prompt for next step, or final success message
pub fn handle_wizard_step(
    session: &mut WizardSession,
    input: &str,
    store: &TinyMushStore,
) -> Result<String, TinyMushError> {
    let input = input.trim();
    
    // Clone the current state to avoid borrow checker issues
    let current_state = session.state.clone();
    
    match current_state {
        WizardState::ChooseObject => {
            // Parse object selection
            let selection: usize = input.parse().map_err(|_| {
                TinyMushError::NotFound("Please enter a number from the list.".to_string())
            })?;
            
            if selection < 1 || selection > session.available_objects.len() {
                return Err(TinyMushError::NotFound(format!(
                    "Invalid selection. Choose 1-{}.",
                    session.available_objects.len()
                )));
            }
            
            let (object_id, object_name) = session.available_objects[selection - 1].clone();
            
            // Move to trigger selection
            session.state = WizardState::ChooseTrigger { object_id, object_name: object_name.clone() };
            
            Ok(format!(
                "🧙 Trigger Wizard - Step 2: Choose Trigger Type\n\n\
                Object: {}\n\n\
                When should the trigger activate?\n\n\
                  1. When examined (player looks at object)\n\
                  2. When used (player uses/activates object)\n\
                  3. When taken (player picks up object)\n\
                  4. When dropped (player drops object)\n\n\
                Reply with the number:",
                object_name
            ))
        }
        
        WizardState::ChooseTrigger { object_id, object_name } => {
            // Parse trigger type selection
            let trigger_type = match input {
                "1" => ObjectTrigger::OnLook,
                "2" => ObjectTrigger::OnUse,
                "3" => ObjectTrigger::OnTake,
                "4" => ObjectTrigger::OnDrop,
                _ => return Err(TinyMushError::NotFound("Please enter 1, 2, 3, or 4.".to_string())),
            };
            
            let trigger_name = match trigger_type {
                ObjectTrigger::OnLook => "examined",
                ObjectTrigger::OnUse => "used",
                ObjectTrigger::OnTake => "taken",
                ObjectTrigger::OnDrop => "dropped",
                _ => "activated",
            };
            
            // Move to action selection
            session.state = WizardState::ChooseAction {
                object_id: object_id.clone(),
                object_name: object_name.clone(),
                trigger_type,
            };
            
            Ok(format!(
                "🧙 Trigger Wizard - Step 3: Choose Action\n\n\
                Object: {}\n\
                Trigger: When {}\n\n\
                What should happen?\n\n\
                  1. Show a message\n\
                  2. Give player health\n\
                  3. Give player an item\n\
                  4. Teleport player to another room\n\
                  5. Custom script (advanced)\n\n\
                Reply with the number:",
                object_name, trigger_name
            ))
        }
        
        WizardState::ChooseAction { object_id, object_name, trigger_type } => {
            // Parse action selection and generate script
            let script = match input {
                "1" => {
                    // Message action - move to customize step
                    session.state = WizardState::CustomizeAction {
                        object_id: object_id.clone(),
                        object_name: object_name.clone(),
                        trigger_type: trigger_type.clone(),
                        action_template: "message".to_string(),
                    };
                    
                    return Ok(
                        "🧙 Trigger Wizard - Step 4: Customize Message\n\n\
                        What message should be displayed?\n\n\
                        Reply with your message text:".to_string()
                    );
                }
                "2" => "Give player 25 health".to_string(),
                "3" => {
                    // Item action - move to customize step
                    session.state = WizardState::CustomizeAction {
                        object_id: object_id.clone(),
                        object_name: object_name.clone(),
                        trigger_type: trigger_type.clone(),
                        action_template: "item".to_string(),
                    };
                    
                    return Ok(
                        "🧙 Trigger Wizard - Step 4: Customize Item\n\n\
                        What item should be given?\n\n\
                        Reply with the item name:".to_string()
                    );
                }
                "4" => {
                    // Teleport action - move to customize step
                    session.state = WizardState::CustomizeAction {
                        object_id: object_id.clone(),
                        object_name: object_name.clone(),
                        trigger_type: trigger_type.clone(),
                        action_template: "teleport".to_string(),
                    };
                    
                    return Ok(
                        "🧙 Trigger Wizard - Step 4: Customize Teleport\n\n\
                        What room ID should player be teleported to?\n\n\
                        Reply with the room ID:".to_string()
                    );
                }
                "5" => {
                    // Custom script - move to customize step
                    session.state = WizardState::CustomizeAction {
                        object_id: object_id.clone(),
                        object_name: object_name.clone(),
                        trigger_type: trigger_type.clone(),
                        action_template: "custom".to_string(),
                    };
                    
                    return Ok(
                        "🧙 Trigger Wizard - Step 4: Custom Script\n\n\
                        Enter your custom script (natural language or DSL):\n\n\
                        Reply with your script:".to_string()
                    );
                }
                _ => return Err(TinyMushError::NotFound("Please enter 1, 2, 3, 4, or 5.".to_string())),
            };
            
            // For non-customizable actions, save immediately
            finalize_wizard_trigger(&object_id, &object_name, trigger_type, &script, store)
        }
        
        WizardState::CustomizeAction { object_id, object_name, trigger_type, action_template } => {
            // Generate final script based on template and user input
            let script = match action_template.as_str() {
                "message" => format!("Say \"{}\"", input),
                "item" => format!("Give player item {}", input),
                "teleport" => format!("Teleport player to {}", input),
                "custom" => input.to_string(),
                _ => input.to_string(),
            };
            
            // Save the trigger
            finalize_wizard_trigger(&object_id, &object_name, trigger_type, &script, store)
        }
    }
}

/// Finalize and save wizard-created trigger
fn finalize_wizard_trigger(
    object_id: &str,
    object_name: &str,
    trigger_type: ObjectTrigger,
    script: &str,
    store: &TinyMushStore,
) -> Result<String, TinyMushError> {
    // Validate script
    let _ast = parse_script(script).map_err(|e| {
        TinyMushError::NotFound(format!("Script error: {}", e))
    })?;
    
    // Get and update object
    let mut object = store.get_object(object_id)?;
    let trigger_type_display = format!("{:?}", trigger_type);
    object.actions.insert(trigger_type, script.to_string());
    store.put_object(object)?;
    
    // Success message
    Ok(format!(
        "✨ Trigger Created!\n\n\
        Object: {}\n\
        Trigger: {}\n\
        Script: {}\n\n\
        Your trigger is now active!",
        object_name,
        trigger_type_display,
        script
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_trigger_type_examine() {
        assert_eq!(parse_trigger_type("examine").unwrap(), ObjectTrigger::OnLook);
        assert_eq!(parse_trigger_type("look").unwrap(), ObjectTrigger::OnLook);
        assert_eq!(parse_trigger_type("inspect").unwrap(), ObjectTrigger::OnLook);
        assert_eq!(parse_trigger_type("EXAMINE").unwrap(), ObjectTrigger::OnLook);
    }
    
    #[test]
    fn test_parse_trigger_type_use() {
        assert_eq!(parse_trigger_type("use").unwrap(), ObjectTrigger::OnUse);
        assert_eq!(parse_trigger_type("activate").unwrap(), ObjectTrigger::OnUse);
    }
    
    #[test]
    fn test_parse_trigger_type_take() {
        assert_eq!(parse_trigger_type("take").unwrap(), ObjectTrigger::OnTake);
        assert_eq!(parse_trigger_type("get").unwrap(), ObjectTrigger::OnTake);
        assert_eq!(parse_trigger_type("pickup").unwrap(), ObjectTrigger::OnTake);
    }
    
    #[test]
    fn test_parse_trigger_type_drop() {
        assert_eq!(parse_trigger_type("drop").unwrap(), ObjectTrigger::OnDrop);
        assert_eq!(parse_trigger_type("put").unwrap(), ObjectTrigger::OnDrop);
    }
    
    #[test]
    fn test_parse_trigger_type_invalid() {
        assert!(parse_trigger_type("invalid").is_err());
        assert!(parse_trigger_type("").is_err());
    }
    
    #[test]
    fn test_can_modify_object_owner() {
        let object = ObjectRecord::new_world("test_id", "Test Object", "A test");
        // World objects can't be modified by regular users
        assert!(!can_modify_object(&object, "alice"));
    }
    
    #[test]
    fn test_script_builder_new() {
        let builder = ScriptBuilder::new(
            "test_id".to_string(),
            "Test Object".to_string(),
            ObjectTrigger::OnUse,
        );
        
        assert_eq!(builder.object_id, "test_id");
        assert_eq!(builder.object_name, "Test Object");
        assert_eq!(builder.trigger_type, ObjectTrigger::OnUse);
        assert_eq!(builder.line_count(), 0);
    }
    
    #[test]
    fn test_script_builder_add_lines() {
        let mut builder = ScriptBuilder::new(
            "test_id".to_string(),
            "Test Object".to_string(),
            ObjectTrigger::OnUse,
        );
        
        builder.add_line("Say \"Hello!\"".to_string());
        builder.add_line("Give player 50 health".to_string());
        
        assert_eq!(builder.line_count(), 2);
        assert_eq!(builder.get_script(), "Say \"Hello!\"\nGive player 50 health");
    }
    
    #[test]
    fn test_script_builder_empty_script() {
        let builder = ScriptBuilder::new(
            "test_id".to_string(),
            "Test Object".to_string(),
            ObjectTrigger::OnUse,
        );
        
        assert_eq!(builder.get_script(), "");
        assert_eq!(builder.line_count(), 0);
    }
    
    #[test]
    fn test_handle_cancel_command() {
        let result = handle_cancel_command();
        assert!(result.contains("cancelled"));
    }
    
    #[test]
    fn test_wizard_session_new() {
        let objects = vec![
            ("obj1".to_string(), "Crystal".to_string()),
            ("obj2".to_string(), "Wand".to_string()),
        ];
        let session = WizardSession::new(objects.clone());
        
        assert_eq!(session.state, WizardState::ChooseObject);
        assert_eq!(session.available_objects.len(), 2);
        assert_eq!(session.available_objects[0].0, "obj1");
        assert_eq!(session.available_objects[1].1, "Wand");
    }
    
    #[test]
    fn test_wizard_state_choose_trigger() {
        let state = WizardState::ChooseTrigger {
            object_id: "obj1".to_string(),
            object_name: "Magic Orb".to_string(),
        };
        
        match state {
            WizardState::ChooseTrigger { object_id, object_name } => {
                assert_eq!(object_id, "obj1");
                assert_eq!(object_name, "Magic Orb");
            }
            _ => panic!("Expected ChooseTrigger state"),
        }
    }
    
    #[test]
    fn test_wizard_state_choose_action() {
        let state = WizardState::ChooseAction {
            object_id: "obj1".to_string(),
            object_name: "Potion".to_string(),
            trigger_type: ObjectTrigger::OnUse,
        };
        
        match state {
            WizardState::ChooseAction { object_id, object_name, trigger_type } => {
                assert_eq!(object_id, "obj1");
                assert_eq!(object_name, "Potion");
                assert_eq!(trigger_type, ObjectTrigger::OnUse);
            }
            _ => panic!("Expected ChooseAction state"),
        }
    }
    
    #[test]
    fn test_wizard_state_customize_action() {
        let state = WizardState::CustomizeAction {
            object_id: "obj1".to_string(),
            object_name: "Sign".to_string(),
            trigger_type: ObjectTrigger::OnLook,
            action_template: "message".to_string(),
        };
        
        match state {
            WizardState::CustomizeAction { object_id, object_name, trigger_type, action_template } => {
                assert_eq!(object_id, "obj1");
                assert_eq!(object_name, "Sign");
                assert_eq!(trigger_type, ObjectTrigger::OnLook);
                assert_eq!(action_template, "message");
            }
            _ => panic!("Expected CustomizeAction state"),
        }
    }
    
    #[test]
    fn test_wizard_state_transitions_object_to_trigger() {
        // Simulates state transition from ChooseObject to ChooseTrigger
        let initial = WizardState::ChooseObject;
        assert_eq!(initial, WizardState::ChooseObject);
        
        // After user selects object, state becomes:
        let next = WizardState::ChooseTrigger {
            object_id: "obj1".to_string(),
            object_name: "Crystal".to_string(),
        };
        
        match next {
            WizardState::ChooseTrigger { .. } => (), // Success
            _ => panic!("State should be ChooseTrigger"),
        }
    }
    
    #[test]
    fn test_wizard_state_transitions_trigger_to_action() {
        // Simulates state transition from ChooseTrigger to ChooseAction
        let _initial = WizardState::ChooseTrigger {
            object_id: "obj1".to_string(),
            object_name: "Crystal".to_string(),
        };
        
        // After user selects trigger type, state becomes:
        let next = WizardState::ChooseAction {
            object_id: "obj1".to_string(),
            object_name: "Crystal".to_string(),
            trigger_type: ObjectTrigger::OnLook,
        };
        
        match next {
            WizardState::ChooseAction { trigger_type, .. } => {
                assert_eq!(trigger_type, ObjectTrigger::OnLook);
            }
            _ => panic!("State should be ChooseAction"),
        }
    }
}

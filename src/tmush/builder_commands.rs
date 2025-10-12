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
}

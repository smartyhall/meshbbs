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
}

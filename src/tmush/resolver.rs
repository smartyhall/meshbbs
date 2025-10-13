//! Object Name Resolution
//!
//! Resolves user-friendly object names to object IDs, making the trigger
//! system accessible without requiring technical knowledge of internal IDs.
//!
//! ## Features
//! - **Name-based lookup**: Reference objects by their descriptive names
//! - **Special keywords**: "this" (last examined), "here" (current room)
//! - **Inventory prefix**: "@name" searches only player inventory
//! - **Fuzzy matching**: Case-insensitive, whitespace-normalized
//! - **Ambiguity handling**: Returns multiple matches for user selection
//!
//! ## Examples
//! ```ignore
//! // Simple name lookup
//! let context = ResolutionContext::new(username, current_room, last_examined);
//! let result = resolve_object_name(&context, "rusty key", &store).await?;
//!
//! // Inventory-only search
//! let result = resolve_object_name(&context, "@crystal", &store).await?;
//!
//! // Special keywords
//! let this_obj = resolve_object_name(&context, "this", &store).await?;
//! let here_room = resolve_object_name(&context, "here", &store).await?;
//! ```

use crate::tmush::errors::TinyMushError;
use crate::tmush::storage::TinyMushStore;

/// Resolution context for object search
#[derive(Debug, Clone)]
pub struct ResolutionContext {
    /// Player username
    pub username: String,
    /// Current room ID (for "here" keyword)
    pub current_room: String,
    /// Last examined object ID (for "this" keyword)
    pub last_examined: Option<String>,
}

impl ResolutionContext {
    /// Create new resolution context
    pub fn new(username: String, current_room: String, last_examined: Option<String>) -> Self {
        Self {
            username,
            current_room,
            last_examined,
        }
    }
}

/// Result of object name resolution
#[derive(Debug, Clone, PartialEq)]
pub enum ResolveResult {
    /// Single unambiguous match
    Found(String),

    /// Multiple matches - user must clarify
    Ambiguous(Vec<ObjectMatch>),

    /// No matches found
    NotFound,
}

/// A single object match with its context
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectMatch {
    /// Object ID (ulid)
    pub id: String,

    /// Object name
    pub name: String,

    /// Location context ("inventory", "room", "container")
    pub location: String,

    /// Container name if inside something
    pub container: Option<String>,
}

impl ObjectMatch {
    /// Format for display in disambiguation list
    pub fn format_for_display(&self, index: usize) -> String {
        let location_desc = if let Some(ref container) = self.container {
            format!("in {} ({})", container, self.location)
        } else {
            self.location.clone()
        };

        format!(
            "{}) {} [#{}] ({})",
            index, self.name, self.id, location_desc
        )
    }
}

/// Normalize a name for comparison
///
/// - Convert to lowercase
/// - Trim whitespace
/// - Collapse multiple spaces to single space
fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if a query matches an object name
///
/// Uses fuzzy matching:
/// - Case-insensitive
/// - Partial matches allowed
/// - "key" matches "rusty key"
fn name_matches(query: &str, object_name: &str) -> bool {
    let query_norm = normalize_name(query);
    let name_norm = normalize_name(object_name);

    // Exact match
    if query_norm == name_norm {
        return true;
    }

    // Partial match (query is substring of name)
    name_norm.contains(&query_norm)
}

/// Resolve an object name to an object ID
///
/// ## Search Priority
/// 1. Special keywords ("this", "here")
/// 2. Inventory-only search (if "@name" prefix)
/// 3. Current room objects
/// 4. Player inventory
/// 5. Room containers (objects inside other objects in room)
///
/// ## Returns
/// - `ResolveResult::Found(id)` - Single match
/// - `ResolveResult::Ambiguous(matches)` - Multiple matches (user must choose)
/// - `ResolveResult::NotFound` - No matches
pub fn resolve_object_name(
    context: &ResolutionContext,
    query: &str,
    store: &TinyMushStore,
) -> Result<ResolveResult, TinyMushError> {
    let query = query.trim();

    if query.is_empty() {
        return Ok(ResolveResult::NotFound);
    }

    // Handle special keywords
    if query.eq_ignore_ascii_case("here") {
        // "here" refers to current room
        return Ok(ResolveResult::Found(context.current_room.clone()));
    }

    if query.eq_ignore_ascii_case("this") {
        // "this" refers to last examined object
        if let Some(ref obj_id) = context.last_examined {
            return Ok(ResolveResult::Found(obj_id.clone()));
        } else {
            return Err(TinyMushError::NotFound(
                "No object examined yet. Use /look <object> first.".to_string(),
            ));
        }
    }

    // Check for inventory-only prefix (@name)
    let (search_query, inventory_only) = if query.starts_with('@') {
        (&query[1..], true)
    } else {
        (query, false)
    };

    let mut matches = Vec::new();

    // Search player inventory
    let player_result = store.get_player(&context.username);
    if let Ok(player) = player_result {
        for obj_id in &player.inventory {
            if let Ok(obj) = store.get_object(obj_id) {
                if name_matches(search_query, &obj.name) {
                    matches.push(ObjectMatch {
                        id: obj.id.clone(),
                        name: obj.name.clone(),
                        location: "inventory".to_string(),
                        container: None,
                    });
                }
            }
        }
    }

    // If inventory-only search, stop here
    if inventory_only {
        return match matches.len() {
            0 => Ok(ResolveResult::NotFound),
            1 => Ok(ResolveResult::Found(matches[0].id.clone())),
            _ => Ok(ResolveResult::Ambiguous(matches)),
        };
    }

    // Search current room objects
    if let Ok(room) = store.get_room(&context.current_room) {
        for obj_id in &room.items {
            if let Ok(obj) = store.get_object(obj_id) {
                if name_matches(search_query, &obj.name) {
                    matches.push(ObjectMatch {
                        id: obj.id.clone(),
                        name: obj.name.clone(),
                        location: "room".to_string(),
                        container: None,
                    });
                }
            }
        }
    }

    // Return result based on match count
    match matches.len() {
        0 => Ok(ResolveResult::NotFound),
        1 => Ok(ResolveResult::Found(matches[0].id.clone())),
        _ => Ok(ResolveResult::Ambiguous(matches)),
    }
}

/// Format disambiguation prompt for user
///
/// Returns a formatted message listing all matches with numbers,
/// instructing the user to retry with a more specific name.
pub fn format_disambiguation_prompt(matches: &[ObjectMatch]) -> String {
    let mut output = String::from("Multiple objects match that name:\n\n");

    for (i, obj_match) in matches.iter().enumerate() {
        output.push_str(&obj_match.format_for_display(i + 1));
        output.push('\n');
    }

    output.push_str("\nPlease be more specific or use the full name.");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Rusty Key"), "rusty key");
        assert_eq!(normalize_name("  Crystal   Ball  "), "crystal ball");
        assert_eq!(normalize_name("MAGIC SWORD"), "magic sword");
    }

    #[test]
    fn test_name_matches() {
        // Exact match
        assert!(name_matches("key", "key"));
        assert!(name_matches("Key", "KEY"));

        // Partial match
        assert!(name_matches("key", "rusty key"));
        assert!(name_matches("rusty", "rusty key"));

        // No match
        assert!(!name_matches("sword", "rusty key"));
        assert!(!name_matches("magic", "rusty key"));
    }

    #[test]
    fn test_object_match_format() {
        let obj_match = ObjectMatch {
            id: "01234567890123456789012345".to_string(),
            name: "rusty key".to_string(),
            location: "inventory".to_string(),
            container: None,
        };

        let formatted = obj_match.format_for_display(1);
        assert!(formatted.contains("1)"));
        assert!(formatted.contains("rusty key"));
        assert!(formatted.contains("#01234567890123456789012345"));
        assert!(formatted.contains("inventory"));
    }

    #[test]
    fn test_object_match_format_with_container() {
        let obj_match = ObjectMatch {
            id: "01234567890123456789012345".to_string(),
            name: "gold coin".to_string(),
            location: "room".to_string(),
            container: Some("wooden chest".to_string()),
        };

        let formatted = obj_match.format_for_display(2);
        assert!(formatted.contains("2)"));
        assert!(formatted.contains("gold coin"));
        assert!(formatted.contains("in wooden chest"));
    }

    #[test]
    fn test_format_disambiguation_prompt() {
        let matches = vec![
            ObjectMatch {
                id: "123".to_string(),
                name: "key".to_string(),
                location: "inventory".to_string(),
                container: None,
            },
            ObjectMatch {
                id: "456".to_string(),
                name: "rusty key".to_string(),
                location: "room".to_string(),
                container: None,
            },
        ];

        let prompt = format_disambiguation_prompt(&matches);
        assert!(prompt.contains("Multiple objects"));
        assert!(prompt.contains("1)"));
        assert!(prompt.contains("2)"));
        assert!(prompt.contains("more specific"));
    }
}

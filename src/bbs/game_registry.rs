/// Registry for door game resources and state.
///
/// This registry holds all game-specific storage, state, and resources
/// in a single location, allowing the BBS core to remain game-agnostic.
/// New door games can be added by extending this registry without
/// modifying the BBS command processor signatures.
use crate::tmush::storage::TinyMushStore;

#[derive(Clone)]
pub struct GameRegistry {
    tinymush_store: Option<TinyMushStore>,
    // Future door games will add their resources here:
    // tinyhack_store: Option<TinyHackStore>,
    // adventure_store: Option<AdventureStore>,
}

impl GameRegistry {
    /// Create an empty game registry with no games registered.
    pub fn new() -> Self {
        GameRegistry {
            tinymush_store: None,
        }
    }

    /// Register the TinyMUSH game with its storage backend.
    pub fn with_tinymush(mut self, store: TinyMushStore) -> Self {
        self.tinymush_store = Some(store);
        self
    }

    /// Get the TinyMUSH storage backend if registered.
    pub fn get_tinymush_store(&self) -> Option<&TinyMushStore> {
        self.tinymush_store.as_ref()
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

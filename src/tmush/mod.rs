//! TinyMUSH data model and persistence scaffolding.
//! Phase 1 introduces foundational data types, Sled-backed storage, and
//! canonical world seeding helpers so higher phases can focus on command
//! routing and session plumbing.

pub mod commands;
pub mod currency;
pub mod errors;
pub mod room_manager;
pub mod state;
pub mod storage;
pub mod types;

pub use commands::{handle_tinymush_command, should_route_to_tinymush, TinyMushCommand};
pub use currency::{
    convert_decimal_to_multi_tier, convert_multi_tier_to_decimal, format_currency,
    parse_currency, STANDARD_CONVERSION_RATIO,
};
pub use errors::TinyMushError;
pub use state::{
    canonical_world_seed, OLD_TOWNE_WORLD_ROOM_IDS, REQUIRED_LANDING_LOCATION_ID,
    REQUIRED_START_LOCATION_ID,
};
pub use storage::{TinyMushStore, TinyMushStoreBuilder};
pub use types::*;

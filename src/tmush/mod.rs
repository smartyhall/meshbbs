//! TinyMUSH data model and persistence scaffolding.
//! Phase 1 introduces foundational data types, Sled-backed storage, and
//! canonical world seeding helpers so higher phases can focus on command
//! routing and session plumbing.

pub mod achievement;
pub mod commands;
pub mod currency;
pub mod errors;
pub mod inventory;
pub mod quest;
pub mod room_manager;
pub mod shop;
pub mod state;
pub mod storage;
pub mod tutorial;
pub mod types;

pub use achievement::{
    award_achievement, check_trigger, get_achievements_by_category, get_available_achievements,
    get_earned_achievements, update_achievement_progress,
};
pub use commands::{handle_tinymush_command, should_route_to_tinymush, TinyMushCommand};
pub use currency::{
    convert_decimal_to_multi_tier, convert_multi_tier_to_decimal, format_currency,
    parse_currency, STANDARD_CONVERSION_RATIO,
};
pub use errors::TinyMushError;
pub use inventory::{
    add_item_to_inventory, calculate_total_weight, can_add_item, format_inventory_compact,
    format_item_examination, get_item_quantity, has_item, remove_item_from_inventory,
};
pub use quest::{
    abandon_quest, accept_quest, can_accept_quest, complete_quest, format_quest_list,
    format_quest_status, get_active_quests, get_available_quests, get_completed_quests,
    update_quest_objective,
};
pub use shop::{format_shop_listing, format_shop_item_detail, ShopConfig, ShopItem, ShopRecord};
pub use state::{
    canonical_world_seed, seed_starter_achievements, seed_starter_quests,
    OLD_TOWNE_WORLD_ROOM_IDS, REQUIRED_LANDING_LOCATION_ID, REQUIRED_START_LOCATION_ID,
};
pub use storage::{TinyMushStore, TinyMushStoreBuilder};
pub use tutorial::{
    advance_tutorial_step, can_advance_from_location, distribute_tutorial_rewards,
    format_tutorial_status, get_tutorial_hint, restart_tutorial, should_auto_start_tutorial,
    skip_tutorial, start_tutorial,
};
pub use types::*;

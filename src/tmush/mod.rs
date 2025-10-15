//! TinyMUSH data model and persistence scaffolding.
//! Phase 1 introduces foundational data types, Sled-backed storage, and
//! canonical world seeding helpers so higher phases can focus on command
//! routing and session plumbing.

pub mod achievement;
pub mod builder_commands;
pub mod clone;
pub mod commands;
pub mod companion;
pub mod currency;
pub mod currency_migration;
pub mod errors;
pub mod housing_cleanup;
pub mod inventory;
pub mod migration;
pub mod quest;
pub mod resolver;
pub mod room_manager;
pub mod seed_loader;
pub mod shop;
pub mod state;
pub mod storage;
pub mod trigger;
pub mod tutorial;
pub mod types;

pub use achievement::{
    award_achievement, check_trigger, get_achievements_by_category, get_available_achievements,
    get_earned_achievements, update_achievement_progress,
};
pub use builder_commands::{
    handle_cancel_command, handle_done_command, handle_remove_command, handle_script_command,
    handle_show_command, handle_test_command, handle_when_command, handle_wizard_command,
    handle_wizard_step, ScriptBuilder, WizardSession, WizardState,
};
pub use clone::{
    clone_object, handle_clone_command, CLONES_PER_HOUR, CLONE_COOLDOWN, MAX_CLONABLE_VALUE,
    MAX_CLONE_DEPTH, MAX_OBJECTS_PER_PLAYER,
};
pub use commands::{handle_tinymush_command, should_route_to_tinymush, TinyMushCommand};
pub use companion::{
    auto_follow_companions, dismount_companion, feed_companion, find_companion_in_room,
    format_companion_list, format_companion_status, get_player_companions, mount_companion,
    move_companion_to_room, pet_companion, release_companion, tame_companion,
};
pub use currency::{
    convert_decimal_to_multi_tier, convert_multi_tier_to_decimal, format_currency, parse_currency,
    STANDARD_CONVERSION_RATIO,
};
pub use errors::TinyMushError;
pub use housing_cleanup::{
    check_and_cleanup_housing, list_abandoned_housing, AbandonedHousingInfo, CleanupConfig,
    CleanupStats,
};
pub use inventory::{
    add_item_to_inventory, calculate_total_weight, can_add_item, format_inventory_compact,
    format_item_examination, get_item_quantity, has_item, remove_item_from_inventory,
};
pub use quest::{
    abandon_quest, accept_quest, can_accept_quest, complete_quest, format_quest_list,
    format_quest_status, get_active_quests, get_available_quests, get_completed_quests,
    update_quest_objective,
};
pub use resolver::{format_disambiguation_prompt, resolve_object_name, ObjectMatch, ResolveResult};
pub use seed_loader::{
    load_achievements_from_json, load_companions_from_json, load_npcs_from_json,
    load_quests_from_json, load_recipes_from_json, load_rooms_from_json,
};
pub use shop::{format_shop_item_detail, format_shop_listing, ShopConfig, ShopItem, ShopRecord};
pub use state::{
    canonical_world_seed, seed_starter_achievements, seed_starter_companions, seed_starter_npcs,
    seed_starter_quests, OLD_TOWNE_WORLD_ROOM_IDS, REQUIRED_LANDING_LOCATION_ID,
    REQUIRED_START_LOCATION_ID,
};
pub use storage::{TinyMushStore, TinyMushStoreBuilder};
pub use tutorial::{
    advance_tutorial_step, can_advance_from_location, distribute_tutorial_rewards,
    format_tutorial_status, get_tutorial_hint, restart_tutorial, should_auto_start_tutorial,
    skip_tutorial, start_tutorial,
};
pub use types::*;

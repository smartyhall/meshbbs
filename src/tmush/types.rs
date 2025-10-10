use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PLAYER_SCHEMA_VERSION: u8 = 1;
pub const ROOM_SCHEMA_VERSION: u8 = 1;
pub const OBJECT_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomVisibility {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomOwner {
    World,
    Player { username: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RoomFlag {
    Safe,
    Dark,
    Indoor,
    Shop,
    QuestLocation,
    PvpEnabled,
    PlayerCreated,
    Private,
    Moderated,
    Instanced,
    Crowded,
    HousingOffice,  // Room provides housing rental/purchase services
    NoTeleportOut,  // Players cannot teleport out of this room (PvP arenas, quest dungeons, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ObjectFlag {
    QuestItem,
    Consumable,
    Equipment,
    KeyItem,
    Container,
    Magical,
    Companion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ObjectTrigger {
    OnEnter,
    OnLook,
    OnTake,
    OnDrop,
    OnUse,
    OnPoke,
    OnFollow,
    OnIdle,
    OnCombat,
    OnHeal,
}

pub type ObjectActions = HashMap<ObjectTrigger, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectOwner {
    World,
    Player { username: String },
}

/// Tutorial progression state for new player onboarding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TutorialState {
    /// Player has not started tutorial
    NotStarted,
    /// Player is in progress at a specific step
    InProgress { step: TutorialStep },
    /// Player has completed tutorial
    Completed { completed_at: DateTime<Utc> },
    /// Player manually skipped tutorial
    Skipped { skipped_at: DateTime<Utc> },
}

impl Default for TutorialState {
    fn default() -> Self {
        Self::NotStarted
    }
}

/// Tutorial steps for Gazebo → Mayor → City Hall flow
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TutorialStep {
    /// Step 1: Learn basics at gazebo
    WelcomeAtGazebo,
    /// Step 2: Navigate to City Hall
    NavigateToCityHall,
    /// Step 3: Meet the Mayor
    MeetTheMayor,
}

/// NPC flags for behavior and classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NpcFlag {
    TutorialNpc,
    QuestGiver,
    Vendor,
    Guard,
    Immortal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: ObjectOwner,
    pub created_at: DateTime<Utc>,
    pub weight: u8,
    /// New currency-aware value field
    #[serde(default)]
    pub currency_value: CurrencyAmount,
    /// Legacy value field for backward compatibility (deprecated)
    #[serde(default)]
    pub value: u32,
    pub takeable: bool,
    pub usable: bool,
    #[serde(default)]
    pub actions: ObjectActions,
    #[serde(default)]
    pub flags: Vec<ObjectFlag>,
    pub schema_version: u8,
}

impl ObjectRecord {
    pub fn new_world(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            owner: ObjectOwner::World,
            created_at: Utc::now(),
            weight: 0,
            currency_value: CurrencyAmount::default(),
            value: 0,
            takeable: false,
            usable: false,
            actions: HashMap::new(),
            flags: Vec::new(),
            schema_version: OBJECT_SCHEMA_VERSION,
        }
    }
}

/// NPC (Non-Player Character) record for tutorial guides, quest givers, and vendors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcRecord {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub room_id: String,
    pub dialog: HashMap<String, String>, // key -> response text
    #[serde(default)]
    pub flags: Vec<NpcFlag>,
    pub created_at: DateTime<Utc>,
    pub schema_version: u8,
}

impl NpcRecord {
    pub fn new(id: &str, name: &str, title: &str, description: &str, room_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            room_id: room_id.to_string(),
            dialog: HashMap::new(),
            flags: Vec::new(),
            created_at: Utc::now(),
            schema_version: 1,
        }
    }

    pub fn with_dialog(mut self, key: &str, response: &str) -> Self {
        self.dialog.insert(key.to_string(), response.to_string());
        self
    }

    pub fn with_flag(mut self, flag: NpcFlag) -> Self {
        if !self.flags.contains(&flag) {
            self.flags.push(flag);
        }
        self
    }
}

// ============================================================================
// Companion System (Phase 6 Week 4)
// ============================================================================

/// Companion types available in the game
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CompanionType {
    Horse,
    Dog,
    Cat,
    Familiar,
    Mercenary,
    Construct,
}

/// Companion behaviors that trigger automatically
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanionBehavior {
    /// Automatically follow owner between rooms
    AutoFollow,
    /// Occasionally say idle messages
    IdleChatter { messages: Vec<String> },
    /// Alert when danger is nearby
    AlertDanger,
    /// Assist in combat
    CombatAssist { damage_bonus: u32 },
    /// Provide healing over time
    Healing { heal_amount: u32, cooldown_seconds: u64 },
    /// Carry extra items (saddlebags, backpacks)
    ExtraStorage { capacity: u32 },
    /// Boost specific skill
    SkillBoost { skill: String, bonus: u32 },
}

/// Companion state and stats
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompanionRecord {
    pub id: String,
    pub name: String,
    pub companion_type: CompanionType,
    pub description: String,
    /// Owner's username (None if unowned/wild)
    pub owner: Option<String>,
    /// Current room location
    pub room_id: String,
    /// Loyalty level (0-100)
    pub loyalty: u32,
    /// Happiness level (0-100)
    pub happiness: u32,
    /// Last time companion was fed
    pub last_fed: Option<DateTime<Utc>>,
    /// Active behaviors
    #[serde(default)]
    pub behaviors: Vec<CompanionBehavior>,
    /// Companion's inventory (for storage behaviors)
    #[serde(default)]
    pub inventory: Vec<String>,
    /// Whether player is currently mounted (for horses)
    #[serde(default)]
    pub is_mounted: bool,
    pub created_at: DateTime<Utc>,
    pub schema_version: u8,
}

impl CompanionRecord {
    pub fn new(id: &str, name: &str, companion_type: CompanionType, room_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            companion_type,
            description: Self::default_description(companion_type),
            owner: None,
            room_id: room_id.to_string(),
            loyalty: 50,
            happiness: 100,
            last_fed: None,
            behaviors: Self::default_behaviors(companion_type),
            inventory: Vec::new(),
            is_mounted: false,
            created_at: Utc::now(),
            schema_version: 1,
        }
    }

    fn default_description(companion_type: CompanionType) -> String {
        match companion_type {
            CompanionType::Horse => "A sturdy horse with a gentle temperament.".to_string(),
            CompanionType::Dog => "A loyal dog with bright, intelligent eyes.".to_string(),
            CompanionType::Cat => "An independent cat with soft fur.".to_string(),
            CompanionType::Familiar => "A magical familiar crackling with arcane energy.".to_string(),
            CompanionType::Mercenary => "A skilled warrior ready for combat.".to_string(),
            CompanionType::Construct => "A mechanical construct powered by ancient magic.".to_string(),
        }
    }

    fn default_behaviors(companion_type: CompanionType) -> Vec<CompanionBehavior> {
        match companion_type {
            CompanionType::Horse => vec![
                CompanionBehavior::AutoFollow,
                CompanionBehavior::ExtraStorage { capacity: 20 },
            ],
            CompanionType::Dog => vec![
                CompanionBehavior::AutoFollow,
                CompanionBehavior::AlertDanger,
                CompanionBehavior::IdleChatter {
                    messages: vec!["*wags tail*".to_string(), "*barks happily*".to_string()],
                },
            ],
            CompanionType::Cat => vec![
                CompanionBehavior::IdleChatter {
                    messages: vec!["*purrs contentedly*".to_string(), "*meows softly*".to_string()],
                },
            ],
            CompanionType::Familiar => vec![
                CompanionBehavior::AutoFollow,
                CompanionBehavior::SkillBoost {
                    skill: "magic".to_string(),
                    bonus: 10,
                },
            ],
            CompanionType::Mercenary => vec![
                CompanionBehavior::AutoFollow,
                CompanionBehavior::CombatAssist { damage_bonus: 15 },
            ],
            CompanionType::Construct => vec![
                CompanionBehavior::AutoFollow,
                CompanionBehavior::ExtraStorage { capacity: 30 },
                CompanionBehavior::CombatAssist { damage_bonus: 10 },
            ],
        }
    }

    pub fn with_owner(mut self, owner: &str) -> Self {
        self.owner = Some(owner.to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn add_behavior(mut self, behavior: CompanionBehavior) -> Self {
        self.behaviors.push(behavior);
        self
    }

    /// Check if companion needs feeding (>24 hours since last_fed)
    pub fn needs_feeding(&self) -> bool {
        if let Some(last_fed) = self.last_fed {
            let hours_since_fed = Utc::now().signed_duration_since(last_fed).num_hours();
            hours_since_fed > 24
        } else {
            true // Never fed
        }
    }

    /// Feed the companion, increasing happiness
    pub fn feed(&mut self) -> u32 {
        self.last_fed = Some(Utc::now());
        let happiness_gain = if self.happiness < 50 { 20 } else { 10 };
        self.happiness = (self.happiness + happiness_gain).min(100);
        happiness_gain
    }

    /// Pet/interact with companion, increasing loyalty
    pub fn pet(&mut self) -> u32 {
        let loyalty_gain = if self.loyalty < 50 { 5 } else { 2 };
        self.loyalty = (self.loyalty + loyalty_gain).min(100);
        loyalty_gain
    }

    /// Get happiness decay rate (decreases over time if not fed)
    pub fn apply_happiness_decay(&mut self) {
        if self.needs_feeding() {
            self.happiness = self.happiness.saturating_sub(10);
        }
    }

    /// Check if companion has auto-follow behavior
    pub fn has_auto_follow(&self) -> bool {
        self.behaviors
            .iter()
            .any(|b| matches!(b, CompanionBehavior::AutoFollow))
    }

    /// Get extra storage capacity from behaviors
    pub fn storage_capacity(&self) -> u32 {
        self.behaviors
            .iter()
            .filter_map(|b| match b {
                CompanionBehavior::ExtraStorage { capacity } => Some(*capacity),
                _ => None,
            })
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomRecord {
    pub id: String,
    pub name: String,
    pub short_desc: String,
    pub long_desc: String,
    pub owner: RoomOwner,
    pub created_at: DateTime<Utc>,
    pub visibility: RoomVisibility,
    pub exits: HashMap<Direction, String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub flags: Vec<RoomFlag>,
    pub max_capacity: u16,
    /// If this is a housing office, filter templates by these tags (empty = show all)
    #[serde(default)]
    pub housing_filter_tags: Vec<String>,
    pub schema_version: u8,
}

impl RoomRecord {
    pub fn world(id: &str, name: &str, short_desc: &str, long_desc: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            short_desc: short_desc.to_string(),
            long_desc: long_desc.to_string(),
            owner: RoomOwner::World,
            created_at: Utc::now(),
            visibility: RoomVisibility::Public,
            exits: HashMap::new(),
            items: Vec::new(),
            flags: Vec::new(),
            max_capacity: 15,
            housing_filter_tags: Vec::new(),
            schema_version: ROOM_SCHEMA_VERSION,
        }
    }

    pub fn with_exit(mut self, direction: Direction, destination: &str) -> Self {
        self.exits.insert(direction, destination.to_string());
        self
    }

    pub fn with_flag(mut self, flag: RoomFlag) -> Self {
        if !self.flags.contains(&flag) {
            self.flags.push(flag);
        }
        self
    }

    pub fn with_capacity(mut self, capacity: u16) -> Self {
        self.max_capacity = capacity;
        self
    }

    pub fn with_created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }
}

// ============================================================================
// Housing System (Phase 7 Week 1-2)
// ============================================================================

/// Permissions that control what housing instance owners can customize
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HousingPermissions {
    /// Can owner customize room descriptions
    pub can_edit_description: bool,
    /// Can owner place furniture and decorations
    pub can_add_objects: bool,
    /// Can owner invite guests to their housing
    pub can_invite_guests: bool,
    /// Can owner modify room structure (rare, premium feature)
    pub can_build: bool,
    /// Can owner set room flags (private, dark, etc.)
    pub can_set_flags: bool,
    /// Can owner set custom exit names
    pub can_rename_exits: bool,
}

impl Default for HousingPermissions {
    fn default() -> Self {
        Self {
            can_edit_description: true,
            can_add_objects: true,
            can_invite_guests: true,
            can_build: false,
            can_set_flags: false,
            can_rename_exits: false,
        }
    }
}

/// Template room within a housing template
/// Describes a single room that will be cloned when housing is instantiated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HousingTemplateRoom {
    /// Local room ID within template (e.g., "living_room", "bedroom")
    pub room_id: String,
    /// Room name
    pub name: String,
    /// Short description
    pub short_desc: String,
    /// Long description
    pub long_desc: String,
    /// Exits to other rooms in this template (local room IDs)
    pub exits: HashMap<Direction, String>,
    /// Room flags to apply
    #[serde(default)]
    pub flags: Vec<RoomFlag>,
    /// Max capacity for this room
    #[serde(default = "default_room_capacity")]
    pub max_capacity: u16,
}

fn default_room_capacity() -> u16 {
    15
}

/// Housing template - blueprint for creating player housing instances
/// Created by world builders, cloned to create actual player housing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HousingTemplate {
    /// Unique template ID (e.g., "basic_apartment", "luxury_flat", "rural_farm")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description shown in housing catalog
    pub description: String,
    /// Rooms that make up this template
    pub rooms: Vec<HousingTemplateRoom>,
    /// Which room is the entry point (local room ID)
    pub entry_room: String,
    /// Cost to rent/purchase this housing (in base currency units)
    pub cost: i64,
    /// Recurring cost (0 for owned, >0 for rented per time period)
    pub recurring_cost: i64,
    /// What owners are allowed to customize
    pub permissions: HousingPermissions,
    /// Maximum number of instances allowed (-1 for unlimited)
    #[serde(default = "unlimited_instances")]
    pub max_instances: i32,
    /// Tags for filtering by theme/type (e.g., ["modern", "urban"], ["fantasy", "burrow"])
    #[serde(default)]
    pub tags: Vec<String>,
    /// Category for grouping (e.g., "apartment", "house", "burrow", "treehouse")
    #[serde(default)]
    pub category: String,
    /// When this template was created
    pub created_at: DateTime<Utc>,
    /// Who created this template
    pub created_by: String,
    /// Schema version
    pub schema_version: u8,
}

fn unlimited_instances() -> i32 {
    -1
}

impl HousingTemplate {
    pub fn new(id: &str, name: &str, description: &str, created_by: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            rooms: Vec::new(),
            entry_room: String::new(),
            cost: 0,
            recurring_cost: 0,
            permissions: HousingPermissions::default(),
            max_instances: -1,
            tags: Vec::new(),
            category: String::new(),
            created_at: Utc::now(),
            created_by: created_by.to_string(),
            schema_version: 1,
        }
    }

    pub fn with_cost(mut self, cost: i64, recurring: i64) -> Self {
        self.cost = cost;
        self.recurring_cost = recurring;
        self
    }

    pub fn with_permissions(mut self, permissions: HousingPermissions) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_room(mut self, room: HousingTemplateRoom) -> Self {
        self.rooms.push(room);
        self
    }

    pub fn with_entry_room(mut self, room_id: &str) -> Self {
        self.entry_room = room_id.to_string();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    pub fn with_max_instances(mut self, max: i32) -> Self {
        self.max_instances = max;
        self
    }

    /// Check if this template matches the given filter tags (empty filter = match all)
    pub fn matches_filter(&self, filter_tags: &[String]) -> bool {
        if filter_tags.is_empty() {
            return true; // No filter = show all
        }
        // Template must have at least one matching tag
        self.tags.iter().any(|tag| filter_tags.contains(tag))
    }
}

/// Active housing instance - a cloned template owned by a player
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HousingInstance {
    /// Unique instance ID
    pub id: String,
    /// Template this was cloned from
    pub template_id: String,
    /// Owner's username
    pub owner: String,
    /// When this instance was created/rented
    pub created_at: DateTime<Utc>,
    /// When rent was last paid (for recurring costs)
    pub last_payment: DateTime<Utc>,
    /// Mapping of template room IDs to actual room IDs
    /// E.g., "living_room" -> "rooms:instance:alice:basic_apartment:living_room"
    pub room_mappings: HashMap<String, String>,
    /// Entry room ID (actual room ID, not template local ID)
    pub entry_room_id: String,
    /// Guest list (usernames allowed to enter)
    #[serde(default)]
    pub guests: Vec<String>,
    /// Whether instance is currently active
    pub active: bool,
    /// Schema version
    pub schema_version: u8,
}

impl HousingInstance {
    pub fn new(
        id: &str,
        template_id: &str,
        owner: &str,
        entry_room_id: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            template_id: template_id.to_string(),
            owner: owner.to_string(),
            created_at: Utc::now(),
            last_payment: Utc::now(),
            room_mappings: HashMap::new(),
            entry_room_id: entry_room_id.to_string(),
            guests: Vec::new(),
            active: true,
            schema_version: 1,
        }
    }

    pub fn add_guest(&mut self, username: &str) {
        if !self.guests.contains(&username.to_string()) {
            self.guests.push(username.to_string());
        }
    }

    pub fn remove_guest(&mut self, username: &str) {
        self.guests.retain(|g| g != username);
    }

    pub fn is_guest(&self, username: &str) -> bool {
        self.guests.contains(&username.to_string())
    }

    pub fn is_owner(&self, username: &str) -> bool {
        self.owner == username
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombatState {
    pub enemy_id: String,
    pub enemy_hp: u32,
    pub enemy_max_hp: u32,
    pub round: u32,
    pub fled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlayerState {
    Exploring,
    InDialog(String),
    InCombat(CombatState),
    Shopping(String),
    ViewingInventory,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerStats {
    pub hp: u32,
    pub max_hp: u32,
    pub mp: u32,
    pub max_mp: u32,
    pub strength: u8,
    pub dexterity: u8,
    pub intelligence: u8,
    pub constitution: u8,
    pub armor_class: u8,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            hp: 20,
            max_hp: 20,
            mp: 10,
            max_mp: 10,
            strength: 10,
            dexterity: 10,
            intelligence: 10,
            constitution: 10,
            armor_class: 10,
        }
    }
}

/// Quest system data structures for Phase 6 Week 2

/// Quest state tracking for player quest progress
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuestState {
    /// Quest is available but not yet accepted
    Available,
    /// Quest is active and in progress
    Active { started_at: DateTime<Utc> },
    /// Quest has been completed
    Completed { completed_at: DateTime<Utc> },
    /// Quest was failed or abandoned
    Failed { failed_at: DateTime<Utc> },
}

impl Default for QuestState {
    fn default() -> Self {
        Self::Available
    }
}

/// Types of quest objectives that can be tracked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveType {
    /// Kill a specific number of enemies
    KillEnemy { enemy_type: String, count: u32 },
    /// Collect a specific number of items
    CollectItem { item_id: String, count: u32 },
    /// Visit a specific location
    VisitLocation { room_id: String },
    /// Talk to a specific NPC
    TalkToNpc { npc_id: String },
    /// Use an item on a target
    UseItem { item_id: String, target: String },
}

/// Individual quest objective with progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuestObjective {
    pub description: String,
    pub objective_type: ObjectiveType,
    pub progress: u32,
    pub required: u32,
}

impl QuestObjective {
    pub fn new(description: &str, objective_type: ObjectiveType, required: u32) -> Self {
        Self {
            description: description.to_string(),
            objective_type,
            progress: 0,
            required,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= self.required
    }

    pub fn increment_progress(&mut self, amount: u32) {
        self.progress = self.progress.saturating_add(amount).min(self.required);
    }
}

/// Quest rewards that can be granted upon completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestRewards {
    #[serde(default)]
    pub currency: Option<CurrencyAmount>,
    #[serde(default)]
    pub experience: u32,
    #[serde(default)]
    pub items: Vec<String>, // Item IDs to grant
    #[serde(default)]
    pub reputation: HashMap<String, i32>, // Faction -> reputation change
}

impl Default for QuestRewards {
    fn default() -> Self {
        Self {
            currency: None,
            experience: 0,
            items: Vec::new(),
            reputation: HashMap::new(),
        }
    }
}

/// Quest record defining a quest template and player-specific progress
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub quest_giver_npc: String, // NPC ID who gives the quest
    pub difficulty: u8,           // 1-5 difficulty rating
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestRewards,
    #[serde(default)]
    pub prerequisites: Vec<String>, // Quest IDs that must be complete
    pub created_at: DateTime<Utc>,
    pub schema_version: u8,
}

impl QuestRecord {
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        quest_giver_npc: &str,
        difficulty: u8,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            quest_giver_npc: quest_giver_npc.to_string(),
            difficulty: difficulty.min(5),
            objectives: Vec::new(),
            rewards: QuestRewards::default(),
            prerequisites: Vec::new(),
            created_at: Utc::now(),
            schema_version: 1,
        }
    }

    pub fn with_objective(mut self, objective: QuestObjective) -> Self {
        self.objectives.push(objective);
        self
    }

    pub fn with_reward_currency(mut self, currency: CurrencyAmount) -> Self {
        self.rewards.currency = Some(currency);
        self
    }

    pub fn with_reward_experience(mut self, experience: u32) -> Self {
        self.rewards.experience = experience;
        self
    }

    pub fn with_reward_item(mut self, item_id: &str) -> Self {
        self.rewards.items.push(item_id.to_string());
        self
    }

    pub fn with_prerequisite(mut self, quest_id: &str) -> Self {
        self.prerequisites.push(quest_id.to_string());
        self
    }

    pub fn all_objectives_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|obj| obj.is_complete())
    }
}

/// Player's quest progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerQuest {
    pub quest_id: String,
    pub state: QuestState,
    /// Snapshot of objectives at accept time (allows tracking progress)
    pub objectives: Vec<QuestObjective>,
}

impl PlayerQuest {
    pub fn new(quest_id: &str, objectives: Vec<QuestObjective>) -> Self {
        Self {
            quest_id: quest_id.to_string(),
            state: QuestState::Active {
                started_at: Utc::now(),
            },
            objectives,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, QuestState::Active { .. })
    }

    pub fn is_complete(&self) -> bool {
        matches!(self.state, QuestState::Completed { .. })
    }

    pub fn all_objectives_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|obj| obj.is_complete())
    }

    pub fn mark_complete(&mut self) {
        self.state = QuestState::Completed {
            completed_at: Utc::now(),
        };
    }

    pub fn mark_failed(&mut self) {
        self.state = QuestState::Failed {
            failed_at: Utc::now(),
        };
    }
}

/// Achievement system data structures for Phase 6 Week 3

/// Achievement category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AchievementCategory {
    Combat,
    Exploration,
    Social,
    Economic,
    Crafting,
    Quest,
    Special,
}

/// Achievement trigger conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AchievementTrigger {
    /// Kill N enemies
    KillCount { required: u32 },
    /// Visit N unique rooms
    RoomVisits { required: u32 },
    /// Make N friends
    FriendCount { required: u32 },
    /// Complete N quests
    QuestCompletion { required: u32 },
    /// Earn N currency
    CurrencyEarned { amount: i64 },
    /// Craft N items
    CraftCount { required: u32 },
    /// Trade N times
    TradeCount { required: u32 },
    /// Send N messages
    MessagesSent { required: u32 },
    /// Reach specific location
    VisitLocation { room_id: String },
    /// Complete specific quest
    CompleteQuest { quest_id: String },
}

/// Achievement record template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub trigger: AchievementTrigger,
    /// Optional title awarded on completion
    pub title: Option<String>,
    /// Hidden achievements don't show until earned
    pub hidden: bool,
    /// Schema version for migrations
    pub schema_version: u8,
    pub created_at: DateTime<Utc>,
}

impl AchievementRecord {
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        category: AchievementCategory,
        trigger: AchievementTrigger,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            category,
            trigger,
            title: None,
            hidden: false,
            schema_version: 1,
            created_at: Utc::now(),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn as_hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

/// Player's achievement progress
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerAchievement {
    pub achievement_id: String,
    /// Current progress toward trigger condition
    pub progress: u32,
    /// Whether achievement has been earned
    pub earned: bool,
    /// When achievement was earned (if earned)
    pub earned_at: Option<DateTime<Utc>>,
}

impl PlayerAchievement {
    pub fn new(achievement_id: &str) -> Self {
        Self {
            achievement_id: achievement_id.to_string(),
            progress: 0,
            earned: false,
            earned_at: None,
        }
    }

    pub fn increment(&mut self, amount: u32) {
        if !self.earned {
            self.progress += amount;
        }
    }

    pub fn mark_earned(&mut self) {
        self.earned = true;
        self.earned_at = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerRecord {
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_room: String,
    pub state: PlayerState,
    pub stats: PlayerStats,
    /// Inventory as item stacks (new system)
    #[serde(default)]
    pub inventory_stacks: Vec<ItemStack>,
    /// Legacy inventory (deprecated, for backward compatibility)
    #[serde(default)]
    pub inventory: Vec<String>,
    /// Current currency on hand (replaces legacy credits field)
    #[serde(default)]
    pub currency: CurrencyAmount,
    /// Currency in bank vault (safe storage)
    #[serde(default)]
    pub banked_currency: CurrencyAmount,
    /// Legacy credits field for backward compatibility (deprecated)
    #[serde(default)]
    pub credits: u32,
    /// Tutorial progression state
    #[serde(default)]
    pub tutorial_state: TutorialState,
    /// Active and completed quests
    #[serde(default)]
    pub quests: Vec<PlayerQuest>,
    /// Achievement progress and earned achievements
    #[serde(default)]
    pub achievements: Vec<PlayerAchievement>,
    /// Currently equipped title (optional)
    #[serde(default)]
    pub equipped_title: Option<String>,
    /// Active companions (companion IDs)
    #[serde(default)]
    pub companions: Vec<String>,
    /// Currently mounted companion (if any)
    #[serde(default)]
    pub mounted_companion: Option<String>,
    /// Primary housing instance ID (for HOME command)
    #[serde(default)]
    pub primary_housing_id: Option<String>,
    /// Last teleport timestamp (for cooldown enforcement)
    #[serde(default)]
    pub last_teleport: Option<DateTime<Utc>>,
    /// Combat state (blocks teleportation when true)
    #[serde(default)]
    pub in_combat: bool,
    pub schema_version: u8,
}

impl PlayerRecord {
    pub fn new(username: &str, display_name: &str, starting_room: &str) -> Self {
        let now = Utc::now();
        Self {
            username: username.to_string(),
            display_name: display_name.to_string(),
            created_at: now,
            updated_at: now,
            current_room: starting_room.to_string(),
            state: PlayerState::Exploring,
            stats: PlayerStats::default(),
            inventory_stacks: Vec::new(),
            inventory: Vec::new(),
            currency: CurrencyAmount::default(),
            banked_currency: CurrencyAmount::default(),
            credits: 0,
            tutorial_state: TutorialState::default(),
            quests: Vec::new(),
            achievements: Vec::new(),
            equipped_title: None,
            companions: Vec::new(),
            mounted_companion: None,
            primary_housing_id: None,
            last_teleport: None,
            in_combat: false,
            schema_version: PLAYER_SCHEMA_VERSION,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Inventory System
// ============================================================================

/// Inventory configuration limits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryConfig {
    /// Maximum number of unique item stacks a player can carry
    pub max_stacks: u32,
    /// Maximum total weight a player can carry (abstract units)
    pub max_weight: u32,
    /// Whether to allow item stacking
    pub allow_stacking: bool,
}

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            max_stacks: 100,
            max_weight: 1000,
            allow_stacking: true,
        }
    }
}

/// Represents a stack of identical items in inventory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItemStack {
    /// Object ID of the item
    pub object_id: String,
    /// Number of items in this stack (1 for non-stackable)
    pub quantity: u32,
    /// When this stack was first added to inventory
    pub added_at: DateTime<Utc>,
}

impl ItemStack {
    pub fn new(object_id: String, quantity: u32) -> Self {
        Self {
            object_id,
            quantity,
            added_at: Utc::now(),
        }
    }

    /// Get total weight of this stack
    pub fn total_weight(&self, item_weight: u8) -> u32 {
        self.quantity * item_weight as u32
    }
}

/// Inventory management result
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryResult {
    /// Item added successfully
    Added { quantity: u32, stacked: bool },
    /// Item removed successfully
    Removed { quantity: u32 },
    /// Operation failed
    Failed { reason: String },
}

pub const BULLETIN_SCHEMA_VERSION: u8 = 1;

/// A single bulletin board message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulletinMessage {
    pub id: u64,
    pub author: String,
    pub subject: String,
    pub body: String,
    pub posted_at: DateTime<Utc>,
    pub board_id: String,
    pub schema_version: u8,
}

impl BulletinMessage {
    pub fn new(author: &str, subject: &str, body: &str, board_id: &str) -> Self {
        Self {
            id: 0, // Will be set by storage layer
            author: author.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            posted_at: Utc::now(),
            board_id: board_id.to_string(),
            schema_version: BULLETIN_SCHEMA_VERSION,
        }
    }
}

/// Configuration and metadata for a bulletin board
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulletinBoard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub room_id: String,
    pub max_messages: u32,
    pub max_message_length: u32,
    pub allow_anonymous: bool,
    pub created_at: DateTime<Utc>,
    pub schema_version: u8,
}

impl BulletinBoard {
    pub fn new(id: &str, name: &str, description: &str, room_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            room_id: room_id.to_string(),
            max_messages: 100,
            max_message_length: 500,
            allow_anonymous: false,
            created_at: Utc::now(),
            schema_version: BULLETIN_SCHEMA_VERSION,
        }
    }
}

pub const MAIL_SCHEMA_VERSION: u8 = 1;

/// Status of a mail message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MailStatus {
    Unread,
    Read,
    Replied,
    Forwarded,
}

/// A mail message between players
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MailMessage {
    pub id: u64,
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub status: MailStatus,
    pub reply_to: Option<u64>,
    pub schema_version: u8,
}

impl MailMessage {
    pub fn new(sender: &str, recipient: &str, subject: &str, body: &str) -> Self {
        Self {
            id: 0, // Will be set by storage layer
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            sent_at: Utc::now(),
            read_at: None,
            status: MailStatus::Unread,
            reply_to: None,
            schema_version: MAIL_SCHEMA_VERSION,
        }
    }

    pub fn reply(sender: &str, subject: &str, body: &str, original: &MailMessage) -> Self {
        let reply_subject = if original.subject.starts_with("Re: ") {
            original.subject.clone()
        } else {
            format!("Re: {}", original.subject)
        };

        Self {
            id: 0,
            sender: sender.to_string(),
            recipient: original.sender.clone(),
            subject: if subject.is_empty() { reply_subject } else { subject.to_string() },
            body: body.to_string(),
            sent_at: Utc::now(),
            read_at: None,
            status: MailStatus::Unread,
            reply_to: Some(original.id),
            schema_version: MAIL_SCHEMA_VERSION,
        }
    }

    pub fn mark_read(&mut self) {
        if self.status == MailStatus::Unread {
            self.status = MailStatus::Read;
            self.read_at = Some(Utc::now());
        }
    }
}

/// Mail system configuration and quotas
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MailConfig {
    pub max_messages_per_player: u32,
    pub max_subject_length: u32,
    pub max_body_length: u32,
    pub auto_cleanup_days: u32,
    pub allow_anonymous: bool,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            max_messages_per_player: 50,
            max_subject_length: 50,
            max_body_length: 1000,
            auto_cleanup_days: 30,
            allow_anonymous: false,
        }
    }
}

// ============================================================================
// Currency System - Dual Mode Support (Decimal vs Multi-Tier)
// ============================================================================

/// Currency system configuration - choose between decimal or multi-tier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CurrencySystem {
    /// Modern/sci-fi style decimal currency (e.g., credits, dollars)
    Decimal(DecimalCurrency),
    /// Fantasy/medieval style multi-tier currency (e.g., gold/silver/copper)
    MultiTier(MultiTierCurrency),
}

/// Decimal currency configuration (single unit with decimal subdivisions)
/// Internally we store as integer minor units (e.g., cents).
/// So $12.34 is stored as 1234 minor units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecimalCurrency {
    /// Display name for the currency (e.g., "credit", "dollar", "gil")
    pub name: String,
    /// Plural form (e.g., "credits", "dollars", "gil")
    pub name_plural: String,
    /// Symbol prefix (e.g., "$", "₹", "¤")
    pub symbol: String,
    /// Number of decimal places (2 for cents, 0 for whole units only)
    pub decimals: u8,
}

impl Default for DecimalCurrency {
    fn default() -> Self {
        Self {
            name: "credit".to_string(),
            name_plural: "credits".to_string(),
            symbol: "¤".to_string(),
            decimals: 2,
        }
    }
}

/// Multi-tier currency configuration (fantasy-style tiers)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiTierCurrency {
    /// Currency tiers from lowest to highest
    /// Each tier has: name, plural, symbol, and conversion ratio to base unit
    pub tiers: Vec<CurrencyTier>,
    /// Name of the base unit (typically the lowest tier, e.g., "copper")
    pub base_unit: String,
}

/// A single tier in a multi-tier currency system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrencyTier {
    /// Singular name (e.g., "copper", "silver", "gold")
    pub name: String,
    /// Plural name (e.g., "coppers", "silvers", "golds")
    pub name_plural: String,
    /// Display symbol (e.g., "c", "s", "g")
    pub symbol: String,
    /// How many base units this tier is worth
    /// Base tier should have ratio=1, silver might be 10, gold might be 100
    pub ratio_to_base: u64,
}

impl Default for MultiTierCurrency {
    fn default() -> Self {
        Self {
            tiers: vec![
                CurrencyTier {
                    name: "copper".to_string(),
                    name_plural: "coppers".to_string(),
                    symbol: "c".to_string(),
                    ratio_to_base: 1,
                },
                CurrencyTier {
                    name: "silver".to_string(),
                    name_plural: "silvers".to_string(),
                    symbol: "s".to_string(),
                    ratio_to_base: 10,
                },
                CurrencyTier {
                    name: "gold".to_string(),
                    name_plural: "golds".to_string(),
                    symbol: "g".to_string(),
                    ratio_to_base: 100,
                },
            ],
            base_unit: "copper".to_string(),
        }
    }
}

/// A currency amount that can represent either decimal or multi-tier currency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CurrencyAmount {
    /// Decimal amount stored as integer minor units (e.g., cents)
    Decimal { minor_units: i64 },
    /// Multi-tier amount stored as base units (e.g., coppers)
    MultiTier { base_units: i64 },
}

impl CurrencyAmount {
    /// Create a decimal currency amount
    pub fn decimal(minor_units: i64) -> Self {
        Self::Decimal { minor_units }
    }

    /// Create a multi-tier currency amount
    pub fn multi_tier(base_units: i64) -> Self {
        Self::MultiTier { base_units }
    }

    /// Get the base value (minor units for decimal, base units for multi-tier)
    pub fn base_value(&self) -> i64 {
        match self {
            Self::Decimal { minor_units } => *minor_units,
            Self::MultiTier { base_units } => *base_units,
        }
    }

    /// Check if this amount is zero or negative
    pub fn is_zero_or_negative(&self) -> bool {
        self.base_value() <= 0
    }

    /// Check if this amount is positive
    pub fn is_positive(&self) -> bool {
        self.base_value() > 0
    }

    /// Add another amount (must be same type)
    pub fn add(&self, other: &CurrencyAmount) -> Result<CurrencyAmount, String> {
        match (self, other) {
            (Self::Decimal { minor_units: a }, Self::Decimal { minor_units: b }) => {
                Ok(Self::Decimal {
                    minor_units: a.saturating_add(*b),
                })
            }
            (Self::MultiTier { base_units: a }, Self::MultiTier { base_units: b }) => {
                Ok(Self::MultiTier {
                    base_units: a.saturating_add(*b),
                })
            }
            _ => Err("Cannot add different currency types".to_string()),
        }
    }

    /// Subtract another amount (must be same type)
    pub fn subtract(&self, other: &CurrencyAmount) -> Result<CurrencyAmount, String> {
        match (self, other) {
            (Self::Decimal { minor_units: a }, Self::Decimal { minor_units: b }) => {
                Ok(Self::Decimal {
                    minor_units: a.saturating_sub(*b),
                })
            }
            (Self::MultiTier { base_units: a }, Self::MultiTier { base_units: b }) => {
                Ok(Self::MultiTier {
                    base_units: a.saturating_sub(*b),
                })
            }
            _ => Err("Cannot subtract different currency types".to_string()),
        }
    }

    /// Check if we have enough currency to afford a cost
    pub fn can_afford(&self, cost: &CurrencyAmount) -> bool {
        match (self, cost) {
            (Self::Decimal { minor_units: have }, Self::Decimal { minor_units: need }) => {
                have >= need
            }
            (Self::MultiTier { base_units: have }, Self::MultiTier { base_units: need }) => {
                have >= need
            }
            _ => false,
        }
    }
}

impl Default for CurrencyAmount {
    fn default() -> Self {
        Self::Decimal { minor_units: 0 }
    }
}

/// Transaction record for audit logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrencyTransaction {
    /// Unique transaction ID
    pub id: String,
    /// Timestamp of transaction
    pub timestamp: DateTime<Utc>,
    /// Source player (None for system transactions)
    pub from: Option<String>,
    /// Destination player (None for system transactions)
    pub to: Option<String>,
    /// Amount transferred
    pub amount: CurrencyAmount,
    /// Reason for transaction
    pub reason: TransactionReason,
    /// Whether this transaction was rolled back
    pub rolled_back: bool,
}

/// Reason for a currency transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionReason {
    /// Initial grant from system
    SystemGrant,
    /// Purchase from shop/vendor
    Purchase,
    /// Sale to shop/vendor
    Sale,
    /// Player-to-player trade
    Trade,
    /// Admin grant
    AdminGrant,
    /// Admin deduction
    AdminDeduct,
    /// Quest reward
    QuestReward,
    /// Room rent payment
    RoomRent,
    /// Bank deposit
    BankDeposit,
    /// Bank withdrawal
    BankWithdrawal,
    /// Combat/loot reward
    CombatLoot,
    /// Transaction rollback
    Rollback,
    /// Other reason
    Other { description: String },
}

// ============================================================================
// Player-to-Player Trading
// ============================================================================

/// Active P2P trade session between two players
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeSession {
    /// Trade session ID
    pub id: String,
    /// First player (initiator)
    pub player1: String,
    /// Second player (recipient)
    pub player2: String,
    /// Items offered by player1
    pub player1_items: Vec<String>,
    /// Currency offered by player1
    pub player1_currency: CurrencyAmount,
    /// Items offered by player2
    pub player2_items: Vec<String>,
    /// Currency offered by player2
    pub player2_currency: CurrencyAmount,
    /// Player1 has accepted
    pub player1_accepted: bool,
    /// Player2 has accepted
    pub player2_accepted: bool,
    /// Trade creation timestamp
    pub created_at: DateTime<Utc>,
    /// Trade expiration timestamp (5 minutes default)
    pub expires_at: DateTime<Utc>,
    /// Trade completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

impl TradeSession {
    /// Create new trade session between two players
    pub fn new(player1: &str, player2: &str) -> Self {
        let now = Utc::now();
        let id = format!("trade_{}_{}", player1, player2);
        Self {
            id,
            player1: player1.to_string(),
            player2: player2.to_string(),
            player1_items: Vec::new(),
            player1_currency: CurrencyAmount::default(),
            player2_items: Vec::new(),
            player2_currency: CurrencyAmount::default(),
            player1_accepted: false,
            player2_accepted: false,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
            completed_at: None,
        }
    }

    /// Check if trade has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if both players have accepted
    pub fn is_ready(&self) -> bool {
        self.player1_accepted && self.player2_accepted
    }

    /// Add currency offer from a player
    pub fn add_currency_offer(&mut self, player: &str, amount: CurrencyAmount) {
        if player == self.player1 {
            self.player1_currency = amount;
            self.player1_accepted = false;
        } else if player == self.player2 {
            self.player2_currency = amount;
            self.player2_accepted = false;
        }
    }

    /// Add item offer from a player
    pub fn add_item_offer(&mut self, player: &str, item_id: String) {
        if player == self.player1 {
            self.player1_items.push(item_id);
            self.player1_accepted = false;
        } else if player == self.player2 {
            self.player2_items.push(item_id);
            self.player2_accepted = false;
        }
    }

    /// Mark player as accepted
    pub fn accept(&mut self, player: &str) {
        if player == self.player1 {
            self.player1_accepted = true;
        } else if player == self.player2 {
            self.player2_accepted = true;
        }
    }

    /// Get summary of offered items and currency
    pub fn get_summary(&self) -> String {
        let p1_items_str = if self.player1_items.is_empty() {
            "nothing".to_string()
        } else {
            format!("{} item(s)", self.player1_items.len())
        };
        let p1_currency_str = format!("{:?}", self.player1_currency);

        let p2_items_str = if self.player2_items.is_empty() {
            "nothing".to_string()
        } else {
            format!("{} item(s)", self.player2_items.len())
        };
        let p2_currency_str = format!("{:?}", self.player2_currency);

        format!(
            "{}: {} + {}\n{}: {} + {}",
            self.player1, p1_items_str, p1_currency_str,
            self.player2, p2_items_str, p2_currency_str
        )
    }
}

/// World configuration for customizable strings and settings
/// This allows world creators to modify system messages without editing source code.
/// All user-facing text is configurable to support internationalization and custom theming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Version of the config schema
    pub version: u8,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Updated by which admin
    pub updated_by: String,
    
    // === BRANDING ===
    /// Welcome message shown when tutorial auto-starts
    pub welcome_message: String,
    /// MOTD shown on login
    pub motd: String,
    /// Name of the world/game
    pub world_name: String,
    /// Short description of the world
    pub world_description: String,
    
    // === HELP SYSTEM TEMPLATES ===
    /// Main help menu text
    pub help_main: String,
    /// Commands reference text
    pub help_commands: String,
    /// Movement guide text
    pub help_movement: String,
    /// Social commands help
    pub help_social: String,
    /// Bulletin board help
    pub help_bulletin: String,
    /// Companion system help
    pub help_companion: String,
    /// Mail system help
    pub help_mail: String,
    
    // === ERROR MESSAGE TEMPLATES ===
    /// Error when trying to move in blocked direction: "You can't go {direction} from here."
    pub err_no_exit: String,
    /// Error when whispering to self: "You can't whisper to yourself!"
    pub err_whisper_self: String,
    /// Error when no shops in room: "There are no shops here."
    pub err_no_shops: String,
    /// Error when item not in inventory: "You don't have any '{item}'."
    pub err_item_not_found: String,
    /// Error when trying to trade with self: "You can't trade with yourself!"
    pub err_trade_self: String,
    /// Prompt when SAY used with no text: "Say what?"
    pub err_say_what: String,
    /// Prompt when EMOTE used with no text: "Emote what?"
    pub err_emote_what: String,
    /// Error when insufficient funds: "Insufficient funds."
    pub err_insufficient_funds: String,
    
    // === SUCCESS MESSAGE TEMPLATES ===
    /// Success depositing to bank: "Deposited {amount} to bank.\nUse BALANCE to check your account."
    pub msg_deposit_success: String,
    /// Success withdrawing from bank: "Withdrew {amount} from bank.\nUse BALANCE to check your account."
    pub msg_withdraw_success: String,
    /// Success buying item: "You bought {quantity} x {item} for {price}."
    pub msg_buy_success: String,
    /// Success selling item: "You sold {quantity} x {item} for {price}."
    pub msg_sell_success: String,
    /// Trade initiated: "Trade initiated with {player}!\nUse OFFER to add items/currency.\nType ACCEPT when ready."
    pub msg_trade_initiated: String,
    
    // === VALIDATION & INPUT ERROR MESSAGES ===
    pub err_whisper_what: String,
    pub err_whisper_whom: String,
    pub err_pose_what: String,
    pub err_ooc_what: String,
    pub err_amount_positive: String,
    pub err_invalid_amount_format: String,
    pub err_transfer_self: String,
    
    // === EMPTY STATE MESSAGES ===
    pub msg_empty_inventory: String,
    pub msg_no_item_quantity: String,
    pub msg_no_shops_available: String,
    pub msg_no_shops_sell_to: String,
    pub msg_no_companions: String,
    pub msg_no_companions_tame_hint: String,
    pub msg_no_companions_follow: String,
    pub msg_no_active_quests: String,
    pub msg_no_achievements: String,
    pub msg_no_achievements_earned: String,
    pub msg_no_titles_unlocked: String,
    pub msg_no_title_equipped: String,
    pub msg_no_active_trade: String,
    pub msg_no_active_trade_hint: String,
    pub msg_no_trade_history: String,
    pub msg_no_players_found: String,
    
    // === SHOP ERROR MESSAGES ===
    pub err_shop_no_sell: String,
    pub err_shop_doesnt_sell: String,
    pub err_shop_insufficient_funds: String,
    pub err_shop_no_buy: String,
    pub err_shop_wont_buy_price: String,
    pub err_item_not_owned: String,
    pub err_only_have_quantity: String,
    
    // === TRADING SYSTEM MESSAGES ===
    pub err_trade_already_active: String,
    pub err_trade_partner_busy: String,
    pub err_trade_player_not_here: String,
    pub err_trade_insufficient_amount: String,
    pub msg_trade_accepted_waiting: String,
    
    // === MOVEMENT & NAVIGATION MESSAGES ===
    pub err_movement_restricted: String,
    pub err_player_not_here: String,
    
    // === QUEST SYSTEM MESSAGES ===
    pub err_quest_cannot_accept: String,
    pub err_quest_not_found: String,
    pub msg_quest_abandoned: String,
    
    // === ACHIEVEMENT SYSTEM MESSAGES ===
    pub err_achievement_unknown_category: String,
    pub msg_no_achievements_category: String,
    
    // === TITLE SYSTEM MESSAGES ===
    pub err_title_not_unlocked: String,
    pub msg_title_equipped: String,
    pub msg_title_equipped_display: String,
    pub err_title_usage: String,
    
    // === COMPANION SYSTEM MESSAGES ===
    pub msg_companion_tamed: String,
    pub err_companion_owned: String,
    pub err_companion_not_found: String,
    pub msg_companion_released: String,
    
    // === BULLETIN BOARD MESSAGES ===
    pub err_board_location_required: String,
    pub err_board_post_location: String,
    pub err_board_read_location: String,
    
    // === NPC & TUTORIAL MESSAGES ===
    pub err_no_npc_here: String,
    pub msg_tutorial_completed: String,
    pub msg_tutorial_not_started: String,
    
    // === HOUSING SYSTEM MESSAGES ===
    pub err_housing_not_at_office: String,
    pub err_housing_no_templates: String,
    pub err_housing_insufficient_funds: String,
    pub err_housing_already_owns: String,
    pub err_housing_template_not_found: String,
    pub msg_housing_rented: String,
    pub msg_housing_list_header: String,
    
    // === HOME/TELEPORT SYSTEM MESSAGES ===
    pub err_teleport_in_combat: String,
    pub err_teleport_restricted: String,
    pub err_teleport_cooldown: String,
    pub err_no_housing: String,
    pub err_teleport_no_access: String,
    pub msg_teleport_success: String,
    pub home_cooldown_seconds: u64,
    pub msg_home_list_header: String,
    pub msg_home_list_empty: String,
    pub msg_home_list_footer_travel: String,
    pub msg_home_list_footer_set: String,
    pub err_home_not_found: String,
    pub msg_home_set_success: String,
    
    // === GUEST/INVITE SYSTEM MESSAGES ===
    pub err_invite_no_housing: String,
    pub err_invite_not_in_housing: String,
    pub err_invite_player_not_found: String,
    pub err_invite_already_guest: String,
    pub msg_invite_success: String,
    pub err_uninvite_not_guest: String,
    pub msg_uninvite_success: String,
    
    // === DESCRIBE/CUSTOMIZATION SYSTEM MESSAGES ===
    pub err_describe_not_in_housing: String,
    pub err_describe_no_permission: String,
    pub err_describe_too_long: String,
    pub msg_describe_success: String,
    pub msg_describe_current: String,
    
    // === TECHNICAL/SYSTEM MESSAGES ===
    pub err_player_load_failed: String,
    pub err_shop_save_failed: String,
    pub err_player_save_failed: String,
    pub err_payment_failed: String,
    pub err_purchase_failed: String,
    pub err_sale_failed: String,
    pub err_tutorial_error: String,
    pub err_reward_error: String,
    pub err_quest_failed: String,
    pub err_shop_find_failed: String,
    pub err_player_list_failed: String,
    pub err_movement_failed: String,
    pub err_movement_save_failed: String,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            version: 1,
            updated_at: Utc::now(),
            updated_by: "system".to_string(),
            
            // Branding
            welcome_message: "=== WELCOME TO OLD TOWNE MESH ===\n".to_string() +
                "You find yourself at the Script Gazebo...\n\n" +
                "This tutorial will guide you through\n" +
                "the basics of exploring our world.\n\n" +
                "Type LOOK to see your surroundings.\n" +
                "Type HELP for more commands.\n" +
                "Type TUTORIAL to check your progress.",
            motd: "Welcome to Old Towne Mesh!\nType HELP for commands.".to_string(),
            world_name: "Old Towne Mesh".to_string(),
            world_description: "A mesh-networked MUD adventure".to_string(),
            
            // Help system templates
            help_main: "=TINYMUSH HELP=\n".to_string() +
                "Move: N/S/E/W/U/D + diagonals\n" +
                "Look: L | I (inv) | WHO | SCORE\n" +
                "Talk: SAY/EMOTE\n" +
                "Board: BOARD/POST/READ\n" +
                "Mail: MAIL/SEND\n" +
                "More: HELP <topic>\n" +
                "Topics: COMMANDS MOVEMENT SOCIAL BOARD MAIL COMPANION",
            help_commands: "=COMMANDS=\n".to_string() +
                "L - look | I - inventory\n" +
                "WHO - players | SCORE - stats\n" +
                "SAY/EMOTE - talk\n" +
                "BOARD/POST/READ - bulletin\n" +
                "MAIL/SEND/RMAIL - messages\n" +
                "SAVE | QUIT",
            help_movement: "=MOVEMENT=\n".to_string() +
                "N/S/E/W - cardinal\n" +
                "U/D - up/down\n" +
                "NE/NW/SE/SW - diagonals\n" +
                "L - look around",
            help_social: "=SOCIAL=\n".to_string() +
                "SAY <txt> - speak aloud\n" +
                "WHISPER <plr> <txt> - private\n" +
                "EMOTE/: <act> - action\n" +
                "POSE/; <pose> - describe\n" +
                "OOC <txt> - out of char\n" +
                "WHO - list players",
            help_bulletin: "=BULLETIN BOARD=\n".to_string() +
                "Town Stump message board\n" +
                "BOARD - view messages\n" +
                "POST <subj> <msg> - post\n" +
                "READ <id> - read\n" +
                "Use at Town Square\n" +
                "Max: 50 char subj, 300 msg",
            help_companion: "=COMPANIONS=\n".to_string() +
                "COMP [LIST] - your pets\n" +
                "COMP TAME <name> - claim\n" +
                "COMP <name> - status\n" +
                "COMP RELEASE <name> - free\n" +
                "COMP STAY/COME - control\n" +
                "COMP INV - storage\n" +
                "FEED/PET <name> - care\n" +
                "MOUNT/DISMOUNT - riding\n" +
                "TRAIN <name> <skill> - teach",
            help_mail: "=MAIL=\n".to_string() +
                "MAIL [folder] - list messages\n" +
                "SEND <plr> <subj> <msg> - send\n" +
                "RMAIL <id> - read message\n" +
                "DMAIL <id> - delete message\n" +
                "Folders: inbox, sent, trash",
            
            // Error messages
            err_no_exit: "You can't go {} from here.".to_string(),
            err_whisper_self: "You can't whisper to yourself!".to_string(),
            err_no_shops: "There are no shops here.".to_string(),
            err_item_not_found: "You don't have any '{}'.".to_string(),
            err_trade_self: "You can't trade with yourself!".to_string(),
            err_say_what: "Say what?".to_string(),
            err_emote_what: "Emote what?".to_string(),
            err_insufficient_funds: "Insufficient funds.".to_string(),
            
            // Success messages
            msg_deposit_success: "Deposited {amount} to bank.\nUse BALANCE to check your account.".to_string(),
            msg_withdraw_success: "Withdrew {amount} from bank.\nUse BALANCE to check your account.".to_string(),
            msg_buy_success: "You bought {quantity} x {item} for {price}.".to_string(),
            msg_sell_success: "You sold {quantity} x {item} for {price}.".to_string(),
            msg_trade_initiated: "Trade initiated with {target}!\nUse OFFER to add items/currency.\nType ACCEPT when ready.".to_string(),
            
            // Validation & input errors
            err_whisper_what: "Whisper what?".to_string(),
            err_whisper_whom: "Whisper to whom?".to_string(),
            err_pose_what: "Strike what pose?".to_string(),
            err_ooc_what: "Say what out of character?".to_string(),
            err_amount_positive: "Amount must be positive.".to_string(),
            err_invalid_amount_format: "Invalid amount format.".to_string(),
            err_transfer_self: "You can't transfer to yourself!".to_string(),
            
            // Empty state messages
            msg_empty_inventory: "You are carrying nothing.".to_string(),
            msg_no_item_quantity: "You only have {quantity} x {item}.".to_string(),
            msg_no_shops_available: "No shops available.".to_string(),
            msg_no_shops_sell_to: "There are no shops here to sell to.".to_string(),
            msg_no_companions: "You don't have any companions.".to_string(),
            msg_no_companions_tame_hint: "You don't have any companions.\nTAME a wild companion to add them to your party!".to_string(),
            msg_no_companions_follow: "No companions with auto-follow are here.".to_string(),
            msg_no_active_quests: "You have no active quests.\nUse QUEST LIST to see available quests.".to_string(),
            msg_no_achievements: "No achievements available.".to_string(),
            msg_no_achievements_earned: "You haven't earned any achievements yet.\nKeep exploring and trying new things!".to_string(),
            msg_no_titles_unlocked: "You haven't unlocked any titles yet.\nEarn achievements to unlock titles!".to_string(),
            msg_no_title_equipped: "You don't have any title equipped.".to_string(),
            msg_no_active_trade: "You have no active trade.".to_string(),
            msg_no_active_trade_hint: "You have no active trade.\nUse TRADE <player> to start one.".to_string(),
            msg_no_trade_history: "No trade history.".to_string(),
            msg_no_players_found: "No players found.".to_string(),
            
            // Shop error messages
            err_shop_no_sell: "No shop here sells '{item}'.".to_string(),
            err_shop_doesnt_sell: "Shop doesn't sell '{item}'.".to_string(),
            err_shop_insufficient_funds: "You don't have enough! Need: {amount}".to_string(),
            err_shop_no_buy: "No shop here buys '{item}'.".to_string(),
            err_shop_wont_buy_price: "Shop doesn't want to buy {item} for more than {price}.".to_string(),
            err_item_not_owned: "You don't have any '{item}'.".to_string(),
            err_only_have_quantity: "You only have {quantity} x {item}.".to_string(),
            
            // Trading system messages
            err_trade_already_active: "You're already trading with {player}!\nType REJECT to cancel.".to_string(),
            err_trade_partner_busy: "{player} is already in a trade.".to_string(),
            err_trade_player_not_here: "{player} is not here!".to_string(),
            err_trade_insufficient_amount: "You don't have that much!".to_string(),
            msg_trade_accepted_waiting: "You accepted the trade.\nWaiting for other player...".to_string(),
            
            // Movement & navigation messages
            err_movement_restricted: "You can't go {direction} right now. The area might be full or restricted.".to_string(),
            err_player_not_here: "Player '{player}' not found in this room.".to_string(),
            
            // Quest system messages
            err_quest_cannot_accept: "Cannot accept that quest (already accepted/completed, or prerequisites not met).".to_string(),
            err_quest_not_found: "Quest '{quest}' not found in your active quests.".to_string(),
            msg_quest_abandoned: "You have abandoned the quest: {quest}".to_string(),
            
            // Achievement system messages
            err_achievement_unknown_category: "Unknown category: {category}\nAvailable: COMBAT, EXPLORATION, SOCIAL, ECONOMIC, CRAFTING, QUEST, SPECIAL".to_string(),
            msg_no_achievements_category: "No achievements found in category: {category}".to_string(),
            
            // Title system messages
            err_title_not_unlocked: "You haven't unlocked the title: {title}".to_string(),
            msg_title_equipped: "Title equipped: {title}".to_string(),
            msg_title_equipped_display: "Title equipped: {title}\nYou are now known as {display}".to_string(),
            err_title_usage: "Usage: TITLE [LIST|EQUIP <name>|UNEQUIP]".to_string(),
            
            // Companion system messages
            msg_companion_tamed: "You've tamed {name}!\nLoyalty: {loyalty}/100".to_string(),
            err_companion_owned: "{name} already has an owner.".to_string(),
            err_companion_not_found: "There's no companion named '{name}' here.".to_string(),
            msg_companion_released: "You've released {name} back to the wild.".to_string(),
            
            // Bulletin board messages
            err_board_location_required: "You must be at the Town Square to access the Town Stump bulletin board.\nHead to the town square and try again.".to_string(),
            err_board_post_location: "You must be at the Town Square to post to the bulletin board.".to_string(),
            err_board_read_location: "You must be at the Town Square to read bulletin board messages.".to_string(),
            
            // NPC & tutorial messages
            err_no_npc_here: "There's nobody here to talk to.".to_string(),
            msg_tutorial_completed: "Mayor Thompson: 'You've already completed the tutorial. Welcome back!'".to_string(),
            msg_tutorial_not_started: "Mayor Thompson: 'Come back when you're ready for the tutorial.'".to_string(),
            
            // Housing system messages
            err_housing_not_at_office: "You need to visit a housing office to inquire about available housing.\nLook for locations with rental services or property management.".to_string(),
            err_housing_no_templates: "No housing is available at this location right now.".to_string(),
            err_housing_insufficient_funds: "You can't afford this housing. It costs {amount} credits (you have {player} credits).".to_string(),
            err_housing_already_owns: "You already own housing! Type HOME to visit it, or HOUSING INFO to see details.".to_string(),
            err_housing_template_not_found: "Housing template '{name}' not found. Type HOUSING LIST to see available options.".to_string(),
            msg_housing_rented: "Congratulations! You've acquired {name}.\nType HOME to visit your new space!".to_string(),
            msg_housing_list_header: "=== Available Housing ===\n\nType RENT <id> to acquire housing.\nType HOUSING INFO <id> for more details.".to_string(),
            
            // Home/teleport system messages
            err_teleport_in_combat: "You can't teleport while in combat!".to_string(),
            err_teleport_restricted: "You can't teleport from this location.".to_string(),
            err_teleport_cooldown: "You must wait {time} before teleporting again.".to_string(),
            err_no_housing: "You don't own any housing! Visit a housing office to rent a place.".to_string(),
            err_teleport_no_access: "You no longer have access to that location.".to_string(),
            msg_teleport_success: "You teleport to {name}...".to_string(),
            home_cooldown_seconds: 300,  // 5 minutes default
            msg_home_list_header: "=== Your Housing ===".to_string(),
            msg_home_list_empty: "You don't own any housing yet.\nVisit a housing office to rent a place!".to_string(),
            msg_home_list_footer_travel: "Use 'HOME <number>' to travel to a specific property.".to_string(),
            msg_home_list_footer_set: "Use 'HOME SET <number>' to change your primary home.".to_string(),
            err_home_not_found: "Housing '{id}' not found. Use HOME LIST to see your properties.".to_string(),
            msg_home_set_success: "Primary home set to: {name}\nUse HOME to teleport there.".to_string(),
            
            // Guest/invite system messages
            err_invite_no_housing: "You don't own any housing to invite guests to.".to_string(),
            err_invite_not_in_housing: "You must be in your housing to invite guests.".to_string(),
            err_invite_player_not_found: "Player '{name}' not found.".to_string(),
            err_invite_already_guest: "{name} is already on the guest list.".to_string(),
            msg_invite_success: "You've invited {name} to your housing.".to_string(),
            err_uninvite_not_guest: "{name} is not on your guest list.".to_string(),
            msg_uninvite_success: "You've removed {name} from your guest list.".to_string(),
            
            // Describe/customization system messages
            err_describe_not_in_housing: "You can only use DESCRIBE in housing you own or have permission to edit.".to_string(),
            err_describe_no_permission: "You don't have permission to edit this room's description.".to_string(),
            err_describe_too_long: "Description too long. Maximum {max} characters (yours: {actual}).".to_string(),
            msg_describe_success: "✓ Room description updated!".to_string(),
            msg_describe_current: "Current: \"{desc}\"\n\nYou have permission to edit.\nUsage: DESCRIBE <new description>".to_string(),
            
            // Technical/system messages
            err_player_load_failed: "Error loading player: {error}".to_string(),
            err_shop_save_failed: "Failed to save shop: {error}".to_string(),
            err_player_save_failed: "Failed to save player: {error}".to_string(),
            err_payment_failed: "Payment failed: {error}".to_string(),
            err_purchase_failed: "Purchase failed: {error}".to_string(),
            err_sale_failed: "Sale failed: {error}".to_string(),
            err_tutorial_error: "Tutorial error: {error}".to_string(),
            err_reward_error: "Reward error: {error}".to_string(),
            err_quest_failed: "Failed to abandon quest: {error}".to_string(),
            err_shop_find_failed: "Error finding shops: {error}".to_string(),
            err_player_list_failed: "Error listing players: {error}".to_string(),
            err_movement_failed: "Movement failed: {error}".to_string(),
            err_movement_save_failed: "Movement failed to save: {error}".to_string(),
        }
    }
}

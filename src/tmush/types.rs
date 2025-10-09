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

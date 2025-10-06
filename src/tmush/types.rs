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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: ObjectOwner,
    pub created_at: DateTime<Utc>,
    pub weight: u8,
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
            value: 0,
            takeable: false,
            usable: false,
            actions: HashMap::new(),
            flags: Vec::new(),
            schema_version: OBJECT_SCHEMA_VERSION,
        }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerRecord {
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_room: String,
    pub state: PlayerState,
    pub stats: PlayerStats,
    #[serde(default)]
    pub inventory: Vec<String>,
    pub credits: u32,
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
            inventory: Vec::new(),
            credits: 0,
            schema_version: PLAYER_SCHEMA_VERSION,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

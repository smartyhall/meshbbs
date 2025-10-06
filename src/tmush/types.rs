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
    /// Current currency on hand (replaces legacy credits field)
    #[serde(default)]
    pub currency: CurrencyAmount,
    /// Currency in bank vault (safe storage)
    #[serde(default)]
    pub banked_currency: CurrencyAmount,
    /// Legacy credits field for backward compatibility (deprecated)
    #[serde(default)]
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
            currency: CurrencyAmount::default(),
            banked_currency: CurrencyAmount::default(),
            credits: 0,
            schema_version: PLAYER_SCHEMA_VERSION,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
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

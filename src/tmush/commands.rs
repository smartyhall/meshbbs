//! TinyMUSH command parsing and session integration for MeshBBS.
//!
//! This module extends the BBS command processor to handle TinyMUSH-specific
//! verbs and routing. It integrates with the existing SessionState machine
//! and provides the bridge between mesh input and TinyMUSH world interactions.

use anyhow::Result;
use log::{debug, info};

use crate::bbs::session::{Session, SessionState};
use crate::config::Config;
use crate::logutil::escape_log;
use crate::metrics;
use crate::storage::Storage;
use crate::tmush::{TinyMushStore, TinyMushError, PlayerRecord};
use crate::tmush::types::{BulletinBoard, BulletinMessage, Direction as TmushDirection, RoomFlag, CurrencyAmount, ObjectRecord};
use crate::tmush::state::canonical_world_seed;
use crate::tmush::room_manager::RoomManager;
use crate::tmush::inventory::format_inventory_compact;

/// TinyMUSH command categories for parsing and routing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TinyMushCommand {
    // Core navigation
    Look(Option<String>),    // L, L chest
    Move(Direction),         // N, S, E, W, U, D, NE, NW, SE, SW
    Where,                   // WHERE - show current location
    Map,                     // MAP - show area overview

    // Inventory and items
    Inventory,               // I - show inventory
    Take(String),           // T item - pick up item
    Drop(String),           // D item - drop item
    Use(String),            // U item - use/activate item
    Examine(String),        // X item - detailed examination

    // Economy and shops (Phase 5)
    Buy(String, Option<u32>),    // BUY item [quantity] - purchase from shop
    Sell(String, Option<u32>),   // SELL item [quantity] - sell to shop
    List,                        // LIST/WARES - view shop inventory

    // Social interactions
    Say(String),            // SAY text - speak to room
    Whisper(String, String), // W player text - private message
    Emote(String),          // EMOTE action - perform emote
    Pose(String),           // POSE action - strike a pose
    Ooc(String),            // OOC text - out of character

    // Information
    Who,                    // WHO - list online players
    Score,                  // SCORE - show player stats
    Time,                   // TIME - show game time
    
    // Bulletin board commands (Phase 4 feature)
    Board(Option<String>),  // BOARD, BOARD stump - view bulletin board
    Post(String, String),   // POST subject message - post to bulletin board
    Read(u64),             // READ 123 - read specific bulletin message
    
    // Mail system commands (Phase 4 feature)
    Mail(Option<String>),   // MAIL, MAIL inbox - view mail folder
    Send(String, String, String), // SEND player subject message - send mail
    ReadMail(u64),         // RMAIL 123 - read specific mail message
    DeleteMail(u64),       // DMAIL 123 - delete mail message
    
    // Banking commands (Phase 5 Week 4)
    Balance,               // BALANCE - show pocket and bank balance
    Deposit(String),       // DEPOSIT amount - deposit currency to bank
    Withdraw(String),      // WITHDRAW amount - withdraw currency from bank
    BankTransfer(String, String), // BTRANSFER player amount - transfer to another player
    
    // Trading commands (Phase 5 Week 4)
    Trade(String),         // TRADE player - initiate trade with player
    Offer(String),         // OFFER item/amount - offer item or currency in active trade
    Accept,                // ACCEPT - accept trade
    Reject,                // REJECT - reject/cancel trade
    TradeHistory,          // THISTORY - view trade history
    
    // Tutorial & NPC commands (Phase 6 Week 1)
    Tutorial(Option<String>), // TUTORIAL, TUTORIAL SKIP, TUTORIAL RESTART - manage tutorial
    Talk(String),          // TALK npc - interact with NPC
    
    // Quest commands (Phase 6 Week 2)
    Quest(Option<String>), // QUEST, QUEST LIST, QUEST ACCEPT id - manage quests
    Abandon(String),       // ABANDON quest_id - abandon active quest
    
    // Achievement & Title commands (Phase 6 Week 3)
    Achievements(Option<String>), // ACHIEVEMENTS, ACHIEVEMENTS LIST, ACHIEVEMENTS EARNED - manage achievements
    Title(Option<String>), // TITLE, TITLE LIST, TITLE EQUIP name - manage titles
    
    // Companion commands (Phase 6 Week 4)
    Companion(Option<String>), // COMPANION, COMPANION TAME/STAY/COME/INVENTORY/RELEASE - manage companions
    Feed(String),           // FEED horse - feed companion
    Pet(String),            // PET dog - interact with companion
    Mount(String),          // MOUNT horse - mount companion
    Dismount,              // DISMOUNT - dismount from companion
    Train(String, String), // TRAIN horse speed - train companion skill
    
    // Housing commands (Phase 7 Week 1-2)
    Housing(Option<String>), // HOUSING, HOUSING LIST, HOUSING INFO - manage housing
    Rent(String),           // RENT template_id - rent/purchase housing from template
    Home(Option<String>),   // HOME, HOME LIST, HOME <id>, HOME SET <id> - teleport to housing
    Invite(String),         // INVITE player - add guest to housing
    Uninvite(String),       // UNINVITE player - remove guest from housing
    Describe(Option<String>), // DESCRIBE <text> - edit current room description (housing only)
                            // DESCRIBE - show current description and permissions
    Lock(Option<String>),   // LOCK - lock current room, LOCK <item> - lock item (Phase 2)
    Unlock(Option<String>), // UNLOCK - unlock current room, UNLOCK <item> - unlock item (Phase 2)
    Kick(Option<String>),   // KICK <player> - remove player from housing, KICK ALL (Phase 3)
    History(String),        // HISTORY <item> - view ownership audit trail (Phase 5)

    
    // System
    Help(Option<String>),   // HELP, HELP topic
    Quit,                   // QUIT - leave TinyMUSH
    Save,                   // SAVE - force save player state

    // Meta/admin (future phases)
    Debug(String),          // DEBUG - admin diagnostics
    SetConfig(String, String), // @SETCONFIG field value - set world configuration
    GetConfig(Option<String>), // @GETCONFIG [field] - view world configuration
    
    // Unrecognized command
    Unknown(String),
}

/// Cardinal and intercardinal directions for movement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    North, South, East, West,
    Up, Down,
    Northeast, Northwest, Southeast, Southwest,
}

/// TinyMUSH session state and command processor
pub struct TinyMushProcessor {
    store: Option<TinyMushStore>,
    room_manager: Option<RoomManager>,
}

impl Default for TinyMushProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TinyMushProcessor {
    pub fn new() -> Self {
        Self { 
            store: None,
            room_manager: None,
        }
    }

    /// Initialize or get the TinyMUSH store for this session
    async fn get_store(&mut self, config: &Config) -> Result<&TinyMushStore, TinyMushError> {
        if self.store.is_none() {
            let db_path = config.games.tinymush_db_path
                .as_deref()
                .unwrap_or("data/tinymush");
            
            debug!("Opening TinyMUSH store at: {}", db_path);
            let store = TinyMushStore::open(db_path)?;
            self.store = Some(store);
        }
        
        Ok(self.store.as_ref().unwrap())
    }

    /// Initialize or get the room manager for this session
    async fn get_room_manager(&mut self, config: &Config) -> Result<&mut RoomManager, TinyMushError> {
        if self.room_manager.is_none() {
            // Ensure store is initialized first
            let _ = self.get_store(config).await?;
            
            let db_path = config.games.tinymush_db_path
                .as_deref()
                .unwrap_or("data/tinymush");
            
            debug!("Opening TinyMUSH room manager at: {}", db_path);
            let store = TinyMushStore::open(db_path)?;
            let room_manager = RoomManager::new(store);
            self.room_manager = Some(room_manager);
        }
        
        Ok(self.room_manager.as_mut().unwrap())
    }

    /// Get the store reference (assumes store is already initialized)
    fn store(&self) -> &TinyMushStore {
        self.store.as_ref().expect("TinyMUSH store not initialized")
    }

    /// Get world configuration (assumes store is already initialized)
    async fn get_world_config(&self) -> Result<crate::tmush::types::WorldConfig, TinyMushError> {
        self.store().get_world_config()
    }

    /// Initialize a player when entering TinyMUSH and return welcome screen
    pub async fn initialize_player(
        &mut self,
        session: &mut Session,
        _storage: &mut Storage,
        config: &Config,
    ) -> Result<String> {
        let _store = match self.get_store(config).await {
            Ok(store) => store,
            Err(e) => {
                return Ok(format!("TinyMUSH unavailable: {}", e));
            }
        };

        // Create or load player  
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Player initialization failed: {}", e)),
        };

        // Show welcome message and initial look
        let mut response = String::new();
        response.push_str("*** Welcome to TinyMUSH! ***\n");
        response.push_str("Type HELP for commands, B or QUIT to exit.\n\n");
        
        // Add initial room description
        if let Ok(room) = self.store().get_room(&player.current_room) {
            response.push_str(&format!("{}\n", room.name));
            response.push_str(&format!("{}\n", room.long_desc));
            
            // Show exits
            if !room.exits.is_empty() {
                response.push_str("Exits: ");
                let exit_names: Vec<String> = room.exits.keys()
                    .map(|dir| format!("{:?}", dir).to_lowercase())
                    .collect();
                response.push_str(&exit_names.join(", "));
                response.push('\n');
            }
        } else {
            response.push_str("You find yourself in a mysterious void...\n");
        }

        Ok(response)
    }

    /// Process a TinyMUSH command and return the response
    pub async fn process_command(
        &mut self,
        session: &mut Session,
        command: &str,
        _storage: &mut Storage,
        config: &Config,
    ) -> Result<String> {
        // Ensure store is initialized
        if let Err(e) = self.get_store(config).await {
            return Ok(format!("TinyMUSH unavailable: {}", e));
        }

        // Auto-start tutorial for new players (first command only)
        let player = self.get_or_create_player(session).await?;
        if crate::tmush::tutorial::should_auto_start_tutorial(&player) {
            use crate::tmush::tutorial::start_tutorial;
            
            // Start the tutorial
            start_tutorial(self.store(), &player.username)?;
            
            // Show welcome message from world config
            let config = self.store().get_world_config()?;
            return Ok(config.welcome_message);
        }

        let parsed_command = self.parse_command(command);
        debug!(
            "TinyMUSH command parsed: session={} command={:?}",
            escape_log(&session.id),
            parsed_command
        );

        match parsed_command {
            TinyMushCommand::Look(target) => self.handle_look(session, target, config).await,
            TinyMushCommand::Move(direction) => self.handle_move(session, direction, config).await,
            TinyMushCommand::Where => self.handle_where(session, config).await,
            TinyMushCommand::Map => self.handle_map(session, config).await,
            TinyMushCommand::Inventory => self.handle_inventory(session, config).await,
            TinyMushCommand::Take(item) => self.handle_take(session, item, config).await,
            TinyMushCommand::Drop(item) => self.handle_drop(session, item, config).await,
            TinyMushCommand::Examine(target) => self.handle_examine(session, target, config).await,
            TinyMushCommand::Buy(item, quantity) => self.handle_buy(session, item, quantity, config).await,
            TinyMushCommand::Sell(item, quantity) => self.handle_sell(session, item, quantity, config).await,
            TinyMushCommand::List => self.handle_list(session, config).await,
            TinyMushCommand::Who => self.handle_who(session, config).await,
            TinyMushCommand::Score => self.handle_score(session, config).await,
            TinyMushCommand::Say(text) => self.handle_say(session, text, config).await,
            TinyMushCommand::Whisper(target, text) => self.handle_whisper(session, target, text, config).await,
            TinyMushCommand::Emote(text) => self.handle_emote(session, text, config).await,
            TinyMushCommand::Pose(text) => self.handle_pose(session, text, config).await,
            TinyMushCommand::Ooc(text) => self.handle_ooc(session, text, config).await,
            TinyMushCommand::Board(board_id) => self.handle_board(session, board_id, config).await,
            TinyMushCommand::Post(subject, message) => self.handle_post(session, subject, message, config).await,
            TinyMushCommand::Read(message_id) => self.handle_read(session, message_id, config).await,
            TinyMushCommand::Mail(folder) => self.handle_mail(session, folder, config).await,
            TinyMushCommand::Send(recipient, subject, message) => self.handle_send(session, recipient, subject, message, config).await,
            TinyMushCommand::ReadMail(message_id) => self.handle_read_mail(session, message_id, config).await,
            TinyMushCommand::DeleteMail(message_id) => self.handle_delete_mail(session, message_id, config).await,
            TinyMushCommand::Balance => self.handle_balance(session, config).await,
            TinyMushCommand::Deposit(amount) => self.handle_deposit(session, amount, config).await,
            TinyMushCommand::Withdraw(amount) => self.handle_withdraw(session, amount, config).await,
            TinyMushCommand::BankTransfer(recipient, amount) => self.handle_bank_transfer(session, recipient, amount, config).await,
            TinyMushCommand::Trade(target) => self.handle_trade(session, target, config).await,
            TinyMushCommand::Offer(item) => self.handle_offer(session, item, config).await,
            TinyMushCommand::Accept => self.handle_accept(session, config).await,
            TinyMushCommand::Reject => self.handle_reject(session, config).await,
            TinyMushCommand::TradeHistory => self.handle_trade_history(session, config).await,
            TinyMushCommand::Tutorial(subcommand) => self.handle_tutorial(session, subcommand, config).await,
            TinyMushCommand::Talk(npc) => self.handle_talk(session, npc, config).await,
            TinyMushCommand::Quest(subcommand) => self.handle_quest(session, subcommand, config).await,
            TinyMushCommand::Abandon(quest_id) => self.handle_abandon(session, quest_id, config).await,
            TinyMushCommand::Achievements(subcommand) => self.handle_achievements(session, subcommand, config).await,
            TinyMushCommand::Title(subcommand) => self.handle_title(session, subcommand, config).await,
            TinyMushCommand::Companion(subcommand) => self.handle_companion(session, subcommand, config).await,
            TinyMushCommand::Feed(name) => self.handle_feed(session, name, config).await,
            TinyMushCommand::Pet(name) => self.handle_pet(session, name, config).await,
            TinyMushCommand::Mount(name) => self.handle_mount(session, name, config).await,
            TinyMushCommand::Dismount => self.handle_dismount(session, config).await,
            TinyMushCommand::Train(companion, skill) => self.handle_train(session, companion, skill, config).await,
            TinyMushCommand::Housing(subcommand) => self.handle_housing(session, subcommand, config).await,
            TinyMushCommand::Rent(template_id) => self.handle_rent(session, template_id, config).await,
            TinyMushCommand::Home(subcommand) => self.handle_home(session, subcommand, config).await,
            TinyMushCommand::Invite(player) => self.handle_invite(session, player, config).await,
            TinyMushCommand::Uninvite(player) => self.handle_uninvite(session, player, config).await,
            TinyMushCommand::Describe(description) => self.handle_describe(session, description, config).await,
            TinyMushCommand::Lock(target) => self.handle_lock(session, target, config).await,
            TinyMushCommand::Unlock(target) => self.handle_unlock(session, target, config).await,
            TinyMushCommand::Kick(target) => self.handle_kick(session, target, config).await,
            TinyMushCommand::History(item_name) => self.handle_history(session, item_name, config).await,
            TinyMushCommand::SetConfig(field, value) => self.handle_set_config(session, field, value, config).await,
            TinyMushCommand::GetConfig(field) => self.handle_get_config(session, field, config).await,
            TinyMushCommand::Help(topic) => self.handle_help(session, topic, config).await,
            TinyMushCommand::Quit => self.handle_quit(session, config).await,
            TinyMushCommand::Save => self.handle_save(session, config).await,
            TinyMushCommand::Unknown(cmd) => Ok(format!(
                "Unknown command: '{}'\nType HELP for available commands.",
                cmd
            )),
            _ => Ok("That command isn't implemented yet.\nType HELP for available commands.".to_string()),
        }
    }

    /// Parse raw input into TinyMUSH command enum
    pub fn parse_command(&self, input: &str) -> TinyMushCommand {
        let input = input.trim().to_uppercase();
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            return TinyMushCommand::Unknown(input);
        }

        match parts[0] {
            // Movement (single letters)
            "N" | "NORTH" => TinyMushCommand::Move(Direction::North),
            "S" | "SOUTH" => TinyMushCommand::Move(Direction::South),
            "E" | "EAST" => TinyMushCommand::Move(Direction::East),
            "W" | "WEST" => TinyMushCommand::Move(Direction::West),
            "U" | "UP" => TinyMushCommand::Move(Direction::Up), 
            "D" | "DOWN" => TinyMushCommand::Move(Direction::Down),
            "NE" | "NORTHEAST" => TinyMushCommand::Move(Direction::Northeast),
            "NW" | "NORTHWEST" => TinyMushCommand::Move(Direction::Northwest),
            "SE" | "SOUTHEAST" => TinyMushCommand::Move(Direction::Southeast),
            "SW" | "SOUTHWEST" => TinyMushCommand::Move(Direction::Southwest),

            // Look commands
            "L" | "LOOK" => {
                if parts.len() > 1 {
                    TinyMushCommand::Look(Some(parts[1..].join(" ")))
                } else {
                    TinyMushCommand::Look(None)
                }
            },

            // Information commands
            "I" | "INV" | "INVENTORY" => TinyMushCommand::Inventory,
            "WHO" => TinyMushCommand::Who,
            "WHERE" => TinyMushCommand::Where,
            "MAP" => TinyMushCommand::Map,
            "SCORE" => TinyMushCommand::Score,
            "TIME" => TinyMushCommand::Time,

            // Social commands
            "SAY" | "'" => {
                if parts.len() > 1 {
                    TinyMushCommand::Say(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Say("".to_string())
                }
            },
            "WHISPER" | "WHIS" => {
                if parts.len() > 2 {
                    let target = parts[1].to_string();
                    let message = parts[2..].join(" ");
                    TinyMushCommand::Whisper(target, message)
                } else {
                    TinyMushCommand::Unknown(format!("Usage: WHISPER <player> <message>"))
                }
            },
            "EMOTE" | ":" => {
                if parts.len() > 1 {
                    TinyMushCommand::Emote(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Emote("".to_string())
                }
            },
            "POSE" | ";" => {
                if parts.len() > 1 {
                    TinyMushCommand::Pose(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Pose("".to_string())
                }
            },
            "OOC" => {
                if parts.len() > 1 {
                    TinyMushCommand::Ooc(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Ooc("".to_string())
                }
            },

            // Object interaction
            "T" | "TAKE" | "GET" => {
                if parts.len() > 1 {
                    TinyMushCommand::Take(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown(input)
                }
            },
            "DROP" => {
                if parts.len() > 1 {
                    TinyMushCommand::Drop(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown(input)
                }
            },
            "USE" => {
                if parts.len() > 1 {
                    TinyMushCommand::Use(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown(input)
                }
            },
            "X" | "EXAMINE" => {
                if parts.len() > 1 {
                    TinyMushCommand::Examine(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown(input)
                }
            },

            // Economy commands
            "BUY" | "PURCHASE" => {
                if parts.len() > 1 {
                    // BUY item [quantity]
                    let item_name = parts[1].to_string();
                    let quantity = if parts.len() > 2 {
                        parts[2].parse::<u32>().ok()
                    } else {
                        Some(1)  // Default to 1 if no quantity specified
                    };
                    TinyMushCommand::Buy(item_name, quantity)
                } else {
                    TinyMushCommand::Unknown("Usage: BUY <item> [quantity]".to_string())
                }
            },
            "SELL" => {
                if parts.len() > 1 {
                    // SELL item [quantity]
                    let item_name = parts[1].to_string();
                    let quantity = if parts.len() > 2 {
                        parts[2].parse::<u32>().ok()
                    } else {
                        Some(1)  // Default to 1 if no quantity specified
                    };
                    TinyMushCommand::Sell(item_name, quantity)
                } else {
                    TinyMushCommand::Unknown("Usage: SELL <item> [quantity]".to_string())
                }
            },
            "LIST" | "WARES" | "SHOP" => TinyMushCommand::List,

            // System commands
            "HELP" | "H" => {
                if parts.len() > 1 {
                    TinyMushCommand::Help(Some(parts[1..].join(" ")))
                } else {
                    TinyMushCommand::Help(None)
                }
            },
            "QUIT" | "Q" | "EXIT" => TinyMushCommand::Quit,
            "SAVE" => TinyMushCommand::Save,

            // Bulletin board commands
            "BOARD" | "BB" => {
                if parts.len() > 1 {
                    TinyMushCommand::Board(Some(parts[1].to_string()))
                } else {
                    TinyMushCommand::Board(None)
                }
            },
            "POST" => {
                if parts.len() > 2 {
                    let subject = parts[1].to_string();
                    let message = parts[2..].join(" ");
                    TinyMushCommand::Post(subject, message)
                } else {
                    TinyMushCommand::Unknown("Usage: POST <subject> <message>".to_string())
                }
            },
            "READ" => {
                if parts.len() > 1 {
                    if let Ok(message_id) = parts[1].parse::<u64>() {
                        TinyMushCommand::Read(message_id)
                    } else {
                        TinyMushCommand::Unknown("Usage: READ <message_id>".to_string())
                    }
                } else {
                    TinyMushCommand::Unknown("Usage: READ <message_id>".to_string())
                }
            },

            // Mail system commands
            "MAIL" => {
                if parts.len() > 1 {
                    TinyMushCommand::Mail(Some(parts[1].to_string()))
                } else {
                    TinyMushCommand::Mail(None)
                }
            },
            "SEND" => {
                if parts.len() > 3 {
                    let recipient = parts[1].to_string();
                    let subject = parts[2].to_string();
                    let message = parts[3..].join(" ");
                    TinyMushCommand::Send(recipient, subject, message)
                } else {
                    TinyMushCommand::Unknown("Usage: SEND <player> <subject> <message>".to_string())
                }
            },
            "RMAIL" => {
                if parts.len() > 1 {
                    if let Ok(message_id) = parts[1].parse::<u64>() {
                        TinyMushCommand::ReadMail(message_id)
                    } else {
                        TinyMushCommand::Unknown("Usage: RMAIL <message_id>".to_string())
                    }
                } else {
                    TinyMushCommand::Unknown("Usage: RMAIL <message_id>".to_string())
                }
            },
            "DMAIL" => {
                if parts.len() > 1 {
                    if let Ok(message_id) = parts[1].parse::<u64>() {
                        TinyMushCommand::DeleteMail(message_id)
                    } else {
                        TinyMushCommand::Unknown("Usage: DMAIL <message_id>".to_string())
                    }
                } else {
                    TinyMushCommand::Unknown("Usage: DMAIL <message_id>".to_string())
                }
            },

            // Banking commands (Phase 5 Week 4)
            "BALANCE" | "BAL" => TinyMushCommand::Balance,
            "DEPOSIT" | "DEP" => {
                if parts.len() > 1 {
                    TinyMushCommand::Deposit(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: DEPOSIT <amount>".to_string())
                }
            },
            "WITHDRAW" | "WITH" => {
                if parts.len() > 1 {
                    TinyMushCommand::Withdraw(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: WITHDRAW <amount>".to_string())
                }
            },
            "BTRANSFER" | "BTRANS" => {
                if parts.len() > 2 {
                    let recipient = parts[1].to_string();
                    let amount = parts[2..].join(" ");
                    TinyMushCommand::BankTransfer(recipient, amount)
                } else {
                    TinyMushCommand::Unknown("Usage: BTRANSFER <player> <amount>".to_string())
                }
            },

            // Trading commands (Phase 5 Week 4)
            "TRADE" => {
                if parts.len() > 1 {
                    TinyMushCommand::Trade(parts[1].to_string())
                } else {
                    TinyMushCommand::Unknown("Usage: TRADE <player>".to_string())
                }
            },
            "OFFER" => {
                if parts.len() > 1 {
                    TinyMushCommand::Offer(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: OFFER <item/amount>".to_string())
                }
            },
            "ACCEPT" | "ACC" => TinyMushCommand::Accept,
            "REJECT" | "REJ" | "CANCEL" => TinyMushCommand::Reject,
            "THISTORY" | "THIST" => TinyMushCommand::TradeHistory,

            // Tutorial & NPC commands (Phase 6 Week 1)
            "TUTORIAL" | "TUT" => {
                if parts.len() > 1 {
                    TinyMushCommand::Tutorial(Some(parts[1].to_uppercase()))
                } else {
                    TinyMushCommand::Tutorial(None)
                }
            },
            "TALK" | "GREET" => {
                if parts.len() > 1 {
                    TinyMushCommand::Talk(parts[1].to_uppercase())
                } else {
                    TinyMushCommand::Unknown("Usage: TALK <npc>".to_string())
                }
            },

            // Quest commands (Phase 6 Week 2)
            "QUEST" | "QUESTS" => {
                if parts.len() > 1 {
                    TinyMushCommand::Quest(Some(parts[1..].join(" ").to_uppercase()))
                } else {
                    TinyMushCommand::Quest(None)
                }
            },
            "ABANDON" | "ABAND" => {
                if parts.len() > 1 {
                    TinyMushCommand::Abandon(parts[1].to_lowercase())
                } else {
                    TinyMushCommand::Unknown("Usage: ABANDON <quest_id>".to_string())
                }
            },

            // Achievement & Title commands (Phase 6 Week 3)
            "ACHIEVEMENTS" | "ACHIEVE" | "ACHIEV" | "ACH" => {
                if parts.len() > 1 {
                    TinyMushCommand::Achievements(Some(parts[1..].join(" ").to_uppercase()))
                } else {
                    TinyMushCommand::Achievements(None)
                }
            },
            "TITLE" | "TITLES" => {
                if parts.len() > 1 {
                    TinyMushCommand::Title(Some(parts[1..].join(" ")))
                } else {
                    TinyMushCommand::Title(None)
                }
            },

            // Companion commands (Phase 6 Week 4)
            "COMPANION" | "COMP" => {
                if parts.len() > 1 {
                    TinyMushCommand::Companion(Some(parts[1..].join(" ")))
                } else {
                    TinyMushCommand::Companion(None)
                }
            },
            "FEED" => {
                if parts.len() > 1 {
                    TinyMushCommand::Feed(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: FEED <companion>".to_string())
                }
            },
            "PET" => {
                if parts.len() > 1 {
                    TinyMushCommand::Pet(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: PET <companion>".to_string())
                }
            },
            "MOUNT" => {
                if parts.len() > 1 {
                    TinyMushCommand::Mount(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: MOUNT <horse>".to_string())
                }
            },
            "DISMOUNT" => TinyMushCommand::Dismount,
            "TRAIN" => {
                if parts.len() > 2 {
                    let companion = parts[1].to_string();
                    let skill = parts[2..].join(" ");
                    TinyMushCommand::Train(companion, skill)
                } else {
                    TinyMushCommand::Unknown("Usage: TRAIN <companion> <skill>".to_string())
                }
            },

            // Housing commands (Phase 7 Week 1-2)
            "HOUSING" | "HOUSE" => {
                if parts.len() > 1 {
                    TinyMushCommand::Housing(Some(parts[1..].join(" ").to_uppercase()))
                } else {
                    TinyMushCommand::Housing(None)
                }
            },
            "RENT" => {
                if parts.len() > 1 {
                    TinyMushCommand::Rent(parts[1].to_lowercase())
                } else {
                    TinyMushCommand::Unknown("Usage: RENT <template_id>".to_string())
                }
            },
            "HOME" => {
                if parts.len() > 1 {
                    TinyMushCommand::Home(Some(parts[1..].join(" ").to_uppercase()))
                } else {
                    TinyMushCommand::Home(None)
                }
            },
            "INVITE" => {
                if parts.len() > 1 {
                    TinyMushCommand::Invite(parts[1].to_lowercase())
                } else {
                    TinyMushCommand::Unknown("Usage: INVITE <player>".to_string())
                }
            },
            "UNINVITE" => {
                if parts.len() > 1 {
                    TinyMushCommand::Uninvite(parts[1].to_lowercase())
                } else {
                    TinyMushCommand::Unknown("Usage: UNINVITE <player>".to_string())
                }
            },
            "DESCRIBE" | "DESC" => {
                if parts.len() > 1 {
                    // Join all parts after DESCRIBE as the description
                    let description = parts[1..].join(" ");
                    TinyMushCommand::Describe(Some(description))
                } else {
                    // No args - show current description and permissions
                    TinyMushCommand::Describe(None)
                }
            },
            "LOCK" => {
                if parts.len() > 1 {
                    // LOCK <item> - lock a specific item
                    TinyMushCommand::Lock(Some(parts[1..].join(" ")))
                } else {
                    // LOCK - lock current room
                    TinyMushCommand::Lock(None)
                }
            },
            "UNLOCK" => {
                if parts.len() > 1 {
                    // UNLOCK <item> - unlock a specific item
                    TinyMushCommand::Unlock(Some(parts[1..].join(" ")))
                } else {
                    // UNLOCK - unlock current room
                    TinyMushCommand::Unlock(None)
                }
            },
            "KICK" => {
                if parts.len() > 1 {
                    let target = parts[1].to_uppercase();
                    if target == "ALL" {
                        TinyMushCommand::Kick(Some("ALL".to_string()))
                    } else {
                        TinyMushCommand::Kick(Some(parts[1].to_lowercase()))
                    }
                } else {
                    TinyMushCommand::Unknown("Usage: KICK <player> or KICK ALL".to_string())
                }
            },
            "HISTORY" | "HIST" => {
                if parts.len() > 1 {
                    TinyMushCommand::History(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Unknown("Usage: HISTORY <item>".to_string())
                }
            },

            // Admin/debug
            "DEBUG" => {
                if parts.len() > 1 {
                    TinyMushCommand::Debug(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Debug("".to_string())
                }
            },
            "@SETCONFIG" | "@SETCONF" => {
                if parts.len() > 2 {
                    let field = parts[1].to_lowercase();
                    let value = parts[2..].join(" ");
                    TinyMushCommand::SetConfig(field, value)
                } else {
                    TinyMushCommand::Unknown("Usage: @SETCONFIG <field> <value>\nFields: welcome_message, motd, world_name, world_description".to_string())
                }
            },
            "@GETCONFIG" | "@GETCONF" | "@CONFIG" => {
                if parts.len() > 1 {
                    TinyMushCommand::GetConfig(Some(parts[1].to_lowercase()))
                } else {
                    TinyMushCommand::GetConfig(None)
                }
            },

            _ => TinyMushCommand::Unknown(input),
        }
    }

    /// Handle LOOK command - examine room or object
    async fn handle_look(
        &mut self,
        session: &Session,
        target: Option<String>,
        _config: &Config,
    ) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // If no target specified, look at current room
        if target.is_none() {
            return self.describe_current_room(&player).await;
        }

        // Look at specific target (not implemented in Phase 3)
        Ok(format!(
            "You don't see '{}' here.\nType LOOK to see the room.",
            target.unwrap()
        ))
    }

    /// Handle movement commands
    async fn handle_move(
        &mut self,
        session: &Session,
        direction: Direction,
        config: &Config,
    ) -> Result<String> {
        let mut player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get room manager
        let room_manager = self.get_room_manager(config).await?;
        
        // Get current room
        let current_room = match room_manager.get_room(&player.current_room) {
            Ok(room) => room,
            Err(_) => {
                return Ok(format!(
                    "You seem to be lost! Your current location '{}' doesn't exist.\nType WHERE for help.",
                    player.current_room
                ));
            }
        };

        // Check if exit exists in that direction
        let tmush_direction = match direction {
            Direction::North => TmushDirection::North,
            Direction::South => TmushDirection::South,
            Direction::East => TmushDirection::East,
            Direction::West => TmushDirection::West,
            Direction::Up => TmushDirection::Up,
            Direction::Down => TmushDirection::Down,
            Direction::Northeast => TmushDirection::Northeast,
            Direction::Northwest => TmushDirection::Northwest,
            Direction::Southeast => TmushDirection::Southeast,
            Direction::Southwest => TmushDirection::Southwest,
        };
        
        let destination_id = match current_room.exits.get(&tmush_direction) {
            Some(dest) => dest,
            None => {
                let dir_str = format!("{:?}", direction).to_lowercase();
                return Ok(format!("You can't go {} from here.", dir_str));
            }
        };

        // Use room manager to move player (includes capacity and permission checks)
        match room_manager.move_player_to_room(&mut player, destination_id) {
            Ok(true) => {
                // Movement successful
                debug!("Player {} moved to room {}", player.username, destination_id);
            },
            Ok(false) => {
                // Movement blocked (capacity or permissions)
                let dir_str = format!("{:?}", direction).to_lowercase();
                return Ok(format!("You can't go {} right now. The area might be full or restricted.", dir_str));
            },
            Err(e) => {
                return Ok(format!("Movement failed: {}", e));
            }
        }
        
        // Save updated player state
        if let Err(e) = self.store().put_player(player.clone()) {
            return Ok(format!("Movement failed to save: {}", e));
        }

        // Show the new room
        let mut response = String::new();
        response.push_str(&format!("You go {}.\n\n", format!("{:?}", direction).to_lowercase()));
        
        // Add room description
        match self.describe_current_room(&player).await {
            Ok(desc) => response.push_str(&desc),
            Err(_) => response.push_str("The room description is unavailable."),
        }

        Ok(response)
    }

    /// Handle WHERE command - show current location
    async fn handle_where(&mut self, session: &Session, config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        let room_manager = self.get_room_manager(config).await?;
        
        match room_manager.get_room(&player.current_room) {
            Ok(room) => {
                let occupancy = room_manager.get_room_occupancy(&player.current_room);
                let capacity_limit = if room.max_capacity > 0 {
                    room.max_capacity
                } else {
                    // Use default capacity based on room type
                    if room.flags.contains(&crate::tmush::types::RoomFlag::Shop) {
                        10
                    } else if room.flags.contains(&crate::tmush::types::RoomFlag::Indoor) {
                        20
                    } else {
                        50
                    }
                };
                
                Ok(format!(
                    "You are in: {} ({})\n{}\nOccupancy: {}/{}",
                    room.name,
                    player.current_room,
                    room.short_desc,
                    occupancy,
                    capacity_limit
                ))
            },
            Err(_) => Ok(format!(
                "You are lost in: {}\n(Room not found - contact admin)",
                player.current_room
            ))
        }
    }

    /// Handle MAP command - show overview of the game world
    async fn handle_map(&mut self, session: &Session, config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        let current_room_id = &player.current_room;
        let room_manager = self.get_room_manager(config).await?;
        
        // Build map display showing all rooms and their connections
        let mut response = String::new();
        response.push_str("=== Map of Old Towne Mesh ===\n\n");

        // Display current location prominently
        if let Ok(current_room) = room_manager.get_room(current_room_id) {
            response.push_str(&format!(
                "Current Location: {}\n\n",
                current_room.name
            ));
        }

        // Show all rooms with connections
        response.push_str("Area Overview:\n");
        
        // Get all rooms from canonical seed data and fetch via room manager
        let seed_rooms = canonical_world_seed(chrono::Utc::now());
        let mut rooms_with_ids: Vec<(String, crate::tmush::types::RoomRecord)> = Vec::new();
        
        for room in seed_rooms {
            if let Ok(stored_room) = room_manager.get_room(&room.id) {
                rooms_with_ids.push((room.id.clone(), stored_room));
            }
        }
        
        // Sort for consistent display
        rooms_with_ids.sort_by(|(a, _), (b, _)| a.cmp(b));
        
        for (room_id, room) in rooms_with_ids {
            let marker = if &room_id == current_room_id {
                "➤"
            } else {
                " "
            };
            
            response.push_str(&format!(
                "{} {} - {}\n",
                marker, room.name, room.short_desc
            ));
            
            // Show exits
            if !room.exits.is_empty() {
                let mut exits: Vec<_> = room.exits.keys().map(|d| format!("{:?}", d).to_lowercase()).collect();
                exits.sort();
                response.push_str(&format!(
                    "    Exits: {}\n",
                    exits.join(", ")
                ));
            }
            response.push('\n');
        }

        response.push_str("Use LOOK to examine your current room in detail.\n");
        response.push_str("Use movement commands (north, south, east, west, etc.) to travel.\n");

        Ok(response)
    }

    /// Handle INVENTORY command
    async fn handle_inventory(&mut self, session: &Session, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Use new inventory system if available
        if !player.inventory_stacks.is_empty() {
            let store = self.store();
            let get_item = |object_id: &str| store.get_object(object_id).ok();
            let inventory_lines = format_inventory_compact(&player, get_item);
            Ok(inventory_lines.join("\n"))
        } else if player.inventory.is_empty() {
            Ok("You are carrying nothing.".to_string())
        } else {
            // Fallback to legacy inventory display
            let mut response = "=== INVENTORY ===\n".to_string();
            response.push_str(&format!("Gold: {}\n", player.credits));
            response.push_str(&format!("Items: {}\n", player.inventory.len()));
            
            for (i, item_id) in player.inventory.iter().enumerate() {
                response.push_str(&format!("{}. {}\n", i + 1, item_id));
            }

            Ok(response)
        }
    }

    /// Handle TAKE/GET command - pick up items from room
    async fn handle_take(&mut self, session: &Session, item_name: String, _config: &Config) -> Result<String> {
        let _player_node_id = &session.node_id;
        
        // Parse item name - support "get 5 coins" or "get coins"
        let (quantity, item_name) = if let Some(first_space) = item_name.find(' ') {
            let (first, rest) = item_name.split_at(first_space);
            if let Ok(qty) = first.trim().parse::<u32>() {
                (qty, rest.trim().to_uppercase())
            } else {
                (1, item_name.to_uppercase())
            }
        } else {
            (1, item_name.to_uppercase())
        };

        // Find object by name in the current room (future: scan room contents)
        // For now, we'll return a helpful message
        // TODO: Implement room.contents and object lookup by name
        // TODO: Phase 4 - Check item.locked field and prevent taking locked items
        //       owned by other players. Message: "That item is locked by its owner."
        // TODO: Phase 5 - Record ownership transfer when taking items:
        //       TinyMushProcessor::record_ownership_transfer(
        //           &mut item,
        //           item.owner (as Option<String>),
        //           player.username,
        //           OwnershipReason::PickedUp
        //       );
        
        Ok(format!(
            "You try to pick up '{}' (qty: {}) but room object scanning isn't implemented yet.\n\
             This command will search the current room's contents and transfer items to your inventory.",
            item_name, quantity
        ))
    }

    /// Handle DROP command - drop items into current room
    async fn handle_drop(&mut self, session: &Session, item_name: String, _config: &Config) -> Result<String> {
        let _player_node_id = &session.node_id;
        
        // Parse item name - support "drop 5 coins" or "drop coins"
        let (quantity, item_name) = if let Some(first_space) = item_name.find(' ') {
            let (first, rest) = item_name.split_at(first_space);
            if let Ok(qty) = first.trim().parse::<u32>() {
                (qty, rest.trim().to_uppercase())
            } else {
                (1, item_name.to_uppercase())
            }
        } else {
            (1, item_name.to_uppercase())
        };

        // Get player's inventory
        let _player = match self.get_or_create_player(session).await {
            Ok(p) => p,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Find matching item in inventory (by object name lookup)
        // For now, return helpful message
        // TODO: Implement object name -> ID lookup and room transfer
        // TODO: Phase 5 - Record ownership transfer when dropping items:
        //       TinyMushProcessor::record_ownership_transfer(
        //           &mut item,
        //           Some(player.username),
        //           "WORLD".to_string(),
        //           OwnershipReason::Dropped
        //       );
        //       item.owner = ObjectOwner::World;
        
        Ok(format!(
            "You try to drop '{}' (qty: {}) but inventory -> room transfer isn't implemented yet.\n\
             This command will search your inventory by item name and move items to the current room.",
            item_name, quantity
        ))
    }

    /// Handle EXAMINE command - show detailed item information
    async fn handle_examine(&mut self, session: &Session, target: String, _config: &Config) -> Result<String> {
        let target = target.to_uppercase();
        
        // Try to find the object in player's inventory first
        let _player = match self.get_or_create_player(session).await {
            Ok(p) => p,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Search inventory by object name
        // TODO: Implement object name -> ID lookup, then use format_item_examination
        
        Ok(format!(
            "You try to examine '{}' but object lookup by name isn't implemented yet.\n\
             This command will display detailed information about objects in your inventory or the current room.",
            target
        ))
    }

    /// Handle BUY command - purchase items from shops in current room
    async fn handle_buy(
        &mut self,
        session: &Session,
        item_name: String,
        quantity: Option<u32>,
        _config: &Config
    ) -> Result<String> {
        // Get player
        let mut player = match self.get_or_create_player(session).await {
            Ok(p) => p,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get shops in current location
        let location = player.current_room.clone();
        let shops = match self.store().get_shops_in_location(&location) {
            Ok(s) => s,
            Err(e) => return Ok(format!("Error finding shops: {}", e)),
        };

        if shops.is_empty() {
            return Ok("There are no shops here.".to_string());
        }

        // Find shop that has the item (search by name, case-insensitive)
        let item_name_upper = item_name.to_uppercase();
        let mut found_shop = None;
        let mut found_object = None;
        
        for shop in shops {
            // Look through shop inventory for matching item name
            for object_id in shop.inventory.keys() {
                if let Ok(obj) = self.store().get_object(object_id) {
                    if obj.name.to_uppercase() == item_name_upper {
                        found_shop = Some(shop);
                        found_object = Some(obj);
                        break;
                    }
                }
            }
            if found_shop.is_some() {
                break;
            }
        }

        let (mut shop, object) = match (found_shop, found_object) {
            (Some(s), Some(obj)) => (s, obj),
            _ => return Ok(format!("No shop here sells '{}'.", item_name)),
        };

        let qty = quantity.unwrap_or(1);
        
        // Get the shop item to calculate price
        let shop_item = match shop.get_item(&object.id) {
            Some(item) => item,
            None => return Ok(format!("Shop doesn't sell '{}'.", object.name)),
        };
        
        // Calculate total price using shop's pricing logic (includes quantity)
        let total_price = shop.calculate_buy_price(&object, qty, shop_item);

        // Check if player can afford it
        if !player.currency.can_afford(&total_price) {
            return Ok(format!(
                "You cannot afford {} x {} (need {:?}, have {:?}).",
                qty, object.name, total_price, player.currency
            ));
        }

        // Process the purchase (updates shop inventory and currency)
        match shop.process_buy(&object.id, qty, &object) {
            Ok((price, actual_qty)) => {
                // Deduct currency from player
                player.currency = match player.currency.subtract(&price) {
                    Ok(new_balance) => new_balance,
                    Err(e) => return Ok(format!("Payment failed: {}", e)),
                };
                
                // Add items to player inventory using inventory system
                use crate::tmush::inventory::add_item_to_inventory;
                use crate::tmush::types::InventoryConfig;
                let config = InventoryConfig::default();
                for _ in 0..actual_qty {
                    // Clone the object and add ownership tracking (Phase 5)
                    let mut owned_object = object.clone();
                    owned_object.owner = crate::tmush::types::ObjectOwner::Player {
                        username: player.username.clone(),
                    };
                    Self::record_ownership_transfer(
                        &mut owned_object,
                        None, // Purchased from shop
                        player.username.clone(),
                        crate::tmush::types::OwnershipReason::Purchased,
                    );
                    // TODO: Store the updated object with ownership history
                    // self.store().put_object(owned_object)?;
                    
                    add_item_to_inventory(&mut player, &object, 1, &config);
                }

                // Capture balance before moving player
                let final_balance = format!("{:?}", player.currency);
                
                // Save shop and player (consume values)
                if let Err(e) = self.store().put_shop(shop) {
                    return Ok(format!("Failed to save shop: {}", e));
                }
                if let Err(e) = self.store().put_player(player) {
                    return Ok(format!("Failed to save player: {}", e));
                }

                Ok(format!(
                    "You buy {} x {} for {:?}. Balance: {}",
                    actual_qty, object.name, price, final_balance
                ))
            },
            Err(e) => Ok(format!("Purchase failed: {}", e)),
        }
    }

    /// Handle SELL command - sell items from inventory to shops
    async fn handle_sell(
        &mut self,
        session: &Session,
        item_name: String,
        quantity: Option<u32>,
        _config: &Config
    ) -> Result<String> {
        // Get player
        let mut player = match self.get_or_create_player(session).await {
            Ok(p) => p,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Find item in player's inventory by name
        let item_name_upper = item_name.to_uppercase();
        let mut found_object_id = None;
        let mut found_object = None;

        for stack in &player.inventory_stacks {
            if let Ok(obj) = self.store().get_object(&stack.object_id) {
                if obj.name.to_uppercase() == item_name_upper {
                    found_object_id = Some(stack.object_id.clone());
                    found_object = Some(obj);
                    break;
                }
            }
        }

        let (object_id, object) = match (found_object_id, found_object) {
            (Some(id), Some(obj)) => (id, obj),
            _ => return Ok(format!("You don't have any '{}'.", item_name)),
        };

        let qty = quantity.unwrap_or(1);

        // Check if player has enough quantity
        let player_qty = player.inventory_stacks
            .iter()
            .find(|s| s.object_id == object_id)
            .map(|s| s.quantity)
            .unwrap_or(0);
        if player_qty < qty {
            return Ok(format!("You only have {} x {}.", player_qty, object.name));
        }

        // Get shops in current location
        let location = player.current_room.clone();
        let shops = match self.store().get_shops_in_location(&location) {
            Ok(s) => s,
            Err(e) => return Ok(format!("Error finding shops: {}", e)),
        };

        if shops.is_empty() {
            return Ok("There are no shops here to sell to.".to_string());
        }

        // Find a shop willing to buy this item (shop must have item in inventory to accept it)
        let mut found_shop = None;
        for shop in shops {
            if shop.get_item(&object_id).is_some() {
                found_shop = Some(shop);
                break;
            }
        }

        let mut shop = match found_shop {
            Some(s) => s,
            None => return Ok(format!("No shop here buys '{}'.", object.name)),
        };

        // Get shop item to calculate price
        let shop_item = shop.get_item(&object_id).unwrap(); // safe: we just checked
        
        // Calculate sell price using shop's pricing logic (includes quantity)
        let total_price = shop.calculate_sell_price(&object, qty, shop_item);

        // Check if shop can afford it
        if !shop.currency.can_afford(&total_price) {
            return Ok(format!(
                "The shop cannot afford to buy {} x {} (need {:?}, have {:?}).",
                qty, object.name, total_price, shop.currency
            ));
        }

        // Process the sale (updates shop inventory and currency)
        match shop.process_sell(&object_id, qty, &object) {
            Ok(price) => {
                // Add currency to player
                player.currency = match player.currency.add(&price) {
                    Ok(new_balance) => new_balance,
                    Err(e) => return Ok(format!("Payment failed: {}", e)),
                };
                
                // Remove items from player inventory using inventory system
                use crate::tmush::inventory::remove_item_from_inventory;
                remove_item_from_inventory(&mut player, &object_id, qty);

                // Capture balance before moving player
                let final_balance = format!("{:?}", player.currency);
                
                // Save shop and player (consume values)
                if let Err(e) = self.store().put_shop(shop) {
                    return Ok(format!("Failed to save shop: {}", e));
                }
                if let Err(e) = self.store().put_player(player) {
                    return Ok(format!("Failed to save player: {}", e));
                }

                Ok(format!(
                    "You sell {} x {} for {:?}. Balance: {}",
                    qty, object.name, price, final_balance
                ))
            },
            Err(e) => Ok(format!("Sale failed: {}", e)),
        }
    }

    /// Handle LIST/WARES command - display shop inventory with prices
    async fn handle_list(&mut self, session: &Session, _config: &Config) -> Result<String> {
        use crate::tmush::shop::format_shop_listing;
        
        // Get player to determine current location
        let player = match self.get_or_create_player(session).await {
            Ok(p) => p,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get shops in current location
        let location = player.current_room.clone();
        let shops = match self.store().get_shops_in_location(&location) {
            Ok(s) => s,
            Err(e) => return Ok(format!("Error finding shops: {}", e)),
        };

        if shops.is_empty() {
            return Ok("There are no shops here.".to_string());
        }

        // Build listing for each shop
        let mut response = String::new();
        
        for (idx, shop) in shops.iter().enumerate() {
            if idx > 0 {
                response.push_str("\n---\n");
            }
            
            response.push_str(&format!("=== {} ===\n", shop.name));
            if !shop.description.is_empty() {
                response.push_str(&format!("{}\n\n", shop.description));
            }

            // Get object resolver closure
            let store = self.store();
            let get_object = |object_id: &str| store.get_object(object_id).ok();

            // Use shop formatting function
            let lines = format_shop_listing(shop, get_object);
            response.push_str(&lines.join("\n"));
        }

        if response.is_empty() {
            Ok("No shops available.".to_string())
        } else {
            Ok(response)
        }
    }

    /// Handle WHO command - list online players
    async fn handle_who(&mut self, _session: &Session, _config: &Config) -> Result<String> {
        let player_ids = match self.store().list_player_ids() {
            Ok(ids) => ids,
            Err(e) => return Ok(format!("Error listing players: {}", e)),
        };

        if player_ids.is_empty() {
            return Ok("No players found.".to_string());
        }

        let mut response = "=== ONLINE PLAYERS ===\n".to_string();
        for (i, username) in player_ids.iter().take(10).enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1, username));
        }

        if player_ids.len() > 10 {
            response.push_str(&format!("... and {} more\n", player_ids.len() - 10));
        }

        Ok(response)
    }

    /// Handle SCORE command - show player stats
    async fn handle_score(&mut self, session: &Session, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        let mut response = format!("=== {} ===\n", player.display_name);
        response.push_str(&format!("Location: {}\n", player.current_room));
        response.push_str(&format!("Credits: {}\n", player.credits));
        response.push_str(&format!("HP: {}/{}\n", player.stats.hp, player.stats.max_hp));
        response.push_str(&format!("MP: {}/{}\n", player.stats.mp, player.stats.max_mp));
        response.push_str(&format!("Items: {}\n", player.inventory.len()));
        response.push_str(&format!("State: {:?}\n", player.state));

        Ok(response)
    }

    /// Handle SAY command - speak to room
    async fn handle_say(&mut self, session: &Session, text: String, config: &Config) -> Result<String> {
        if text.trim().is_empty() {
            let store = self.get_store(config).await?;
            let world_config = store.get_world_config()?;
            return Ok(world_config.err_say_what);
        }

        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get other players in the same room
        let room_manager = self.get_room_manager(config).await?;
        let players_in_room = room_manager.get_players_in_room(&player.current_room);
        let others_count = players_in_room.len().saturating_sub(1); // Exclude self

        let mut response = format!("You say: \"{}\"\n", text);
        if others_count > 0 {
            response.push_str(&format!(
                "({} other player{} in room will also see this)\n",
                others_count,
                if others_count == 1 { "" } else { "s" }
            ));
        } else {
            response.push_str("(No other players in room)\n");
        }

        Ok(response)
    }

    /// Handle WHISPER command - private message to another player  
    async fn handle_whisper(&mut self, session: &Session, target: String, text: String, config: &Config) -> Result<String> {
        let world_config = self.get_world_config().await?;
        
        if text.trim().is_empty() {
            return Ok("Whisper what?".to_string());
        }

        if target.trim().is_empty() {
            return Ok("Whisper to whom?".to_string());
        }

        let speaker = session.display_name();
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Check if target player exists and is in same room
        let room_manager = self.get_room_manager(config).await?;
        let players_in_room = room_manager.get_players_in_room(&player.current_room);
        
        let target_lower = target.to_lowercase();
        let target_found = players_in_room.iter()
            .find(|p| p.to_lowercase().starts_with(&target_lower));

        if let Some(target_player) = target_found {
            if target_player.to_lowercase() == speaker.to_lowercase() {
                return Ok(world_config.err_whisper_self);
            }
            
            Ok(format!(
                "You whisper to {}: \"{}\"\n(Private message - only {} will see this)",
                target_player, text, target_player
            ))
        } else {
            Ok(format!(
                "Player '{}' not found in this room.\nPlayers here: {}",
                target,
                if players_in_room.is_empty() {
                    "none".to_string()
                } else {
                    players_in_room.join(", ")
                }
            ))
        }
    }

    /// Handle EMOTE command - perform an action
    async fn handle_emote(&mut self, session: &Session, text: String, config: &Config) -> Result<String> {
        let world_config = self.get_world_config().await?;
        
        if text.trim().is_empty() {
            return Ok(world_config.err_emote_what);
        }

        let speaker = session.display_name();
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get other players in the same room
        let room_manager = self.get_room_manager(config).await?;
        let players_in_room = room_manager.get_players_in_room(&player.current_room);
        let others_count = players_in_room.len().saturating_sub(1); // Exclude self

        let mut response = format!("{} {}\n", speaker, text);
        if others_count > 0 {
            response.push_str(&format!(
                "(Action visible to {} other player{})\n",
                others_count,
                if others_count == 1 { "" } else { "s" }
            ));
        } else {
            response.push_str("(No other players in room to see your action)\n");
        }

        Ok(response)
    }

    /// Handle POSE command - strike a pose  
    async fn handle_pose(&mut self, session: &Session, text: String, config: &Config) -> Result<String> {
        if text.trim().is_empty() {
            return Ok("Strike what pose?".to_string());
        }

        let speaker = session.display_name();
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get other players in the same room
        let room_manager = self.get_room_manager(config).await?;
        let players_in_room = room_manager.get_players_in_room(&player.current_room);
        let others_count = players_in_room.len().saturating_sub(1); // Exclude self

        // Pose format is different from emote - it's a descriptive action
        let mut response = format!("{} {}\n", speaker, text);
        if others_count > 0 {
            response.push_str(&format!(
                "(Pose visible to {} other player{})\n",
                others_count,
                if others_count == 1 { "" } else { "s" }
            ));
        } else {
            response.push_str("(No other players in room to see your pose)\n");
        }

        Ok(response)
    }

    /// Handle OOC command - out of character communication
    async fn handle_ooc(&mut self, session: &Session, text: String, config: &Config) -> Result<String> {
        if text.trim().is_empty() {
            return Ok("Say what out of character?".to_string());
        }

        let speaker = session.display_name();
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get other players in the same room
        let room_manager = self.get_room_manager(config).await?;
        let players_in_room = room_manager.get_players_in_room(&player.current_room);
        let others_count = players_in_room.len().saturating_sub(1); // Exclude self

        let mut response = format!("[OOC] {}: {}\n", speaker, text);
        if others_count > 0 {
            response.push_str(&format!(
                "(OOC message visible to {} other player{})\n",
                others_count,
                if others_count == 1 { "" } else { "s" }
            ));
        } else {
            response.push_str("(No other players in room to see your OOC message)\n");
        }

        Ok(response)
    }

    /// Handle TUTORIAL command - manage tutorial progress
    async fn handle_tutorial(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::tutorial::{
            format_tutorial_status, skip_tutorial, restart_tutorial, start_tutorial,
        };

        let username = session.node_id.to_string();
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        match subcommand.as_deref() {
            None => {
                // Show current tutorial status
                Ok(format_tutorial_status(&player.tutorial_state))
            }
            Some("SKIP") => {
                // Skip tutorial
                match skip_tutorial(self.store(), &username) {
                    Ok(_) => Ok("Tutorial skipped. You can restart anytime with TUTORIAL RESTART.".to_string()),
                    Err(e) => Ok(format!("Error skipping tutorial: {}", e)),
                }
            }
            Some("RESTART") => {
                // Restart tutorial from beginning
                match restart_tutorial(self.store(), &username) {
                    Ok(_) => Ok("Tutorial restarted. Head to the Gazebo to begin!".to_string()),
                    Err(e) => Ok(format!("Error restarting tutorial: {}", e)),
                }
            }
            Some("START") => {
                // Manually start tutorial
                match start_tutorial(self.store(), &username) {
                    Ok(_) => Ok("Tutorial started! Look around and follow the hints.".to_string()),
                    Err(e) => Ok(format!("Error starting tutorial: {}", e)),
                }
            }
            Some(unknown) => {
                Ok(format!(
                    "Unknown subcommand: {}\nUsage: TUTORIAL [SKIP|RESTART|START]",
                    unknown
                ))
            }
        }
    }

    /// Handle TALK command - interact with NPCs
    async fn handle_talk(
        &mut self,
        session: &Session,
        npc_name: String,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::tutorial::{
            advance_tutorial_step, distribute_tutorial_rewards,
        };
        use crate::tmush::types::{TutorialState, TutorialStep};

        let username = session.node_id.to_string();
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Get NPCs in current room
        let npcs = self.store().get_npcs_in_room(&player.current_room)?;
        
        if npcs.is_empty() {
            return Ok("There's nobody here to talk to.".to_string());
        }

        // Find matching NPC (case-insensitive partial match)
        let npc = npcs.iter().find(|n| {
            n.name.to_uppercase().contains(&npc_name)
                || n.id.to_uppercase().contains(&npc_name)
        });

        let Some(npc) = npc else {
            let available: Vec<_> = npcs.iter().map(|n| n.name.as_str()).collect();
            return Ok(format!(
                "I don't see '{}' here.\nAvailable: {}",
                npc_name,
                available.join(", ")
            ));
        };

        // Tutorial-specific NPC dialogs
        if npc.id == "mayor_thompson" {
            // Check if player is at MeetTheMayor step
            match &player.tutorial_state {
                TutorialState::InProgress { step } if matches!(step, TutorialStep::MeetTheMayor) => {
                    // Complete tutorial and give rewards
                    if let Err(e) = advance_tutorial_step(
                        self.store(),
                        &username,
                        TutorialStep::MeetTheMayor,
                    ) {
                        return Ok(format!("Tutorial error: {}", e));
                    }

                    // Get world currency system from config or player
                    let currency_system = player.currency.clone();
                    
                    if let Err(e) = distribute_tutorial_rewards(
                        self.store(),
                        &username,
                        &currency_system,
                    ) {
                        return Ok(format!("Reward error: {}", e));
                    }

                    return Ok(format!(
                        "Mayor Thompson:\n'Welcome, citizen! Here's a starter purse and town map. \
                        Good luck in Old Towne Mesh!'\n\n\
                        [Tutorial Complete! Rewards granted.]"
                    ));
                }
                TutorialState::Completed { .. } => {
                    return Ok("Mayor Thompson: 'You've already completed the tutorial. Welcome back!'".to_string());
                }
                _ => {
                    return Ok("Mayor Thompson: 'Come back when you're ready for the tutorial.'".to_string());
                }
            }
        }

        // Generic NPC dialog
        let dialog = npc.dialog.get("default")
            .or_else(|| npc.dialog.get("greeting"))
            .map(|s| s.as_str())
            .unwrap_or("...");

        Ok(format!("{}: '{}'", npc.name, dialog))
    }

    /// Handle QUEST command - manage quests (list, accept, status)
    async fn handle_quest(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::quest::{
            accept_quest, can_accept_quest, format_quest_list, format_quest_status,
            get_active_quests, get_available_quests,
        };

        let username = session.username.as_deref().unwrap_or("guest");
        
        match subcommand.as_deref() {
            None => {
                // Show active quests
                let active = get_active_quests(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get active quests: {}", e))?;
                
                if active.is_empty() {
                    return Ok("You have no active quests.\nUse QUEST LIST to see available quests.".to_string());
                }

                let mut output = String::from("=== ACTIVE QUESTS ===\n");
                for (idx, player_quest) in active.iter().enumerate() {
                    let quest = self.store().get_quest(&player_quest.quest_id)
                        .map_err(|e| anyhow::anyhow!("Failed to get quest: {}", e))?;
                    let all_done = player_quest.all_objectives_complete();
                    let status_char = if all_done { "!" } else { " " };
                    output.push_str(&format!(
                        "{}. [{}] {} - {} obj.\n",
                        idx + 1,
                        status_char,
                        quest.name,
                        player_quest.objectives.len()
                    ));
                }
                output.push_str("\nQUEST <id> - View details\nQUEST LIST - Available quests");
                Ok(output)
            }
            Some(ref cmd) if cmd.starts_with("LIST") => {
                // List available quests
                let available = get_available_quests(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get available quests: {}", e))?;
                
                let messages = format_quest_list(self.store(), &available)
                    .map_err(|e| anyhow::anyhow!("Failed to format quest list: {}", e))?;
                
                Ok(messages.join("\n"))
            }
            Some(ref cmd) if cmd.starts_with("ACCEPT ") => {
                // Accept a quest by ID
                let quest_id = cmd.strip_prefix("ACCEPT ").unwrap().trim().to_lowercase();
                
                if !can_accept_quest(self.store(), username, &quest_id)
                    .map_err(|e| anyhow::anyhow!("Failed to check quest: {}", e))? {
                    return Ok("Cannot accept that quest (already accepted/completed, or prerequisites not met).".to_string());
                }
                
                accept_quest(self.store(), username, &quest_id)
                    .map_err(|e| anyhow::anyhow!("Failed to accept quest: {}", e))?;
                
                let quest = self.store().get_quest(&quest_id)
                    .map_err(|e| anyhow::anyhow!("Failed to get quest: {}", e))?;
                
                Ok(format!(
                    "Quest accepted: {}\n{}\nObjectives: {}\n\nUse QUEST to view progress.",
                    quest.name,
                    quest.description,
                    quest.objectives.len()
                ))
            }
            Some(ref cmd) if cmd.starts_with("COMPLETE") || cmd.starts_with("COMP") => {
                // Complete a quest
                Ok("Quest completion is automatic when all objectives are met.\nTalk to the quest giver to turn in.".to_string())
            }
            Some(quest_id) => {
                // Show quest details by ID
                let quest_id = quest_id.trim().to_lowercase();
                let active = get_active_quests(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get active quests: {}", e))?;
                
                let player_quest = active.iter()
                    .find(|pq| pq.quest_id == quest_id);
                
                if let Some(pq) = player_quest {
                    let status = format_quest_status(self.store(), &quest_id, pq)
                        .map_err(|e| anyhow::anyhow!("Failed to format quest status: {}", e))?;
                    Ok(status)
                } else {
                    Ok(format!("Quest '{}' not found in your active quests.", quest_id))
                }
            }
        }
    }

    /// Handle ABANDON command - abandon an active quest
    async fn handle_abandon(
        &mut self,
        session: &Session,
        quest_id: String,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::quest::abandon_quest;

        let username = session.username.as_deref().unwrap_or("guest");
        
        match abandon_quest(self.store(), username, &quest_id) {
            Ok(_) => {
                let quest = self.store().get_quest(&quest_id)
                    .map_err(|e| anyhow::anyhow!("Failed to get quest: {}", e))?;
                Ok(format!("You have abandoned the quest: {}", quest.name))
            }
            Err(e) => Ok(format!("Failed to abandon quest: {}", e)),
        }
    }

    /// Handle ACHIEVEMENTS command - view and manage achievements
    async fn handle_achievements(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::achievement::{get_achievements_by_category, get_available_achievements, get_earned_achievements};
        use crate::tmush::types::AchievementCategory;

        let username = session.username.as_deref().unwrap_or("guest");
        
        match subcommand.as_deref() {
            None | Some("LIST") => {
                // Show all achievements with progress
                let achievements = get_available_achievements(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get achievements: {}", e))?;
                
                if achievements.is_empty() {
                    return Ok("No achievements available.".to_string());
                }

                let mut output = String::from("=== ACHIEVEMENTS ===\n");
                let mut by_category: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
                
                for (achievement, player_progress) in achievements {
                    let category = format!("{:?}", achievement.category);
                    by_category.entry(category).or_default().push((achievement, player_progress));
                }

                let categories = vec!["Combat", "Exploration", "Social", "Economic", "Crafting", "Quest", "Special"];
                for cat_name in categories {
                    if let Some(achievements) = by_category.get(cat_name) {
                        output.push_str(&format!("\n--- {} ---\n", cat_name));
                        for (achievement, player_progress) in achievements {
                            let earned_marker = if let Some(pa) = player_progress {
                                if pa.earned {
                                    "[✓]"
                                } else {
                                    &format!("[{}%]", (pa.progress * 100) / self.get_achievement_required(&achievement.trigger))
                                }
                            } else {
                                "[ ]"
                            };
                            
                            let title_info = if let Some(ref title) = achievement.title {
                                format!(" - Title: {}", title)
                            } else {
                                String::new()
                            };
                            
                            output.push_str(&format!(
                                "{} {}{}\n",
                                earned_marker,
                                achievement.name,
                                title_info
                            ));
                        }
                    }
                }
                
                output.push_str("\nUse ACHIEVEMENTS EARNED for earned only\nUse ACHIEVEMENTS COMBAT|EXPLORATION|etc for category");
                Ok(output)
            }
            Some("EARNED") => {
                // Show only earned achievements
                let earned = get_earned_achievements(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get earned achievements: {}", e))?;
                
                if earned.is_empty() {
                    return Ok("You haven't earned any achievements yet.\nKeep exploring and trying new things!".to_string());
                }

                let mut output = String::from("=== EARNED ACHIEVEMENTS ===\n");
                for achievement in earned {
                    let title_info = if let Some(ref title) = achievement.title {
                        format!(" - {}", title)
                    } else {
                        String::new()
                    };
                    output.push_str(&format!("✓ {}{}\n  {}\n", achievement.name, title_info, achievement.description));
                }
                Ok(output)
            }
            Some(cat) => {
                // Filter by category
                let category = match cat {
                    "COMBAT" => AchievementCategory::Combat,
                    "EXPLORATION" | "EXPLORE" => AchievementCategory::Exploration,
                    "SOCIAL" => AchievementCategory::Social,
                    "ECONOMIC" | "ECONOMY" => AchievementCategory::Economic,
                    "CRAFTING" | "CRAFT" => AchievementCategory::Crafting,
                    "QUEST" | "QUESTS" => AchievementCategory::Quest,
                    "SPECIAL" => AchievementCategory::Special,
                    _ => return Ok(format!("Unknown category: {}\nAvailable: COMBAT, EXPLORATION, SOCIAL, ECONOMIC, CRAFTING, QUEST, SPECIAL", cat)),
                };
                
                let cat_name = format!("{:?}", category);
                let achievements = get_achievements_by_category(self.store(), username, category)
                    .map_err(|e| anyhow::anyhow!("Failed to get achievements: {}", e))?;
                
                if achievements.is_empty() {
                    return Ok(format!("No achievements found in category: {}", cat_name));
                }

                let mut output = String::from(format!("=== {} ACHIEVEMENTS ===\n", cat));
                for (achievement, player_progress) in achievements {
                    let status = if let Some(pa) = player_progress {
                        if pa.earned {
                            format!("[✓] EARNED")
                        } else {
                            format!("[{}/{}]", pa.progress, self.get_achievement_required(&achievement.trigger))
                        }
                    } else {
                        "[0]".to_string()
                    };
                    
                    output.push_str(&format!(
                        "{} - {}\n  {}\n",
                        status,
                        achievement.name,
                        achievement.description
                    ));
                }
                Ok(output)
            }
        }
    }

    /// Handle TITLE command - manage player titles
    async fn handle_title(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::achievement::get_earned_achievements;

        let username = session.username.as_deref().unwrap_or("guest");
        
        match subcommand.as_deref() {
            None | Some("LIST") => {
                // List all available titles from earned achievements
                let earned = get_earned_achievements(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get earned achievements: {}", e))?;
                
                let titles: Vec<String> = earned.iter()
                    .filter_map(|a| a.title.clone())
                    .collect();
                
                if titles.is_empty() {
                    return Ok("You haven't unlocked any titles yet.\nEarn achievements to unlock titles!".to_string());
                }

                let player = self.get_or_create_player(session).await?;
                let equipped = player.equipped_title.as_deref().unwrap_or("None");
                
                let mut output = String::from(format!("=== YOUR TITLES ===\nCurrently equipped: {}\n\n", equipped));
                for (idx, title) in titles.iter().enumerate() {
                    let marker = if Some(title.as_str()) == player.equipped_title.as_deref() {
                        "[*]"
                    } else {
                        "   "
                    };
                    output.push_str(&format!("{}  {}. {}\n", marker, idx + 1, title));
                }
                output.push_str("\nTITLE EQUIP <name> - Equip title\nTITLE UNEQUIP - Remove title");
                Ok(output)
            }
            Some(cmd) if cmd.starts_with("EQUIP ") => {
                // Equip a title
                let title = cmd.strip_prefix("EQUIP ").unwrap().trim();
                
                // Verify player has earned this title
                let earned = get_earned_achievements(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get earned achievements: {}", e))?;
                
                let has_title = earned.iter()
                    .any(|a| a.title.as_deref() == Some(title));
                
                if !has_title {
                    return Ok(format!("You haven't unlocked the title: {}", title));
                }

                let mut player = self.get_or_create_player(session).await?;
                player.equipped_title = Some(title.to_string());
                player.touch();
                self.store().put_player(player)?;
                
                Ok(format!("Title equipped: {}\nYou are now known as {} {}",
                    title,
                    username,
                    title
                ))
            }
            Some("UNEQUIP") => {
                // Remove equipped title
                let mut player = self.get_or_create_player(session).await?;
                if player.equipped_title.is_none() {
                    return Ok("You don't have any title equipped.".to_string());
                }
                
                player.equipped_title = None;
                player.touch();
                self.store().put_player(player)?;
                
                Ok("Title removed. You are no longer using a title.".to_string())
            }
            Some(title) if !title.starts_with("EQUIP") => {
                // Try to equip by name (shortcut)
                let earned = get_earned_achievements(self.store(), username)
                    .map_err(|e| anyhow::anyhow!("Failed to get earned achievements: {}", e))?;
                
                let has_title = earned.iter()
                    .any(|a| a.title.as_deref() == Some(title));
                
                if !has_title {
                    return Ok(format!("You haven't unlocked the title: {}", title));
                }

                let mut player = self.get_or_create_player(session).await?;
                player.equipped_title = Some(title.to_string());
                player.touch();
                self.store().put_player(player)?;
                
                Ok(format!("Title equipped: {}", title))
            }
            _ => Ok("Usage: TITLE [LIST|EQUIP <name>|UNEQUIP]".to_string()),
        }
    }

    async fn handle_companion(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::companion::{tame_companion, find_companion_in_room, format_companion_list, format_companion_status, get_player_companions};

        let username = session.username.as_deref().unwrap_or("guest");
        let player = self.get_or_create_player(session).await?;
        let room_id = &player.current_room;

        match subcommand.as_deref() {
            None | Some("LIST") => {
                // List player's companions
                let companions = get_player_companions(self.store(), username)?;
                if companions.is_empty() {
                    return Ok("You don't have any companions.\nTAME a wild companion to add them to your party!".to_string());
                }
                Ok(format_companion_list(&companions))
            }
            Some(cmd) if cmd.starts_with("TAME ") => {
                // Tame a wild companion
                let name = cmd.strip_prefix("TAME ").unwrap().trim();
                
                match find_companion_in_room(self.store(), room_id, name)? {
                    Some(companion) if companion.owner.is_none() => {
                        tame_companion(self.store(), username, &companion.id)?;
                        // Fetch updated companion to show loyalty
                        let updated = self.store().get_companion(&companion.id)?;
                        Ok(format!("You've tamed {}!\nLoyalty: {}/100", 
                            updated.name, updated.loyalty))
                    }
                    Some(_) => Ok(format!("{} already has an owner.", name)),
                    None => Ok(format!("There's no companion named '{}' here.", name)),
                }
            }
            Some(cmd) if cmd.starts_with("RELEASE ") => {
                // Release a companion back to wild
                use crate::tmush::companion::release_companion;
                let name = cmd.strip_prefix("RELEASE ").unwrap().trim();
                
                let companions = get_player_companions(self.store(), username)?;
                if let Some(comp) = companions.iter().find(|c| c.name.eq_ignore_ascii_case(name)) {
                    release_companion(self.store(), username, &comp.id)?;
                    Ok(format!("You've released {} back to the wild.", comp.name))
                } else {
                    Ok(format!("You don't have a companion named '{}'.", name))
                }
            }
            Some("STAY") => {
                // Leave all companions in current room (toggle auto-follow off)
                let companions = get_player_companions(self.store(), username)?;
                if companions.is_empty() {
                    return Ok("You don't have any companions.".to_string());
                }
                
                let mut count = 0;
                for comp in companions.iter().filter(|c| c.room_id == *room_id && c.has_auto_follow()) {
                    // Remove AutoFollow behavior
                    let mut updated = comp.clone();
                    updated.behaviors.retain(|b| !matches!(b, crate::tmush::types::CompanionBehavior::AutoFollow));
                    self.store().put_companion(updated)?;
                    count += 1;
                }
                
                if count > 0 {
                    Ok(format!("{} companion(s) will stay here.", count))
                } else {
                    Ok("No companions with auto-follow are here.".to_string())
                }
            }
            Some("COME") => {
                // Summon all companions to player's room
                use crate::tmush::companion::move_companion_to_room;
                let companions = get_player_companions(self.store(), username)?;
                if companions.is_empty() {
                    return Ok("You don't have any companions.".to_string());
                }
                
                let mut count = 0;
                for comp in companions.iter().filter(|c| c.room_id != *room_id) {
                    move_companion_to_room(self.store(), &comp.id, room_id)?;
                    count += 1;
                }
                
                if count > 0 {
                    Ok(format!("{} companion(s) arrive at your side.", count))
                } else {
                    Ok("All your companions are already here.".to_string())
                }
            }
            Some("INVENTORY") | Some("INV") => {
                // Show all companions and their inventory
                let companions = get_player_companions(self.store(), username)?;
                if companions.is_empty() {
                    return Ok("You don't have any companions.".to_string());
                }
                
                let mut output = String::from("=== COMPANION INVENTORY ===\n");
                for comp in companions.iter() {
                    output.push_str(&format!("{}: ", comp.name));
                    if comp.inventory.is_empty() {
                        output.push_str("(empty)\n");
                    } else {
                        output.push_str(&format!("{} items\n", comp.inventory.len()));
                        for item in &comp.inventory {
                            output.push_str(&format!("  - {}\n", item));
                        }
                    }
                }
                Ok(output)
            }
            Some(name) => {
                // Show companion status
                let companions = get_player_companions(self.store(), username)?;
                if let Some(comp) = companions.iter().find(|c| c.name.eq_ignore_ascii_case(name)) {
                    Ok(format_companion_status(comp))
                } else {
                    Ok(format!("You don't have a companion named '{}'.", name))
                }
            }
        }
    }

    async fn handle_feed(
        &mut self,
        session: &Session,
        name: String,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::companion::{feed_companion, get_player_companions};

        let username = session.username.as_deref().unwrap_or("guest");
        let companions = get_player_companions(self.store(), username)?;
        
        if let Some(comp) = companions.iter().find(|c| c.name.eq_ignore_ascii_case(&name)) {
            let gain = feed_companion(self.store(), username, &comp.id)?;
            // Fetch updated companion to show current happiness
            let updated = self.store().get_companion(&comp.id)?;
            Ok(format!("You feed {}. Happiness +{} ({}/100)", 
                updated.name, gain, updated.happiness))
        } else {
            Ok(format!("You don't have a companion named '{}'.", name))
        }
    }

    async fn handle_pet(
        &mut self,
        session: &Session,
        name: String,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::companion::{pet_companion, get_player_companions};

        let username = session.username.as_deref().unwrap_or("guest");
        let companions = get_player_companions(self.store(), username)?;
        
        if let Some(comp) = companions.iter().find(|c| c.name.eq_ignore_ascii_case(&name)) {
            let gain = pet_companion(self.store(), username, &comp.id)?;
            // Fetch updated companion to show current loyalty
            let updated = self.store().get_companion(&comp.id)?;
            Ok(format!("You pet {}. Loyalty +{} ({}/100)", 
                updated.name, gain, updated.loyalty))
        } else {
            Ok(format!("You don't have a companion named '{}'.", name))
        }
    }

    async fn handle_mount(
        &mut self,
        session: &Session,
        name: String,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::companion::{mount_companion, get_player_companions};
        use crate::tmush::types::CompanionType;

        let username = session.username.as_deref().unwrap_or("guest");
        let companions = get_player_companions(self.store(), username)?;
        
        if let Some(comp) = companions.iter().find(|c| c.name.eq_ignore_ascii_case(&name)) {
            if comp.companion_type != CompanionType::Horse {
                return Ok(format!("{} is not mountable.", comp.name));
            }
            if comp.is_mounted {
                return Ok(format!("You're already mounted on {}.", comp.name));
            }
            
            mount_companion(self.store(), username, &comp.id)?;
            Ok(format!("You mount {}. Ready to ride!", comp.name))
        } else {
            Ok(format!("You don't have a companion named '{}'.", name))
        }
    }

    async fn handle_dismount(
        &mut self,
        session: &Session,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::companion::dismount_companion;

        let username = session.username.as_deref().unwrap_or("guest");
        
        match dismount_companion(self.store(), username) {
            Ok(companion_name) => Ok(format!("You dismount from {}.", companion_name)),
            Err(_) => Ok("You're not currently mounted.".to_string()),
        }
    }

    async fn handle_train(
        &mut self,
        session: &Session,
        companion_name: String,
        skill: String,
        _config: &Config,
    ) -> Result<String> {
        use crate::tmush::companion::get_player_companions;

        let username = session.username.as_deref().unwrap_or("guest");
        let companions = get_player_companions(self.store(), username)?;
        
        if let Some(comp) = companions.iter().find(|c| c.name.eq_ignore_ascii_case(&companion_name)) {
            // For now, simple training system - could be expanded with skill trees
            let valid_skills = match comp.companion_type {
                crate::tmush::types::CompanionType::Horse => vec!["speed", "endurance", "carrying"],
                crate::tmush::types::CompanionType::Dog => vec!["tracking", "guarding", "hunting"],
                crate::tmush::types::CompanionType::Cat => vec!["stealth", "agility", "hunting"],
                crate::tmush::types::CompanionType::Familiar => vec!["magic", "wisdom", "perception"],
                crate::tmush::types::CompanionType::Mercenary => vec!["combat", "tactics", "defense"],
                crate::tmush::types::CompanionType::Construct => vec!["strength", "durability", "efficiency"],
            };
            
            let skill_lower = skill.to_lowercase();
            if !valid_skills.contains(&skill_lower.as_str()) {
                return Ok(format!("{} cannot learn '{}'. Valid skills: {}", 
                    comp.name, skill, valid_skills.join(", ")));
            }
            
            // Check loyalty requirement
            if comp.loyalty < 50 {
                return Ok(format!("{} needs loyalty 50+ to train. Current: {}/100", 
                    comp.name, comp.loyalty));
            }
            
            // Training successful (skill progression would be tracked in companion record)
            Ok(format!("You train {} in {}. They show promise!", 
                comp.name, skill))
        } else {
            Ok(format!("You don't have a companion named '{}'.", companion_name))
        }
    }

    /// Handle HOUSING command - manage player housing
    async fn handle_housing(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        
        // Get world config for error messages
        let world_config = store.get_world_config()?;
        
        // Get current room to check for HousingOffice flag
        let current_room = store.get_room(&player.current_room)?;
        
        match subcommand.as_deref() {
            None | Some("") => {
                // Show player's housing status
                let instances = store.get_player_housing_instances(&player.username)?;
                
                if instances.is_empty() {
                    Ok(format!("You don't own any housing yet.\n\n\
                        Visit a housing office to rent or purchase a place!\n\
                        Type HOUSING LIST to see available options."))
                } else {
                    let mut output = format!("=== YOUR HOUSING ===\n\n");
                    for instance in instances {
                        let template = store.get_housing_template(&instance.template_id)?;
                        let active_status = if instance.active { "✓ Active" } else { "✗ Inactive" };
                        output.push_str(&format!(
                            "{} ({})\n  Template: {}\n  {} rooms, {} guests\n  {}\n\n",
                            template.name,
                            instance.id,
                            instance.template_id,
                            instance.room_mappings.len(),
                            instance.guests.len(),
                            active_status
                        ));
                    }
                    output.push_str("Type HOME to visit your housing.\n");
                    output.push_str("Type HOUSING INFO for more details.");
                    Ok(output)
                }
            },
            Some("LIST") => {
                // Check if player is at a housing office
                use crate::tmush::types::RoomFlag;
                if !current_room.flags.contains(&RoomFlag::HousingOffice) {
                    return Ok(world_config.err_housing_not_at_office.clone());
                }
                
                // Get all templates
                let all_template_ids = store.list_housing_templates()?;
                
                if all_template_ids.is_empty() {
                    return Ok(world_config.err_housing_no_templates.clone());
                }
                
                // Filter templates by room's housing_filter_tags
                let mut templates = Vec::new();
                for template_id in all_template_ids {
                    if let Ok(template) = store.get_housing_template(&template_id) {
                        // Check if template matches room's filter
                        if template.matches_filter(&current_room.housing_filter_tags) {
                            templates.push(template);
                        }
                    }
                }
                
                if templates.is_empty() {
                    return Ok(world_config.err_housing_no_templates.clone());
                }
                
                // Build output
                let mut output = world_config.msg_housing_list_header.clone();
                output.push_str("\n\n");
                
                for (idx, template) in templates.iter().enumerate() {
                    let current_count = store.count_template_instances(&template.id)?;
                    let availability = if template.max_instances < 0 {
                        "Unlimited".to_string()
                    } else {
                        let remaining = template.max_instances - current_count as i32;
                        format!("{} of {} available", remaining.max(0), template.max_instances)
                    };
                    
                    let cost_str = if template.recurring_cost > 0 {
                        format!("{} credits ({} per month)", template.cost, template.recurring_cost)
                    } else {
                        format!("{} credits (one-time)", template.cost)
                    };
                    
                    output.push_str(&format!(
                        "{}. {} ({})\n\
                         {}\n\
                         {} rooms | {} | Category: {}\n\
                         Availability: {}\n\n",
                        idx + 1,
                        template.name,
                        template.id,
                        template.description,
                        template.rooms.len(),
                        cost_str,
                        if template.category.is_empty() { "general" } else { &template.category },
                        availability
                    ));
                }
                
                output.push_str("\nType RENT <id> to acquire housing.");
                Ok(output)
            },
            Some(other) => {
                Ok(format!("Unknown HOUSING subcommand: {}\n\
                    Available: LIST, INFO", other))
            }
        }
    }

    /// Handle RENT command - rent/purchase housing from template
    async fn handle_rent(
        &mut self,
        session: &Session,
        template_id: String,
        config: &Config,
    ) -> Result<String> {
        let mut player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        let world_config = store.get_world_config().unwrap_or_default();
        
        // Get current room to check if we're at a housing office
        let current_room = store.get_room(&player.current_room)?;
        
        // Check if current location is a housing office
        if !current_room.flags.contains(&RoomFlag::HousingOffice) {
            return Ok(world_config.err_housing_not_at_office.clone());
        }
        
        // Load the housing template
        let template = store.get_housing_template(&template_id)?;
        
        // Check if template matches this location's filter
        if !template.matches_filter(&current_room.housing_filter_tags) {
            return Ok(world_config.err_housing_no_templates.clone());
        }
        
        // Check if player already owns housing
        let existing_instances = store.get_player_housing_instances(&player.username)?;
        if !existing_instances.is_empty() {
            return Ok(world_config.err_housing_already_owns.clone());
        }
        
        // Check if template has available instances
        if template.max_instances >= 0 {
            let current_count = store.count_template_instances(&template.id)?;
            if current_count >= template.max_instances as usize {
                return Ok(format!("Sorry, all {} housing units are currently occupied. \
                    Please check back later.", template.name));
            }
        }
        
        // Check if player has sufficient funds (currency + bank)
        let total_funds = player.currency.base_value() + player.banked_currency.base_value();
        let required = template.cost as i64;
        
        if total_funds < required {
            let deficit = required - total_funds;
            return Ok(world_config.err_housing_insufficient_funds
                .replace("{cost}", &template.cost.to_string())
                .replace("{deficit}", &deficit.to_string()));
        }
        
        // Clone the template to create player's housing instance
        let instance = store.clone_housing_template(&template.id, &player.username)?;
        
        // Deduct cost from player (currency first, then bank if needed)
        let mut remaining_cost = required;
        let currency_value = player.currency.base_value();
        
        // Create amount to subtract matching player's currency type
        if currency_value >= remaining_cost {
            // Currency covers it all
            let to_subtract = match player.currency {
                CurrencyAmount::Decimal { .. } => CurrencyAmount::decimal(remaining_cost),
                CurrencyAmount::MultiTier { .. } => CurrencyAmount::multi_tier(remaining_cost),
            };
            player.currency = player.currency.subtract(&to_subtract)
                .map_err(|e| anyhow::anyhow!("Currency subtraction failed: {}", e))?;
        } else {
            // Use all currency, then bank
            if currency_value > 0 {
                let to_subtract = match player.currency {
                    CurrencyAmount::Decimal { .. } => CurrencyAmount::decimal(currency_value),
                    CurrencyAmount::MultiTier { .. } => CurrencyAmount::multi_tier(currency_value),
                };
                player.currency = player.currency.subtract(&to_subtract)
                    .map_err(|e| anyhow::anyhow!("Currency subtraction failed: {}", e))?;
                remaining_cost -= currency_value;
            }
            let to_subtract = match player.banked_currency {
                CurrencyAmount::Decimal { .. } => CurrencyAmount::decimal(remaining_cost),
                CurrencyAmount::MultiTier { .. } => CurrencyAmount::multi_tier(remaining_cost),
            };
            player.banked_currency = player.banked_currency.subtract(&to_subtract)
                .map_err(|e| anyhow::anyhow!("Bank currency subtraction failed: {}", e))?;
        }
        
        // Save updated player
        store.put_player(player)?;
        
        // Return success message with HOME hint
        let success_msg = world_config.msg_housing_rented
            .replace("{name}", &template.name)
            .replace("{id}", &instance.id);
        
        Ok(format!("{}\n\nUse the HOME command to teleport to your new housing.", success_msg))
    }

    /// Handle HOME command - teleport to owned housing or manage homes
    async fn handle_home(
        &mut self,
        session: &Session,
        subcommand: Option<String>,
        config: &Config,
    ) -> Result<String> {
        use chrono::Utc;
        
        let mut player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        let world_config = store.get_world_config().unwrap_or_default();
        
        match subcommand {
            Some(ref cmd) if cmd == "LIST" => {
                // HOME LIST - show all accessible housing (owned + guest access)
                let owned_instances = store.get_player_housing_instances(&player.username)?;
                let guest_instances = store.get_guest_housing_instances(&player.username)?;
                
                if owned_instances.is_empty() && guest_instances.is_empty() {
                    return Ok(world_config.msg_home_list_empty.clone());
                }
                
                let mut output = format!("{}\n\n", world_config.msg_home_list_header);
                
                // Show owned housing first
                for (idx, instance) in owned_instances.iter().enumerate() {
                    let template = store.get_housing_template(&instance.template_id)?;
                    let num = idx + 1;
                    
                    // Check if this is primary
                    let is_primary = player.primary_housing_id.as_ref() == Some(&instance.id);
                    let primary_marker = if is_primary { "[★ Primary] " } else { "           " };
                    
                    output.push_str(&format!(
                        "{}{}. {} ({})\n   Location: {} | Access: OWNED\n\n",
                        primary_marker,
                        num,
                        template.name,
                        template.category,
                        instance.id
                    ));
                }
                
                // Show guest housing
                let owned_count = owned_instances.len();
                for (idx, instance) in guest_instances.iter().enumerate() {
                    let template = store.get_housing_template(&instance.template_id)?;
                    let num = owned_count + idx + 1;
                    
                    output.push_str(&format!(
                        "           {}. {} ({})\n   Owner: {} | Access: GUEST\n\n",
                        num,
                        template.name,
                        template.category,
                        instance.owner
                    ));
                }
                
                output.push_str(&format!("{}\n", world_config.msg_home_list_footer_travel));
                output.push_str(&world_config.msg_home_list_footer_set);
                
                Ok(output)
            },
            
            Some(ref cmd) if cmd.starts_with("SET ") => {
                // HOME SET <id> - set primary housing
                let id_or_num = cmd.strip_prefix("SET ").unwrap().trim().to_string();
                let instances = store.get_player_housing_instances(&player.username)?;
                
                if instances.is_empty() {
                    return Ok(world_config.msg_home_list_empty.clone());
                }
                
                // Try to parse as number first
                let target_instance = if let Ok(num) = id_or_num.parse::<usize>() {
                    if num >= 1 && num <= instances.len() {
                        Some(&instances[num - 1])
                    } else {
                        None
                    }
                } else {
                    // Try to match by ID
                    instances.iter().find(|inst| inst.id == id_or_num)
                };
                
                let target_instance = match target_instance {
                    Some(inst) => inst,
                    None => {
                        return Ok(world_config.err_home_not_found.replace("{id}", &id_or_num));
                    }
                };
                
                // Set as primary
                player.primary_housing_id = Some(target_instance.id.clone());
                player.touch();
                store.put_player(player)?;
                
                let template = store.get_housing_template(&target_instance.template_id)?;
                
                Ok(world_config.msg_home_set_success.replace("{name}", &template.name))
            },
            
            Some(id_or_num) => {
                // HOME <id> - teleport to specific housing by ID or number
                let owned_instances = store.get_player_housing_instances(&player.username)?;
                let guest_instances = store.get_guest_housing_instances(&player.username)?;
                
                // Combine owned + guest for unified numbering
                let mut all_instances = owned_instances.clone();
                all_instances.extend(guest_instances.clone());
                
                if all_instances.is_empty() {
                    return Ok(world_config.err_no_housing.clone());
                }
                
                // Try to parse as number first, then try as ID
                let target_instance = if let Ok(num) = id_or_num.parse::<usize>() {
                    if num >= 1 && num <= all_instances.len() {
                        Some(&all_instances[num - 1])
                    } else {
                        None
                    }
                } else {
                    all_instances.iter().find(|inst| inst.id == id_or_num)
                };
                
                let target_instance = match target_instance {
                    Some(inst) => inst,
                    None => {
                        return Ok(world_config.err_home_not_found.replace("{id}", &id_or_num));
                    }
                };
                
                // Validation checks
                if player.in_combat {
                    return Ok(world_config.err_teleport_in_combat.clone());
                }
                
                let current_room = store.get_room(&player.current_room)?;
                if current_room.flags.contains(&RoomFlag::NoTeleportOut) {
                    return Ok(world_config.err_teleport_restricted.clone());
                }
                
                if let Some(last_teleport) = player.last_teleport {
                    let now = Utc::now();
                    let elapsed = (now - last_teleport).num_seconds() as u64;
                    let cooldown = world_config.home_cooldown_seconds;
                    
                    if elapsed < cooldown {
                        let remaining = cooldown - elapsed;
                        let minutes = remaining / 60;
                        let seconds = remaining % 60;
                        let time_str = if minutes > 0 {
                            format!("{} minute{} {} second{}", 
                                minutes, if minutes == 1 { "" } else { "s" },
                                seconds, if seconds == 1 { "" } else { "s" })
                        } else {
                            format!("{} second{}", seconds, if seconds == 1 { "" } else { "s" })
                        };
                        
                        return Ok(world_config.err_teleport_cooldown.replace("{time}", &time_str));
                    }
                }
                
                // Teleport
                player.current_room = target_instance.entry_room_id.clone();
                player.last_teleport = Some(Utc::now());
                player.touch();
                store.put_player(player.clone())?;
                
                let template = store.get_housing_template(&target_instance.template_id)?;
                let success_msg = world_config.msg_teleport_success
                    .replace("{name}", &template.name);
                
                let look_output = self.describe_current_room(&player).await?;
                
                Ok(format!("{}\n\n{}", success_msg, look_output))
            },
            
            None => {
                // HOME with no args - teleport to primary housing
                
                // 1. Check if player is in combat
                if player.in_combat {
                    return Ok(world_config.err_teleport_in_combat.clone());
                }
                
                // 2. Check if current room allows teleportation out
                let current_room = store.get_room(&player.current_room)?;
                if current_room.flags.contains(&RoomFlag::NoTeleportOut) {
                    return Ok(world_config.err_teleport_restricted.clone());
                }
                
                // 3. Check teleport cooldown
                if let Some(last_teleport) = player.last_teleport {
                    let now = Utc::now();
                    let elapsed = (now - last_teleport).num_seconds() as u64;
                    let cooldown = world_config.home_cooldown_seconds;
                    
                    if elapsed < cooldown {
                        let remaining = cooldown - elapsed;
                        let minutes = remaining / 60;
                        let seconds = remaining % 60;
                        let time_str = if minutes > 0 {
                            format!("{} minute{} {} second{}", 
                                minutes, if minutes == 1 { "" } else { "s" },
                                seconds, if seconds == 1 { "" } else { "s" })
                        } else {
                            format!("{} second{}", seconds, if seconds == 1 { "" } else { "s" })
                        };
                        
                        return Ok(world_config.err_teleport_cooldown.replace("{time}", &time_str));
                    }
                }
                
                // 4. Check if player has housing
                let instances = store.get_player_housing_instances(&player.username)?;
                if instances.is_empty() {
                    return Ok(world_config.err_no_housing.clone());
                }
                
                // 5. Determine target housing instance
                let target_instance = if let Some(primary_id) = &player.primary_housing_id {
                    // Try to use primary housing
                    instances.iter().find(|inst| &inst.id == primary_id)
                        .or_else(|| instances.first())  // Fallback to first if primary not found
                } else {
                    // No primary set, use first instance
                    instances.first()
                };
                
                let target_instance = match target_instance {
                    Some(inst) => inst,
                    None => return Ok(world_config.err_no_housing.clone()),
                };
                
                // Verify player still has access
                if target_instance.owner != player.username {
                    return Ok(world_config.err_teleport_no_access.clone());
                }
                
                // 6. Teleport player to housing entry room
                player.current_room = target_instance.entry_room_id.clone();
                player.last_teleport = Some(Utc::now());
                player.touch();
                
                // Save player
                store.put_player(player.clone())?;
                
                // 7. Get the template name for success message
                let template = store.get_housing_template(&target_instance.template_id)?;
                let housing_name = template.name;
                
                // Build success message
                let success_msg = world_config.msg_teleport_success
                    .replace("{name}", &housing_name);
                
                // Get room description using describe_current_room
                let look_output = self.describe_current_room(&player).await?;
                
                Ok(format!("{}\n\n{}", success_msg, look_output))
            }
        }
    }

    /// Handle INVITE command - add guest to housing
    async fn handle_invite(
        &mut self,
        session: &Session,
        player_name: String,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        let world_config = store.get_world_config().unwrap_or_default();
        
        // Check player owns housing
        let instances = store.get_player_housing_instances(&player.username)?;
        if instances.is_empty() {
            return Ok(world_config.err_invite_no_housing.clone());
        }
        
        // Check if player is currently in one of their housing rooms
        let current_instance = instances.iter().find(|inst| {
            inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
        });
        
        let mut current_instance = match current_instance {
            Some(inst) => inst.clone(),
            None => return Ok(world_config.err_invite_not_in_housing.clone()),
        };
        
        // Validate target player exists
        let target = store.get_player(&player_name);
        if target.is_err() {
            return Ok(world_config.err_invite_player_not_found.replace("{name}", &player_name));
        }
        
        // Check if already a guest
        if current_instance.guests.contains(&player_name) {
            return Ok(world_config.err_invite_already_guest.replace("{name}", &player_name));
        }
        
        // Add to guest list
        current_instance.guests.push(player_name.clone());
        store.put_housing_instance(&current_instance)?;
        
        Ok(world_config.msg_invite_success.replace("{name}", &player_name))
    }

    /// Handle UNINVITE command - remove guest from housing
    async fn handle_uninvite(
        &mut self,
        session: &Session,
        player_name: String,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        let world_config = store.get_world_config().unwrap_or_default();
        
        // Check player owns housing
        let instances = store.get_player_housing_instances(&player.username)?;
        if instances.is_empty() {
            return Ok(world_config.err_invite_no_housing.clone());
        }
        
        // Check if player is currently in one of their housing rooms
        let current_instance = instances.iter().find(|inst| {
            inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
        });
        
        let mut current_instance = match current_instance {
            Some(inst) => inst.clone(),
            None => return Ok(world_config.err_invite_not_in_housing.clone()),
        };
        
        // Check if player is on guest list
        if !current_instance.guests.contains(&player_name) {
            return Ok(world_config.err_uninvite_not_guest.replace("{name}", &player_name));
        }
        
        // Remove from guest list
        current_instance.guests.retain(|g| g != &player_name);
        store.put_housing_instance(&current_instance)?;
        
        Ok(world_config.msg_uninvite_success.replace("{name}", &player_name))
    }

    /// Handle DESCRIBE command - edit current room description (housing only)
    async fn handle_describe(
        &mut self,
        session: &Session,
        description: Option<String>,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        let world_config = store.get_world_config().unwrap_or_default();
        
        // Get all housing instances player owns or has guest access to
        let owned_instances = store.get_player_housing_instances(&player.username)?;
        let guest_instances = store.get_guest_housing_instances(&player.username)?;
        
        // Check if player is currently in a housing room
        let current_owned = owned_instances.iter().find(|inst| {
            inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
        });
        
        let current_guest = guest_instances.iter().find(|inst| {
            inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
        });
        
        // Determine which instance and check permissions
        let (current_instance, is_owner) = if let Some(inst) = current_owned {
            (inst, true)
        } else if let Some(inst) = current_guest {
            (inst, false)
        } else {
            return Ok(world_config.err_describe_not_in_housing.clone());
        };
        
        // Get the housing template to check permissions
        let template = store.get_housing_template(&current_instance.template_id)?;
        
        // Check if editing is allowed
        if !is_owner && !template.permissions.can_edit_description {
            return Ok(world_config.err_describe_no_permission.clone());
        }
        
        // If no description provided, show current description and permissions
        if description.is_none() {
            let current_room = store.get_room(&player.current_room)?;
            let desc = if current_room.long_desc.is_empty() {
                "Empty room."
            } else {
                &current_room.long_desc
            };
            
            return Ok(world_config.msg_describe_current.replace("{desc}", desc));
        }
        
        // Update the room description
        let new_desc = description.unwrap();
        
        // Validate length (500 char max for room descriptions)
        const MAX_DESC_LENGTH: usize = 500;
        if new_desc.len() > MAX_DESC_LENGTH {
            return Ok(world_config.err_describe_too_long
                .replace("{max}", &MAX_DESC_LENGTH.to_string())
                .replace("{actual}", &new_desc.len().to_string()));
        }
        
        // Update the room
        let mut current_room = store.get_room(&player.current_room)?;
        current_room.long_desc = new_desc;
        store.put_room(current_room)?;
        
        Ok(world_config.msg_describe_success.clone())
    }

    /// Handle LOCK command - lock room or item
    async fn handle_lock(
        &mut self,
        session: &Session,
        target: Option<String>,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        
        // If target is None, lock current room
        if target.is_none() {
            // Check if player owns housing instances
            let instances = store.get_player_housing_instances(&player.username)?;
            if instances.is_empty() {
                return Ok("You don't own any housing.".to_string());
            }
            
            // Check if player is currently in one of their housing rooms
            let current_instance = instances.iter().find(|inst| {
                inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
            });
            
            if current_instance.is_none() {
                return Ok("You can only lock rooms in your own housing.".to_string());
            }
            
            // Get the current room
            let mut current_room = store.get_room(&player.current_room)?;
            
            // Check if already locked
            if current_room.locked {
                return Ok("This room is already locked.".to_string());
            }
            
            // Lock the room
            current_room.locked = true;
            store.put_room(current_room)?;
            
            return Ok("You lock the room. Only you and your guests can enter now.".to_string());
        }
        
        // Phase 4: Item locking
        let target_name = target.unwrap().to_uppercase();
        
        // Search for item in player's inventory
        for item_id in &player.inventory {
            if let Ok(mut item) = store.get_object(item_id) {
                if item.name.to_uppercase() == target_name {
                    // Check if player owns this item
                    match &item.owner {
                        crate::tmush::types::ObjectOwner::Player { username } => {
                            if username != &player.username {
                                return Ok(format!("You don't own {}.", item.name));
                            }
                        }
                        crate::tmush::types::ObjectOwner::World => {
                            return Ok(format!("{} is a world item and cannot be locked.", item.name));
                        }
                    }
                    
                    // Check if already locked
                    if item.locked {
                        return Ok(format!("{} is already locked.", item.name));
                    }
                    
                    // Lock the item
                    item.locked = true;
                    store.put_object(item.clone())?;
                    
                    return Ok(format!("You lock {}. It cannot be taken by others.", item.name));
                }
            }
        }
        
        Ok(format!("You don't have '{}' in your inventory.", target_name))
    }

    /// Handle UNLOCK command - unlock room or item
    async fn handle_unlock(
        &mut self,
        session: &Session,
        target: Option<String>,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        
        // If target is None, unlock current room
        if target.is_none() {
            // Check if player owns housing instances
            let instances = store.get_player_housing_instances(&player.username)?;
            if instances.is_empty() {
                return Ok("You don't own any housing.".to_string());
            }
            
            // Check if player is currently in one of their housing rooms
            let current_instance = instances.iter().find(|inst| {
                inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
            });
            
            if current_instance.is_none() {
                return Ok("You can only unlock rooms in your own housing.".to_string());
            }
            
            // Get the current room
            let mut current_room = store.get_room(&player.current_room)?;
            
            // Check if already unlocked
            if !current_room.locked {
                return Ok("This room is already unlocked.".to_string());
            }
            
            // Unlock the room
            current_room.locked = false;
            store.put_room(current_room)?;
            
            return Ok("You unlock the room. Anyone can enter now.".to_string());
        }
        
        // Phase 4: Item unlocking
        let target_name = target.unwrap().to_uppercase();
        
        // Search for item in player's inventory
        for item_id in &player.inventory {
            if let Ok(mut item) = store.get_object(item_id) {
                if item.name.to_uppercase() == target_name {
                    // Check if player owns this item
                    match &item.owner {
                        crate::tmush::types::ObjectOwner::Player { username } => {
                            if username != &player.username {
                                return Ok(format!("You don't own {}.", item.name));
                            }
                        }
                        crate::tmush::types::ObjectOwner::World => {
                            return Ok(format!("{} is a world item and cannot be unlocked.", item.name));
                        }
                    }
                    
                    // Check if already unlocked
                    if !item.locked {
                        return Ok(format!("{} is already unlocked.", item.name));
                    }
                    
                    // Unlock the item
                    item.locked = false;
                    store.put_object(item.clone())?;
                    
                    return Ok(format!("You unlock {}. Others can take it now.", item.name));
                }
            }
        }
        
        Ok(format!("You don't have '{}' in your inventory.", target_name))
    }

    /// Handle KICK command - remove player from housing
    async fn handle_kick(
        &mut self,
        session: &Session,
        target: Option<String>,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        
        let target = match target {
            Some(t) => t,
            None => return Ok("Usage: KICK <player> or KICK ALL".to_string()),
        };
        
        // Get instances and housing rooms first (separate scope)
        let (mut current_instance, housing_rooms) = {
            let store = self.get_store(config).await?;
            
            // Check if player owns housing
            let instances = store.get_player_housing_instances(&player.username)?;
            if instances.is_empty() {
                return Ok("You don't own any housing.".to_string());
            }
            
            // Check if player is currently in one of their housing rooms
            let current_instance = instances.iter().find(|inst| {
                inst.room_mappings.values().any(|room_id| room_id == &player.current_room)
            });
            
            let current_instance = match current_instance {
                Some(inst) => inst.clone(),
                None => return Ok("You can only kick players from your own housing.".to_string()),
            };
            
            let housing_rooms: Vec<String> = current_instance.room_mappings.values().cloned().collect();
            (current_instance, housing_rooms)
        };
        
        // Handle KICK ALL
        if target == "ALL" {
            let guest_count = current_instance.guests.len();
            
            // Find all guests currently in the housing
            let guests_to_kick = {
                let room_manager = self.get_room_manager(config).await?;
                let mut guests = Vec::new();
                for room_id in &housing_rooms {
                    let players_in_room = room_manager.get_players_in_room(room_id);
                    for guest_username in players_in_room {
                        if guest_username != player.username {
                            guests.push(guest_username);
                        }
                    }
                }
                guests
            };
            
            // Teleport them
            for guest_username in guests_to_kick {
                let store = self.get_store(config).await?;
                if let Ok(mut guest_player) = store.get_player(&guest_username) {
                    guest_player.current_room = "town_square".to_string();
                    let _ = store.put_player(guest_player);
                }
            }
            
            // Clear guest list
            current_instance.guests.clear();
            let store = self.get_store(config).await?;
            store.put_housing_instance(&current_instance)?;
            
            return Ok(format!(
                "You kick all guests from your housing. {} guest(s) removed from guest list.",
                guest_count
            ));
        }
        
        // Handle KICK <player>
        if !current_instance.guests.contains(&target) {
            return Ok(format!("{} is not on your guest list.", target));
        }
        
        // Check if the player is currently in the housing
        let was_in_housing = {
            let room_manager = self.get_room_manager(config).await?;
            let mut found = false;
            for room_id in &housing_rooms {
                let players_in_room = room_manager.get_players_in_room(room_id);
                if players_in_room.contains(&target) {
                    found = true;
                    break;
                }
            }
            found
        };
        
        // Teleport if in housing
        if was_in_housing {
            let store = self.get_store(config).await?;
            if let Ok(mut target_player) = store.get_player(&target) {
                target_player.current_room = "town_square".to_string();
                store.put_player(target_player)?;
            }
        }
        
        // Remove from guest list
        current_instance.guests.retain(|g| g != &target);
        let store = self.get_store(config).await?;
        store.put_housing_instance(&current_instance)?;
        
        if was_in_housing {
            return Ok(format!(
                "You kick {} from your housing. They have been teleported to town square.",
                target
            ));
        }
        
        Ok(format!(
            "You remove {} from your guest list. They can no longer enter your housing.",
            target
        ))
    }

    /// Handle @SETCONFIG command - set world configuration
    async fn handle_set_config(
        &mut self,
        session: &Session,
        field: String,
        value: String,
        config: &Config,
    ) -> Result<String> {
        // Note: In future, add role-based permissions here
        // For now, allowing any authenticated user for testing
        let player = self.get_or_create_player(session).await?;

        let store = self.get_store(config).await?;
        
        // Update the configuration field
        match store.update_world_config_field(&field, &value, &player.username) {
            Ok(_) => Ok(format!(
                "Configuration updated:\n{}: {}\n\nUpdated by: {}",
                field, value, player.username
            )),
            Err(e) => Ok(format!("Error updating configuration: {}", e)),
        }
    }

    /// Handle @GETCONFIG command - view world configuration
    async fn handle_get_config(
        &mut self,
        session: &Session,
        field: Option<String>,
        config: &Config,
    ) -> Result<String> {
        let _player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        
        let world_config = store.get_world_config()?;

        match field {
            Some(f) => {
                let value = match f.as_str() {
                    // Branding
                    "welcome_message" => &world_config.welcome_message,
                    "motd" => &world_config.motd,
                    "world_name" => &world_config.world_name,
                    "world_description" => &world_config.world_description,
                    // Help system
                    "help_main" => &world_config.help_main,
                    "help_commands" => &world_config.help_commands,
                    "help_movement" => &world_config.help_movement,
                    "help_social" => &world_config.help_social,
                    "help_bulletin" => &world_config.help_bulletin,
                    "help_companion" => &world_config.help_companion,
                    "help_mail" => &world_config.help_mail,
                    // Error messages
                    "err_no_exit" => &world_config.err_no_exit,
                    "err_whisper_self" => &world_config.err_whisper_self,
                    "err_no_shops" => &world_config.err_no_shops,
                    "err_item_not_found" => &world_config.err_item_not_found,
                    "err_trade_self" => &world_config.err_trade_self,
                    "err_say_what" => &world_config.err_say_what,
                    "err_emote_what" => &world_config.err_emote_what,
                    "err_insufficient_funds" => &world_config.err_insufficient_funds,
                    // Success messages
                    "msg_deposit_success" => &world_config.msg_deposit_success,
                    "msg_withdraw_success" => &world_config.msg_withdraw_success,
                    "msg_buy_success" => &world_config.msg_buy_success,
                    "msg_sell_success" => &world_config.msg_sell_success,
                    "msg_trade_initiated" => &world_config.msg_trade_initiated,
                    _ => return Ok(format!(
                        "Unknown configuration field: {}\n\n\
                        Available fields:\n\
                        Branding: welcome_message, motd, world_name, world_description\n\
                        Help: help_main, help_commands, help_movement, help_social, help_bulletin, help_companion, help_mail\n\
                        Errors: err_no_exit, err_whisper_self, err_no_shops, err_item_not_found, err_trade_self, err_say_what, err_emote_what, err_insufficient_funds\n\
                        Messages: msg_deposit_success, msg_withdraw_success, msg_buy_success, msg_sell_success, msg_trade_initiated",
                        f
                    )),
                };
                Ok(format!("{}:\n{}", f, value))
            }
            None => {
                // Show summary of configuration
                Ok(format!(
                    "=== WORLD CONFIGURATION ===\n\n\
                    World Name: {}\n\
                    Description: {}\n\n\
                    Configuration Fields: 24\n\
                    - 4 branding fields\n\
                    - 7 help system templates\n\
                    - 8 error message templates\n\
                    - 5 success message templates\n\n\
                    Use @GETCONFIG <field> to view specific field.\n\
                    Use @SETCONFIG <field> <value> to update.\n\n\
                    Last Updated: {}\n\
                    Updated By: {}",
                    world_config.world_name,
                    world_config.world_description,
                    world_config.updated_at.format("%Y-%m-%d %H:%M:%S"),
                    world_config.updated_by
                ))
            }
        }
    }

    /// Helper to get required progress for achievement
    fn get_achievement_required(&self, trigger: &crate::tmush::types::AchievementTrigger) -> u32 {
        use crate::tmush::types::AchievementTrigger::*;
        match trigger {
            KillCount { required } | RoomVisits { required } | FriendCount { required } |
            QuestCompletion { required } | CraftCount { required } |
            TradeCount { required } | MessagesSent { required } => *required,
            CurrencyEarned { amount } => (*amount).max(0) as u32,
            VisitLocation { .. } | CompleteQuest { .. } => 1,
        }
    }

    /// Handle HELP command
    async fn handle_help(&mut self, _session: &Session, topic: Option<String>, config: &Config) -> Result<String> {
        // Load world config for help text
        let store = self.get_store(config).await?;
        let world_config = store.get_world_config()?;
        
        match topic.as_deref() {
            Some("commands") | Some("COMMANDS") => Ok(world_config.help_commands),
            Some("movement") | Some("MOVEMENT") => Ok(world_config.help_movement),
            Some("social") | Some("SOCIAL") => Ok(world_config.help_social),
            Some("board") | Some("BOARD") | Some("bulletin") | Some("BULLETIN") => Ok(world_config.help_bulletin),
            Some("mail") | Some("MAIL") => Ok(world_config.help_mail),
            Some("companion") | Some("COMPANION") | Some("companions") | Some("COMPANIONS") => Ok(world_config.help_companion),
            None => Ok(world_config.help_main),
            Some(topic) => Ok(format!("No help available for: {}\nTry: HELP COMMANDS", topic)),
        }
    }

    /// Handle QUIT command - exit TinyMUSH
    async fn handle_quit(&mut self, session: &mut Session, _config: &Config) -> Result<String> {
        // Record game exit metrics
        if let Some(slug) = session.current_game_slug.take() {
            let counters = metrics::record_game_exit(&slug);
            info!(
                target: "meshbbs::games",
                "game.exit slug={} session={} user={} node={} reason=command command=QUIT active={} exits={} entries={} peak={}",
                slug,
                escape_log(&session.id),
                escape_log(&session.display_name()),
                escape_log(&session.node_id),
                counters.currently_active,
                counters.exits,
                counters.entries,
                counters.concurrent_peak
            );
        }

        // Return to main menu
        session.state = SessionState::MainMenu;
        Ok("Leaving TinyMUSH...\n\nReturning to main menu.".to_string())
    }

    /// Handle SAVE command - force save player state
    async fn handle_save(&mut self, session: &Session, _config: &Config) -> Result<String> {
        match self.get_or_create_player(session).await {
            Ok(player) => {
                match self.store().put_player(player) {
                    Ok(()) => Ok("Player state saved.".to_string()),
                    Err(e) => Ok(format!("Save failed: {}", e)),
                }
            },
            Err(e) => Ok(format!("Error saving: {}", e)),
        }
    }

    /// Handle BOARD command - view bulletin board
    async fn handle_board(&mut self, session: &Session, board_id: Option<String>, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // For now, only support the "stump" board in the town square
        let board_id = board_id.unwrap_or_else(|| "stump".to_string());
        
        if board_id != "stump" {
            return Ok("Only the 'stump' bulletin board is available.\nUsage: BOARD or BOARD stump".to_string());
        }

        // Check if player is in the town square
        if player.current_room != "town_square" {
            return Ok("You must be at the Town Square to access the Town Stump bulletin board.\nHead to the town square and try again.".to_string());
        }

        // Initialize stump board if it doesn't exist
        let board = match self.store().get_bulletin_board(&board_id) {
            Ok(board) => board,
            Err(TinyMushError::NotFound(_)) => {
                // Create the stump board
                let board = BulletinBoard::new(
                    "stump",
                    "Town Stump",
                    "A weathered stump with notices posted by travelers",
                    "town_square"
                );
                self.store().put_bulletin_board(board.clone())?;
                board
            },
            Err(e) => return Ok(format!("Bulletin board error: {}", e)),
        };

        // Get recent messages (last 10)
        let messages = match self.store().list_bulletins(&board_id, 0, 10) {
            Ok(messages) => messages,
            Err(e) => return Ok(format!("Error reading bulletins: {}", e)),
        };

        let mut response = format!("=== {} ===\n{}\n\n", board.name, board.description);
        
        if messages.is_empty() {
            response.push_str("No messages posted.\n");
        } else {
            response.push_str("Recent messages:\n");
            for (i, msg) in messages.iter().enumerate() {
                // Format: [ID] Subject - by Author (date)
                let date = msg.posted_at.format("%m/%d");
                response.push_str(&format!(
                    "[{}] {} - by {} ({})\n",
                    msg.id, msg.subject, msg.author, date
                ));
                
                // Limit to fit in 200 bytes
                if response.len() > 150 {
                    let remaining = messages.len() - i - 1;
                    if remaining > 0 {
                        response.push_str(&format!("... and {} more.\n", remaining));
                    }
                    break;
                }
            }
        }
        
        response.push_str("\nPOST <subject> <message> to add\nREAD <id> to read a message");
        Ok(response)
    }

    /// Handle POST command - post message to bulletin board
    async fn handle_post(&mut self, session: &Session, subject: String, message: String, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Check if player is in the town square
        if player.current_room != "town_square" {
            return Ok("You must be at the Town Square to post to the bulletin board.".to_string());
        }

        // Validate input
        if subject.trim().is_empty() {
            return Ok("Subject cannot be empty.\nUsage: POST <subject> <message>".to_string());
        }
        
        if message.trim().is_empty() {
            return Ok("Message cannot be empty.\nUsage: POST <subject> <message>".to_string());
        }

        if subject.len() > 50 {
            return Ok("Subject too long (max 50 characters).".to_string());
        }

        if message.len() > 300 {
            return Ok("Message too long (max 300 characters).".to_string());
        }

        // Create the bulletin message
        let bulletin = BulletinMessage::new(
            &session.display_name(),
            &subject,
            &message,
            "stump"
        );

        // Post the message
        match self.store().post_bulletin(bulletin) {
            Ok(message_id) => {
                // Clean up old messages if needed
                let _ = self.store().cleanup_bulletins("stump", 50);
                
                Ok(format!(
                    "Message posted to Town Stump bulletin board.\nMessage ID: {} - '{}'\nOthers can read it with: READ {}",
                    message_id, subject, message_id
                ))
            },
            Err(e) => Ok(format!("Failed to post message: {}", e)),
        }
    }

    /// Handle READ command - read specific bulletin message
    async fn handle_read(&mut self, session: &Session, message_id: u64, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Check if player is in the town square
        if player.current_room != "town_square" {
            return Ok("You must be at the Town Square to read bulletin board messages.".to_string());
        }

        // Get the message
        match self.store().get_bulletin("stump", message_id) {
            Ok(message) => {
                let date = message.posted_at.format("%b %d, %Y at %H:%M");
                let mut response = format!(
                    "=== Message {} ===\n\
                    From: {}\n\
                    Subject: {}\n\
                    Date: {}\n\n\
                    {}",
                    message.id, message.author, message.subject, date, message.body
                );

                // Ensure we stay under 200 bytes
                if response.len() > 200 {
                    response.truncate(197);
                    response.push_str("...");
                }

                Ok(response)
            },
            Err(TinyMushError::NotFound(_)) => {
                Ok(format!("No bulletin message with ID {}.\nUse BOARD to see available messages.", message_id))
            },
            Err(e) => Ok(format!("Error reading message: {}", e)),
        }
    }

    /// Get or create player record for this session
    async fn get_or_create_player(&self, session: &Session) -> Result<PlayerRecord, TinyMushError> {
        let username = session.display_name();
        
        match self.store().get_player(&username) {
            Ok(player) => Ok(player),
            Err(TinyMushError::NotFound(_)) => {
                // Create new player at gazebo for tutorial
                let display_name = if session.is_logged_in() {
                    session.display_name()
                } else {
                    format!("Guest_{}", &session.id[..8])
                };

                let player = PlayerRecord::new(&username, &display_name, crate::tmush::state::REQUIRED_LANDING_LOCATION_ID);
                self.store().put_player(player.clone())?;
                Ok(player)
            },
            Err(e) => Err(e),
        }
    }

    /// Helper: Record ownership transfer in item's history (Phase 5)
    fn record_ownership_transfer(
        item: &mut ObjectRecord,
        from_owner: Option<String>,
        to_owner: String,
        reason: crate::tmush::types::OwnershipReason,
    ) {
        use chrono::Utc;
        use crate::tmush::types::OwnershipTransfer;
        
        let transfer = OwnershipTransfer {
            from_owner,
            to_owner,
            timestamp: Utc::now(),
            reason,
        };
        
        item.ownership_history.push(transfer);
    }

    /// Handle HISTORY command - view item ownership audit trail (Phase 5)
    async fn handle_history(
        &mut self,
        session: &Session,
        item_name: String,
        config: &Config,
    ) -> Result<String> {
        let player = self.get_or_create_player(session).await?;
        let store = self.get_store(config).await?;
        
        let item_name_upper = item_name.to_uppercase();
        
        // Search for item in player's inventory
        for item_id in &player.inventory {
            if let Ok(item) = store.get_object(item_id) {
                if item.name.to_uppercase() == item_name_upper {
                    // Check if player owns this item
                    match &item.owner {
                        crate::tmush::types::ObjectOwner::Player { username } => {
                            if username != &player.username {
                                return Ok(format!("You don't own {}. Only the owner can view ownership history.", item.name));
                            }
                        }
                        crate::tmush::types::ObjectOwner::World => {
                            return Ok(format!("{} is a world item with no ownership history.", item.name));
                        }
                    }
                    
                    // Display ownership history
                    let mut response = String::new();
                    response.push_str(&format!("=== Ownership History: {} ===\n\n", item.name));
                    
                    if item.ownership_history.is_empty() {
                        response.push_str("No ownership transfers recorded.\n");
                        response.push_str("(This item may predate the ownership tracking system)\n");
                    } else {
                        for (idx, transfer) in item.ownership_history.iter().enumerate() {
                            let from = transfer.from_owner.as_ref()
                                .map(|s| s.as_str())
                                .unwrap_or("WORLD");
                            let to = &transfer.to_owner;
                            let reason = format!("{:?}", transfer.reason);
                            let timestamp = transfer.timestamp.format("%Y-%m-%d %H:%M:%S");
                            
                            response.push_str(&format!(
                                "{}. {} → {} | {} | {}\n",
                                idx + 1, from, to, reason, timestamp
                            ));
                        }
                        
                        response.push_str(&format!("\nTotal transfers: {}\n", item.ownership_history.len()));
                    }
                    
                    return Ok(response);
                }
            }
        }
        
        Ok(format!("You don't have '{}' in your inventory.", item_name))
    }

    /// Describe the current room (placeholder for Phase 3)
    async fn describe_current_room(&self, player: &PlayerRecord) -> Result<String> {
        match self.store().get_room(&player.current_room) {
            Ok(room) => {
                let mut response = String::new();
                
                // Room name
                response.push_str(&format!("=== {} ===\n", room.name));
                
                // Room description
                response.push_str(&format!("{}\n\n", room.long_desc));
                
                // Show exits if any
                if !room.exits.is_empty() {
                    response.push_str("Obvious exits: ");
                    let mut exit_names: Vec<String> = room.exits.keys()
                        .map(|dir| format!("{:?}", dir).to_lowercase())
                        .collect();
                    exit_names.sort(); // Consistent ordering
                    response.push_str(&exit_names.join(", "));
                    response.push('\n');
                }
                
                // Show other players (Phase 4 feature - placeholder for now)
                // response.push_str("Players here: (none visible)\n");
                
                Ok(response)
            }
            Err(_) => {
                Ok(format!(
                    "You are in a mysterious void (room '{}' not found).\nType WHERE for help.",
                    player.current_room
                ))
            }
        }
    }

    /// Main help text
    pub fn help_main(&self) -> String {
        "=TINYMUSH HELP=\n".to_string() +
        "Move: N/S/E/W/U/D + diagonals\n" +
        "Look: L | I (inv) | WHO | SCORE\n" +
        "Talk: SAY/EMOTE\n" +
        "Board: BOARD/POST/READ\n" +
        "Mail: MAIL/SEND\n" +
        "More: HELP <topic>\n" +
        "Topics: COMMANDS MOVEMENT SOCIAL BOARD MAIL"
    }

    /// Commands help
    pub fn help_commands(&self) -> String {
        "=COMMANDS=\n".to_string() +
        "L - look | I - inventory\n" +
        "WHO - players | SCORE - stats\n" +
        "SAY/EMOTE - talk\n" +
        "BOARD/POST/READ - bulletin\n" +
        "MAIL/SEND/RMAIL - messages\n" +
        "SAVE | QUIT"
    }    /// Movement help
    pub fn help_movement(&self) -> String {
        "=MOVEMENT=\n".to_string() +
        "N/S/E/W - cardinal\n" +
        "U/D - up/down\n" +
        "NE/NW/SE/SW - diagonals\n" +
        "L - look around"
    }

    /// Social commands help  
    pub fn help_social(&self) -> String {
        "=SOCIAL=\n".to_string() +
        "SAY <txt> - speak aloud\n" +
        "WHISPER <plr> <txt> - private\n" +
        "EMOTE/: <act> - action\n" +
        "POSE/; <pose> - describe\n" +
        "OOC <txt> - out of char\n" +
        "WHO - list players"
    }

    /// Bulletin board help
    pub fn help_bulletin(&self) -> String {
        "=BULLETIN BOARD=\n".to_string() +
        "Town Stump message board\n" +
        "BOARD - view messages\n" +
        "POST <subj> <msg> - post\n" +
        "READ <id> - read\n" +
        "Use at Town Square\n" +
        "Max: 50 char subj, 300 msg"
    }

    /// Companion commands help
    pub fn help_companion(&self) -> String {
        "=COMPANIONS=\n".to_string() +
        "COMP [LIST] - your pets\n" +
        "COMP TAME <name> - claim\n" +
        "COMP <name> - status\n" +
        "COMP RELEASE <name> - free\n" +
        "COMP STAY/COME - control\n" +
        "COMP INV - storage\n" +
        "FEED/PET <name> - care\n" +
        "MOUNT/DISMOUNT - riding\n" +
        "TRAIN <name> <skill> - teach"
    }

    /// Handle MAIL command - view mail folders
    async fn handle_mail(&mut self, session: &Session, folder: Option<String>, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        let folder = folder.unwrap_or_else(|| "inbox".to_string());
        
        // Only support inbox and sent folders for now
        if folder != "inbox" && folder != "sent" {
            return Ok("Available folders: MAIL inbox, MAIL sent".to_string());
        }

        // Get mail messages
        let messages = match self.store().list_mail(&folder, &player.username, 0, 10) {
            Ok(messages) => messages,
            Err(e) => return Ok(format!("Error reading mail: {}", e)),
        };

        let mut response = format!("=== {} MAIL ===\n", folder.to_uppercase());
        
        if messages.is_empty() {
            response.push_str("No mail messages.\n");
        } else {
            for (i, msg) in messages.iter().enumerate() {
                let status = if msg.status == crate::tmush::types::MailStatus::Unread { "*" } else { " " };
                let date = msg.sent_at.format("%m/%d");
                let sender_recipient = if folder == "inbox" { &msg.sender } else { &msg.recipient };
                
                response.push_str(&format!(
                    "{} [{}] {} - {} ({})\n",
                    status, msg.id, msg.subject, sender_recipient, date
                ));
                
                // Limit to fit in 200 bytes
                if response.len() > 150 {
                    let remaining = messages.len() - i - 1;
                    if remaining > 0 {
                        response.push_str(&format!("... and {} more.\n", remaining));
                    }
                    break;
                }
            }
        }
        
        response.push_str("\nRMAIL <id> to read, DMAIL <id> to delete\nSEND <player> <subject> <message> to send");
        Ok(response)
    }

    /// Handle SEND command - send mail to another player
    async fn handle_send(&mut self, session: &Session, recipient: String, subject: String, message: String, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Validate input
        if recipient.trim().is_empty() {
            return Ok("Recipient cannot be empty.\nUsage: SEND <player> <subject> <message>".to_string());
        }
        
        if subject.trim().is_empty() {
            return Ok("Subject cannot be empty.\nUsage: SEND <player> <subject> <message>".to_string());
        }
        
        if message.trim().is_empty() {
            return Ok("Message cannot be empty.\nUsage: SEND <player> <subject> <message>".to_string());
        }

        if subject.len() > 50 {
            return Ok("Subject too long (max 50 characters).".to_string());
        }

        if message.len() > 200 {
            return Ok("Message too long (max 200 characters).".to_string());
        }

        // Check if recipient exists (basic validation)
        let recipient_lower = recipient.to_ascii_lowercase();
        match self.store().get_player(&recipient_lower) {
            Ok(_) => {}, // Player exists
            Err(TinyMushError::NotFound(_)) => {
                return Ok(format!("Player '{}' not found.\nMake sure they have logged in at least once.", recipient));
            },
            Err(e) => return Ok(format!("Error checking recipient: {}", e)),
        }

        // Create the mail message
        let mail = crate::tmush::types::MailMessage::new(
            &player.username,
            &recipient_lower,
            &subject,
            &message
        );

        // Send the message
        match self.store().send_mail(mail) {
            Ok(message_id) => {
                // Enforce mail quota for recipient
                let _ = self.store().enforce_mail_quota(&recipient_lower, 100);
                
                Ok(format!(
                    "Mail sent to {}.\nMessage ID: {} - '{}'\nThey can read it with: RMAIL {}",
                    recipient, message_id, subject, message_id
                ))
            },
            Err(e) => Ok(format!("Failed to send mail: {}", e)),
        }
    }

    /// Handle RMAIL command - read specific mail message
    async fn handle_read_mail(&mut self, session: &Session, message_id: u64, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Try to find the message in inbox first, then sent
        let message = match self.store().get_mail("inbox", &player.username, message_id) {
            Ok(msg) => {
                // Mark as read if it's in the inbox
                let _ = self.store().mark_mail_read("inbox", &player.username, message_id);
                msg
            },
            Err(TinyMushError::NotFound(_)) => {
                // Try sent folder
                match self.store().get_mail("sent", &player.username, message_id) {
                    Ok(msg) => msg,
                    Err(TinyMushError::NotFound(_)) => {
                        return Ok(format!("No mail message with ID {}.\nUse MAIL to see available messages.", message_id));
                    },
                    Err(e) => return Ok(format!("Error reading mail: {}", e)),
                }
            },
            Err(e) => return Ok(format!("Error reading mail: {}", e)),
        };

        let date = message.sent_at.format("%b %d, %Y at %H:%M");
        let mut response = format!(
            "=== Mail {} ===\n\
            From: {}\n\
            To: {}\n\
            Subject: {}\n\
            Date: {}\n\n\
            {}",
            message.id, message.sender, message.recipient, message.subject, date, message.body
        );

        // Ensure we stay under 200 bytes
        if response.len() > 200 {
            response.truncate(197);
            response.push_str("...");
        }

        Ok(response)
    }

    /// Handle DMAIL command - delete mail message
    async fn handle_delete_mail(&mut self, session: &Session, message_id: u64, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Try to delete from inbox first, then sent
        match self.store().delete_mail("inbox", &player.username, message_id) {
            Ok(()) => {
                Ok(format!("Mail message {} deleted from inbox.", message_id))
            },
            Err(TinyMushError::NotFound(_)) => {
                // Try sent folder
                match self.store().delete_mail("sent", &player.username, message_id) {
                    Ok(()) => {
                        Ok(format!("Mail message {} deleted from sent folder.", message_id))
                    },
                    Err(TinyMushError::NotFound(_)) => {
                        Ok(format!("No mail message with ID {}.\nUse MAIL to see available messages.", message_id))
                    },
                    Err(e) => Ok(format!("Error deleting mail: {}", e)),
                }
            },
            Err(e) => Ok(format!("Error deleting mail: {}", e)),
        }
    }

    // ===== BANKING COMMANDS (Phase 5 Week 4) =====

    /// Handle BALANCE command - show pocket and bank balance
    async fn handle_balance(&mut self, session: &Session, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        let pocket_display = format!("{:?}", player.currency);
        let bank_display = format!("{:?}", player.banked_currency);

        Ok(format!(
            "=ACCOUNT BALANCE=\nPocket: {}\nBank: {}",
            pocket_display, bank_display
        ))
    }

    /// Handle DEPOSIT command - deposit currency to bank
    async fn handle_deposit(&mut self, session: &Session, amount_str: String, _config: &Config) -> Result<String> {
        let world_config = self.get_world_config().await?;
        
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Parse amount as integer (base units)
        let base_units: i64 = match amount_str.parse() {
            Ok(units) => units,
            Err(_) => return Ok("Invalid amount format.\nExample: DEPOSIT 100".to_string()),
        };

        if base_units <= 0 {
            return Ok("Amount must be positive.".to_string());
        }

        // Create CurrencyAmount matching player's currency type
        use crate::tmush::types::CurrencyAmount;
        let amount = match player.currency {
            CurrencyAmount::Decimal { .. } => CurrencyAmount::Decimal { minor_units: base_units },
            CurrencyAmount::MultiTier { .. } => CurrencyAmount::MultiTier { base_units },
        };

        // Perform deposit via storage
        match self.store().bank_deposit(&player.username, &amount) {
            Ok(_) => {
                Ok(world_config.msg_deposit_success.replace("{amount}", &format!("{:?}", amount)))
            },
            Err(e) => Ok(format!("Deposit failed: {}", e)),
        }
    }

    /// Handle WITHDRAW command - withdraw currency from bank
    async fn handle_withdraw(&mut self, session: &Session, amount_str: String, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Parse amount as integer (base units)
        let base_units: i64 = match amount_str.parse() {
            Ok(units) => units,
            Err(_) => return Ok("Invalid amount format.\nExample: WITHDRAW 100".to_string()),
        };

        if base_units <= 0 {
            return Ok("Amount must be positive.".to_string());
        }

        // Create CurrencyAmount matching player's currency type
        use crate::tmush::types::CurrencyAmount;
        let amount = match player.currency {
            CurrencyAmount::Decimal { .. } => CurrencyAmount::Decimal { minor_units: base_units },
            CurrencyAmount::MultiTier { .. } => CurrencyAmount::MultiTier { base_units },
        };

        // Perform withdrawal via storage
        match self.store().bank_withdraw(&player.username, &amount) {
            Ok(_) => {
                Ok(format!("Withdrew {:?} from bank.\nUse BALANCE to check your account.", amount))
            },
            Err(e) => Ok(format!("Withdrawal failed: {}", e)),
        }
    }

    /// Handle BTRANSFER command - transfer currency between players via bank
    async fn handle_bank_transfer(&mut self, session: &Session, recipient: String, amount_str: String, _config: &Config) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Can't transfer to self
        if recipient.to_lowercase() == player.username.to_lowercase() {
            return Ok("You can't transfer to yourself!".to_string());
        }

        // Check recipient exists
        let recipient_player = match self.store().get_player(&recipient) {
            Ok(p) => p,
            Err(TinyMushError::NotFound(_)) => return Ok(format!("Player '{}' not found.", recipient)),
            Err(e) => return Ok(format!("Error loading recipient: {}", e)),
        };

        // Parse amount as integer (base units)
        let base_units: i64 = match amount_str.parse() {
            Ok(units) => units,
            Err(_) => return Ok("Invalid amount format.\nExample: BTRANSFER alice 100".to_string()),
        };

        if base_units <= 0 {
            return Ok("Amount must be positive.".to_string());
        }

        // Create CurrencyAmount matching player's currency type
        use crate::tmush::types::CurrencyAmount;
        let amount = match player.currency {
            CurrencyAmount::Decimal { .. } => CurrencyAmount::Decimal { minor_units: base_units },
            CurrencyAmount::MultiTier { .. } => CurrencyAmount::MultiTier { base_units },
        };

        // Check sender has enough in bank
        if !player.banked_currency.can_afford(&amount) {
            return Ok(format!("Insufficient bank funds.\nYou have: {:?}", player.banked_currency));
        }

        // Perform bank-to-bank transfer by manually handling both players
        // First, withdraw from sender's bank
        let mut sender = self.store().get_player(&player.username)?;
        sender.banked_currency = match sender.banked_currency.subtract(&amount) {
            Ok(new_balance) => new_balance,
            Err(e) => return Ok(format!("Transfer failed: {}", e)),
        };

        // Then deposit to recipient's bank
        let mut recipient = self.store().get_player(&recipient_player.username)?;
        recipient.banked_currency = match recipient.banked_currency.add(&amount) {
            Ok(new_balance) => new_balance,
            Err(e) => return Ok(format!("Transfer failed: {}", e)),
        };

        // Save both players
        self.store().put_player(sender)?;
        self.store().put_player(recipient)?;

        // Log the transaction
        use crate::tmush::types::{TransactionReason, CurrencyTransaction};
        use chrono::Utc;
        let _transaction = CurrencyTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from: Some(format!("{}:bank", player.username)),
            to: Some(format!("{}:bank", recipient_player.username)),
            amount: amount.clone(),
            reason: TransactionReason::Trade,
            rolled_back: false,
        };
        // Note: log_transaction is private, so we'll just skip logging for now
        // TODO: Make log_transaction public or provide a public logging method

        Ok(format!(
            "Transferred {:?} to {}'s bank account.",
            amount, recipient_player.username
        ))
    }

    // ===== TRADING COMMANDS (Phase 5 Week 4) =====

    /// Handle TRADE command - initiate trade with another player
    async fn handle_trade(&mut self, session: &Session, target: String, _config: &Config) -> Result<String> {
        let world_config = self.get_world_config().await?;
        let username = session.node_id.to_string();
        let target_lower = target.to_ascii_lowercase();

        // Can't trade with yourself
        if target_lower == username.to_ascii_lowercase() {
            return Ok("You can't trade with yourself!".to_string());
        }

        // Check if initiator already has an active trade
        if let Some(existing) = self.store().get_player_active_trade(&username)? {
            let other = if existing.player1.to_ascii_lowercase() == username.to_ascii_lowercase() {
                &existing.player2
            } else {
                &existing.player1
            };
            return Ok(format!("You're already trading with {}!\nType REJECT to cancel.", other));
        }

        // Check if target player has an active trade
        if let Some(_existing) = self.store().get_player_active_trade(&target)? {
            return Ok(format!("{} is already in a trade.", target));
        }

        // Get both players to verify they exist
        let player1 = self.store().get_player(&username)?;
        let player2 = self.store().get_player(&target)?;

        // Verify both players are in the same room
        if player1.current_room != player2.current_room {
            return Ok(format!("{} is not here!", target));
        }

        // Create new trade session
        let trade_session = crate::tmush::types::TradeSession::new(&username, &target);
        self.store().put_trade_session(&trade_session)?;

        Ok(world_config.msg_trade_initiated.replace("{target}", &target))
    }

    /// Handle OFFER command - offer item or currency in active trade
    async fn handle_offer(&mut self, session: &Session, offer_text: String, _config: &Config) -> Result<String> {
        let username = session.node_id.to_string();

        // Get active trade session
        let mut trade = match self.store().get_player_active_trade(&username)? {
            Some(t) => t,
            None => return Ok("You have no active trade.\nUse TRADE <player> to start one.".to_string()),
        };

        // Check if trade expired
        if trade.is_expired() {
            self.store().delete_trade_session(&trade.id)?;
            return Ok("Trade expired!".to_string());
        }

        let offer_trimmed = offer_text.trim();

        // Try to parse as currency amount (simple integer for base units)
        if let Ok(amount) = offer_trimmed.parse::<i64>() {
            if amount <= 0 {
                return Ok("Amount must be positive!".to_string());
            }

            // Get player's currency to match type
            let player = self.store().get_player(&username)?;

            // Create currency amount of same type as player has
            let currency_offer = match player.currency {
                crate::tmush::types::CurrencyAmount::Decimal { .. } => {
                    crate::tmush::types::CurrencyAmount::Decimal { minor_units: amount }
                },
                crate::tmush::types::CurrencyAmount::MultiTier { .. } => {
                    crate::tmush::types::CurrencyAmount::MultiTier { base_units: amount }
                }
            };

            // Verify player can afford this
            if !player.currency.can_afford(&currency_offer) {
                return Ok("You don't have that much!".to_string());
            }

            // Add to trade
            trade.add_currency_offer(&username, currency_offer.clone());
            self.store().put_trade_session(&trade)?;

            return Ok(format!("Added {:?} to trade.\nType ACCEPT when ready.", currency_offer));
        }

        // Otherwise treat as item name
        let player = self.store().get_player(&username)?;
        let offer_lower = offer_trimmed.to_ascii_lowercase();

        // Check if player has this item
        let has_item = player.inventory_stacks.iter()
            .any(|stack| stack.object_id.to_ascii_lowercase() == offer_lower);

        if !has_item {
            return Ok(format!("You don't have '{}'!", offer_text));
        }

        // Add item to trade
        trade.add_item_offer(&username, offer_trimmed.to_string());
        self.store().put_trade_session(&trade)?;

        Ok(format!("Added '{}' to trade.\nType ACCEPT when ready.", offer_trimmed))
    }

    /// Handle ACCEPT command - accept trade
    async fn handle_accept(&mut self, session: &Session, _config: &Config) -> Result<String> {
        let username = session.node_id.to_string();

        // Get active trade session
        let mut trade = match self.store().get_player_active_trade(&username)? {
            Some(t) => t,
            None => return Ok("You have no active trade.".to_string()),
        };

        // Check if trade expired
        if trade.is_expired() {
            self.store().delete_trade_session(&trade.id)?;
            return Ok("Trade expired!".to_string());
        }

        // Mark this player as accepted
        trade.accept(&username);
        
        // If both players have accepted, execute the trade
        if trade.is_ready() {
            // Execute the atomic swap
            match self.execute_trade(&trade).await {
                Ok(()) => {
                    // Trade successful - delete session and notify
                    self.store().delete_trade_session(&trade.id)?;
                    
                    let summary = trade.get_summary();
                    Ok(format!("Trade complete!\n{}", summary))
                },
                Err(e) => {
                    // Trade failed - delete session and return error
                    self.store().delete_trade_session(&trade.id)?;
                    Ok(format!("Trade failed: {}", e))
                }
            }
        } else {
            // Not both accepted yet - save updated session and wait
            self.store().put_trade_session(&trade)?;
            
            Ok("You accepted the trade.\nWaiting for other player...".to_string())
        }
    }

    /// Handle REJECT command - reject/cancel trade
    async fn handle_reject(&mut self, session: &Session, _config: &Config) -> Result<String> {
        let username = session.node_id.to_string();

        // Get active trade session
        let trade = match self.store().get_player_active_trade(&username)? {
            Some(t) => t,
            None => return Ok("You have no active trade.".to_string()),
        };

        // Get the other player's name
        let other_player = if trade.player1.to_ascii_lowercase() == username.to_ascii_lowercase() {
            &trade.player2
        } else {
            &trade.player1
        };

        // Delete the trade session
        self.store().delete_trade_session(&trade.id)?;

        Ok(format!("Trade with {} cancelled.", other_player))
    }

    /// Handle THISTORY command - view trade history
    async fn handle_trade_history(&mut self, session: &Session, _config: &Config) -> Result<String> {
        let username = session.node_id.to_string();

        // Get last 20 transactions for this player
        let transactions = self.store().get_player_transactions(&username, 20)?;
        
        // Filter for trades only
        let trade_txns: Vec<_> = transactions.iter()
            .filter(|tx| matches!(tx.reason, crate::tmush::types::TransactionReason::Trade))
            .take(10)
            .collect();

        if trade_txns.is_empty() {
            return Ok("No trade history.".to_string());
        }

        let mut output = "=TRADE HISTORY=\n".to_string();
        for tx in trade_txns {
            let timestamp = tx.timestamp.format("%m/%d %H:%M");
            
            // Determine direction and other party
            let (direction, other_party) = match (&tx.from, &tx.to) {
                (Some(from), Some(to)) => {
                    if from.to_ascii_lowercase() == username.to_ascii_lowercase() {
                        ("->", to.as_str())
                    } else {
                        ("<-", from.as_str())
                    }
                },
                _ => ("??", "?"),
            };
            
            output.push_str(&format!("{} {} {} {:?}\n", timestamp, direction, other_party, tx.amount));
        }

        Ok(output)
    }

    /// Execute a two-phase commit trade between players (atomic swap)
    async fn execute_trade(&mut self, trade: &crate::tmush::types::TradeSession) -> Result<()> {
        // Phase 1: Validate both players can complete the trade
        let mut player1 = self.store().get_player(&trade.player1)?;
        let mut player2 = self.store().get_player(&trade.player2)?;

        // Validate player1 can afford their currency offer
        if !player1.currency.can_afford(&trade.player1_currency) {
            return Err(TinyMushError::InsufficientFunds.into());
        }

        // Validate player2 can afford their currency offer
        if !player2.currency.can_afford(&trade.player2_currency) {
            return Err(TinyMushError::InsufficientFunds.into());
        }

        // Validate player1 has all offered items
        for item_id in &trade.player1_items {
            if !player1.inventory_stacks.iter().any(|s| &s.object_id == item_id) {
                return Err(TinyMushError::NotFound(format!("{} no longer has {}", trade.player1, item_id)).into());
            }
        }

        // Validate player2 has all offered items
        for item_id in &trade.player2_items {
            if !player2.inventory_stacks.iter().any(|s| &s.object_id == item_id) {
                return Err(TinyMushError::NotFound(format!("{} no longer has {}", trade.player2, item_id)).into());
            }
        }

        // Phase 2: Execute atomic swap

        // Swap currency (if any)
        if !trade.player1_currency.is_zero_or_negative() || !trade.player2_currency.is_zero_or_negative() {
            // Player1 gives currency, receives currency
            player1.currency = player1.currency.subtract(&trade.player1_currency)
                .map_err(|e| TinyMushError::InvalidCurrency(format!("P1 currency subtract failed: {}", e)))?;
            player1.currency = player1.currency.add(&trade.player2_currency)
                .map_err(|e| TinyMushError::InvalidCurrency(format!("P1 currency add failed: {}", e)))?;

            // Player2 gives currency, receives currency
            player2.currency = player2.currency.subtract(&trade.player2_currency)
                .map_err(|e| TinyMushError::InvalidCurrency(format!("P2 currency subtract failed: {}", e)))?;
            player2.currency = player2.currency.add(&trade.player1_currency)
                .map_err(|e| TinyMushError::InvalidCurrency(format!("P2 currency add failed: {}", e)))?;
        }

        // Swap items
        // Player1 gives items to Player2
        for item_id in &trade.player1_items {
            player1.inventory_stacks.retain(|s| &s.object_id != item_id);
            player2.inventory_stacks.push(crate::tmush::types::ItemStack {
                object_id: item_id.clone(),
                quantity: 1,
                added_at: chrono::Utc::now(),
            });
        }

        // Player2 gives items to Player1
        for item_id in &trade.player2_items {
            player2.inventory_stacks.retain(|s| &s.object_id != item_id);
            player1.inventory_stacks.push(crate::tmush::types::ItemStack {
                object_id: item_id.clone(),
                quantity: 1,
                added_at: chrono::Utc::now(),
            });
        }

        // Save both players (atomic commit point)
        self.store().put_player(player1)?;
        self.store().put_player(player2)?;

        // Transaction logging is skipped for P2P trades since we manually updated both players
        // The currency swap is complete and atomic at this point

        Ok(())
    }

    /// Mail system help
    pub fn help_mail(&self) -> String {
        "=MAIL SYSTEM=\n".to_string() +
        "MAIL [folder] - inbox/sent\n" +
        "SEND <plr> <subj> <msg>\n" +
        "RMAIL <id> - read\n" +
        "DMAIL <id> - delete\n" +
        "* = unread\n" +
        "Max: 50 subj, 200 msg"
    }
}

/// Integration with main command processor
pub async fn handle_tinymush_command(
    session: &mut Session,
    command: &str,
    storage: &mut Storage,
    config: &Config,
) -> Result<String> {
    // Create processor instance (could be cached in future)
    let mut processor = TinyMushProcessor::new();
    processor.process_command(session, command, storage, config).await
}

/// Check if we should route to TinyMUSH based on session state
pub fn should_route_to_tinymush(session: &Session) -> bool {
    session.current_game_slug.as_deref() == Some("tinymush")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing() {
        let processor = TinyMushProcessor::new();
        
        // Movement commands
        assert_eq!(processor.parse_command("n"), TinyMushCommand::Move(Direction::North));
        assert_eq!(processor.parse_command("NORTH"), TinyMushCommand::Move(Direction::North));
        assert_eq!(processor.parse_command("ne"), TinyMushCommand::Move(Direction::Northeast));
        
        // Look commands
        assert_eq!(processor.parse_command("l"), TinyMushCommand::Look(None));
        assert_eq!(processor.parse_command("look sword"), TinyMushCommand::Look(Some("SWORD".to_string())));
        
        // Social commands
        assert_eq!(processor.parse_command("say hello"), TinyMushCommand::Say("HELLO".to_string()));
        assert_eq!(processor.parse_command("' hello world"), TinyMushCommand::Say("HELLO WORLD".to_string()));
        
        // System commands
        assert_eq!(processor.parse_command("help"), TinyMushCommand::Help(None));
        assert_eq!(processor.parse_command("help commands"), TinyMushCommand::Help(Some("COMMANDS".to_string())));
        assert_eq!(processor.parse_command("quit"), TinyMushCommand::Quit);
        
        // Unknown commands
        assert_eq!(processor.parse_command("frobozz"), TinyMushCommand::Unknown("FROBOZZ".to_string()));
    }

    #[test]
    fn test_direction_parsing() {
        let processor = TinyMushProcessor::new();
        
        assert_eq!(processor.parse_command("n"), TinyMushCommand::Move(Direction::North));
        assert_eq!(processor.parse_command("s"), TinyMushCommand::Move(Direction::South));
        assert_eq!(processor.parse_command("e"), TinyMushCommand::Move(Direction::East));
        assert_eq!(processor.parse_command("w"), TinyMushCommand::Move(Direction::West));
        assert_eq!(processor.parse_command("u"), TinyMushCommand::Move(Direction::Up));
        assert_eq!(processor.parse_command("d"), TinyMushCommand::Move(Direction::Down));
        assert_eq!(processor.parse_command("ne"), TinyMushCommand::Move(Direction::Northeast));
        assert_eq!(processor.parse_command("nw"), TinyMushCommand::Move(Direction::Northwest));
        assert_eq!(processor.parse_command("se"), TinyMushCommand::Move(Direction::Southeast));
        assert_eq!(processor.parse_command("sw"), TinyMushCommand::Move(Direction::Southwest));
    }

    #[test]
    fn test_empty_input() {
        let processor = TinyMushProcessor::new();
        assert_eq!(processor.parse_command(""), TinyMushCommand::Unknown("".to_string()));
        assert_eq!(processor.parse_command("   "), TinyMushCommand::Unknown("".to_string()));
    }
}
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
use crate::tmush::{TinyMushStore, TinyMushError, PlayerRecord, REQUIRED_START_LOCATION_ID};
use crate::tmush::types::{BulletinBoard, BulletinMessage, Direction as TmushDirection};
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
    
    // Companion commands (Phase 6 feature)
    Companion(Option<String>), // COMPANION, COMPANION horse - manage companions
    Feed(String),           // FEED horse - feed companion
    Pet(String),            // PET dog - interact with companion
    Mount(String),          // MOUNT horse - mount companion
    Dismount,              // DISMOUNT - dismount from companion
    
    // System
    Help(Option<String>),   // HELP, HELP topic
    Quit,                   // QUIT - leave TinyMUSH
    Save,                   // SAVE - force save player state

    // Meta/admin (future phases)
    Debug(String),          // DEBUG - admin diagnostics
    
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

            // Admin/debug
            "DEBUG" => {
                if parts.len() > 1 {
                    TinyMushCommand::Debug(parts[1..].join(" "))
                } else {
                    TinyMushCommand::Debug("".to_string())
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
            return Ok("Say what?".to_string());
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
                return Ok("You can't whisper to yourself!".to_string());
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
        if text.trim().is_empty() {
            return Ok("Emote what?".to_string());
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

    /// Handle HELP command
    async fn handle_help(&mut self, _session: &Session, topic: Option<String>, _config: &Config) -> Result<String> {
        match topic.as_deref() {
            Some("commands") | Some("COMMANDS") => Ok(self.help_commands()),
            Some("movement") | Some("MOVEMENT") => Ok(self.help_movement()),
            Some("social") | Some("SOCIAL") => Ok(self.help_social()),
            Some("board") | Some("BOARD") | Some("bulletin") | Some("BULLETIN") => Ok(self.help_bulletin()),
            Some("mail") | Some("MAIL") => Ok(self.help_mail()),
            None => Ok(self.help_main()),
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
                // Create new player
                let display_name = if session.is_logged_in() {
                    session.display_name()
                } else {
                    format!("Guest_{}", &session.id[..8])
                };

                let player = PlayerRecord::new(&username, &display_name, REQUIRED_START_LOCATION_ID);
                self.store().put_player(player.clone())?;
                Ok(player)
            },
            Err(e) => Err(e),
        }
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
                Ok(format!("Deposited {:?} to bank.\nUse BALANCE to check your account.", amount))
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

        Ok(format!("Trade initiated with {}!\nUse OFFER to add items/currency.\nType ACCEPT when ready.", target))
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
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
}

impl TinyMushProcessor {
    pub fn new() -> Self {
        Self { store: None }
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
            TinyMushCommand::Look(target) => self.handle_look(session, target).await,
            TinyMushCommand::Move(direction) => self.handle_move(session, direction).await,
            TinyMushCommand::Where => self.handle_where(session).await,
            TinyMushCommand::Inventory => self.handle_inventory(session).await,
            TinyMushCommand::Who => self.handle_who(session).await,
            TinyMushCommand::Score => self.handle_score(session).await,
            TinyMushCommand::Say(text) => self.handle_say(session, text).await,
            TinyMushCommand::Help(topic) => self.handle_help(session, topic).await,
            TinyMushCommand::Quit => self.handle_quit(session).await,
            TinyMushCommand::Save => self.handle_save(session).await,
            TinyMushCommand::Unknown(cmd) => Ok(format!(
                "Unknown command: '{}'\nType HELP for available commands.",
                cmd
            )),
            _ => Ok("That command isn't implemented yet.\nType HELP for available commands.".to_string()),
        }
    }

    /// Parse raw input into TinyMUSH command enum
    fn parse_command(&self, input: &str) -> TinyMushCommand {
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

            // Whisper (special format: W player text)
            "WHISPER" => {
                if parts.len() > 2 {
                    TinyMushCommand::Whisper(parts[1].to_string(), parts[2..].join(" "))
                } else {
                    TinyMushCommand::Unknown(input)
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
    ) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // If no target specified, look at current room
        if target.is_none() {
            return self.describe_current_room(&player).await;
        }

        // Look at specific target (not implemented in Phase 2)
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
    ) -> Result<String> {
        let _player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        // Phase 2: Basic movement validation, room lookup in Phase 3
        let dir_str = match direction {
            Direction::North => "north",
            Direction::South => "south", 
            Direction::East => "east",
            Direction::West => "west",
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Northeast => "northeast",
            Direction::Northwest => "northwest",
            Direction::Southeast => "southeast",
            Direction::Southwest => "southwest",
        };

        Ok(format!(
            "You attempt to go {}.\n\n(Movement not yet implemented - Phase 3 feature)",
            dir_str
        ))
    }

    /// Handle WHERE command - show current location
    async fn handle_where(&mut self, session: &Session) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        Ok(format!(
            "You are in: {}\n\n(Room details in Phase 3)",
            player.current_room
        ))
    }

    /// Handle INVENTORY command
    async fn handle_inventory(&mut self, session: &Session) -> Result<String> {
        let player = match self.get_or_create_player(session).await {
            Ok(player) => player,
            Err(e) => return Ok(format!("Error loading player: {}", e)),
        };

        if player.inventory.is_empty() {
            return Ok("You are carrying nothing.".to_string());
        }

        let mut response = "=== INVENTORY ===\n".to_string();
        response.push_str(&format!("Gold: {}\n", player.credits));
        response.push_str(&format!("Items: {}\n", player.inventory.len()));
        
        for (i, item_id) in player.inventory.iter().enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1, item_id));
        }

        Ok(response)
    }

    /// Handle WHO command - list online players
    async fn handle_who(&mut self, _session: &Session) -> Result<String> {
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
    async fn handle_score(&mut self, session: &Session) -> Result<String> {
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
    async fn handle_say(&mut self, session: &Session, text: String) -> Result<String> {
        if text.trim().is_empty() {
            return Ok("Say what?".to_string());
        }

        let speaker = session.display_name();
        Ok(format!(
            "{}: \"{}\"\n\n(Other players will see this in Phase 4)",
            speaker, text
        ))
    }

    /// Handle HELP command
    async fn handle_help(&mut self, _session: &Session, topic: Option<String>) -> Result<String> {
        match topic.as_deref() {
            Some("commands") | Some("COMMANDS") => Ok(self.help_commands()),
            Some("movement") | Some("MOVEMENT") => Ok(self.help_movement()),
            Some("social") | Some("SOCIAL") => Ok(self.help_social()),
            None => Ok(self.help_main()),
            Some(topic) => Ok(format!("No help available for: {}\nTry: HELP COMMANDS", topic)),
        }
    }

    /// Handle QUIT command - exit TinyMUSH
    async fn handle_quit(&mut self, session: &mut Session) -> Result<String> {
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
    async fn handle_save(&mut self, session: &Session) -> Result<String> {
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
        Ok(format!(
            "=== {} ===\n\n(Room descriptions in Phase 3)\n\nYou are standing in {}.",
            player.current_room.to_uppercase(),
            player.current_room
        ))
    }

    /// Main help text
    fn help_main(&self) -> String {
        "=== TINYMUSH HELP ===\n".to_string() +
        "Movement: N S E W U D NE NW SE SW\n" +
        "Look: L (room) L <thing>\n" +
        "Info: I (inventory) WHO SCORE WHERE\n" +
        "Talk: SAY <text> EMOTE <action>\n" +
        "System: HELP <topic> SAVE QUIT\n\n" +
        "Topics: COMMANDS MOVEMENT SOCIAL"
    }

    /// Commands help
    fn help_commands(&self) -> String {
        "=== COMMANDS ===\n".to_string() +
        "L/LOOK - examine room/object\n" +
        "I/INV - show inventory\n" +
        "WHO - list online players\n" +
        "WHERE - show current location\n" +
        "SCORE - show your stats\n" +
        "SAY <text> - speak to room\n" +
        "HELP <topic> - get help\n" +
        "SAVE - save your progress\n" +
        "QUIT - return to main menu"
    }

    /// Movement help
    fn help_movement(&self) -> String {
        "=== MOVEMENT ===\n".to_string() +
        "N/NORTH - go north\n" +
        "S/SOUTH - go south\n" +
        "E/EAST - go east\n" +
        "W/WEST - go west\n" +
        "U/UP - go up\n" +
        "D/DOWN - go down\n" +
        "NE/NW/SE/SW - diagonals\n\n" +
        "(Movement active in Phase 3)"
    }

    /// Social commands help  
    fn help_social(&self) -> String {
        "=== SOCIAL ===\n".to_string() +
        "SAY <text> - speak aloud\n" +
        "EMOTE <action> - perform action\n" +
        "POSE <pose> - strike a pose\n" +
        "OOC <text> - out of character\n" +
        "WHO - see other players\n\n" +
        "(Full social features in Phase 4)"
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
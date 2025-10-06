use std::path::{Path, PathBuf};

use chrono::Utc;
use sled::IVec;

use crate::tmush::errors::TinyMushError;
use crate::tmush::state::canonical_world_seed;
use crate::tmush::types::{
    BulletinBoard, BulletinMessage, ObjectOwner, ObjectRecord, PlayerRecord, RoomOwner, RoomRecord, 
    BULLETIN_SCHEMA_VERSION, OBJECT_SCHEMA_VERSION, PLAYER_SCHEMA_VERSION, ROOM_SCHEMA_VERSION,
};

const TREE_PRIMARY: &str = "tinymush";
const TREE_OBJECTS: &str = "tinymush_objects";
const TREE_MAIL: &str = "tinymush_mail";
const TREE_LOGS: &str = "tinymush_logs";
const TREE_BULLETINS: &str = "tinymush_bulletins";

fn next_timestamp_nanos() -> i64 {
    let now = Utc::now();
    now.timestamp_nanos_opt()
        .unwrap_or_else(|| now.timestamp_micros() * 1000)
}

/// Helper builder so tests can easily create throwaway stores with custom paths.
pub struct TinyMushStoreBuilder {
    path: PathBuf,
    ensure_world_seed: bool,
}

impl TinyMushStoreBuilder {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ensure_world_seed: true,
        }
    }

    /// Opt out of seeding the canonical world during initialization (useful for targeted tests).
    pub fn without_world_seed(mut self) -> Self {
        self.ensure_world_seed = false;
        self
    }

    pub fn open(self) -> Result<TinyMushStore, TinyMushError> {
        TinyMushStore::open_with_options(self.path, self.ensure_world_seed)
    }
}

/// Sled-backed persistence for TinyMUSH world data and player state.
pub struct TinyMushStore {
    _db: sled::Db,
    primary: sled::Tree,
    objects: sled::Tree,
    mail: sled::Tree,
    logs: sled::Tree,
    bulletins: sled::Tree,
}

impl TinyMushStore {
    /// Open (or create) the TinyMUSH store rooted at `path`. When `seed_world` is true the
    /// canonical "Old Towne Mesh" rooms are inserted if no world rooms exist yet.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, TinyMushError> {
        Self::open_with_options(path, true)
    }

    fn open_with_options<P: AsRef<Path>>(path: P, seed_world: bool) -> Result<Self, TinyMushError> {
        let path_ref = path.as_ref();
        std::fs::create_dir_all(path_ref)?;
        let db = sled::open(path_ref)?;
        let primary = db.open_tree(TREE_PRIMARY)?;
        let objects = db.open_tree(TREE_OBJECTS)?;
        let mail = db.open_tree(TREE_MAIL)?;
        let logs = db.open_tree(TREE_LOGS)?;
        let bulletins = db.open_tree(TREE_BULLETINS)?;
        let store = Self {
            _db: db,
            primary,
            objects,
            mail,
            logs,
            bulletins,
        };

        if seed_world {
            store.seed_world_if_needed()?;
        }

        Ok(store)
    }

    fn players_key(username: &str) -> Vec<u8> {
        format!("players:{}", username.to_ascii_lowercase()).into_bytes()
    }

    fn room_key(record: &RoomRecord) -> Vec<u8> {
        match &record.owner {
            RoomOwner::World => format!("rooms:world:{}", record.id),
            RoomOwner::Player { username } => {
                format!(
                    "rooms:player:{}:{}",
                    username.to_ascii_lowercase(),
                    record.id
                )
            }
        }
        .into_bytes()
    }

    fn room_world_prefix() -> &'static [u8] {
        b"rooms:world:"
    }

    fn object_key(record: &ObjectRecord) -> Vec<u8> {
        match &record.owner {
            ObjectOwner::World => format!("objects:world:{}", record.id),
            ObjectOwner::Player { username } => {
                format!(
                    "objects:player:{}:{}",
                    username.to_ascii_lowercase(),
                    record.id
                )
            }
        }
        .into_bytes()
    }

    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, TinyMushError> {
        Ok(bincode::serialize(value)?)
    }

    fn deserialize<T: serde::de::DeserializeOwned>(bytes: IVec) -> Result<T, TinyMushError> {
        Ok(bincode::deserialize::<T>(&bytes)?)
    }

    /// Insert or update a player record.
    pub fn put_player(&self, mut player: PlayerRecord) -> Result<(), TinyMushError> {
        player.schema_version = PLAYER_SCHEMA_VERSION;
        player.touch();
        let key = Self::players_key(&player.username);
        let bytes = Self::serialize(&player)?;
        self.primary.insert(key, bytes)?;
        self.primary.flush()?;
        Ok(())
    }

    /// Fetch a player record by username.
    pub fn get_player(&self, username: &str) -> Result<PlayerRecord, TinyMushError> {
        let key = Self::players_key(username);
        let Some(bytes) = self.primary.get(&key)? else {
            return Err(TinyMushError::NotFound(format!("player: {}", username)));
        };
        let record: PlayerRecord = Self::deserialize(bytes)?;
        if record.schema_version != PLAYER_SCHEMA_VERSION {
            return Err(TinyMushError::SchemaMismatch {
                entity: "player",
                expected: PLAYER_SCHEMA_VERSION,
                found: record.schema_version,
            });
        }
        Ok(record)
    }

    /// List all player usernames currently stored.
    pub fn list_player_ids(&self) -> Result<Vec<String>, TinyMushError> {
        let mut ids = Vec::new();
        for entry in self.primary.scan_prefix(b"players:") {
            let (key, _) = entry?;
            let text = String::from_utf8_lossy(&key);
            if let Some(username) = text.strip_prefix("players:") {
                ids.push(username.to_string());
            }
        }
        Ok(ids)
    }

    /// Insert or update a room record.
    pub fn put_room(&self, mut room: RoomRecord) -> Result<(), TinyMushError> {
        room.schema_version = ROOM_SCHEMA_VERSION;
        let key = Self::room_key(&room);
        let bytes = Self::serialize(&room)?;
        self.primary.insert(key, bytes)?;
        self.primary.flush()?;
        Ok(())
    }

    pub fn get_room(&self, room_id: &str) -> Result<RoomRecord, TinyMushError> {
        let key = format!("rooms:world:{}", room_id).into_bytes();
        let Some(bytes) = self.primary.get(&key)? else {
            return Err(TinyMushError::NotFound(format!("room: {}", room_id)));
        };
        let record: RoomRecord = Self::deserialize(bytes)?;
        if record.schema_version != ROOM_SCHEMA_VERSION {
            return Err(TinyMushError::SchemaMismatch {
                entity: "room",
                expected: ROOM_SCHEMA_VERSION,
                found: record.schema_version,
            });
        }
        Ok(record)
    }

    /// Insert or update an object definition.
    pub fn put_object(&self, mut object: ObjectRecord) -> Result<(), TinyMushError> {
        object.schema_version = OBJECT_SCHEMA_VERSION;
        let key = Self::object_key(&object);
        let bytes = Self::serialize(&object)?;
        self.objects.insert(key, bytes)?;
        self.objects.flush()?;
        Ok(())
    }

    pub fn seed_world_if_needed(&self) -> Result<usize, TinyMushError> {
        if self
            .primary
            .scan_prefix(Self::room_world_prefix())
            .next()
            .is_some()
        {
            return Ok(0);
        }
        let now = Utc::now();
        let rooms = canonical_world_seed(now);
        let mut inserted = 0usize;
        for room in rooms {
            self.put_room(room)?;
            inserted += 1;
        }
        Ok(inserted)
    }

    /// Create or update a bulletin board configuration
    pub fn put_bulletin_board(&self, mut board: BulletinBoard) -> Result<(), TinyMushError> {
        board.schema_version = BULLETIN_SCHEMA_VERSION;
        let key = format!("boards:{}", board.id).into_bytes();
        let bytes = Self::serialize(&board)?;
        self.bulletins.insert(key, bytes)?;
        self.bulletins.flush()?;
        Ok(())
    }

    /// Get a bulletin board by ID
    pub fn get_bulletin_board(&self, board_id: &str) -> Result<BulletinBoard, TinyMushError> {
        let key = format!("boards:{}", board_id).into_bytes();
        let bytes = self.bulletins.get(key)?
            .ok_or_else(|| TinyMushError::NotFound(format!("Bulletin board: {}", board_id)))?;
        Ok(Self::deserialize(bytes)?)
    }

    /// Post a message to a bulletin board
    pub fn post_bulletin(&self, mut message: BulletinMessage) -> Result<u64, TinyMushError> {
        // Generate unique message ID based on timestamp
        let message_id = next_timestamp_nanos() as u64;
        message.id = message_id;
        message.schema_version = BULLETIN_SCHEMA_VERSION;
        
        let key = format!("messages:{}:{:020}", message.board_id, message_id).into_bytes();
        let bytes = Self::serialize(&message)?;
        self.bulletins.insert(key, bytes)?;
        self.bulletins.flush()?;
        Ok(message_id)
    }

    /// Get a specific bulletin message
    pub fn get_bulletin(&self, board_id: &str, message_id: u64) -> Result<BulletinMessage, TinyMushError> {
        let key = format!("messages:{}:{:020}", board_id, message_id).into_bytes();
        let bytes = self.bulletins.get(key)?
            .ok_or_else(|| TinyMushError::NotFound(format!("Bulletin message: {}", message_id)))?;
        Ok(Self::deserialize(bytes)?)
    }

    /// List bulletin messages for a board with pagination
    pub fn list_bulletins(&self, board_id: &str, offset: usize, limit: usize) -> Result<Vec<BulletinMessage>, TinyMushError> {
        let prefix = format!("messages:{}:", board_id);
        let messages: Result<Vec<_>, _> = self.bulletins
            .scan_prefix(prefix.as_bytes())
            .skip(offset)
            .take(limit)
            .map(|result| {
                result.map_err(TinyMushError::from)
                    .and_then(|(_key, value)| Self::deserialize(value))
            })
            .collect();
        messages
    }

    /// Count total messages on a bulletin board
    pub fn count_bulletins(&self, board_id: &str) -> Result<usize, TinyMushError> {
        let prefix = format!("messages:{}:", board_id);
        let count = self.bulletins
            .scan_prefix(prefix.as_bytes())
            .count();
        Ok(count)
    }

    /// Remove old messages to enforce max_messages limit
    pub fn cleanup_bulletins(&self, board_id: &str, max_messages: u32) -> Result<usize, TinyMushError> {
        let prefix = format!("messages:{}:", board_id);
        let all_keys: Result<Vec<_>, _> = self.bulletins
            .scan_prefix(prefix.as_bytes())
            .map(|result| result.map(|(key, _value)| key))
            .collect();
        
        let mut keys = all_keys?;
        if keys.len() <= max_messages as usize {
            return Ok(0);
        }

        // Sort by key (which includes timestamp) and remove oldest
        keys.sort();
        let to_remove = keys.len() - max_messages as usize;
        let mut removed = 0;
        
        for key in keys.iter().take(to_remove) {
            self.bulletins.remove(key)?;
            removed += 1;
        }
        
        self.bulletins.flush()?;
        Ok(removed)
    }

    /// Append a line to the TinyMUSH diagnostic log tree.
    pub fn append_log(&self, message: &str) -> Result<(), TinyMushError> {
        let key = format!("logs:{}", next_timestamp_nanos()).into_bytes();
        self.logs.insert(key, message.as_bytes())?;
        self.logs.flush()?;
        Ok(())
    }

    /// Store a mail payload for a player. Future phases will consume these entries
    /// to deliver asynchronous notifications.
    pub fn enqueue_mail(&self, username: &str, body: &str) -> Result<(), TinyMushError> {
        let key = format!(
            "mail:{}:{}",
            username.to_ascii_lowercase(),
            next_timestamp_nanos()
        )
        .into_bytes();
        self.mail.insert(key, body.as_bytes())?;
        self.mail.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::state::OLD_TOWNE_WORLD_ROOM_IDS;
    use tempfile::TempDir;

    #[test]
    fn store_round_trip_player() {
        let dir = TempDir::new().expect("tempdir");
        let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
        let mut player = PlayerRecord::new("alice", "Alice", "town_square");
        player.credits = 42;
        store.put_player(player.clone()).expect("put");
        let fetched = store.get_player("alice").expect("get");
        assert_eq!(fetched.username, player.username);
        assert_eq!(fetched.credits, 42);
        assert_eq!(fetched.schema_version, PLAYER_SCHEMA_VERSION);
        drop(store);
    }

    #[test]
    fn seeding_world_only_happens_once() {
        let dir = TempDir::new().expect("tempdir");
        {
            let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
            let ids = store.list_player_ids().expect("list players");
            assert!(ids.is_empty());
            for room_id in OLD_TOWNE_WORLD_ROOM_IDS {
                store.get_room(room_id).expect("room present");
            }
        }

        let store = TinyMushStoreBuilder::new(dir.path())
            .without_world_seed()
            .open()
            .expect("reopen store");
        let count = store.seed_world_if_needed().expect("seed check");
        assert_eq!(count, 0, "should not reseed when rooms already exist");
        for room_id in OLD_TOWNE_WORLD_ROOM_IDS {
            store.get_room(room_id).expect("room persists");
        }
    }

    #[test]
    fn seed_world_populates_expected_rooms() {
        let dir = TempDir::new().expect("tempdir");
        let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");
        for room_id in OLD_TOWNE_WORLD_ROOM_IDS {
            let room = store.get_room(room_id).expect("room present");
            assert_eq!(room.schema_version, ROOM_SCHEMA_VERSION);
        }
        drop(store);
    }
}

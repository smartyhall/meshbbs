use std::path::{Path, PathBuf};

use chrono::Utc;
use sled::IVec;

use crate::tmush::errors::TinyMushError;
use crate::tmush::state::canonical_world_seed;
use crate::tmush::types::{
    BulletinBoard, BulletinMessage, CurrencyAmount, CurrencyTransaction, MailMessage,
    MailStatus, ObjectOwner, ObjectRecord, PlayerRecord, RoomOwner, RoomRecord,
    TransactionReason, BULLETIN_SCHEMA_VERSION, MAIL_SCHEMA_VERSION, OBJECT_SCHEMA_VERSION,
    PLAYER_SCHEMA_VERSION, ROOM_SCHEMA_VERSION,
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

    /// Send a mail message between players
    pub fn send_mail(&self, mut message: MailMessage) -> Result<u64, TinyMushError> {
        // Generate unique message ID based on timestamp
        let message_id = next_timestamp_nanos() as u64;
        message.id = message_id;
        message.schema_version = MAIL_SCHEMA_VERSION;

        // Store in recipient's inbox
        let inbox_key = format!(
            "mail:inbox:{}:{}:{:020}",
            message.recipient.to_ascii_lowercase(),
            message.sender.to_ascii_lowercase(),
            message_id
        ).into_bytes();
        
        // Store in sender's sent folder
        let sent_key = format!(
            "mail:sent:{}:{}:{:020}",
            message.sender.to_ascii_lowercase(),
            message.recipient.to_ascii_lowercase(),
            message_id
        ).into_bytes();

        let bytes = Self::serialize(&message)?;
        
        self.mail.insert(inbox_key, bytes.clone())?;
        self.mail.insert(sent_key, bytes)?;
        self.mail.flush()?;
        
        Ok(message_id)
    }

    /// Get a specific mail message
    pub fn get_mail(&self, folder: &str, username: &str, message_id: u64) -> Result<MailMessage, TinyMushError> {
        // Try to find the message in the specified folder
        let prefix = format!("mail:{}:{}:", folder, username.to_ascii_lowercase());
        
        for result in self.mail.scan_prefix(prefix.as_bytes()) {
            let (key, value) = result?;
            let key_str = String::from_utf8_lossy(&key);
            if key_str.ends_with(&format!(":{:020}", message_id)) {
                let message: MailMessage = Self::deserialize(value)?;
                return Ok(message);
            }
        }
        
        Err(TinyMushError::NotFound(format!("Mail message: {}", message_id)))
    }

    /// List mail messages for a player's folder
    pub fn list_mail(&self, folder: &str, username: &str, offset: usize, limit: usize) -> Result<Vec<MailMessage>, TinyMushError> {
        let prefix = format!("mail:{}:{}:", folder, username.to_ascii_lowercase());
        
        let messages: Result<Vec<_>, _> = self.mail
            .scan_prefix(prefix.as_bytes())
            .skip(offset)
            .take(limit)
            .map(|result| {
                result.map_err(TinyMushError::from)
                    .and_then(|(_key, value)| Self::deserialize::<MailMessage>(value))
            })
            .collect();
        
        let mut messages = messages?;
        // Sort by sent date, newest first
        messages.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
        Ok(messages)
    }

    /// Mark a mail message as read
    pub fn mark_mail_read(&self, folder: &str, username: &str, message_id: u64) -> Result<(), TinyMushError> {
        let prefix = format!("mail:{}:{}:", folder, username.to_ascii_lowercase());
        
        for result in self.mail.scan_prefix(prefix.as_bytes()) {
            let (key, value) = result?;
            let key_str = String::from_utf8_lossy(&key);
            if key_str.ends_with(&format!(":{:020}", message_id)) {
                let mut message: MailMessage = Self::deserialize(value)?;
                message.mark_read();
                let updated_bytes = Self::serialize(&message)?;
                self.mail.insert(key, updated_bytes)?;
                self.mail.flush()?;
                return Ok(());
            }
        }
        
        Err(TinyMushError::NotFound(format!("Mail message: {}", message_id)))
    }

    /// Delete a mail message
    pub fn delete_mail(&self, folder: &str, username: &str, message_id: u64) -> Result<(), TinyMushError> {
        let prefix = format!("mail:{}:{}:", folder, username.to_ascii_lowercase());
        
        for result in self.mail.scan_prefix(prefix.as_bytes()) {
            let (key, _value) = result?;
            let key_str = String::from_utf8_lossy(&key);
            if key_str.ends_with(&format!(":{:020}", message_id)) {
                self.mail.remove(key)?;
                self.mail.flush()?;
                return Ok(());
            }
        }
        
        Err(TinyMushError::NotFound(format!("Mail message: {}", message_id)))
    }

    /// Count mail messages in a folder
    pub fn count_mail(&self, folder: &str, username: &str) -> Result<usize, TinyMushError> {
        let prefix = format!("mail:{}:{}:", folder, username.to_ascii_lowercase());
        let count = self.mail.scan_prefix(prefix.as_bytes()).count();
        Ok(count)
    }

    /// Count unread mail messages in inbox
    pub fn count_unread_mail(&self, username: &str) -> Result<usize, TinyMushError> {
        let prefix = format!("mail:inbox:{}:", username.to_ascii_lowercase());
        let mut unread_count = 0;
        
        for result in self.mail.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = result?;
            let message: MailMessage = Self::deserialize(value)?;
            if message.status == MailStatus::Unread {
                unread_count += 1;
            }
        }
        
        Ok(unread_count)
    }

    /// Cleanup old read mail messages
    pub fn cleanup_old_mail(&self, username: &str, days_old: u32) -> Result<usize, TinyMushError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_old as i64);
        let prefix = format!("mail:inbox:{}:", username.to_ascii_lowercase());
        let mut removed = 0;
        
        let mut keys_to_remove = Vec::new();
        
        for result in self.mail.scan_prefix(prefix.as_bytes()) {
            let (key, value) = result?;
            let message: MailMessage = Self::deserialize(value)?;
            
            // Only delete read messages older than cutoff
            if message.status != MailStatus::Unread && message.sent_at < cutoff_date {
                keys_to_remove.push(key);
            }
        }
        
        for key in keys_to_remove {
            self.mail.remove(key)?;
            removed += 1;
        }
        
        if removed > 0 {
            self.mail.flush()?;
        }
        
        Ok(removed)
    }

    /// Enforce mail quota for a player
    pub fn enforce_mail_quota(&self, username: &str, max_messages: u32) -> Result<usize, TinyMushError> {
        let inbox_count = self.count_mail("inbox", username)?;
        
        if inbox_count <= max_messages as usize {
            return Ok(0);
        }
        
        // Get all messages sorted by date (oldest first)
        let prefix = format!("mail:inbox:{}:", username.to_ascii_lowercase());
        let mut messages_with_keys = Vec::new();
        
        for result in self.mail.scan_prefix(prefix.as_bytes()) {
            let (key, value) = result?;
            let message: MailMessage = Self::deserialize(value)?;
            messages_with_keys.push((key, message));
        }
        
        // Sort by sent date, oldest first
        messages_with_keys.sort_by(|a, b| a.1.sent_at.cmp(&b.1.sent_at));
        
        // Remove oldest messages to get under quota
        let to_remove = inbox_count - max_messages as usize;
        let mut removed = 0;
        
        for (key, message) in messages_with_keys.iter().take(to_remove) {
            // Only remove read messages to preserve unread mail
            if message.status != MailStatus::Unread {
                self.mail.remove(key)?;
                removed += 1;
            }
        }
        
        if removed > 0 {
            self.mail.flush()?;
        }
        
        Ok(removed)
    }

    /// Legacy method for simple mail notifications (kept for compatibility)
    pub fn enqueue_mail(&self, username: &str, body: &str) -> Result<(), TinyMushError> {
        let message = MailMessage::new("system", username, "System Message", body);
        self.send_mail(message)?;
        Ok(())
    }

    // ========================================================================
    // Currency & Economy Methods
    // ========================================================================

    /// Transfer currency from one player to another (atomic transaction)
    pub fn transfer_currency(
        &self,
        from_username: &str,
        to_username: &str,
        amount: &CurrencyAmount,
        reason: TransactionReason,
    ) -> Result<CurrencyTransaction, TinyMushError> {
        // Validate amount is positive
        if amount.is_zero_or_negative() {
            return Err(TinyMushError::InvalidCurrency(
                "Transfer amount must be positive".to_string(),
            ));
        }

        // Get both players (locks the records)
        let mut from_player = self.get_player(from_username)?;
        let mut to_player = self.get_player(to_username)?;

        // Check if sender can afford it
        if !from_player.currency.can_afford(amount) {
            return Err(TinyMushError::InsufficientFunds);
        }

        // Perform the transfer
        from_player.currency = from_player.currency.subtract(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency subtraction failed: {}", e))
        })?;

        to_player.currency = to_player.currency.add(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency addition failed: {}", e))
        })?;

        // Save both players
        self.put_player(from_player)?;
        self.put_player(to_player)?;

        // Create transaction record
        let transaction = CurrencyTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from: Some(from_username.to_string()),
            to: Some(to_username.to_string()),
            amount: amount.clone(),
            reason,
            rolled_back: false,
        };

        // Log the transaction
        self.log_transaction(&transaction)?;

        Ok(transaction)
    }

    /// Grant currency to a player from the system (admin/quest rewards)
    pub fn grant_currency(
        &self,
        username: &str,
        amount: &CurrencyAmount,
        reason: TransactionReason,
    ) -> Result<CurrencyTransaction, TinyMushError> {
        if amount.is_zero_or_negative() {
            return Err(TinyMushError::InvalidCurrency(
                "Grant amount must be positive".to_string(),
            ));
        }

        let mut player = self.get_player(username)?;
        player.currency = player.currency.add(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency addition failed: {}", e))
        })?;
        self.put_player(player)?;

        let transaction = CurrencyTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from: None, // System grant
            to: Some(username.to_string()),
            amount: amount.clone(),
            reason,
            rolled_back: false,
        };

        self.log_transaction(&transaction)?;
        Ok(transaction)
    }

    /// Deduct currency from a player to the system (admin/rent payments)
    pub fn deduct_currency(
        &self,
        username: &str,
        amount: &CurrencyAmount,
        reason: TransactionReason,
    ) -> Result<CurrencyTransaction, TinyMushError> {
        if amount.is_zero_or_negative() {
            return Err(TinyMushError::InvalidCurrency(
                "Deduct amount must be positive".to_string(),
            ));
        }

        let mut player = self.get_player(username)?;
        if !player.currency.can_afford(amount) {
            return Err(TinyMushError::InsufficientFunds);
        }

        player.currency = player.currency.subtract(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency subtraction failed: {}", e))
        })?;
        self.put_player(player)?;

        let transaction = CurrencyTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from: Some(username.to_string()),
            to: None, // System deduction
            amount: amount.clone(),
            reason,
            rolled_back: false,
        };

        self.log_transaction(&transaction)?;
        Ok(transaction)
    }

    /// Deposit currency to bank (move from pocket to vault)
    pub fn bank_deposit(
        &self,
        username: &str,
        amount: &CurrencyAmount,
    ) -> Result<CurrencyTransaction, TinyMushError> {
        if amount.is_zero_or_negative() {
            return Err(TinyMushError::InvalidCurrency(
                "Deposit amount must be positive".to_string(),
            ));
        }

        let mut player = self.get_player(username)?;
        if !player.currency.can_afford(amount) {
            return Err(TinyMushError::InsufficientFunds);
        }

        // Move from pocket to bank
        player.currency = player.currency.subtract(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency subtraction failed: {}", e))
        })?;

        player.banked_currency = player.banked_currency.add(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency addition failed: {}", e))
        })?;

        self.put_player(player)?;

        let transaction = CurrencyTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from: Some(username.to_string()),
            to: Some(format!("{}:bank", username)),
            amount: amount.clone(),
            reason: TransactionReason::BankDeposit,
            rolled_back: false,
        };

        self.log_transaction(&transaction)?;
        Ok(transaction)
    }

    /// Withdraw currency from bank (move from vault to pocket)
    pub fn bank_withdraw(
        &self,
        username: &str,
        amount: &CurrencyAmount,
    ) -> Result<CurrencyTransaction, TinyMushError> {
        if amount.is_zero_or_negative() {
            return Err(TinyMushError::InvalidCurrency(
                "Withdrawal amount must be positive".to_string(),
            ));
        }

        let mut player = self.get_player(username)?;
        if !player.banked_currency.can_afford(amount) {
            return Err(TinyMushError::InsufficientFunds);
        }

        // Move from bank to pocket
        player.banked_currency = player.banked_currency.subtract(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency subtraction failed: {}", e))
        })?;

        player.currency = player.currency.add(amount).map_err(|e| {
            TinyMushError::InvalidCurrency(format!("Currency addition failed: {}", e))
        })?;

        self.put_player(player)?;

        let transaction = CurrencyTransaction {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            from: Some(format!("{}:bank", username)),
            to: Some(username.to_string()),
            amount: amount.clone(),
            reason: TransactionReason::BankWithdrawal,
            rolled_back: false,
        };

        self.log_transaction(&transaction)?;
        Ok(transaction)
    }

    /// Log a transaction to the audit tree
    fn log_transaction(&self, transaction: &CurrencyTransaction) -> Result<(), TinyMushError> {
        let key = format!("transaction:{}", transaction.id);
        self.mail
            .insert(key.as_bytes(), Self::serialize(transaction)?)?;
        self.mail.flush()?;
        Ok(())
    }

    /// Get a transaction by ID
    pub fn get_transaction(&self, id: &str) -> Result<CurrencyTransaction, TinyMushError> {
        let key = format!("transaction:{}", id);
        match self.mail.get(key.as_bytes())? {
            Some(bytes) => Ok(Self::deserialize(bytes)?),
            None => Err(TinyMushError::TransactionNotFound),
        }
    }

    /// Rollback a transaction (reverse the currency movement)
    pub fn rollback_transaction(&self, transaction_id: &str) -> Result<(), TinyMushError> {
        let mut transaction = self.get_transaction(transaction_id)?;

        if transaction.rolled_back {
            return Err(TinyMushError::InvalidCurrency(
                "Transaction already rolled back".to_string(),
            ));
        }

        // Reverse the transaction based on type
        match (&transaction.from, &transaction.to) {
            (Some(from), Some(to)) if to.contains(":bank") => {
                // Reverse bank deposit
                let username = from;
                let mut player = self.get_player(username)?;

                player.currency = player.currency.subtract(&transaction.amount).map_err(|e| {
                    TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                })?;

                player.banked_currency =
                    player.banked_currency.add(&transaction.amount).map_err(|e| {
                        TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                    })?;

                self.put_player(player)?;
            }
            (Some(from), Some(to)) if from.contains(":bank") => {
                // Reverse bank withdrawal
                let username = to;
                let mut player = self.get_player(username)?;

                player.banked_currency =
                    player.banked_currency.subtract(&transaction.amount).map_err(|e| {
                        TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                    })?;

                player.currency = player.currency.add(&transaction.amount).map_err(|e| {
                    TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                })?;

                self.put_player(player)?;
            }
            (Some(from), Some(to)) => {
                // Reverse player-to-player transfer
                let mut from_player = self.get_player(from)?;
                let mut to_player = self.get_player(to)?;

                from_player.currency =
                    from_player.currency.add(&transaction.amount).map_err(|e| {
                        TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                    })?;

                to_player.currency =
                    to_player.currency.subtract(&transaction.amount).map_err(|e| {
                        TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                    })?;

                self.put_player(from_player)?;
                self.put_player(to_player)?;
            }
            (None, Some(to)) => {
                // Reverse system grant
                let mut player = self.get_player(to)?;
                player.currency =
                    player.currency.subtract(&transaction.amount).map_err(|e| {
                        TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                    })?;
                self.put_player(player)?;
            }
            (Some(from), None) => {
                // Reverse system deduction
                let mut player = self.get_player(from)?;
                player.currency = player.currency.add(&transaction.amount).map_err(|e| {
                    TinyMushError::InvalidCurrency(format!("Rollback failed: {}", e))
                })?;
                self.put_player(player)?;
            }
            _ => {
                return Err(TinyMushError::InvalidCurrency(
                    "Invalid transaction format".to_string(),
                ))
            }
        }

        // Mark transaction as rolled back
        transaction.rolled_back = true;
        self.log_transaction(&transaction)?;

        Ok(())
    }

    /// Get transaction history for a player
    pub fn get_player_transactions(
        &self,
        username: &str,
        limit: usize,
    ) -> Result<Vec<CurrencyTransaction>, TinyMushError> {
        let mut transactions = Vec::new();
        let prefix = b"transaction:";

        for result in self.mail.scan_prefix(prefix) {
            let (_key, value) = result?;
            let transaction: CurrencyTransaction = Self::deserialize(value)?;

            // Check if player is involved in this transaction
            let involved = match (&transaction.from, &transaction.to) {
                (Some(from), Some(to)) => {
                    from == username
                        || to == username
                        || from == &format!("{}:bank", username)
                        || to == &format!("{}:bank", username)
                }
                (Some(from), None) => from == username,
                (None, Some(to)) => to == username,
                _ => false,
            };

            if involved {
                transactions.push(transaction);
            }

            if transactions.len() >= limit {
                break;
            }
        }

        // Sort by timestamp, newest first
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        transactions.truncate(limit);

        Ok(transactions)
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

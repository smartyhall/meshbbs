//! Room manager with LRU caching and capacity enforcement for TinyMUSH.
//!
//! This module provides efficient room access with caching and enforces room
//! capacity limits. It supports both world rooms and instanced rooms (apartments,
//! hotel rooms) with proper permissions and quotas.

use std::collections::HashMap;
use std::time::Instant;
use anyhow::Result;
use log::debug;

use crate::tmush::types::{RoomRecord, RoomFlag};
use crate::tmush::storage::TinyMushStore;
use crate::tmush::{TinyMushError, PlayerRecord};

/// Default maximum number of rooms to cache
const DEFAULT_CACHE_SIZE: usize = 50;

/// Default room capacity limits by type
const DEFAULT_STANDARD_CAPACITY: u16 = 20;
const DEFAULT_SHOP_CAPACITY: u16 = 10;
const DEFAULT_SOCIAL_CAPACITY: u16 = 50;

/// Cache entry for room data with access tracking
#[derive(Debug, Clone)]
struct CachedRoom {
    room: RoomRecord,
    last_accessed: Instant,
    access_count: u64,
}

/// Room manager with LRU caching and capacity enforcement
pub struct RoomManager {
    store: TinyMushStore,
    cache: HashMap<String, CachedRoom>,
    max_cache_size: usize,
    player_locations: HashMap<String, String>, // player_id -> room_id
}

impl RoomManager {
    /// Create a new room manager with the given store
    pub fn new(store: TinyMushStore) -> Self {
        Self {
            store,
            cache: HashMap::new(),
            max_cache_size: DEFAULT_CACHE_SIZE,
            player_locations: HashMap::new(),
        }
    }

    /// Create a room manager with custom cache size
    pub fn with_cache_size(store: TinyMushStore, cache_size: usize) -> Self {
        Self {
            store,
            cache: HashMap::new(),
            max_cache_size: cache_size,
            player_locations: HashMap::new(),
        }
    }

    /// Get a room by ID, using cache when possible
    pub fn get_room(&mut self, room_id: &str) -> Result<RoomRecord, TinyMushError> {
        // Check cache first
        if let Some(cached) = self.cache.get_mut(room_id) {
            cached.last_accessed = Instant::now();
            cached.access_count += 1;
            debug!("Room cache hit: {} (access_count: {})", room_id, cached.access_count);
            return Ok(cached.room.clone());
        }

        // Cache miss - load from store
        debug!("Room cache miss: {}", room_id);
        let room = self.store.get_room(room_id)?;
        
        // Add to cache
        self.cache_room(room_id.to_string(), room.clone());
        
        Ok(room)
    }

    /// Check if a player can enter a room (capacity and permissions)
    pub fn can_enter_room(&mut self, player: &PlayerRecord, room_id: &str) -> Result<bool, TinyMushError> {
        let room = self.get_room(room_id)?;
        
        // Check room capacity
        let current_occupancy = self.get_room_occupancy(room_id);
        let capacity_limit = self.get_room_capacity_limit(&room);
        
        if current_occupancy >= capacity_limit {
            debug!("Room {} at capacity: {}/{}", room_id, current_occupancy, capacity_limit);
            return Ok(false);
        }

        // Check room permissions (for private rooms, etc.)
        if room.flags.contains(&RoomFlag::Private) {
            // Private rooms - only owner can enter
            match &room.owner {
                crate::tmush::types::RoomOwner::Player { username } => {
                    if username != &player.username {
                        debug!("Player {} denied access to private room {} (owner: {})", 
                               player.username, room_id, username);
                        return Ok(false);
                    }
                }
                crate::tmush::types::RoomOwner::World => {
                    // World private rooms require special permission (future enhancement)
                    return Ok(true);
                }
            }
        }

        Ok(true)
    }

    /// Move a player to a room (with capacity and permission checks)
    pub fn move_player_to_room(&mut self, player: &mut PlayerRecord, room_id: &str) -> Result<bool, TinyMushError> {
        // Check if player can enter the room
        if !self.can_enter_room(player, room_id)? {
            return Ok(false);
        }

        // Remove from old location
        if let Some(old_room_id) = self.player_locations.remove(&player.username) {
            debug!("Player {} leaving room {}", player.username, old_room_id);
        }

        // Add to new location
        self.player_locations.insert(player.username.clone(), room_id.to_string());
        player.current_room = room_id.to_string();
        
        debug!("Player {} entered room {}", player.username, room_id);
        Ok(true)
    }

    /// Get current occupancy of a room
    pub fn get_room_occupancy(&self, room_id: &str) -> u16 {
        self.player_locations
            .values()
            .filter(|&location| location == room_id)
            .count() as u16
    }

    /// Get capacity limit for a room based on its type and flags
    fn get_room_capacity_limit(&self, room: &RoomRecord) -> u16 {
        // Use configured capacity if set
        if room.max_capacity > 0 {
            return room.max_capacity;
        }

        // Determine capacity by room type
        if room.flags.contains(&RoomFlag::Shop) {
            DEFAULT_SHOP_CAPACITY
        } else if room.flags.contains(&RoomFlag::Indoor) || room.flags.contains(&RoomFlag::Private) {
            DEFAULT_STANDARD_CAPACITY
        } else {
            DEFAULT_SOCIAL_CAPACITY
        }
    }

    /// Add a room to the cache, evicting LRU entries if needed
    fn cache_room(&mut self, room_id: String, room: RoomRecord) {
        // If at capacity, evict least recently used
        if self.cache.len() >= self.max_cache_size {
            self.evict_lru();
        }

        let cached_room = CachedRoom {
            room,
            last_accessed: Instant::now(),
            access_count: 1,
        };

        self.cache.insert(room_id.clone(), cached_room);
        debug!("Cached room: {} (cache size: {})", room_id, self.cache.len());
    }

    /// Evict the least recently used room from cache
    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        let oldest_key = self.cache
            .iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            self.cache.remove(&key);
            debug!("Evicted LRU room from cache: {}", key);
        }
    }

    /// Get cache statistics for debugging/monitoring
    pub fn cache_stats(&self) -> CacheStats {
        let total_accesses: u64 = self.cache.values().map(|c| c.access_count).sum();
        
        CacheStats {
            cache_size: self.cache.len(),
            max_cache_size: self.max_cache_size,
            total_accesses,
            total_rooms: self.player_locations.len(),
        }
    }

    /// Clear the cache (useful for testing)
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        debug!("Room cache cleared");
    }

    /// Get all players in a specific room
    pub fn get_players_in_room(&self, room_id: &str) -> Vec<String> {
        self.player_locations
            .iter()
            .filter(|(_, location)| *location == room_id)
            .map(|(player, _)| player.clone())
            .collect()
    }

    /// Remove a player from tracking (when they disconnect)
    pub fn remove_player(&mut self, username: &str) {
        if let Some(room_id) = self.player_locations.remove(username) {
            debug!("Removed player {} from room tracking (was in {})", username, room_id);
        }
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cache_size: usize,
    pub max_cache_size: usize,
    pub total_accesses: u64,
    pub total_rooms: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::types::{RoomRecord, RoomOwner, RoomVisibility};
    use crate::tmush::storage::TinyMushStoreBuilder;
    use tempfile::TempDir;
    use chrono::Utc;

    fn create_test_store() -> (TinyMushStore, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let store = TinyMushStoreBuilder::new(temp_dir.path())
            .without_world_seed()
            .open()
            .expect("Failed to open store");
        (store, temp_dir)
    }

    fn create_test_room(id: &str, name: &str, max_capacity: u16) -> RoomRecord {
        RoomRecord {
            id: id.to_string(),
            name: name.to_string(),
            short_desc: format!("{} - Test Room", name),
            long_desc: format!("This is a test room named {}.", name),
            owner: RoomOwner::World,
            created_at: Utc::now(),
            visibility: RoomVisibility::Public,
            exits: HashMap::new(),
            items: Vec::new(),
            flags: Vec::new(),
            max_capacity,
            housing_filter_tags: Vec::new(),
            locked: false, // Test rooms unlocked
            schema_version: 1,
        }
    }

    fn create_test_player(username: &str, room_id: &str) -> PlayerRecord {
        PlayerRecord::new(username, username, room_id)
    }

    #[test]
    fn test_room_cache_basic_operations() {
        let (store, _temp_dir) = create_test_store();
        let mut manager = RoomManager::new(store);

        // Create and store a test room
        let room = create_test_room("test_room", "Test Room", 10);
        manager.store.put_room(room.clone()).expect("Failed to store room");

        // Test cache miss and hit
        let retrieved1 = manager.get_room("test_room").expect("Failed to get room");
        assert_eq!(retrieved1.id, "test_room");

        let retrieved2 = manager.get_room("test_room").expect("Failed to get room");
        assert_eq!(retrieved2.id, "test_room");

        // Verify cache stats
        let stats = manager.cache_stats();
        assert_eq!(stats.cache_size, 1);
        assert_eq!(stats.total_accesses, 2);
    }

    #[test]
    fn test_lru_eviction() {
        let (store, _temp_dir) = create_test_store();
        let mut manager = RoomManager::with_cache_size(store, 2);

        // Create and store test rooms
        for i in 1..=3 {
            let room = create_test_room(&format!("room_{}", i), &format!("Room {}", i), 10);
            manager.store.put_room(room).expect("Failed to store room");
        }

        // Access rooms 1 and 2 (should be cached)
        manager.get_room("room_1").expect("Failed to get room_1");
        manager.get_room("room_2").expect("Failed to get room_2");

        // Access room 3 (should evict room_1 as LRU)
        manager.get_room("room_3").expect("Failed to get room_3");

        let stats = manager.cache_stats();
        assert_eq!(stats.cache_size, 2); // Cache should be at max size
    }

    #[test]
    fn test_room_capacity_enforcement() {
        let (store, _temp_dir) = create_test_store();
        let mut manager = RoomManager::new(store);

        // Create a small capacity room
        let room = create_test_room("small_room", "Small Room", 2);
        manager.store.put_room(room).expect("Failed to store room");

        // Create test players
        let mut player1 = create_test_player("player1", "spawn");
        let mut player2 = create_test_player("player2", "spawn");
        let mut player3 = create_test_player("player3", "spawn");

        // Move first two players (should succeed)
        assert!(manager.move_player_to_room(&mut player1, "small_room").expect("Move failed"));
        assert!(manager.move_player_to_room(&mut player2, "small_room").expect("Move failed"));

        // Try to move third player (should fail due to capacity)
        assert!(!manager.move_player_to_room(&mut player3, "small_room").expect("Move failed"));

        // Check occupancy
        assert_eq!(manager.get_room_occupancy("small_room"), 2);
    }

    #[test]
    fn test_player_tracking() {
        let (store, _temp_dir) = create_test_store();
        let mut manager = RoomManager::new(store);

        // Create test rooms
        let room1 = create_test_room("room1", "Room 1", 10);
        let room2 = create_test_room("room2", "Room 2", 10);
        manager.store.put_room(room1).expect("Failed to store room1");
        manager.store.put_room(room2).expect("Failed to store room2");

        // Create and move player
        let mut player = create_test_player("testuser", "spawn");
        
        manager.move_player_to_room(&mut player, "room1").expect("Move to room1 failed");
        assert_eq!(manager.get_room_occupancy("room1"), 1);
        assert_eq!(manager.get_room_occupancy("room2"), 0);

        // Move to different room
        manager.move_player_to_room(&mut player, "room2").expect("Move to room2 failed");
        assert_eq!(manager.get_room_occupancy("room1"), 0);
        assert_eq!(manager.get_room_occupancy("room2"), 1);

        // Get players in room
        let players_in_room2 = manager.get_players_in_room("room2");
        assert_eq!(players_in_room2, vec!["testuser"]);

        // Remove player
        manager.remove_player("testuser");
        assert_eq!(manager.get_room_occupancy("room2"), 0);
    }
}
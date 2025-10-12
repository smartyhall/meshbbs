//! Backup & Recovery System for MeshBBS
//!
//! Provides automated and manual backup capabilities for the Sled database,
//! including retention policies, verification, and point-in-time recovery.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

/// Backup metadata stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Unique backup identifier (timestamp-based)
    pub id: String,
    /// Human-readable name (optional)
    pub name: Option<String>,
    /// Timestamp when backup was created
    pub created_at: DateTime<Utc>,
    /// Size of backup file in bytes
    pub size_bytes: u64,
    /// Backup type (manual, automatic, daily, weekly, monthly)
    pub backup_type: BackupType,
    /// SHA256 checksum for verification
    pub checksum: String,
    /// Whether backup has been verified
    pub verified: bool,
    /// Path to backup file (relative to backup directory)
    pub path: PathBuf,
}

/// Type of backup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupType {
    Manual,
    Automatic,
    Daily,
    Weekly,
    Monthly,
}

/// Backup retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Keep last N daily backups
    pub daily_count: usize,
    /// Keep last N weekly backups
    pub weekly_count: usize,
    /// Keep last N monthly backups
    pub monthly_count: usize,
    /// Keep manual backups forever
    pub keep_manual: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            daily_count: 7,     // Keep 1 week of daily backups
            weekly_count: 4,    // Keep 1 month of weekly backups
            monthly_count: 12,  // Keep 1 year of monthly backups
            keep_manual: true,  // Never delete manual backups
        }
    }
}

/// Backup scheduler and manager
pub struct BackupManager {
    /// Path to database directory
    db_path: PathBuf,
    /// Path to backup storage directory
    backup_path: PathBuf,
    /// Retention policy
    retention: RetentionPolicy,
    /// In-memory cache of backup metadata
    backups: HashMap<String, BackupMetadata>,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(db_path: PathBuf, backup_path: PathBuf, retention: RetentionPolicy) -> io::Result<Self> {
        // Ensure backup directory exists
        fs::create_dir_all(&backup_path)?;
        
        let mut manager = Self {
            db_path,
            backup_path,
            retention,
            backups: HashMap::new(),
        };
        
        // Load existing backup metadata
        manager.load_metadata()?;
        
        Ok(manager)
    }
    
    /// Load backup metadata from disk
    fn load_metadata(&mut self) -> io::Result<()> {
        let metadata_path = self.backup_path.join("backups.json");
        
        if metadata_path.exists() {
            let contents = fs::read_to_string(&metadata_path)?;
            self.backups = serde_json::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        }
        
        Ok(())
    }
    
    /// Save backup metadata to disk
    fn save_metadata(&self) -> io::Result<()> {
        let metadata_path = self.backup_path.join("backups.json");
        let contents = serde_json::to_string_pretty(&self.backups)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&metadata_path, contents)?;
        Ok(())
    }
    
    /// Create a new backup
    pub fn create_backup(&mut self, name: Option<String>, backup_type: BackupType) -> io::Result<BackupMetadata> {
        let timestamp = Utc::now();
        let id = format!("backup_{}", timestamp.format("%Y%m%d_%H%M%S_%3f")); // Add milliseconds for uniqueness
        let filename = format!("{}.tar.gz", id);
        let backup_file = self.backup_path.join(&filename);
        
        log::info!("Creating backup: {} (type: {:?})", id, backup_type);
        
        // Create tar.gz archive of database directory
        let tar_gz = File::create(&backup_file)?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);
        
        // Add database directory to archive
        tar.append_dir_all("data", &self.db_path)?;
        
        // IMPORTANT: Finish and flush the archive before calculating checksum
        let enc = tar.into_inner()?;
        enc.finish()?;
        
        // Calculate checksum after file is completely written
        let checksum = self.calculate_checksum(&backup_file)?;
        
        // Get file size
        let size_bytes = fs::metadata(&backup_file)?.len();
        
        // Create metadata
        let metadata = BackupMetadata {
            id: id.clone(),
            name,
            created_at: timestamp,
            size_bytes,
            backup_type,
            checksum,
            verified: false,
            path: PathBuf::from(&filename),
        };
        
        // Store metadata
        self.backups.insert(id.clone(), metadata.clone());
        self.save_metadata()?;
        
        log::info!("Backup created successfully: {} ({} bytes)", id, size_bytes);
        
        Ok(metadata)
    }
    
    /// Verify a backup's integrity
    pub fn verify_backup(&mut self, backup_id: &str) -> io::Result<bool> {
        let metadata = self.backups.get(backup_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Backup not found"))?;
        
        let backup_file = self.backup_path.join(&metadata.path);
        
        if !backup_file.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Backup file missing"));
        }
        
        // Verify checksum
        let current_checksum = self.calculate_checksum(&backup_file)?;
        let valid = current_checksum == metadata.checksum;
        
        if valid {
            log::info!("Backup verification passed: {}", backup_id);
            // Mark as verified
            if let Some(meta) = self.backups.get_mut(backup_id) {
                meta.verified = true;
            }
            self.save_metadata()?;
        } else {
            log::error!("Backup verification FAILED: {} (checksum mismatch)", backup_id);
        }
        
        Ok(valid)
    }
    
    /// Restore from a backup
    pub fn restore_backup(&self, backup_id: &str, restore_path: &Path) -> io::Result<()> {
        let metadata = self.backups.get(backup_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Backup not found"))?;
        
        let backup_file = self.backup_path.join(&metadata.path);
        
        if !backup_file.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Backup file missing"));
        }
        
        log::info!("Restoring backup: {} to {:?}", backup_id, restore_path);
        
        // Verify checksum before restoring
        let current_checksum = self.calculate_checksum(&backup_file)?;
        if current_checksum != metadata.checksum {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Backup checksum mismatch"));
        }
        
        // Create restore directory
        fs::create_dir_all(restore_path)?;
        
        // Extract tar.gz archive
        let tar_gz = File::open(&backup_file)?;
        let dec = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(dec);
        archive.unpack(restore_path)?;
        
        log::info!("Backup restored successfully: {}", backup_id);
        
        Ok(())
    }
    
    /// Apply retention policy and clean up old backups
    pub fn apply_retention_policy(&mut self) -> io::Result<Vec<String>> {
        let mut deleted = Vec::new();
        
        // Group backups by type
        let mut daily: Vec<_> = self.backups.values()
            .filter(|b| b.backup_type == BackupType::Daily)
            .collect();
        let mut weekly: Vec<_> = self.backups.values()
            .filter(|b| b.backup_type == BackupType::Weekly)
            .collect();
        let mut monthly: Vec<_> = self.backups.values()
            .filter(|b| b.backup_type == BackupType::Monthly)
            .collect();
        
        // Sort by creation time (newest first)
        daily.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        weekly.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        monthly.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Delete old daily backups
        for backup in daily.iter().skip(self.retention.daily_count) {
            deleted.push(backup.id.clone());
        }
        
        // Delete old weekly backups
        for backup in weekly.iter().skip(self.retention.weekly_count) {
            deleted.push(backup.id.clone());
        }
        
        // Delete old monthly backups
        for backup in monthly.iter().skip(self.retention.monthly_count) {
            deleted.push(backup.id.clone());
        }
        
        // Delete the files and metadata
        for backup_id in &deleted {
            if let Some(metadata) = self.backups.remove(backup_id) {
                let backup_file = self.backup_path.join(&metadata.path);
                if backup_file.exists() {
                    fs::remove_file(&backup_file)?;
                    log::info!("Deleted old backup: {}", backup_id);
                }
            }
        }
        
        if !deleted.is_empty() {
            self.save_metadata()?;
        }
        
        Ok(deleted)
    }
    
    /// List all available backups
    pub fn list_backups(&self) -> Vec<BackupMetadata> {
        let mut backups: Vec<_> = self.backups.values().cloned().collect();
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Newest first
        backups
    }
    
    /// Get backup by ID
    pub fn get_backup(&self, backup_id: &str) -> Option<&BackupMetadata> {
        self.backups.get(backup_id)
    }
    
    /// Delete a specific backup
    pub fn delete_backup(&mut self, backup_id: &str) -> io::Result<()> {
        let metadata = self.backups.remove(backup_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Backup not found"))?;
        
        // Don't allow deletion of manual backups if policy says to keep them
        if metadata.backup_type == BackupType::Manual && self.retention.keep_manual {
            self.backups.insert(backup_id.to_string(), metadata);
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, 
                "Cannot delete manual backups (retention policy)"));
        }
        
        let backup_file = self.backup_path.join(&metadata.path);
        if backup_file.exists() {
            fs::remove_file(&backup_file)?;
        }
        
        self.save_metadata()?;
        log::info!("Deleted backup: {}", backup_id);
        
        Ok(())
    }
    
    /// Calculate SHA256 checksum of a file
    fn calculate_checksum(&self, path: &Path) -> io::Result<String> {
        use sha2::{Sha256, Digest};
        
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 8192];
        
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// Get backup statistics
    pub fn get_stats(&self) -> BackupStats {
        let mut stats = BackupStats::default();
        
        stats.total_backups = self.backups.len();
        
        for backup in self.backups.values() {
            stats.total_size_bytes += backup.size_bytes;
            
            match backup.backup_type {
                BackupType::Manual => stats.manual_count += 1,
                BackupType::Automatic => stats.automatic_count += 1,
                BackupType::Daily => stats.daily_count += 1,
                BackupType::Weekly => stats.weekly_count += 1,
                BackupType::Monthly => stats.monthly_count += 1,
            }
            
            if backup.verified {
                stats.verified_count += 1;
            }
        }
        
        if let Some(latest) = self.backups.values()
            .max_by_key(|b| b.created_at) {
            stats.latest_backup = Some(latest.created_at);
        }
        
        stats
    }
}

/// Backup statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_backups: usize,
    pub total_size_bytes: u64,
    pub manual_count: usize,
    pub automatic_count: usize,
    pub daily_count: usize,
    pub weekly_count: usize,
    pub monthly_count: usize,
    pub verified_count: usize,
    pub latest_backup: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_db(path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)?;
        fs::write(path.join("test.db"), b"test data")?;
        fs::write(path.join("metadata.json"), b"{\"version\": 1}")?;
        Ok(())
    }
    
    #[test]
    fn test_create_backup() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db");
        let backup_path = temp.path().join("backups");
        
        create_test_db(&db_path).unwrap();
        
        let mut manager = BackupManager::new(
            db_path.clone(),
            backup_path.clone(),
            RetentionPolicy::default(),
        ).unwrap();
        
        let metadata = manager.create_backup(Some("test_backup".to_string()), BackupType::Manual).unwrap();
        
        assert_eq!(metadata.name, Some("test_backup".to_string()));
        assert_eq!(metadata.backup_type, BackupType::Manual);
        assert!(metadata.size_bytes > 0);
        assert!(!metadata.checksum.is_empty());
        
        // Verify backup file exists
        let backup_file = backup_path.join(&metadata.path);
        assert!(backup_file.exists());
    }
    
    #[test]
    fn test_verify_backup() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db");
        let backup_path = temp.path().join("backups");
        
        create_test_db(&db_path).unwrap();
        
        let mut manager = BackupManager::new(db_path, backup_path, RetentionPolicy::default()).unwrap();
        let metadata = manager.create_backup(None, BackupType::Manual).unwrap();
        
        // Verify backup
        let valid = manager.verify_backup(&metadata.id).unwrap();
        assert!(valid);
        
        // Check that verified flag is set
        let updated_meta = manager.get_backup(&metadata.id).unwrap();
        assert!(updated_meta.verified);
    }
    
    #[test]
    fn test_restore_backup() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db");
        let backup_path = temp.path().join("backups");
        let restore_path = temp.path().join("restore");
        
        create_test_db(&db_path).unwrap();
        
        let mut manager = BackupManager::new(db_path, backup_path, RetentionPolicy::default()).unwrap();
        let metadata = manager.create_backup(None, BackupType::Manual).unwrap();
        
        // Restore backup
        manager.restore_backup(&metadata.id, &restore_path).unwrap();
        
        // Verify restored files exist
        assert!(restore_path.join("data/test.db").exists());
        assert!(restore_path.join("data/metadata.json").exists());
    }
    
    #[test]
    fn test_retention_policy() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db");
        let backup_path = temp.path().join("backups");
        
        create_test_db(&db_path).unwrap();
        
        let policy = RetentionPolicy {
            daily_count: 2,
            weekly_count: 1,
            monthly_count: 1,
            keep_manual: true,
        };
        
        let mut manager = BackupManager::new(db_path, backup_path, policy).unwrap();
        
        // Create 5 daily backups (with small delays to ensure unique timestamps)
        for i in 0..5 {
            manager.create_backup(Some(format!("daily_{}", i)), BackupType::Daily).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        assert_eq!(manager.list_backups().len(), 5);
        
        // Apply retention policy
        let deleted = manager.apply_retention_policy().unwrap();
        
        // Should keep only 2 daily backups
        assert_eq!(deleted.len(), 3);
        assert_eq!(manager.list_backups().len(), 2);
    }
    
    #[test]
    fn test_list_backups() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db");
        let backup_path = temp.path().join("backups");
        
        create_test_db(&db_path).unwrap();
        
        let mut manager = BackupManager::new(db_path, backup_path, RetentionPolicy::default()).unwrap();
        
        // Get initial backup count (should be 0 but check in case of metadata)
        let initial_count = manager.list_backups().len();
        
        manager.create_backup(Some("backup1".to_string()), BackupType::Manual).unwrap();
        manager.create_backup(Some("backup2".to_string()), BackupType::Daily).unwrap();
        manager.create_backup(Some("backup3".to_string()), BackupType::Weekly).unwrap();
        
        let backups = manager.list_backups();
        assert_eq!(backups.len(), initial_count + 3);
        
        // Find backup3 in the list (should be one of the newest)
        let backup3 = backups.iter().find(|b| b.name == Some("backup3".to_string()));
        assert!(backup3.is_some(), "backup3 should exist in backup list");
    }
    
    #[test]
    fn test_backup_stats() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("db");
        let backup_path = temp.path().join("backups");
        
        create_test_db(&db_path).unwrap();
        
        let mut manager = BackupManager::new(db_path, backup_path, RetentionPolicy::default()).unwrap();
        
        manager.create_backup(None, BackupType::Manual).unwrap();
        manager.create_backup(None, BackupType::Daily).unwrap();
        manager.create_backup(None, BackupType::Weekly).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_backups, 3);
        assert_eq!(stats.manual_count, 1);
        assert_eq!(stats.daily_count, 1);
        assert_eq!(stats.weekly_count, 1);
        assert!(stats.total_size_bytes > 0);
        assert!(stats.latest_backup.is_some());
    }
}

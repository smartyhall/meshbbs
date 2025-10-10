//! Housing abandonment cleanup task (Phase 7)
//!
//! This module implements automated lifecycle management for abandoned housing instances.
//! It provides tools for checking inactive housing and taking progressive actions based on
//! how long the owner has been away.
//!
//! Timeline:
//! * 30 days: Move items to reclaim box, notify owner (if online)
//! * 60 days: Mark housing for reclamation
//! * 80 days: Final warning (if owner logs in)
//! * 90 days: Delete reclaim box items permanently
//!
//! Design Philosophy:
//! * Trust-based with safety nets: Give owners ample time to return
//! * Progressive escalation: Warnings before irreversible actions
//! * Item preservation: 90-day retention window for recovery
//! * Admin oversight: Tools to view and manage abandoned housing

use chrono::{DateTime, Utc};
use log::{debug, info};

use crate::storage::Storage;
use crate::tmush::{TinyMushStore, TinyMushError};

/// Configuration for the housing cleanup task
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    /// Days before moving items to reclaim box (default: 30)
    pub items_to_reclaim_days: i64,
    
    /// Days before marking for reclamation (default: 60)
    pub mark_reclaim_days: i64,
    
    /// Days before final warning (default: 80)
    pub final_warning_days: i64,
    
    /// Days before permanent deletion (default: 90)
    pub permanent_deletion_days: i64,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            items_to_reclaim_days: 30,
            mark_reclaim_days: 60,
            final_warning_days: 80,
            permanent_deletion_days: 90,
        }
    }
}

/// Statistics for cleanup operations
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub checks_performed: u64,
    pub items_moved_to_reclaim: u64,
    pub housing_marked_for_reclaim: u64,
    pub reclaim_boxes_deleted: u64,
    pub warnings_issued: u64,
}

/// Admin command: List all abandoned or at-risk housing
/// 
/// This function scans all housing instances and returns those that have been
/// inactive for 20+ days (approaching the 30-day threshold).
pub async fn list_abandoned_housing(
    tmush_store: &TinyMushStore,
    storage: &Storage,
) -> Result<Vec<AbandonedHousingInfo>, TinyMushError> {
    let now = Utc::now();
    let mut results = Vec::new();
    
    let instance_ids = tmush_store.list_housing_instances()?;
    
    for instance_id in instance_ids {
        if let Ok(instance) = tmush_store.get_housing_instance(&instance_id) {
            let owner_username = &instance.owner;
            
            // Get owner's last login
            if let Ok(Some(owner)) = storage.get_user(owner_username).await {
                let days_inactive = now.signed_duration_since(owner.last_login).num_days();
                
                // Show housing that's been inactive for 20+ days (approaching 30-day threshold)
                if days_inactive >= 20 {
                    results.push(AbandonedHousingInfo {
                        instance_id: instance_id.clone(),
                        template_id: instance.template_id.clone(),
                        owner_username: owner_username.clone(),
                        days_inactive,
                        inactive_since: instance.inactive_since,
                        active: instance.active,
                        reclaim_box_items: instance.reclaim_box.len(),
                    });
                }
            }
        }
    }
    
    // Sort by days inactive (most critical first)
    results.sort_by(|a, b| b.days_inactive.cmp(&a.days_inactive));
    
    Ok(results)
}

/// Check all housing for abandonment and take appropriate actions
///
/// This function performs a full scan of all housing instances and takes
/// progressive actions based on the abandonment timeline. It's designed to be
/// called periodically (e.g., daily via cron or a background task).
pub async fn check_and_cleanup_housing(
    tmush_store: &TinyMushStore,
    storage: &Storage,
    config: &CleanupConfig,
    stats: &mut CleanupStats,
) -> Result<(), TinyMushError> {
    stats.checks_performed += 1;
    let now = Utc::now();
    
    info!("Housing cleanup check #{}: scanning all instances", stats.checks_performed);
    
    // Get all housing instances
    let instance_ids = tmush_store.list_housing_instances()?;
    let instance_count = instance_ids.len();
    
    for instance_id in instance_ids {
        if let Ok(mut instance) = tmush_store.get_housing_instance(&instance_id) {
            // Get owner's last login
            let owner_username = &instance.owner;
            if let Ok(Some(owner)) = storage.get_user(owner_username).await {
                let days_inactive = now.signed_duration_since(owner.last_login).num_days();
                
                // Phase 1: Move items to reclaim box at 30 days
                if days_inactive >= config.items_to_reclaim_days && instance.active {
                    info!(
                        "Housing {} (template: {}) owner {} inactive for {} days: marking for reclaim",
                        instance.id, instance.template_id, owner_username, days_inactive
                    );
                    
                    // Mark instance as inactive (items will be moved when housing is accessed)
                    instance.inactive_since = Some(now);
                    instance.active = false;
                    tmush_store.put_housing_instance(&instance)?;
                    
                    stats.items_moved_to_reclaim += 1;
                    
                    // TODO: Move items to reclaim box (call move_housing_to_reclaim_box from commands.rs)
                    // TODO: Send notification to owner (if online)
                }
                
                // Phase 2: Delete reclaim box items at 90 days
                if let Some(inactive_since) = instance.inactive_since {
                    let days_in_reclaim = now.signed_duration_since(inactive_since).num_days();
                    
                    if days_in_reclaim >= config.permanent_deletion_days {
                        info!(
                            "Housing {} (template: {}) inactive for {} days: clearing reclaim box",
                            instance.id, instance.template_id, days_inactive
                        );
                        
                        // Clear reclaim box (permanent deletion)
                        if !instance.reclaim_box.is_empty() {
                            let item_count = instance.reclaim_box.len();
                            instance.reclaim_box.clear();
                            tmush_store.put_housing_instance(&instance)?;
                            stats.reclaim_boxes_deleted += 1;
                            
                            info!(
                                "Deleted {} items from reclaim box for housing {}",
                                item_count, instance.id
                            );
                        }
                        
                        // TODO: Delete the housing instance entirely at this point?
                        // Or keep the shell for historical records?
                    }
                }
            }
        }
    }
    
    debug!(
        "Housing cleanup check complete: {} housing processed, {} marked, {} reclaim boxes cleared",
        instance_count, stats.items_moved_to_reclaim, stats.reclaim_boxes_deleted
    );
    
    Ok(())
}

/// Information about abandoned or at-risk housing
#[derive(Debug, Clone)]
pub struct AbandonedHousingInfo {
    pub instance_id: String,
    pub template_id: String,
    pub owner_username: String,
    pub days_inactive: i64,
    pub inactive_since: Option<DateTime<Utc>>,
    pub active: bool,
    pub reclaim_box_items: usize,
}

impl AbandonedHousingInfo {
    /// Get a status message describing the housing's abandonment state
    pub fn status_message(&self) -> String {
        match self.days_inactive {
            0..=29 => format!("⚠️  Warning: {} days inactive (approaching 30-day threshold)", self.days_inactive),
            30..=59 => "⚠️  Items moved to reclaim box (30+ days)".to_string(),
            60..=79 => "🚨 Marked for reclamation (60+ days)".to_string(),
            80..=89 => "🔴 FINAL WARNING (80+ days, deletion imminent)".to_string(),
            _ => "💀 Reclaim box deleted (90+ days)".to_string(),
        }
    }
    
    /// Format a compact one-line summary
    pub fn summary_line(&self) -> String {
        format!(
            "{} | Owner: {} | {} days | Items: {} | {}",
            self.instance_id,
            self.owner_username,
            self.days_inactive,
            self.reclaim_box_items,
            if self.active { "Active" } else { "Inactive" }
        )
    }
}

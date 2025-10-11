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
use log::{debug, info, warn};
use std::sync::Arc;

use crate::storage::Storage;
use crate::tmush::{TinyMushStore, TinyMushError, WorldConfig};

/// Notification callback for housing events
/// Arguments: (username, housing_name, message)
pub type NotificationCallback = Arc<dyn Fn(&str, &str, &str) + Send + Sync>;

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
///
/// ## Parameters
/// * `tmush_store` - TinyMUSH storage handle
/// * `storage` - Main BBS storage (for user login data)
/// * `config` - Cleanup configuration (thresholds)
/// * `stats` - Statistics tracker
/// * `world_config` - World configuration for notification messages
/// * `notification_fn` - Optional callback for sending notifications to online players
///
/// ## Notification Function
/// The notification function receives: `(username, housing_name, message_text)`
/// It should handle checking if the player is online and sending the message appropriately.
pub async fn check_and_cleanup_housing(
    tmush_store: &TinyMushStore,
    storage: &Storage,
    config: &CleanupConfig,
    stats: &mut CleanupStats,
    world_config: &WorldConfig,
    notification_fn: Option<NotificationCallback>,
) -> Result<(), TinyMushError> {
    stats.checks_performed += 1;
    let now = Utc::now();
    
    info!("Housing cleanup check #{}: scanning all instances", stats.checks_performed);
    
    // Get all housing instances
    let instance_ids = tmush_store.list_housing_instances()?;
    let instance_count = instance_ids.len();
    
    for instance_id in instance_ids {
        if let Ok(mut instance) = tmush_store.get_housing_instance(&instance_id) {
            // Get housing template for name display
            let housing_name = tmush_store.get_housing_template(&instance.template_id)
                .ok()
                .map(|t| t.name)
                .unwrap_or_else(|| instance.template_id.clone());
            
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
                    
                    // Send notification to owner (if callback provided)
                    if let Some(ref notify) = notification_fn {
                        let message = world_config.msg_housing_inactive_warning
                            .replace("{name}", &housing_name)
                            .replace("{days}", &days_inactive.to_string());
                        notify(owner_username, &housing_name, &message);
                        stats.warnings_issued += 1;
                    }
                    
                    // TODO: Move items to reclaim box (call move_housing_to_reclaim_box from commands.rs)
                }
                
                // Phase 1.5: Send warnings at 60 and 80 days
                if !instance.active && instance.inactive_since.is_some() {
                    let days_in_reclaim = now.signed_duration_since(instance.inactive_since.unwrap()).num_days();
                    
                    // 60-day warning (housing marked for reclamation)
                    if days_in_reclaim >= config.mark_reclaim_days 
                        && days_in_reclaim < config.final_warning_days 
                        && notification_fn.is_some() {
                        if let Some(ref notify) = notification_fn {
                            let message = world_config.msg_housing_inactive_warning
                                .replace("{name}", &housing_name)
                                .replace("{days}", &days_in_reclaim.to_string());
                            notify(owner_username, &housing_name, &message);
                            stats.warnings_issued += 1;
                        }
                        stats.housing_marked_for_reclaim += 1;
                    }
                    
                    // 80-day final warning
                    if days_in_reclaim >= config.final_warning_days 
                        && days_in_reclaim < config.permanent_deletion_days 
                        && notification_fn.is_some() {
                        if let Some(ref notify) = notification_fn {
                            let message = world_config.msg_housing_final_warning
                                .replace("{name}", &housing_name)
                                .replace("{days}", &days_in_reclaim.to_string());
                            notify(owner_username, &housing_name, &message);
                            stats.warnings_issued += 1;
                        }
                    }
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

/// Process recurring housing payments for all active housing instances
///
/// This function checks all housing instances to see if monthly payments are due.
/// It attempts to deduct from the player's wallet first, then bank if needed.
/// If payment fails, the housing is marked inactive and items moved to reclaim box.
///
/// ## Parameters
/// * `tmush_store` - TinyMUSH storage handle
/// * `storage` - Main BBS storage (for player data)
/// * `world_config` - World configuration for notification messages
/// * `notification_fn` - Optional callback for sending payment notifications
///
/// ## Returns
/// * `Ok((payments_processed, payments_failed))` - Count of successful and failed payments
pub async fn process_recurring_payments(
    tmush_store: &TinyMushStore,
    storage: &Storage,
    world_config: &WorldConfig,
    notification_fn: Option<NotificationCallback>,
) -> Result<(u32, u32), TinyMushError> {
    let now = Utc::now();
    let mut payments_processed = 0;
    let mut payments_failed = 0;
    
    info!("Processing recurring housing payments");
    
    // Get all housing instances
    let instance_ids = tmush_store.list_housing_instances()?;
    
    for instance_id in instance_ids {
        if let Ok(mut instance) = tmush_store.get_housing_instance(&instance_id) {
            // Skip inactive housing
            if !instance.active {
                continue;
            }
            
            // Get the template to check for recurring_cost
            let template = match tmush_store.get_housing_template(&instance.template_id) {
                Ok(t) => t,
                Err(e) => {
                    warn!("Failed to get template {} for instance {}: {}", 
                          instance.template_id, instance.id, e);
                    continue;
                }
            };
            
            // Skip if no recurring cost
            if template.recurring_cost <= 0 {
                continue;
            }
            
            // Check if payment is due (30 days since last payment)
            let days_since_payment = now.signed_duration_since(instance.last_payment).num_days();
            if days_since_payment < 30 {
                continue; // Not due yet
            }
            
            info!(
                "Housing {} ({}) payment due: {} credits (last payment {} days ago)",
                instance.id, template.name, template.recurring_cost, days_since_payment
            );
            
            // Get housing name for messages
            let housing_name = &template.name;
            let owner_username = &instance.owner;
            
            // Get owner's player record
            let mut player = match tmush_store.get_player(owner_username) {
                Ok(p) => p,
                Err(e) => {
                    warn!("Failed to get player {} for housing {}: {}", 
                          owner_username, instance.id, e);
                    continue;
                }
            };
            
            // Try to deduct payment
            let cost = template.recurring_cost as i64;
            let wallet_value = player.currency.base_value();
            let bank_value = player.banked_currency.base_value();
            let total_value = wallet_value + bank_value;
            
            if total_value >= cost {
                // Player can afford it - deduct from wallet first, then bank if needed
                let mut remaining_cost = cost;
                
                // Deduct from wallet first
                if wallet_value > 0 {
                    let to_deduct = wallet_value.min(remaining_cost);
                    let deduct_amount = match player.currency {
                        crate::tmush::types::CurrencyAmount::Decimal { .. } => 
                            crate::tmush::types::CurrencyAmount::decimal(to_deduct),
                        crate::tmush::types::CurrencyAmount::MultiTier { .. } => 
                            crate::tmush::types::CurrencyAmount::multi_tier(to_deduct),
                    };
                    
                    if let Ok(new_currency) = player.currency.subtract(&deduct_amount) {
                        player.currency = new_currency;
                        remaining_cost -= to_deduct;
                    }
                }
                
                // Deduct remainder from bank if needed
                if remaining_cost > 0 && bank_value > 0 {
                    let deduct_amount = match player.banked_currency {
                        crate::tmush::types::CurrencyAmount::Decimal { .. } => 
                            crate::tmush::types::CurrencyAmount::decimal(remaining_cost),
                        crate::tmush::types::CurrencyAmount::MultiTier { .. } => 
                            crate::tmush::types::CurrencyAmount::multi_tier(remaining_cost),
                    };
                    
                    if let Ok(new_bank) = player.banked_currency.subtract(&deduct_amount) {
                        player.banked_currency = new_bank;
                    }
                }
                
                // Update player record
                if let Err(e) = tmush_store.put_player(player.clone()) {
                    warn!("Failed to update player {} after housing payment: {}", owner_username, e);
                    payments_failed += 1;
                    continue;
                }
                
                // Update last payment date
                instance.last_payment = now;
                tmush_store.put_housing_instance(&instance)?;
                
                payments_processed += 1;
                
                info!("Successfully processed payment for housing {} ({}): {} credits", 
                      instance.id, housing_name, cost);
                
                // Send success notification
                if let Some(ref notify) = notification_fn {
                    let message = world_config.msg_housing_payment_success
                        .replace("{name}", housing_name)
                        .replace("{amount}", &cost.to_string());
                    notify(owner_username, housing_name, &message);
                }
            } else {
                // Payment failed - mark housing inactive
                warn!(
                    "Payment failed for housing {} ({}): insufficient funds ({} credits needed, {} available)",
                    instance.id, housing_name, cost, total_value
                );
                
                instance.active = false;
                instance.inactive_since = Some(now);
                tmush_store.put_housing_instance(&instance)?;
                
                payments_failed += 1;
                
                // Send failure notification
                if let Some(ref notify) = notification_fn {
                    let message = world_config.msg_housing_payment_failed
                        .replace("{name}", housing_name)
                        .replace("{amount}", &cost.to_string());
                    notify(owner_username, housing_name, &message);
                }
                
                // TODO: Move items to reclaim box
            }
        }
    }
    
    info!(
        "Recurring payment processing complete: {} successful, {} failed",
        payments_processed, payments_failed
    );
    
    Ok((payments_processed, payments_failed))
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

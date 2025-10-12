//! Shop and vendor system for TinyMUSH economy.
//!
//! This module provides shop functionality including:
//! - Shop data structures with vendor inventory
//! - Dynamic pricing with markup/markdown
//! - Buy/sell operations with currency conversion
//! - Stock management and restocking
//! - Shop persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};

use crate::tmush::types::{ObjectRecord, CurrencyAmount};

/// Configuration for shop behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopConfig {
    /// Maximum number of unique items a shop can stock
    pub max_unique_items: usize,
    /// Maximum quantity of any single item
    pub max_item_quantity: u32,
    /// Default buy markup percentage (shop sells at base_price * markup)
    pub default_buy_markup: f64,
    /// Default sell markdown percentage (shop buys at base_price * markdown)
    pub default_sell_markdown: f64,
    /// Whether shops restock automatically
    pub enable_restocking: bool,
    /// Restock interval in seconds
    pub restock_interval_secs: i64,
}

impl Default for ShopConfig {
    fn default() -> Self {
        Self {
            max_unique_items: 50,
            max_item_quantity: 999,
            default_buy_markup: 1.2,      // Sell for 120% of base
            default_sell_markdown: 0.7,   // Buy for 70% of base
            enable_restocking: true,
            restock_interval_secs: 86400, // 24 hours
        }
    }
}

/// An item available in a shop's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    /// Reference to the object being sold
    pub object_id: String,
    /// Current quantity in stock (None = infinite)
    pub quantity: Option<u32>,
    /// Price markup multiplier (overrides shop default)
    pub markup: Option<f64>,
    /// Price markdown for selling back (overrides shop default)
    pub markdown: Option<f64>,
    /// Minimum stock level before restocking
    pub restock_threshold: Option<u32>,
    /// Quantity to restock to
    pub restock_to: Option<u32>,
    /// Last restock timestamp
    pub last_restock: Option<DateTime<Utc>>,
}

impl ShopItem {
    /// Create a new shop item with default settings
    pub fn new(object_id: String, quantity: Option<u32>) -> Self {
        Self {
            object_id,
            quantity,
            markup: None,
            markdown: None,
            restock_threshold: None,
            restock_to: None,
            last_restock: Some(Utc::now()),
        }
    }

    /// Create an infinite-stock item (no quantity limit)
    pub fn infinite(object_id: String) -> Self {
        Self::new(object_id, None)
    }

    /// Create a limited-stock item
    pub fn limited(object_id: String, quantity: u32) -> Self {
        Self::new(object_id, Some(quantity))
    }

    /// Check if item is in stock
    pub fn in_stock(&self) -> bool {
        self.quantity.is_none_or(|q| q > 0)
    }

    /// Get available quantity (None = infinite)
    pub fn available(&self) -> Option<u32> {
        self.quantity
    }

    /// Reduce stock by amount (returns actual amount reduced)
    pub fn reduce_stock(&mut self, amount: u32) -> u32 {
        if let Some(qty) = self.quantity {
            let actual = amount.min(qty);
            self.quantity = Some(qty - actual);
            actual
        } else {
            amount // Infinite stock
        }
    }

    /// Increase stock by amount
    pub fn increase_stock(&mut self, amount: u32) {
        if let Some(qty) = self.quantity {
            self.quantity = Some(qty.saturating_add(amount));
        }
        // Infinite stock items ignore this
    }

    /// Check if item needs restocking
    pub fn needs_restock(&self) -> bool {
        if let (Some(qty), Some(threshold)) = (self.quantity, self.restock_threshold) {
            qty <= threshold
        } else {
            false
        }
    }

    /// Perform restock if needed
    pub fn restock(&mut self, interval_secs: i64) -> bool {
        if !self.needs_restock() {
            return false;
        }

        // Check if enough time has passed
        if let Some(last) = self.last_restock {
            let elapsed = Utc::now().signed_duration_since(last).num_seconds();
            if elapsed < interval_secs {
                return false;
            }
        }

        // Restock to target
        if let Some(target) = self.restock_to {
            self.quantity = Some(target);
            self.last_restock = Some(Utc::now());
            true
        } else {
            false
        }
    }
}

/// A shop with vendor inventory and pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopRecord {
    /// Unique shop identifier
    pub id: String,
    /// Shop name/title
    pub name: String,
    /// Shop description
    pub description: String,
    /// Location (room ID where shop is located)
    pub location: String,
    /// Shop owner (player or NPC ID)
    pub owner: String,
    /// Items for sale
    pub inventory: HashMap<String, ShopItem>,
    /// Shop's currency reserves
    pub currency: CurrencyAmount,
    /// Shop configuration
    pub config: ShopConfig,
    /// When shop was created
    pub created_at: DateTime<Utc>,
    /// When shop was last modified
    pub updated_at: DateTime<Utc>,
}

impl ShopRecord {
    /// Create a new shop
    pub fn new(
        id: String,
        name: String,
        location: String,
        owner: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description: String::new(),
            location,
            owner,
            inventory: HashMap::new(),
            currency: CurrencyAmount::default(),
            config: ShopConfig::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an item to shop inventory
    pub fn add_item(&mut self, item: ShopItem) -> Result<()> {
        if self.inventory.len() >= self.config.max_unique_items {
            return Err(anyhow!("Shop inventory full ({} items)", self.config.max_unique_items));
        }

        let object_id = item.object_id.clone();
        self.inventory.insert(object_id, item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Remove an item from shop inventory
    pub fn remove_item(&mut self, object_id: &str) -> Option<ShopItem> {
        let item = self.inventory.remove(object_id);
        if item.is_some() {
            self.updated_at = Utc::now();
        }
        item
    }

    /// Get an item from shop inventory
    pub fn get_item(&self, object_id: &str) -> Option<&ShopItem> {
        self.inventory.get(object_id)
    }

    /// Get a mutable item reference
    pub fn get_item_mut(&mut self, object_id: &str) -> Option<&mut ShopItem> {
        self.inventory.get_mut(object_id)
    }

    /// Calculate buy price (what player pays to shop)
    pub fn calculate_buy_price(&self, object: &ObjectRecord, quantity: u32, shop_item: &ShopItem) -> CurrencyAmount {
        let markup = shop_item.markup.unwrap_or(self.config.default_buy_markup);
        
        // Use currency_value if set, otherwise fall back to legacy value
        let base_value = if object.currency_value.is_positive() {
            object.currency_value.base_value()
        } else {
            object.value as i64
        };
        
        // Apply markup and multiply by quantity
        let total = (base_value as f64 * markup * quantity as f64).round() as i64;
        
        // Match the currency type from the object
        match &object.currency_value {
            CurrencyAmount::Decimal { .. } => CurrencyAmount::decimal(total),
            CurrencyAmount::MultiTier { .. } => CurrencyAmount::multi_tier(total),
        }
    }

    /// Calculate sell price (what shop pays to player)
    pub fn calculate_sell_price(&self, object: &ObjectRecord, quantity: u32, shop_item: &ShopItem) -> CurrencyAmount {
        let markdown = shop_item.markdown.unwrap_or(self.config.default_sell_markdown);
        
        // Use currency_value if set, otherwise fall back to legacy value
        let base_value = if object.currency_value.is_positive() {
            object.currency_value.base_value()
        } else {
            object.value as i64
        };
        
        // Apply markdown and multiply by quantity
        let total = (base_value as f64 * markdown * quantity as f64).round() as i64;
        
        // Match the currency type from the object
        match &object.currency_value {
            CurrencyAmount::Decimal { .. } => CurrencyAmount::decimal(total),
            CurrencyAmount::MultiTier { .. } => CurrencyAmount::multi_tier(total),
        }
    }

    /// Process a buy transaction (player buys from shop)
    /// Returns (price_paid, quantity_received)
    pub fn process_buy(
        &mut self,
        object_id: &str,
        quantity: u32,
        object: &ObjectRecord,
    ) -> Result<(CurrencyAmount, u32)> {
        // First check if item exists and get quantity info
        let (in_stock, available_qty) = {
            let shop_item = self.get_item(object_id)
                .ok_or_else(|| anyhow!("Item not in shop inventory"))?;
            (shop_item.in_stock(), shop_item.available())
        };

        if !in_stock {
            return Err(anyhow!("Item out of stock"));
        }

        // Calculate actual quantity (limited by stock)
        let actual_qty = if let Some(available) = available_qty {
            quantity.min(available)
        } else {
            quantity
        };

        if actual_qty == 0 {
            return Err(anyhow!("Cannot buy 0 items"));
        }

        // Calculate price before mutating
        let price = {
            let shop_item = self.get_item(object_id).unwrap();
            self.calculate_buy_price(object, actual_qty, shop_item)
        };

        // Now mutate: reduce shop stock
        let shop_item = self.get_item_mut(object_id).unwrap();
        shop_item.reduce_stock(actual_qty);
        
        // Add currency to shop
        self.currency = self.currency.add(&price)
            .map_err(|e| anyhow!("Currency overflow: {}", e))?;
        
        self.updated_at = Utc::now();
        Ok((price, actual_qty))
    }

    /// Process a sell transaction (player sells to shop)
    /// Returns price_paid
    pub fn process_sell(
        &mut self,
        object_id: &str,
        quantity: u32,
        object: &ObjectRecord,
    ) -> Result<CurrencyAmount> {
        // Check if shop accepts this item and calculate price before mutating
        let price = {
            let shop_item = self.get_item(object_id)
                .ok_or_else(|| anyhow!("Shop does not buy this item"))?;
            self.calculate_sell_price(object, quantity, shop_item)
        };

        // Check if shop has enough currency
        if !self.currency.can_afford(&price) {
            return Err(anyhow!("Shop cannot afford to buy this item"));
        }

        // Now mutate: increase shop stock
        let shop_item = self.get_item_mut(object_id).unwrap();
        shop_item.increase_stock(quantity);
        
        // Deduct currency from shop
        self.currency = self.currency.subtract(&price)
            .map_err(|e| anyhow!("Currency error: {}", e))?;
        
        self.updated_at = Utc::now();
        Ok(price)
    }

    /// Restock all items that need it
    pub fn restock_all(&mut self) -> usize {
        let mut restocked = 0;
        for item in self.inventory.values_mut() {
            if item.restock(self.config.restock_interval_secs) {
                restocked += 1;
            }
        }
        if restocked > 0 {
            self.updated_at = Utc::now();
        }
        restocked
    }

    /// List all items for sale
    pub fn list_items(&self) -> Vec<(&String, &ShopItem)> {
        self.inventory.iter().collect()
    }
}

/// Format shop inventory listing (compact for Meshtastic)
pub fn format_shop_listing(
    shop: &ShopRecord,
    get_object: impl Fn(&str) -> Option<ObjectRecord>,
) -> Vec<String> {
    let mut lines = Vec::new();
    
    lines.push(format!("=== {} ===", shop.name));
    
    if shop.inventory.is_empty() {
        lines.push("No items for sale.".to_string());
        return lines;
    }

    for (idx, (object_id, shop_item)) in shop.inventory.iter().enumerate() {
        if let Some(object) = get_object(object_id) {
            let price = shop.calculate_buy_price(&object, 1, shop_item);
            let stock_str = if let Some(qty) = shop_item.quantity {
                format!(" ({})", qty)
            } else {
                String::new()
            };
            
            let price_str = match price {
                CurrencyAmount::Decimal { minor_units } => format!("${:.2}", minor_units as f64 / 100.0),
                CurrencyAmount::MultiTier { base_units } => format!("{}c", base_units),
            };
            
            lines.push(format!(
                "{}. {}{} - {}",
                idx + 1,
                object.name,
                stock_str,
                price_str
            ));
        }
    }

    lines
}

/// Format shop item details for examination
pub fn format_shop_item_detail(
    shop: &ShopRecord,
    object: &ObjectRecord,
    shop_item: &ShopItem,
) -> Vec<String> {
    let mut lines = Vec::new();
    
    lines.push(format!("=== {} ===", object.name));
    lines.push(object.description.clone());
    
    let buy_price = shop.calculate_buy_price(object, 1, shop_item);
    let sell_price = shop.calculate_sell_price(object, 1, shop_item);
    
    let buy_str = match buy_price {
        CurrencyAmount::Decimal { minor_units } => format!("${:.2}", minor_units as f64 / 100.0),
        CurrencyAmount::MultiTier { base_units } => format!("{}c", base_units),
    };
    
    let sell_str = match sell_price {
        CurrencyAmount::Decimal { minor_units } => format!("${:.2}", minor_units as f64 / 100.0),
        CurrencyAmount::MultiTier { base_units } => format!("{}c", base_units),
    };
    
    lines.push(format!("Buy: {}", buy_str));
    lines.push(format!("Sell: {}", sell_str));
    
    if let Some(qty) = shop_item.quantity {
        lines.push(format!("Stock: {}", qty));
    } else {
        lines.push("Stock: Unlimited".to_string());
    }
    
    lines.push(format!("Weight: {}", object.weight));
    
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmush::types::ObjectOwner;

    fn test_object() -> ObjectRecord {
        ObjectRecord {
            id: "sword1".to_string(),
            name: "Iron Sword".to_string(),
            description: "A basic iron sword".to_string(),
            owner: ObjectOwner::World,
            created_at: Utc::now(),
            weight: 5,
            currency_value: CurrencyAmount::decimal(100), // 100 pennies = $1.00
            value: 100, // Legacy value
            takeable: true,
            usable: false,
            actions: HashMap::new(),
            clone_depth: 0,
            clone_source_id: None,
            clone_count: 0,
            created_by: "world".to_string(),
            flags: Vec::new(),
            locked: false, // Shop items unlocked by default
            ownership_history: vec![], // New items have no history
            schema_version: 1,
        }
    }

    #[test]
    fn test_shop_item_stock_management() {
        let mut item = ShopItem::limited("sword1".to_string(), 10);
        
        assert!(item.in_stock());
        assert_eq!(item.available(), Some(10));
        
        let reduced = item.reduce_stock(3);
        assert_eq!(reduced, 3);
        assert_eq!(item.available(), Some(7));
        
        item.increase_stock(5);
        assert_eq!(item.available(), Some(12));
    }

    #[test]
    fn test_shop_item_infinite_stock() {
        let mut item = ShopItem::infinite("potion1".to_string());
        
        assert!(item.in_stock());
        assert_eq!(item.available(), None);
        
        let reduced = item.reduce_stock(100);
        assert_eq!(reduced, 100);
        assert!(item.in_stock());
        assert_eq!(item.available(), None);
    }

    #[test]
    fn test_shop_pricing() {
        let shop = ShopRecord::new(
            "shop1".to_string(),
            "Weapon Shop".to_string(),
            "room1".to_string(),
            "vendor1".to_string(),
        );
        
        let object = test_object();
        let shop_item = ShopItem::limited("sword1".to_string(), 5);
        
        // Default markup 1.2 = 120 pennies
        let buy_price = shop.calculate_buy_price(&object, 1, &shop_item);
        assert_eq!(buy_price.base_value(), 120);
        
        // Default markdown 0.7 = 70 pennies
        let sell_price = shop.calculate_sell_price(&object, 1, &shop_item);
        assert_eq!(sell_price.base_value(), 70);
    }

    #[test]
    fn test_shop_buy_transaction() {
        let mut shop = ShopRecord::new(
            "shop1".to_string(),
            "Weapon Shop".to_string(),
            "room1".to_string(),
            "vendor1".to_string(),
        );
        
        let object = test_object();
        let shop_item = ShopItem::limited("sword1".to_string(), 5);
        shop.add_item(shop_item).unwrap();
        
        let result = shop.process_buy("sword1", 2, &object);
        assert!(result.is_ok());
        
        let (price, qty) = result.unwrap();
        assert_eq!(qty, 2);
        assert_eq!(price.base_value(), 240); // 120 * 2
        
        // Check stock reduced
        let item = shop.get_item("sword1").unwrap();
        assert_eq!(item.available(), Some(3));
    }

    #[test]
    fn test_shop_buy_exceeds_stock() {
        let mut shop = ShopRecord::new(
            "shop1".to_string(),
            "Weapon Shop".to_string(),
            "room1".to_string(),
            "vendor1".to_string(),
        );
        
        let object = test_object();
        let shop_item = ShopItem::limited("sword1".to_string(), 2);
        shop.add_item(shop_item).unwrap();
        
        // Try to buy 5, only 2 available
        let result = shop.process_buy("sword1", 5, &object);
        assert!(result.is_ok());
        
        let (price, qty) = result.unwrap();
        assert_eq!(qty, 2); // Only 2 were available
        assert_eq!(price.base_value(), 240); // 120 * 2
    }

    #[test]
    fn test_shop_sell_transaction() {
        let mut shop = ShopRecord::new(
            "shop1".to_string(),
            "Weapon Shop".to_string(),
            "room1".to_string(),
            "vendor1".to_string(),
        );
        
        // Give shop currency
        shop.currency = shop.currency.add(&CurrencyAmount::decimal(1000)).unwrap();
        
        let object = test_object();
        let shop_item = ShopItem::limited("sword1".to_string(), 0);
        shop.add_item(shop_item).unwrap();
        
        let result = shop.process_sell("sword1", 2, &object);
        assert!(result.is_ok());
        
        let price = result.unwrap();
        assert_eq!(price.base_value(), 140); // 70 * 2
        
        // Check stock increased
        let item = shop.get_item("sword1").unwrap();
        assert_eq!(item.available(), Some(2));
        
        // Check shop currency reduced
        assert_eq!(shop.currency.base_value(), 860); // 1000 - 140
    }

    #[test]
    fn test_shop_sell_insufficient_funds() {
        let mut shop = ShopRecord::new(
            "shop1".to_string(),
            "Weapon Shop".to_string(),
            "room1".to_string(),
            "vendor1".to_string(),
        );
        
        // Shop only has 50 pennies
        shop.currency = shop.currency.add(&CurrencyAmount::decimal(50)).unwrap();
        
        let object = test_object();
        let shop_item = ShopItem::limited("sword1".to_string(), 0);
        shop.add_item(shop_item).unwrap();
        
        // Try to sell for 70 pennies
        let result = shop.process_sell("sword1", 1, &object);
        assert!(result.is_err());
    }

    #[test]
    fn test_shop_restocking() {
        let mut item = ShopItem::limited("sword1".to_string(), 2);
        item.restock_threshold = Some(5);
        item.restock_to = Some(10);
        item.last_restock = Some(Utc::now() - chrono::Duration::days(2));
        
        // Should restock (below threshold and time elapsed)
        assert!(item.needs_restock());
        assert!(item.restock(86400)); // 24 hour interval
        assert_eq!(item.available(), Some(10));
    }
}

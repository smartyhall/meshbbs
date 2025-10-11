// Currency Migration System
// Provides tools for converting between Decimal and MultiTier currency systems

use crate::tmush::types::CurrencyAmount;
use crate::tmush::storage::TinyMushStore;

/// Result of a currency conversion operation
#[derive(Debug, Default, Clone)]
pub struct ConversionResult {
    pub success_count: usize,
    pub failure_count: usize,
    pub total_converted: i64,
    pub errors: Vec<String>,
}

impl ConversionResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_success(&mut self, amount: i64) {
        self.success_count += 1;
        self.total_converted += amount;
    }

    pub fn add_failure(&mut self, error: String) {
        self.failure_count += 1;
        self.errors.push(error);
    }

    pub fn merge(&mut self, other: ConversionResult) {
        self.success_count += other.success_count;
        self.failure_count += other.failure_count;
        self.total_converted += other.total_converted;
        self.errors.extend(other.errors);
    }
}

/// Convert Decimal currency to MultiTier currency
pub fn convert_decimal_to_multitier(amount: &CurrencyAmount) -> Result<CurrencyAmount, String> {
    match amount {
        CurrencyAmount::Decimal { minor_units } => {
            Ok(CurrencyAmount::MultiTier { base_units: *minor_units })
        }
        CurrencyAmount::MultiTier { .. } => {
            Err("Already in MultiTier format".to_string())
        }
    }
}

/// Convert MultiTier currency to Decimal currency
pub fn convert_multitier_to_decimal(amount: &CurrencyAmount) -> Result<CurrencyAmount, String> {
    match amount {
        CurrencyAmount::MultiTier { base_units } => {
            Ok(CurrencyAmount::Decimal { minor_units: *base_units })
        }
        CurrencyAmount::Decimal { .. } => {
            Err("Already in Decimal format".to_string())
        }
    }
}

/// Convert all player wallets
pub fn convert_all_player_wallets(
    storage: &TinyMushStore,
    to_multitier: bool,
    dry_run: bool,
) -> Result<ConversionResult, String> {
    let mut result = ConversionResult::new();
    
    let player_ids = storage.list_player_ids()
        .map_err(|e| format!("Failed to list players: {}", e))?;
    
    for username in player_ids.iter() {
        match convert_player_wallet(storage, username, to_multitier, dry_run) {
            Ok(amount) => result.add_success(amount),
            Err(e) => result.add_failure(format!("Player {}: {}", username, e)),
        }
    }
    
    Ok(result)
}

fn convert_player_wallet(
    storage: &TinyMushStore,
    username: &str,
    to_multitier: bool,
    dry_run: bool,
) -> Result<i64, String> {
    let mut player = storage.get_player(username)
        .map_err(|e| format!("Failed to get player: {}", e))?;
    
    // Convert player's on-hand currency
    let converted_currency = if to_multitier {
        convert_decimal_to_multitier(&player.currency)?
    } else {
        convert_multitier_to_decimal(&player.currency)?
    };
    
    let base_value = converted_currency.base_value();
    
    if !dry_run {
        player.currency = converted_currency;
        storage.put_player(player)
            .map_err(|e| format!("Failed to update player: {}", e))?;
    }
    
    Ok(base_value)
}

pub fn convert_all_bank_accounts(
    storage: &TinyMushStore,
    to_multitier: bool,
    dry_run: bool,
) -> Result<ConversionResult, String> {
    let mut result = ConversionResult::new();
    
    let player_ids = storage.list_player_ids()
        .map_err(|e| format!("Failed to list players: {}", e))?;
    
    for username in player_ids.iter() {
        match convert_player_bank(storage, username, to_multitier, dry_run) {
            Ok(amount) => result.add_success(amount),
            Err(e) => result.add_failure(format!("Player {}: {}", username, e)),
        }
    }
    
    Ok(result)
}

fn convert_player_bank(
    storage: &TinyMushStore,
    username: &str,
    to_multitier: bool,
    dry_run: bool,
) -> Result<i64, String> {
    let mut player = storage.get_player(username)
        .map_err(|e| format!("Failed to get player: {}", e))?;
    
    // Convert player's banked currency
    let converted_bank = if to_multitier {
        convert_decimal_to_multitier(&player.banked_currency)?
    } else {
        convert_multitier_to_decimal(&player.banked_currency)?
    };
    
    let base_value = converted_bank.base_value();
    
    if !dry_run {
        player.banked_currency = converted_bank;
        storage.put_player(player)
            .map_err(|e| format!("Failed to update player: {}", e))?;
    }
    
    Ok(base_value)
}

/// Convert all shop currency reserves
pub fn convert_all_shop_currency(
    storage: &TinyMushStore,
    to_multitier: bool,
    dry_run: bool,
) -> Result<ConversionResult, String> {
    let mut result = ConversionResult::new();
    
    let shop_ids = storage.list_shop_ids()
        .map_err(|e| format!("Failed to list shops: {}", e))?;
    
    for shop_id in shop_ids.iter() {
        match convert_shop_currency(storage, shop_id, to_multitier, dry_run) {
            Ok(amount) => result.add_success(amount),
            Err(e) => result.add_failure(format!("Shop {}: {}", shop_id, e)),
        }
    }
    
    Ok(result)
}

/// Convert a single shop's currency reserves
fn convert_shop_currency(
    storage: &TinyMushStore,
    shop_id: &str,
    to_multitier: bool,
    dry_run: bool,
) -> Result<i64, String> {
    let mut shop = storage.get_shop(shop_id)
        .map_err(|e| format!("Failed to get shop: {}", e))?;
    
    // Convert the shop's currency reserves
    let converted_currency = if to_multitier {
        convert_decimal_to_multitier(&shop.currency)?
    } else {
        convert_multitier_to_decimal(&shop.currency)?
    };
    
    let base_value = converted_currency.base_value();
    
    if !dry_run {
        shop.currency = converted_currency;
        storage.put_shop(shop)
            .map_err(|e| format!("Failed to update shop: {}", e))?;
    }
    
    Ok(base_value)
}

/// Migrate all currency in the world
/// This is a comprehensive migration that converts all currency types:
/// - Player currency (on hand)
/// - Player banked currency
/// - Shop currency reserves
pub fn migrate_all_currency(
    storage: &TinyMushStore,
    to_multitier: bool,
    dry_run: bool,
) -> Result<ConversionResult, String> {
    let mut result = ConversionResult::new();
    
    result.merge(convert_all_player_wallets(storage, to_multitier, dry_run)?);
    result.merge(convert_all_bank_accounts(storage, to_multitier, dry_run)?);
    result.merge(convert_all_shop_currency(storage, to_multitier, dry_run)?);
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_to_multitier() {
        let decimal = CurrencyAmount::Decimal { minor_units: 150 };
        let result = convert_decimal_to_multitier(&decimal).unwrap();
        
        match result {
            CurrencyAmount::MultiTier { base_units } => {
                assert_eq!(base_units, 150);
            }
            _ => panic!("Expected MultiTier currency"),
        }
    }

    #[test]
    fn test_multitier_to_decimal() {
        let multitier = CurrencyAmount::MultiTier { base_units: 250 };
        let result = convert_multitier_to_decimal(&multitier).unwrap();
        
        match result {
            CurrencyAmount::Decimal { minor_units } => {
                assert_eq!(minor_units, 250);
            }
            _ => panic!("Expected Decimal currency"),
        }
    }

    #[test]
    fn test_decimal_to_multitier_already_converted() {
        let multitier = CurrencyAmount::MultiTier { base_units: 100 };
        let result = convert_decimal_to_multitier(&multitier);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Already in MultiTier format");
    }

    #[test]
    fn test_multitier_to_decimal_already_converted() {
        let decimal = CurrencyAmount::Decimal { minor_units: 100 };
        let result = convert_multitier_to_decimal(&decimal);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Already in Decimal format");
    }

    #[test]
    fn test_conversion_result_merge() {
        let mut result1 = ConversionResult::new();
        result1.add_success(100);
        result1.add_failure("Error 1".to_string());
        
        let mut result2 = ConversionResult::new();
        result2.add_success(200);
        result2.add_success(300);
        result2.add_failure("Error 2".to_string());
        
        result1.merge(result2);
        
        assert_eq!(result1.success_count, 3);
        assert_eq!(result1.failure_count, 2);
        assert_eq!(result1.total_converted, 600);
        assert_eq!(result1.errors.len(), 2);
    }
}

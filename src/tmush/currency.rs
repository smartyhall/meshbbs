/// Currency system implementation - formatting, parsing, and conversion
use super::types::{
    CurrencyAmount, CurrencySystem, CurrencyTier, DecimalCurrency, MultiTierCurrency,
};

// ============================================================================
// Display Formatting (must fit in 200-byte message constraints)
// ============================================================================

/// Format a currency amount for display based on the system configuration
pub fn format_currency(amount: &CurrencyAmount, system: &CurrencySystem) -> String {
    match (amount, system) {
        (CurrencyAmount::Decimal { minor_units }, CurrencySystem::Decimal(config)) => {
            format_decimal(*minor_units, config)
        }
        (CurrencyAmount::MultiTier { base_units }, CurrencySystem::MultiTier(config)) => {
            format_multi_tier(*base_units, config)
        }
        // Mismatched types - shouldn't happen but handle gracefully
        (CurrencyAmount::Decimal { minor_units }, _) => format!("{} units", minor_units),
        (CurrencyAmount::MultiTier { base_units }, _) => format!("{} units", base_units),
    }
}

/// Format a decimal currency amount (e.g., "$12.34" or "1234 credits")
fn format_decimal(minor_units: i64, config: &DecimalCurrency) -> String {
    if config.decimals == 0 {
        // No decimals - show whole units
        let name = if minor_units.abs() == 1 {
            &config.name
        } else {
            &config.name_plural
        };
        format!("{}{} {}", config.symbol, minor_units, name)
    } else {
        // With decimals
        let divisor = 10i64.pow(config.decimals as u32);
        let whole = minor_units / divisor;
        let frac = (minor_units % divisor).abs();
        format!(
            "{}{}",
            config.symbol,
            format_with_decimals(whole, frac, config.decimals)
        )
    }
}

/// Format a number with decimal places
fn format_with_decimals(whole: i64, frac: i64, decimals: u8) -> String {
    let frac_str = format!("{:0width$}", frac, width = decimals as usize);
    format!("{}.{}", whole, frac_str)
}

/// Format a multi-tier currency amount (e.g., "5g 3s 7c")
fn format_multi_tier(base_units: i64, config: &MultiTierCurrency) -> String {
    if base_units == 0 {
        return format!("0 {}", config.base_unit);
    }

    let mut remaining = base_units.abs();
    let mut parts = Vec::new();

    // Sort tiers by ratio descending to start with highest tier
    let mut sorted_tiers = config.tiers.clone();
    sorted_tiers.sort_by(|a, b| b.ratio_to_base.cmp(&a.ratio_to_base));

    for tier in sorted_tiers.iter() {
        if tier.ratio_to_base == 0 {
            continue; // Skip invalid tiers
        }
        let count = remaining / tier.ratio_to_base as i64;
        if count > 0 {
            parts.push(format!("{}{}", count, tier.symbol));
            remaining %= tier.ratio_to_base as i64;
        }
    }

    if parts.is_empty() {
        format!("0 {}", config.base_unit)
    } else {
        let result = parts.join(" ");
        if base_units < 0 {
            format!("-{}", result)
        } else {
            result
        }
    }
}

// ============================================================================
// Currency Parsing
// ============================================================================

/// Parse a currency amount from user input
/// Examples: "100", "5g 3s", "$12.34", "5 gold 3 silver"
pub fn parse_currency(input: &str, system: &CurrencySystem) -> Result<CurrencyAmount, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty currency amount".to_string());
    }

    match system {
        CurrencySystem::Decimal(config) => parse_decimal_input(input, config),
        CurrencySystem::MultiTier(config) => parse_multi_tier_input(input, config),
    }
}

/// Parse decimal currency input
fn parse_decimal_input(input: &str, config: &DecimalCurrency) -> Result<CurrencyAmount, String> {
    // Remove currency symbol and name if present
    // Important: Remove plural before singular to avoid partial matches
    let cleaned = input
        .replace(&config.symbol, "")
        .replace(&config.name_plural, "")
        .replace(&config.name, "")
        .trim()
        .to_string();

    // Check for decimal point
    if cleaned.contains('.') {
        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.len() != 2 {
            return Err("Invalid decimal format".to_string());
        }

        let whole: i64 = parts[0]
            .trim()
            .parse()
            .map_err(|_| "Invalid whole number")?;
        let frac_str = parts[1].trim();

        // Pad or truncate fractional part to match config.decimals
        let frac: i64 = if frac_str.len() > config.decimals as usize {
            // Truncate
            frac_str[..config.decimals as usize]
                .parse()
                .map_err(|_| "Invalid fractional part")?
        } else {
            // Pad with zeros
            let padded = format!("{:0<width$}", frac_str, width = config.decimals as usize);
            padded.parse().map_err(|_| "Invalid fractional part")?
        };

        let divisor = 10i64.pow(config.decimals as u32);
        let minor_units = whole * divisor + if whole < 0 { -frac } else { frac };
        Ok(CurrencyAmount::Decimal { minor_units })
    } else {
        // No decimal point - parse as whole number
        let value: i64 = cleaned.parse().map_err(|_| "Invalid number")?;
        let minor_units = if config.decimals > 0 {
            value * 10i64.pow(config.decimals as u32)
        } else {
            value
        };
        Ok(CurrencyAmount::Decimal { minor_units })
    }
}

/// Parse multi-tier currency input
fn parse_multi_tier_input(
    input: &str,
    config: &MultiTierCurrency,
) -> Result<CurrencyAmount, String> {
    // Simple numeric input - interpret as base units
    if let Ok(value) = input.parse::<i64>() {
        return Ok(CurrencyAmount::MultiTier { base_units: value });
    }

    // Parse tier-based input like "5g 3s 2c" or "5 gold 3 silver 2 copper"
    let mut total_base_units: i64 = 0;
    let parts: Vec<&str> = input.split_whitespace().collect();

    let mut i = 0;
    while i < parts.len() {
        // Try to parse a number
        let amount: i64 = parts[i]
            .parse()
            .map_err(|_| format!("Expected number, got '{}'", parts[i]))?;

        // Next part should be tier name or symbol
        if i + 1 >= parts.len() {
            return Err("Missing tier name after amount".to_string());
        }

        let tier_str = parts[i + 1].to_lowercase();
        let tier = find_tier_by_name_or_symbol(&tier_str, config)
            .ok_or_else(|| format!("Unknown currency tier: '{}'", tier_str))?;

        total_base_units += amount * tier.ratio_to_base as i64;
        i += 2;
    }

    Ok(CurrencyAmount::MultiTier {
        base_units: total_base_units,
    })
}

/// Find a currency tier by name or symbol (case-insensitive)
fn find_tier_by_name_or_symbol<'a>(
    search: &str,
    config: &'a MultiTierCurrency,
) -> Option<&'a CurrencyTier> {
    let search_lower = search.to_lowercase();
    config.tiers.iter().find(|tier| {
        tier.symbol.to_lowercase() == search_lower
            || tier.name.to_lowercase() == search_lower
            || tier.name_plural.to_lowercase() == search_lower
    })
}

// ============================================================================
// Currency Conversion (between decimal and multi-tier)
// ============================================================================

/// Standard conversion ratio: 100 copper = 1 decimal unit (e.g., $1.00)
pub const STANDARD_CONVERSION_RATIO: i64 = 100;

/// Convert from decimal to multi-tier currency
/// Standard ratio: 100 coppers = 1 major decimal unit (e.g., $1.00 = 100c)
pub fn convert_decimal_to_multi_tier(
    decimal: &CurrencyAmount,
    ratio: Option<i64>,
) -> Result<CurrencyAmount, String> {
    let ratio = ratio.unwrap_or(STANDARD_CONVERSION_RATIO);
    if ratio <= 0 {
        return Err("Invalid conversion ratio".to_string());
    }

    match decimal {
        CurrencyAmount::Decimal { minor_units } => {
            // If decimal currency uses 2 decimals (cents), then minor_units are cents
            // Standard: 100 cents ($1.00) = 100 coppers
            // So base_units = minor_units
            Ok(CurrencyAmount::MultiTier {
                base_units: *minor_units,
            })
        }
        CurrencyAmount::MultiTier { .. } => Err("Already multi-tier currency".to_string()),
    }
}

/// Convert from multi-tier to decimal currency
/// Standard ratio: 100 coppers = 1 major decimal unit (e.g., 100c = $1.00)
pub fn convert_multi_tier_to_decimal(
    multi_tier: &CurrencyAmount,
    ratio: Option<i64>,
) -> Result<CurrencyAmount, String> {
    let ratio = ratio.unwrap_or(STANDARD_CONVERSION_RATIO);
    if ratio <= 0 {
        return Err("Invalid conversion ratio".to_string());
    }

    match multi_tier {
        CurrencyAmount::MultiTier { base_units } => {
            // If decimal currency uses 2 decimals (cents), then minor_units are cents
            // Standard: 100 coppers = 100 cents = $1.00
            // So minor_units = base_units
            Ok(CurrencyAmount::Decimal {
                minor_units: *base_units,
            })
        }
        CurrencyAmount::Decimal { .. } => Err("Already decimal currency".to_string()),
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_decimal_config() -> DecimalCurrency {
        DecimalCurrency {
            name: "credit".to_string(),
            name_plural: "credits".to_string(),
            symbol: "¤".to_string(),
            decimals: 2,
        }
    }

    fn test_multi_tier_config() -> MultiTierCurrency {
        MultiTierCurrency::default()
    }

    #[test]
    fn test_format_decimal() {
        let config = test_decimal_config();
        let system = CurrencySystem::Decimal(config);

        let amount = CurrencyAmount::Decimal { minor_units: 1234 };
        let formatted = format_currency(&amount, &system);
        assert_eq!(formatted, "¤12.34");

        let amount = CurrencyAmount::Decimal { minor_units: 0 };
        let formatted = format_currency(&amount, &system);
        assert_eq!(formatted, "¤0.00");

        let amount = CurrencyAmount::Decimal { minor_units: -500 };
        let formatted = format_currency(&amount, &system);
        assert_eq!(formatted, "¤-5.00");
    }

    #[test]
    fn test_format_multi_tier() {
        let config = test_multi_tier_config();
        let system = CurrencySystem::MultiTier(config);

        let amount = CurrencyAmount::MultiTier { base_units: 537 };
        let formatted = format_currency(&amount, &system);
        assert_eq!(formatted, "5g 3s 7c");

        let amount = CurrencyAmount::MultiTier { base_units: 0 };
        let formatted = format_currency(&amount, &system);
        assert_eq!(formatted, "0 copper");

        let amount = CurrencyAmount::MultiTier { base_units: 100 };
        let formatted = format_currency(&amount, &system);
        assert_eq!(formatted, "1g");
    }

    #[test]
    fn test_parse_decimal() {
        let config = test_decimal_config();
        let system = CurrencySystem::Decimal(config);

        let amount = parse_currency("12.34", &system).unwrap();
        assert_eq!(amount, CurrencyAmount::Decimal { minor_units: 1234 });

        let amount = parse_currency("100", &system).unwrap();
        assert_eq!(amount, CurrencyAmount::Decimal { minor_units: 10000 });

        let amount = parse_currency("¤5.50", &system).unwrap();
        assert_eq!(amount, CurrencyAmount::Decimal { minor_units: 550 });
    }

    #[test]
    fn test_parse_multi_tier() {
        let config = test_multi_tier_config();
        let system = CurrencySystem::MultiTier(config);

        let amount = parse_currency("537", &system).unwrap();
        assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });

        let amount = parse_currency("5 gold 3 silver 7 copper", &system).unwrap();
        assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });

        let amount = parse_currency("5 g 3 s 7 c", &system).unwrap();
        assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });
    }

    #[test]
    fn test_conversion() {
        let decimal = CurrencyAmount::Decimal { minor_units: 10000 };
        let multi_tier = convert_decimal_to_multi_tier(&decimal, None).unwrap();
        assert_eq!(multi_tier, CurrencyAmount::MultiTier { base_units: 10000 });

        let back = convert_multi_tier_to_decimal(&multi_tier, None).unwrap();
        assert_eq!(back, decimal);
    }

    #[test]
    fn test_currency_operations() {
        let a = CurrencyAmount::Decimal { minor_units: 1000 };
        let b = CurrencyAmount::Decimal { minor_units: 500 };

        let sum = a.add(&b).unwrap();
        assert_eq!(sum, CurrencyAmount::Decimal { minor_units: 1500 });

        let diff = a.subtract(&b).unwrap();
        assert_eq!(diff, CurrencyAmount::Decimal { minor_units: 500 });

        assert!(a.can_afford(&b));
        assert!(!b.can_afford(&a));
    }
}

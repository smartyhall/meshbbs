/// Tests for the dual currency system (decimal and multi-tier)
use tempfile::TempDir;

use meshbbs::tmush::{
    format_currency, parse_currency, CurrencyAmount, CurrencySystem, CurrencyTier,
    DecimalCurrency, MultiTierCurrency, PlayerRecord, TinyMushError, TinyMushStoreBuilder,
    TransactionReason,
};

#[test]
fn test_decimal_currency_formatting() {
    let config = DecimalCurrency {
        name: "credit".to_string(),
        name_plural: "credits".to_string(),
        symbol: "¤".to_string(),
        decimals: 2,
    };
    let system = CurrencySystem::Decimal(config);

    // Positive amount
    let amount = CurrencyAmount::Decimal { minor_units: 1234 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "¤12.34");

    // Zero
    let amount = CurrencyAmount::Decimal { minor_units: 0 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "¤0.00");

    // Negative amount
    let amount = CurrencyAmount::Decimal { minor_units: -500 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "¤-5.00");

    // Large amount
    let amount = CurrencyAmount::Decimal {
        minor_units: 1_000_000,
    };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "¤10000.00");
}

#[test]
fn test_decimal_currency_no_decimals() {
    let config = DecimalCurrency {
        name: "credit".to_string(),
        name_plural: "credits".to_string(),
        symbol: "¤".to_string(),
        decimals: 0,
    };
    let system = CurrencySystem::Decimal(config);

    let amount = CurrencyAmount::Decimal { minor_units: 100 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "¤100 credits");

    let amount = CurrencyAmount::Decimal { minor_units: 1 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "¤1 credit");
}

#[test]
fn test_multi_tier_currency_formatting() {
    let config = MultiTierCurrency {
        tiers: vec![
            CurrencyTier {
                name: "copper".to_string(),
                name_plural: "coppers".to_string(),
                symbol: "c".to_string(),
                ratio_to_base: 1,
            },
            CurrencyTier {
                name: "silver".to_string(),
                name_plural: "silvers".to_string(),
                symbol: "s".to_string(),
                ratio_to_base: 10,
            },
            CurrencyTier {
                name: "gold".to_string(),
                name_plural: "golds".to_string(),
                symbol: "g".to_string(),
                ratio_to_base: 100,
            },
        ],
        base_unit: "copper".to_string(),
    };
    let system = CurrencySystem::MultiTier(config);

    // Mixed tiers
    let amount = CurrencyAmount::MultiTier { base_units: 537 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "5g 3s 7c");

    // Single tier
    let amount = CurrencyAmount::MultiTier { base_units: 100 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "1g");

    // Zero
    let amount = CurrencyAmount::MultiTier { base_units: 0 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "0 copper");

    // Only lower tiers
    let amount = CurrencyAmount::MultiTier { base_units: 37 };
    let formatted = format_currency(&amount, &system);
    assert_eq!(formatted, "3s 7c");
}

#[test]
fn test_decimal_currency_parsing() {
    let config = DecimalCurrency {
        name: "credit".to_string(),
        name_plural: "credits".to_string(),
        symbol: "¤".to_string(),
        decimals: 2,
    };
    let system = CurrencySystem::Decimal(config);

    // With decimal point
    let amount = parse_currency("12.34", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::Decimal { minor_units: 1234 });

    // Whole number
    let amount = parse_currency("100", &system).unwrap();
    assert_eq!(
        amount,
        CurrencyAmount::Decimal {
            minor_units: 10000
        }
    );

    // With symbol
    let amount = parse_currency("¤5.50", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::Decimal { minor_units: 550 });

    // With currency name
    let amount = parse_currency("10 credits", &system).unwrap();
    assert_eq!(
        amount,
        CurrencyAmount::Decimal {
            minor_units: 1000
        }
    );

    // Edge case: just decimals
    let amount = parse_currency("0.99", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::Decimal { minor_units: 99 });
}

#[test]
fn test_multi_tier_currency_parsing() {
    let config = MultiTierCurrency {
        tiers: vec![
            CurrencyTier {
                name: "copper".to_string(),
                name_plural: "coppers".to_string(),
                symbol: "c".to_string(),
                ratio_to_base: 1,
            },
            CurrencyTier {
                name: "silver".to_string(),
                name_plural: "silvers".to_string(),
                symbol: "s".to_string(),
                ratio_to_base: 10,
            },
            CurrencyTier {
                name: "gold".to_string(),
                name_plural: "golds".to_string(),
                symbol: "g".to_string(),
                ratio_to_base: 100,
            },
        ],
        base_unit: "copper".to_string(),
    };
    let system = CurrencySystem::MultiTier(config);

    // Simple numeric
    let amount = parse_currency("537", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });

    // Full names
    let amount = parse_currency("5 gold 3 silver 7 copper", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });

    // Symbols
    let amount = parse_currency("5 g 3 s 7 c", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });

    // Mixed case
    let amount = parse_currency("5 GOLD 3 Silver 7 c", &system).unwrap();
    assert_eq!(amount, CurrencyAmount::MultiTier { base_units: 537 });

    // Single tier
    let amount = parse_currency("10 gold", &system).unwrap();
    assert_eq!(
        amount,
        CurrencyAmount::MultiTier {
            base_units: 1000
        }
    );
}

#[test]
fn test_currency_operations() {
    // Addition
    let a = CurrencyAmount::Decimal { minor_units: 1000 };
    let b = CurrencyAmount::Decimal { minor_units: 500 };
    let sum = a.add(&b).unwrap();
    assert_eq!(sum, CurrencyAmount::Decimal { minor_units: 1500 });

    // Subtraction
    let diff = a.subtract(&b).unwrap();
    assert_eq!(diff, CurrencyAmount::Decimal { minor_units: 500 });

    // Can afford check
    assert!(a.can_afford(&b));
    assert!(!b.can_afford(&a));

    // Mismatched types should fail
    let c = CurrencyAmount::MultiTier { base_units: 100 };
    assert!(a.add(&c).is_err());
    assert!(a.subtract(&c).is_err());
    assert!(!a.can_afford(&c));
}

#[test]
fn test_currency_transfer() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    // Create two players
    let alice = PlayerRecord::new("alice", "Alice", "room1");
    let bob = PlayerRecord::new("bob", "Bob", "room1");
    store.put_player(alice).expect("put alice");
    store.put_player(bob).expect("put bob");

    // Grant Alice some money
    let amount = CurrencyAmount::Decimal { minor_units: 1000 };
    store
        .grant_currency("alice", &amount, TransactionReason::SystemGrant)
        .expect("grant");

    // Verify Alice received it
    let alice = store.get_player("alice").expect("get alice");
    assert_eq!(alice.currency, amount);

    // Transfer from Alice to Bob
    let transfer_amount = CurrencyAmount::Decimal { minor_units: 300 };
    let transaction = store
        .transfer_currency("alice", "bob", &transfer_amount, TransactionReason::Trade)
        .expect("transfer");

    // Verify balances
    let alice = store.get_player("alice").expect("get alice");
    let bob = store.get_player("bob").expect("get bob");
    assert_eq!(
        alice.currency,
        CurrencyAmount::Decimal { minor_units: 700 }
    );
    assert_eq!(bob.currency, transfer_amount);

    // Verify transaction was logged
    assert_eq!(transaction.from, Some("alice".to_string()));
    assert_eq!(transaction.to, Some("bob".to_string()));
    assert_eq!(transaction.amount, transfer_amount);
    assert!(!transaction.rolled_back);
}

#[test]
fn test_insufficient_funds() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    let bob = PlayerRecord::new("bob", "Bob", "room1");
    store.put_player(alice).expect("put alice");
    store.put_player(bob).expect("put bob");

    // Try to transfer without funds
    let amount = CurrencyAmount::Decimal { minor_units: 1000 };
    let result = store.transfer_currency("alice", "bob", &amount, TransactionReason::Trade);

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        TinyMushError::InsufficientFunds => {}
        _ => panic!("Expected InsufficientFunds error, got: {:?}", err),
    }
}

#[test]
fn test_bank_deposit_and_withdrawal() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    store.put_player(alice).expect("put alice");

    // Grant Alice some money
    let initial = CurrencyAmount::Decimal { minor_units: 1000 };
    store
        .grant_currency("alice", &initial, TransactionReason::SystemGrant)
        .expect("grant");

    // Deposit half to bank
    let deposit_amount = CurrencyAmount::Decimal { minor_units: 500 };
    store
        .bank_deposit("alice", &deposit_amount)
        .expect("deposit");

    // Verify pocket and bank balances
    let alice = store.get_player("alice").expect("get alice");
    assert_eq!(
        alice.currency,
        CurrencyAmount::Decimal { minor_units: 500 }
    );
    assert_eq!(alice.banked_currency, deposit_amount);

    // Withdraw some
    let withdraw_amount = CurrencyAmount::Decimal { minor_units: 200 };
    store
        .bank_withdraw("alice", &withdraw_amount)
        .expect("withdraw");

    // Verify balances
    let alice = store.get_player("alice").expect("get alice");
    assert_eq!(
        alice.currency,
        CurrencyAmount::Decimal { minor_units: 700 }
    );
    assert_eq!(
        alice.banked_currency,
        CurrencyAmount::Decimal { minor_units: 300 }
    );
}

#[test]
fn test_transaction_rollback() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    let bob = PlayerRecord::new("bob", "Bob", "room1");
    store.put_player(alice).expect("put alice");
    store.put_player(bob).expect("put bob");

    // Grant Alice money
    let amount = CurrencyAmount::Decimal { minor_units: 1000 };
    store
        .grant_currency("alice", &amount, TransactionReason::SystemGrant)
        .expect("grant");

    // Transfer to Bob
    let transfer_amount = CurrencyAmount::Decimal { minor_units: 300 };
    let transaction = store
        .transfer_currency("alice", "bob", &transfer_amount, TransactionReason::Trade)
        .expect("transfer");

    // Verify initial state
    let alice_before = store.get_player("alice").expect("get alice");
    let bob_before = store.get_player("bob").expect("get bob");
    assert_eq!(
        alice_before.currency,
        CurrencyAmount::Decimal { minor_units: 700 }
    );
    assert_eq!(bob_before.currency, transfer_amount);

    // Rollback the transaction
    store
        .rollback_transaction(&transaction.id)
        .expect("rollback");

    // Verify rollback
    let alice_after = store.get_player("alice").expect("get alice");
    let bob_after = store.get_player("bob").expect("get bob");
    assert_eq!(alice_after.currency, amount);
    assert_eq!(
        bob_after.currency,
        CurrencyAmount::Decimal { minor_units: 0 }
    );

    // Verify transaction is marked as rolled back
    let rolled_back_tx = store.get_transaction(&transaction.id).expect("get tx");
    assert!(rolled_back_tx.rolled_back);
}

#[test]
fn test_transaction_history() {
    let dir = TempDir::new().expect("tempdir");
    let store = TinyMushStoreBuilder::new(dir.path()).open().expect("store");

    let alice = PlayerRecord::new("alice", "Alice", "room1");
    store.put_player(alice).expect("put alice");

    // Create several transactions
    let grant1 = CurrencyAmount::Decimal { minor_units: 1000 };
    store
        .grant_currency("alice", &grant1, TransactionReason::SystemGrant)
        .expect("grant1");

    let grant2 = CurrencyAmount::Decimal { minor_units: 500 };
    store
        .grant_currency("alice", &grant2, TransactionReason::QuestReward)
        .expect("grant2");

    let deposit = CurrencyAmount::Decimal { minor_units: 300 };
    store.bank_deposit("alice", &deposit).expect("deposit");

    // Get transaction history
    let history = store
        .get_player_transactions("alice", 10)
        .expect("get history");

    assert_eq!(history.len(), 3);
    // Should be in reverse chronological order
    assert_eq!(history[0].reason, TransactionReason::BankDeposit);
    assert_eq!(history[1].reason, TransactionReason::QuestReward);
    assert_eq!(history[2].reason, TransactionReason::SystemGrant);
}

#[test]
fn test_currency_saturation() {
    // Test that operations saturate instead of overflow
    let max = CurrencyAmount::Decimal {
        minor_units: i64::MAX,
    };
    let one = CurrencyAmount::Decimal { minor_units: 1 };

    // Addition should saturate at i64::MAX
    let result = max.add(&one).unwrap();
    assert_eq!(result, max);

    // Subtraction should saturate at i64::MIN
    let min = CurrencyAmount::Decimal {
        minor_units: i64::MIN,
    };
    let result = min.subtract(&one).unwrap();
    assert_eq!(result, min);
}

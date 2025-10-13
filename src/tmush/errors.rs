use thiserror::Error;

/// Errors that can arise while interacting with the TinyMUSH storage layer.
#[derive(Debug, Error)]
pub enum TinyMushError {
    /// Wrapper around sled's error type.
    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),

    /// Wrapper around bincode serialization and deserialization errors.
    #[error("serialization error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Wrapper around IO errors (directory creation, etc.).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Returned when fetching a record that is not present.
    #[error("record not found: {0}")]
    NotFound(String),

    /// Returned when deserializing a record with an unexpected schema version.
    #[error("schema mismatch for {entity}: expected {expected}, got {found}")]
    SchemaMismatch {
        entity: &'static str,
        expected: u8,
        found: u8,
    },

    /// Currency operation errors
    #[error("invalid currency operation: {0}")]
    InvalidCurrency(String),

    /// Insufficient funds for transaction
    #[error("insufficient funds")]
    InsufficientFunds,

    /// Transaction not found
    #[error("transaction not found")]
    TransactionNotFound,

    /// UTF-8 encoding error
    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// Permission denied (admin-only command)
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Container has nested containers (cannot delete until empty)
    #[error("container not empty: {0}")]
    ContainerNotEmpty(String),

    /// Internal error (task join errors, unexpected conditions)
    #[error("internal error: {0}")]
    Internal(String),
}

use std::path::PathBuf;
use thiserror::Error;

pub type NexusResult<T> = Result<T, NexusError>;

#[derive(Debug, Error)]
pub enum NexusError {
    #[error("project not initialized — run `nexus init` first")]
    NotInitialized,

    #[error("already initialized at {0}")]
    AlreadyInitialized(PathBuf),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("database error: {0}")]
    Database(String),

    #[error("git error: {0}")]
    Git(String),

    #[error("scan error: {0}")]
    Scan(String),

    #[error("snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("invalid nxp: {0}")]
    InvalidNxp(String),

    #[error("{0}")]
    Other(String),
}

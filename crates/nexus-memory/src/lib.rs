mod db;
mod json_store;
mod memory_engine;

pub use memory_engine::MemoryEngine;

pub fn db_err(e: rusqlite::Error) -> nexus_core::NexusError {
    nexus_core::NexusError::Database(e.to_string())
}

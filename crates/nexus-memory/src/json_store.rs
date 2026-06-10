use std::fs;
use std::path::Path;

use nexus_core::NexusResult;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> NexusResult<T> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> NexusResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn read_json_or_default<T: DeserializeOwned + Default>(path: &Path) -> NexusResult<T> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(T::default())
    }
}

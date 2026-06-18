use std::fs;
use std::io::Write;
use std::path::Path;

use nexus_core::NexusResult;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> NexusResult<T> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

/// Write JSON atomically.
///
/// Serialization happens into a sibling temp file which is flushed and fsynced,
/// then renamed over the target. `rename` is atomic on a single filesystem, so a
/// crash, power loss, or Ctrl-C mid-write leaves the previous file fully intact
/// rather than a truncated/corrupt document — essential for a memory layer.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> NexusResult<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let data = serde_json::to_string_pretty(value)?;

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("nexus");
    let tmp = parent.join(format!(".{file_name}.tmp"));

    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data.as_bytes())?;
        f.flush()?;
        // Best-effort durability; not fatal on filesystems that reject fsync.
        let _ = f.sync_all();
    }

    // std::fs::rename replaces an existing destination on both Unix and Windows.
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

pub fn read_json_or_default<T: DeserializeOwned + Default>(path: &Path) -> NexusResult<T> {
    if path.exists() {
        read_json(path)
    } else {
        Ok(T::default())
    }
}

use std::fs;

use chrono::Utc;
use nexus_core::{NexusResult, SnapshotManifest, TimelineEvent, TimelineEventKind};
use nexus_memory::{db_err, MemoryEngine};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct SnapshotEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> SnapshotEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn create(
        &self,
        label: impl Into<String>,
        description: Option<String>,
    ) -> NexusResult<SnapshotManifest> {
        let label = label.into();
        let id = Uuid::new_v4();
        let snapshot_dir = self.memory.paths.snapshots_dir().join(id.to_string());
        fs::create_dir_all(&snapshot_dir)?;

        let files_to_copy = [
            self.memory.paths.memory_file(),
            self.memory.paths.architecture_file(),
            self.memory.paths.decisions_file(),
            self.memory.paths.tasks_file(),
            self.memory.paths.timeline_file(),
        ];

        let mut memory_hash = String::new();
        let mut architecture_hash = String::new();

        for src in &files_to_copy {
            if src.exists() {
                let content = fs::read(src)?;
                let hash = hex_hash(&content);
                let dest = snapshot_dir.join(src.file_name().unwrap());
                fs::copy(src, &dest)?;

                if src.ends_with("memory.json") {
                    memory_hash = hash;
                } else if src.ends_with("architecture.json") {
                    architecture_hash = hash;
                }
            }
        }

        let decisions = self.memory.load_decisions()?;
        let tasks = self.memory.load_tasks()?;

        let manifest = SnapshotManifest {
            id,
            label: label.clone(),
            created_at: Utc::now(),
            description: description.clone(),
            memory_hash,
            architecture_hash,
            decisions_count: decisions.decisions.len(),
            tasks_count: tasks.tasks.len(),
        };

        let manifest_path = snapshot_dir.join("manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        self.memory.connection().execute(
            "INSERT INTO snapshots (id, label, created_at, description, memory_hash, architecture_hash, decisions_count, tasks_count, archive_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                manifest.id.to_string(),
                manifest.label,
                manifest.created_at.to_rfc3339(),
                manifest.description,
                manifest.memory_hash,
                manifest.architecture_hash,
                manifest.decisions_count,
                manifest.tasks_count,
                snapshot_dir.to_string_lossy().to_string(),
            ],
        ).map_err(db_err)?;

        self.memory.append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::Snapshot,
            title: format!("Snapshot: {label}"),
            description,
            timestamp: manifest.created_at,
            metadata: serde_json::json!({ "snapshot_id": manifest.id }),
        })?;

        Ok(manifest)
    }

    pub fn list(&self) -> NexusResult<Vec<SnapshotManifest>> {
        let mut stmt = self.memory.connection().prepare(
            "SELECT id, label, created_at, description, memory_hash, architecture_hash, decisions_count, tasks_count
             FROM snapshots ORDER BY created_at DESC",
        ).map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(SnapshotManifest {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?)
                        .unwrap_or_else(|_| Uuid::new_v4()),
                    label: row.get(1)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    description: row.get(3)?,
                    memory_hash: row.get(4)?,
                    architecture_hash: row.get(5)?,
                    decisions_count: row.get(6)?,
                    tasks_count: row.get(7)?,
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn restore(&self, snapshot_id: impl AsRef<str>) -> NexusResult<SnapshotManifest> {
        let id_str = snapshot_id.as_ref();
        let manifest = self
            .list()?
            .into_iter()
            .find(|s| s.id.to_string().starts_with(id_str) || s.label == id_str)
            .ok_or_else(|| nexus_core::NexusError::SnapshotNotFound(id_str.to_string()))?;

        let snapshot_dir = self
            .memory
            .paths
            .snapshots_dir()
            .join(manifest.id.to_string());
        if !snapshot_dir.is_dir() {
            return Err(nexus_core::NexusError::SnapshotNotFound(id_str.to_string()));
        }

        let restore_map = [
            ("memory.json", self.memory.paths.memory_file()),
            ("architecture.json", self.memory.paths.architecture_file()),
            ("decisions.json", self.memory.paths.decisions_file()),
            ("tasks.json", self.memory.paths.tasks_file()),
            ("timeline.json", self.memory.paths.timeline_file()),
        ];

        for (name, dest) in restore_map {
            let src = snapshot_dir.join(name);
            if src.exists() {
                fs::copy(&src, &dest)?;
            }
        }

        self.memory.append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::SnapshotRestore,
            title: format!("Restored snapshot: {}", manifest.label),
            description: None,
            timestamp: Utc::now(),
            metadata: serde_json::json!({ "snapshot_id": manifest.id }),
        })?;

        Ok(manifest)
    }
}

fn hex_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())[..16].to_string()
}

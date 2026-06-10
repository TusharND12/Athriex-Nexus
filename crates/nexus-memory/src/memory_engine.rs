use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use nexus_core::{
    AiSession, Architecture, Decision, DecisionStore, NexusPaths, NexusResult, ProjectMemory, Task,
    TaskStore, Timeline, TimelineEvent, TimelineEventKind,
};
use rusqlite::Connection;
use uuid::Uuid;

use crate::db::open_and_migrate;
use crate::db_err;
use crate::json_store::{read_json, read_json_or_default, write_json};

pub struct MemoryEngine {
    pub paths: NexusPaths,
    conn: Connection,
}

impl MemoryEngine {
    pub fn open(paths: NexusPaths) -> NexusResult<Self> {
        if !paths.is_initialized() {
            return Err(nexus_core::NexusError::NotInitialized);
        }
        let conn = open_and_migrate(&paths.knowledge_db())?;
        Ok(Self { paths, conn })
    }

    pub fn init(paths: NexusPaths, project_name: impl Into<String>) -> NexusResult<Self> {
        if paths.is_initialized() {
            return Err(nexus_core::NexusError::AlreadyInitialized(
                paths.nexus_root.clone(),
            ));
        }

        fs::create_dir_all(&paths.nexus_root)?;
        fs::create_dir_all(paths.sessions_dir())?;
        fs::create_dir_all(paths.snapshots_dir())?;

        let memory = ProjectMemory::new(project_name);
        write_json(&paths.memory_file(), &memory)?;

        let architecture = Architecture {
            version: nexus_core::NEXUS_VERSION.to_string(),
            updated_at: Utc::now(),
            ..Default::default()
        };
        write_json(&paths.architecture_file(), &architecture)?;

        let timeline = Timeline {
            version: nexus_core::NEXUS_VERSION.to_string(),
            events: vec![TimelineEvent {
                id: Uuid::new_v4(),
                kind: TimelineEventKind::Init,
                title: "Nexus initialized".to_string(),
                description: Some("Project memory layer created".to_string()),
                timestamp: Utc::now(),
                metadata: serde_json::json!({}),
            }],
        };
        write_json(&paths.timeline_file(), &timeline)?;

        write_json(
            &paths.decisions_file(),
            &DecisionStore {
                version: nexus_core::NEXUS_VERSION.to_string(),
                decisions: vec![],
            },
        )?;

        write_json(
            &paths.tasks_file(),
            &TaskStore {
                version: nexus_core::NEXUS_VERSION.to_string(),
                tasks: vec![],
            },
        )?;

        Self::open(paths)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn load_memory(&self) -> NexusResult<ProjectMemory> {
        read_json(&self.paths.memory_file())
    }

    pub fn save_memory(&self, memory: &ProjectMemory) -> NexusResult<()> {
        write_json(&self.paths.memory_file(), memory)
    }

    pub fn load_architecture(&self) -> NexusResult<Architecture> {
        read_json_or_default(&self.paths.architecture_file())
    }

    pub fn save_architecture(&self, architecture: &Architecture) -> NexusResult<()> {
        write_json(&self.paths.architecture_file(), architecture)
    }

    pub fn load_decisions(&self) -> NexusResult<DecisionStore> {
        read_json_or_default(&self.paths.decisions_file())
    }

    pub fn save_decisions(&self, store: &DecisionStore) -> NexusResult<()> {
        write_json(&self.paths.decisions_file(), store)
    }

    pub fn load_tasks(&self) -> NexusResult<TaskStore> {
        read_json_or_default(&self.paths.tasks_file())
    }

    pub fn save_tasks(&self, store: &TaskStore) -> NexusResult<()> {
        write_json(&self.paths.tasks_file(), store)
    }

    pub fn load_timeline(&self) -> NexusResult<Timeline> {
        read_json_or_default(&self.paths.timeline_file())
    }

    pub fn save_timeline(&self, timeline: &Timeline) -> NexusResult<()> {
        write_json(&self.paths.timeline_file(), timeline)
    }

    pub fn append_timeline_event(&self, event: TimelineEvent) -> NexusResult<()> {
        let mut timeline = self.load_timeline()?;
        timeline.events.push(event);
        self.save_timeline(&timeline)
    }

    pub fn persist_decision_to_db(&self, decision: &Decision) -> NexusResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO decisions (id, content, rationale, tags, created_at, author, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                decision.id.to_string(),
                decision.content,
                decision.rationale,
                serde_json::to_string(&decision.tags).unwrap(),
                decision.created_at.to_rfc3339(),
                decision.author,
                format!("{:?}", decision.status).to_lowercase(),
            ],
        ).map_err(db_err)?;
        self.index_fts(
            &decision.id.to_string(),
            "decision",
            &format!(
                "{} {}",
                decision.content,
                decision.rationale.as_deref().unwrap_or("")
            ),
        )?;
        Ok(())
    }

    pub fn persist_task_to_db(&self, task: &Task) -> NexusResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (id, title, description, status, priority, created_at, updated_at, related_files, blocked_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                task.id.to_string(),
                task.title,
                task.description,
                format!("{:?}", task.status).to_lowercase(),
                format!("{:?}", task.priority).to_lowercase(),
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                serde_json::to_string(&task.related_files).unwrap(),
                serde_json::to_string(&task.blocked_by).unwrap(),
            ],
        ).map_err(db_err)?;
        self.index_fts(
            &task.id.to_string(),
            "task",
            &format!(
                "{} {}",
                task.title,
                task.description.as_deref().unwrap_or("")
            ),
        )?;
        Ok(())
    }

    pub fn save_session(&self, session: &AiSession) -> NexusResult<PathBuf> {
        let filename = format!(
            "{}_{}.json",
            session.timestamp.format("%Y%m%d_%H%M%S"),
            session.id
        );
        let path = self.paths.sessions_dir().join(filename);
        write_json(&path, session)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO sessions (id, tool, prompt, response, files_modified, timestamp, notes, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                session.id.to_string(),
                session.tool,
                session.prompt,
                session.response,
                serde_json::to_string(&session.files_modified).unwrap(),
                session.timestamp.to_rfc3339(),
                session.notes,
                serde_json::to_string(&session.tags).unwrap(),
            ],
        ).map_err(db_err)?;

        self.index_fts(
            &session.id.to_string(),
            "session",
            &format!("{} {}", session.prompt, session.response),
        )?;

        Ok(path)
    }

    pub fn load_sessions(&self, limit: usize) -> NexusResult<Vec<AiSession>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, tool, prompt, response, files_modified, timestamp, notes, tags
             FROM sessions ORDER BY timestamp DESC LIMIT ?1",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(AiSession {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?)
                        .unwrap_or_else(|_| Uuid::new_v4()),
                    tool: row.get(1)?,
                    prompt: row.get(2)?,
                    response: row.get(3)?,
                    files_modified: serde_json::from_str(&row.get::<_, String>(4)?)
                        .unwrap_or_default(),
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    notes: row.get(6)?,
                    tags: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                })
            })
            .map_err(db_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_err)
    }

    pub fn record_scan(&self, files_analyzed: usize, result_json: &str) -> NexusResult<()> {
        self.conn.execute(
            "INSERT INTO scan_history (scanned_at, files_analyzed, result_json) VALUES (?1, ?2, ?3)",
            rusqlite::params![Utc::now().to_rfc3339(), files_analyzed, result_json],
        ).map_err(db_err)?;
        Ok(())
    }

    fn index_fts(&self, doc_id: &str, source: &str, content: &str) -> NexusResult<()> {
        self.conn
            .execute("DELETE FROM memory_fts WHERE doc_id = ?1", [doc_id])
            .map_err(db_err)?;
        self.conn
            .execute(
                "INSERT INTO memory_fts (doc_id, source, content) VALUES (?1, ?2, ?3)",
                rusqlite::params![doc_id, source, content],
            )
            .map_err(db_err)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn init_creates_nexus_layout() {
        let tmp = TempDir::new().unwrap();
        let paths = NexusPaths::from_project_root(tmp.path());
        let engine = MemoryEngine::init(paths.clone(), "test-project").unwrap();

        assert!(paths.memory_file().exists());
        assert!(paths.architecture_file().exists());
        assert!(paths.knowledge_db().exists());
        assert!(paths.sessions_dir().exists());

        let memory = engine.load_memory().unwrap();
        assert_eq!(memory.project_name, "test-project");
    }
}

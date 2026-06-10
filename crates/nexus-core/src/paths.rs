use std::path::{Path, PathBuf};

use crate::NexusResult;

pub const NEXUS_DIR: &str = ".nexus";
pub const MEMORY_FILE: &str = "memory.json";
pub const ARCHITECTURE_FILE: &str = "architecture.json";
pub const TIMELINE_FILE: &str = "timeline.json";
pub const DECISIONS_FILE: &str = "decisions.json";
pub const TASKS_FILE: &str = "tasks.json";
pub const KNOWLEDGE_DB: &str = "knowledge.db";
pub const NXP_FILE: &str = "project.nxp";
pub const SESSIONS_DIR: &str = "sessions";
pub const SNAPSHOTS_DIR: &str = "snapshots";

/// Canonical filesystem layout for a Nexus-enabled project.
#[derive(Debug, Clone)]
pub struct NexusPaths {
    pub project_root: PathBuf,
    pub nexus_root: PathBuf,
}

impl NexusPaths {
    pub fn discover(start: impl AsRef<Path>) -> NexusResult<Self> {
        let start = start.as_ref().canonicalize()?;
        let mut current = start.clone();

        loop {
            let candidate = current.join(NEXUS_DIR);
            if candidate.is_dir() {
                return Ok(Self {
                    project_root: current.clone(),
                    nexus_root: candidate,
                });
            }
            if !current.pop() {
                break;
            }
        }

        Err(crate::NexusError::NotInitialized)
    }

    pub fn from_project_root(root: impl AsRef<Path>) -> Self {
        let project_root = root.as_ref().to_path_buf();
        let nexus_root = project_root.join(NEXUS_DIR);
        Self {
            project_root,
            nexus_root,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.nexus_root.is_dir() && self.memory_file().exists()
    }

    pub fn memory_file(&self) -> PathBuf {
        self.nexus_root.join(MEMORY_FILE)
    }

    pub fn architecture_file(&self) -> PathBuf {
        self.nexus_root.join(ARCHITECTURE_FILE)
    }

    pub fn timeline_file(&self) -> PathBuf {
        self.nexus_root.join(TIMELINE_FILE)
    }

    pub fn decisions_file(&self) -> PathBuf {
        self.nexus_root.join(DECISIONS_FILE)
    }

    pub fn tasks_file(&self) -> PathBuf {
        self.nexus_root.join(TASKS_FILE)
    }

    pub fn knowledge_db(&self) -> PathBuf {
        self.nexus_root.join(KNOWLEDGE_DB)
    }

    pub fn nxp_file(&self) -> PathBuf {
        self.nexus_root.join(NXP_FILE)
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.nexus_root.join(SESSIONS_DIR)
    }

    pub fn snapshots_dir(&self) -> PathBuf {
        self.nexus_root.join(SNAPSHOTS_DIR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nexus_dir_constants() {
        assert_eq!(NEXUS_DIR, ".nexus");
        assert_eq!(NXP_FILE, "project.nxp");
    }

    #[test]
    fn paths_layout() {
        let paths = NexusPaths::from_project_root("/tmp/myproject");
        assert_eq!(paths.nexus_root, PathBuf::from("/tmp/myproject/.nexus"));
        assert!(paths.memory_file().ends_with(".nexus/memory.json"));
    }
}

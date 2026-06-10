use std::fs;

use chrono::Utc;
use nexus_core::{NexusResult, NxpDocument, NXP_PROTOCOL_VERSION};
use nexus_git::GitEngine;
use nexus_knowledge::KnowledgeEngine;
use nexus_memory::MemoryEngine;

pub struct NxpEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> NxpEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn export(&self) -> NexusResult<NxpDocument> {
        let git = GitEngine::open(&self.memory.paths.project_root)?;
        let knowledge_graph = KnowledgeEngine::new(self.memory).rebuild()?;
        let sessions = self.memory.load_sessions(20)?;

        Ok(NxpDocument {
            protocol_version: NXP_PROTOCOL_VERSION.to_string(),
            nexus_version: nexus_core::NEXUS_VERSION.to_string(),
            exported_at: Utc::now(),
            project: self.memory.load_memory()?,
            architecture: self.memory.load_architecture()?,
            decisions: self.memory.load_decisions()?,
            tasks: self.memory.load_tasks()?,
            timeline: self.memory.load_timeline()?,
            knowledge_graph,
            recent_sessions: sessions,
            git_summary: Some(git.summarize(25)?),
            extensions: serde_json::json!({}),
        })
    }

    pub fn write_to_file(&self) -> NexusResult<std::path::PathBuf> {
        let doc = self.export()?;
        let path = self.memory.paths.nxp_file();
        let toml_body = toml::to_string_pretty(&NxpTomlWrapper::from(&doc))?;
        fs::write(&path, toml_body)?;
        Ok(path)
    }

    pub fn read_from_file(path: &std::path::Path) -> NexusResult<NxpDocument> {
        let content = fs::read_to_string(path)?;
        let wrapper: NxpTomlWrapper = toml::from_str(&content)
            .map_err(|e| nexus_core::NexusError::InvalidNxp(e.to_string()))?;
        wrapper.into_document()
    }

    pub fn import(&self, doc: &NxpDocument) -> NexusResult<()> {
        self.memory.save_memory(&doc.project)?;
        self.memory.save_architecture(&doc.architecture)?;
        self.memory.save_decisions(&doc.decisions)?;
        self.memory.save_tasks(&doc.tasks)?;
        self.memory.save_timeline(&doc.timeline)?;

        for decision in &doc.decisions.decisions {
            self.memory.persist_decision_to_db(decision)?;
        }
        for task in &doc.tasks.tasks {
            self.memory.persist_task_to_db(task)?;
        }

        KnowledgeEngine::new(self.memory).persist_graph_from(&doc.knowledge_graph)?;

        self.write_to_file()?;
        Ok(())
    }
}

/// TOML-serializable wrapper — embeds JSON blobs for complex nested structures.
#[derive(serde::Serialize, serde::Deserialize)]
struct NxpTomlWrapper {
    protocol_version: String,
    nexus_version: String,
    exported_at: String,
    #[serde(rename = "project")]
    project_json: String,
    architecture_json: String,
    decisions_json: String,
    tasks_json: String,
    timeline_json: String,
    knowledge_graph_json: String,
    recent_sessions_json: String,
    git_summary_json: String,
    extensions_json: String,
}

impl From<&NxpDocument> for NxpTomlWrapper {
    fn from(doc: &NxpDocument) -> Self {
        Self {
            protocol_version: doc.protocol_version.clone(),
            nexus_version: doc.nexus_version.clone(),
            exported_at: doc.exported_at.to_rfc3339(),
            project_json: serde_json::to_string(&doc.project).unwrap(),
            architecture_json: serde_json::to_string(&doc.architecture).unwrap(),
            decisions_json: serde_json::to_string(&doc.decisions).unwrap(),
            tasks_json: serde_json::to_string(&doc.tasks).unwrap(),
            timeline_json: serde_json::to_string(&doc.timeline).unwrap(),
            knowledge_graph_json: serde_json::to_string(&doc.knowledge_graph).unwrap(),
            recent_sessions_json: serde_json::to_string(&doc.recent_sessions).unwrap(),
            git_summary_json: serde_json::to_string(&doc.git_summary).unwrap(),
            extensions_json: doc.extensions.to_string(),
        }
    }
}

impl NxpTomlWrapper {
    fn into_document(self) -> NexusResult<NxpDocument> {
        Ok(NxpDocument {
            protocol_version: self.protocol_version,
            nexus_version: self.nexus_version,
            exported_at: chrono::DateTime::parse_from_rfc3339(&self.exported_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            project: serde_json::from_str(&self.project_json)?,
            architecture: serde_json::from_str(&self.architecture_json)?,
            decisions: serde_json::from_str(&self.decisions_json)?,
            tasks: serde_json::from_str(&self.tasks_json)?,
            timeline: serde_json::from_str(&self.timeline_json)?,
            knowledge_graph: serde_json::from_str(&self.knowledge_graph_json)?,
            recent_sessions: serde_json::from_str(&self.recent_sessions_json)?,
            git_summary: serde_json::from_str(&self.git_summary_json)?,
            extensions: serde_json::from_str(&self.extensions_json).unwrap_or_default(),
        })
    }
}

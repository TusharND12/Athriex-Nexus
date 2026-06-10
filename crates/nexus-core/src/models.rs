use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const NEXUS_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NXP_PROTOCOL_VERSION: &str = "1.0.0";

// ─── Project Memory ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMemory {
    pub version: String,
    pub project_name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub technologies: Vec<String>,
    pub current_focus: String,
    pub completed_work: Vec<String>,
    pub risks: Vec<String>,
    pub metadata: serde_json::Value,
}

impl ProjectMemory {
    pub fn new(project_name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            version: NEXUS_VERSION.to_string(),
            project_name: project_name.into(),
            description: String::new(),
            created_at: now,
            updated_at: now,
            technologies: Vec::new(),
            current_focus: String::new(),
            completed_work: Vec::new(),
            risks: Vec::new(),
            metadata: serde_json::json!({}),
        }
    }
}

// ─── Architecture ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Architecture {
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub layers: Vec<ArchitectureLayer>,
    pub technologies: Vec<Technology>,
    pub dependencies: Vec<DependencyInfo>,
    pub entry_points: Vec<String>,
    pub important_files: Vec<ImportantFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    pub name: String,
    pub description: String,
    pub components: Vec<ArchitectureComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureComponent {
    pub name: String,
    pub path: String,
    pub kind: ComponentKind,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ComponentKind {
    Frontend,
    Backend,
    Api,
    Database,
    Shared,
    Infrastructure,
    Test,
    Config,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technology {
    pub name: String,
    pub category: String,
    pub version: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub name: String,
    pub version: Option<String>,
    pub kind: String,
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportantFile {
    pub path: String,
    pub reason: String,
    pub relevance: f32,
}

// ─── Decisions ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub content: String,
    pub rationale: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub author: Option<String>,
    pub status: DecisionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    #[default]
    Active,
    Superseded,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecisionStore {
    pub version: String,
    pub decisions: Vec<Decision>,
}

// ─── Tasks ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub related_files: Vec<String>,
    pub blocked_by: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub version: String,
    pub tasks: Vec<Task>,
}

// ─── Timeline ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: Uuid,
    pub kind: TimelineEventKind,
    pub title: String,
    pub description: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimelineEventKind {
    Init,
    Scan,
    Decision,
    Snapshot,
    SnapshotRestore,
    Session,
    Milestone,
    TaskUpdate,
    Handoff,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Timeline {
    pub version: String,
    pub events: Vec<TimelineEvent>,
}

// ─── Sessions ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSession {
    pub id: Uuid,
    pub tool: Option<String>,
    pub prompt: String,
    pub response: String,
    pub files_modified: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

// ─── Snapshots ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotManifest {
    pub id: Uuid,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub memory_hash: String,
    pub architecture_hash: String,
    pub decisions_count: usize,
    pub tasks_count: usize,
}

// ─── Knowledge Graph ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    File,
    Function,
    Service,
    Api,
    Database,
    Task,
    Decision,
    Module,
    Package,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub id: Uuid,
    pub kind: NodeKind,
    pub name: String,
    pub path: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeRelation {
    Imports,
    Calls,
    DependsOn,
    Implements,
    RelatedTo,
    Owns,
    Documents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub relation: EdgeRelation,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

// ─── Scan Results ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scanned_at: DateTime<Utc>,
    pub files_analyzed: usize,
    pub languages: Vec<LanguageStats>,
    pub frameworks: Vec<String>,
    pub architecture: Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub file_count: usize,
    pub line_count: usize,
}

// ─── Health ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_score: f32,
    pub technical_debt: HealthMetric,
    pub dependency_risk: HealthMetric,
    pub documentation: HealthMetric,
    pub complexity: HealthMetric,
    pub maintainability: HealthMetric,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetric {
    pub score: f32,
    pub summary: String,
}

// ─── Continuation Context ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuationContext {
    pub generated_at: DateTime<Utc>,
    pub project_overview: String,
    pub completed_work: Vec<String>,
    pub current_task: String,
    pub important_decisions: Vec<String>,
    pub important_files: Vec<ImportantFile>,
    pub architecture_summary: String,
    pub risks: Vec<String>,
    pub next_recommended_action: String,
    pub ai_continuation_prompt: String,
    pub compressed: bool,
    pub token_estimate: usize,
}

// ─── NXP Protocol ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxpDocument {
    pub protocol_version: String,
    pub nexus_version: String,
    pub exported_at: DateTime<Utc>,
    pub project: ProjectMemory,
    pub architecture: Architecture,
    pub decisions: DecisionStore,
    pub tasks: TaskStore,
    pub timeline: Timeline,
    pub knowledge_graph: KnowledgeGraph,
    pub recent_sessions: Vec<AiSession>,
    pub git_summary: Option<GitSummary>,
    pub extensions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitSummary {
    pub branch: String,
    pub commit_count: usize,
    pub recent_commits: Vec<CommitSummary>,
    pub dirty_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

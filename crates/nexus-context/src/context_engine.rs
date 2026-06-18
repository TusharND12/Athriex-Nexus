use std::collections::{HashMap, HashSet};

use chrono::Utc;
use nexus_compression::{estimate_tokens, CompressionEngine};
use nexus_core::{ContinuationContext, KnowledgeNode, NexusResult, NodeKind, TaskStatus};
use nexus_decision::DecisionEngine;
use nexus_git::GitEngine;
use nexus_knowledge::KnowledgeEngine;
use nexus_memory::MemoryEngine;
use nexus_scanner::structure::format_architecture_tree;
use nexus_search::SearchEngine;

pub struct ContextEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> ContextEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn continue_context(
        &self,
        compress: bool,
        max_tokens: usize,
    ) -> NexusResult<ContinuationContext> {
        self.continue_context_for(compress, max_tokens, None)
    }

    /// Build continuation context, optionally focused on a specific task.
    ///
    /// When `focus_task` is set, it becomes the current task and the project's
    /// memory (decisions + important files) is re-ranked by relevance to that
    /// task via the FTS5 search index — turning `continue` from a full dump into
    /// a targeted query.
    pub fn continue_context_for(
        &self,
        compress: bool,
        max_tokens: usize,
        focus_task: Option<String>,
    ) -> NexusResult<ContinuationContext> {
        let memory = self.memory.load_memory()?;
        let architecture = self.memory.load_architecture()?;
        let mut decisions = DecisionEngine::new(self.memory).list_active()?;
        let tasks = self.memory.load_tasks()?;
        let git = GitEngine::open(&self.memory.paths.project_root)?;
        let git_summary = git.summarize(10)?;
        let graph = KnowledgeEngine::new(self.memory).rebuild()?;
        let centrality = file_centrality(&graph);

        let focus = focus_task
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string());

        // Pull task-relevant decisions/files out of the search index.
        let (relevant_decisions, relevant_files) = match &focus {
            Some(task) => {
                let mut dec = HashSet::new();
                let mut files = HashSet::new();
                for hit in SearchEngine::new(self.memory)
                    .ask(task, 30)
                    .unwrap_or_default()
                {
                    match hit.source.as_str() {
                        "decision" => {
                            dec.insert(hit.doc_id);
                        }
                        "file" => {
                            files.insert(hit.doc_id);
                        }
                        _ => {}
                    }
                }
                (dec, files)
            }
            None => (HashSet::new(), HashSet::new()),
        };

        // Surface task-relevant decisions first (stable: false sorts before true).
        if !relevant_decisions.is_empty() {
            decisions.sort_by_key(|d| !relevant_decisions.contains(&d.id.to_string()));
        }

        let current_task = focus.clone().unwrap_or_else(|| {
            tasks
                .tasks
                .iter()
                .find(|t| t.status == TaskStatus::InProgress)
                .or_else(|| tasks.tasks.iter().find(|t| t.status == TaskStatus::Pending))
                .map(|t| t.title.clone())
                .unwrap_or_else(|| memory.current_focus.clone())
        });

        let important_decisions: Vec<String> = decisions
            .iter()
            .take(15)
            .map(|d| {
                if let Some(r) = &d.rationale {
                    format!("{} — {}", d.content, r)
                } else {
                    d.content.clone()
                }
            })
            .collect();

        let architecture_summary = if architecture.layers.is_empty() {
            "No architecture scan available. Run `nexus scan`.".to_string()
        } else {
            format_architecture_tree(&architecture.layers)
        };

        let completed_work = if memory.completed_work.is_empty() {
            git_summary
                .recent_commits
                .iter()
                .take(10)
                .map(|c| format!("[{}] {}", c.hash, c.message))
                .collect()
        } else {
            memory.completed_work.clone()
        };

        let risks = if memory.risks.is_empty() {
            let mut auto_risks = Vec::new();
            if !git_summary.dirty_files.is_empty() {
                auto_risks.push(format!(
                    "{} uncommitted files may cause context drift",
                    git_summary.dirty_files.len()
                ));
            }
            if architecture.dependencies.len() > 50 {
                auto_risks.push("High dependency count increases supply chain risk".to_string());
            }
            auto_risks
        } else {
            memory.risks.clone()
        };

        let next_action = derive_next_action(&current_task, &tasks.tasks, &risks);

        // Re-rank important files by scanner relevance plus knowledge-graph
        // centrality, so files wired into many tasks/modules surface first.
        let mut ranked_files = architecture.important_files.clone();
        let file_score = |f: &nexus_core::ImportantFile| {
            let centrality_boost = 0.15 * *centrality.get(&f.path).unwrap_or(&0) as f32;
            let task_boost = if relevant_files.contains(&f.path) {
                1.0
            } else {
                0.0
            };
            f.relevance + centrality_boost + task_boost
        };
        ranked_files.sort_by(|a, b| {
            file_score(b)
                .partial_cmp(&file_score(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let project_overview = format!(
            "Project: {}\n{}\nTechnologies: {}\nBranch: {} | Commits: {} | Last updated: {}",
            memory.project_name,
            if memory.description.is_empty() {
                "No description set.".to_string()
            } else {
                memory.description.clone()
            },
            memory.technologies.join(", "),
            git_summary.branch,
            git_summary.commit_count,
            memory.updated_at.format("%Y-%m-%d %H:%M UTC")
        );

        let mut ctx = ContinuationContext {
            generated_at: Utc::now(),
            project_overview: project_overview.clone(),
            completed_work: completed_work.clone(),
            current_task: current_task.clone(),
            important_decisions: important_decisions.clone(),
            important_files: ranked_files,
            architecture_summary: architecture_summary.clone(),
            risks: risks.clone(),
            next_recommended_action: next_action.clone(),
            ai_continuation_prompt: String::new(),
            compressed: false,
            token_estimate: 0,
        };

        ctx.ai_continuation_prompt = build_full_prompt(&ctx);

        if compress {
            ctx = CompressionEngine::compress(&ctx, max_tokens);
        } else {
            ctx.token_estimate = estimate_tokens(&ctx.ai_continuation_prompt);
        }

        Ok(ctx)
    }

    pub fn format_continue_output(&self, ctx: &ContinuationContext) -> String {
        format!(
            r#"╔══════════════════════════════════════════════════════════════╗
║              ATHREIX NEXUS — PROJECT CONTINUATION            ║
║         Never explain your project to an AI twice.           ║
╚══════════════════════════════════════════════════════════════╝

Generated: {}
Token estimate: ~{}{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PROJECT OVERVIEW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
COMPLETED WORK
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CURRENT TASK
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
IMPORTANT DECISIONS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
IMPORTANT FILES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ARCHITECTURE SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RISKS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
NEXT RECOMMENDED ACTION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
AI CONTINUATION PROMPT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}
"#,
            ctx.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            ctx.token_estimate,
            if ctx.compressed { " (compressed)" } else { "" },
            ctx.project_overview,
            if ctx.completed_work.is_empty() {
                "  (none recorded)".to_string()
            } else {
                ctx.completed_work
                    .iter()
                    .map(|w| format!("  • {w}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            ctx.current_task,
            if ctx.important_decisions.is_empty() {
                "  (none recorded — use `nexus decide`)".to_string()
            } else {
                ctx.important_decisions
                    .iter()
                    .map(|d| format!("  • {d}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            if ctx.important_files.is_empty() {
                "  (run `nexus scan`)".to_string()
            } else {
                ctx.important_files
                    .iter()
                    .take(15)
                    .map(|f| format!("  • {} — {}", f.path, f.reason))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            ctx.architecture_summary,
            if ctx.risks.is_empty() {
                "  (none identified)".to_string()
            } else {
                ctx.risks
                    .iter()
                    .map(|r| format!("  • {r}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            ctx.next_recommended_action,
            ctx.ai_continuation_prompt,
        )
    }
}

/// Degree centrality per file path: how many graph edges touch each file node.
fn file_centrality(graph: &nexus_core::KnowledgeGraph) -> HashMap<String, usize> {
    let node_by_id: HashMap<String, &KnowledgeNode> =
        graph.nodes.iter().map(|n| (n.id.to_string(), n)).collect();

    let mut degree: HashMap<String, usize> = HashMap::new();
    for edge in &graph.edges {
        for endpoint in [edge.from.to_string(), edge.to.to_string()] {
            if let Some(node) = node_by_id.get(&endpoint) {
                if node.kind == NodeKind::File {
                    if let Some(path) = &node.path {
                        *degree.entry(path.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    degree
}

fn derive_next_action(current_task: &str, tasks: &[nexus_core::Task], risks: &[String]) -> String {
    if !current_task.is_empty() {
        return format!("Continue: {current_task}");
    }
    if let Some(task) = tasks.iter().find(|t| t.status == TaskStatus::Pending) {
        return format!("Start task: {}", task.title);
    }
    if !risks.is_empty() {
        return format!("Address risk: {}", risks[0]);
    }
    "Run `nexus scan` to refresh project understanding".to_string()
}

fn build_full_prompt(ctx: &ContinuationContext) -> String {
    format!(
        r#"You are continuing work on an existing software project. Full context has been reconstructed by Athreix Nexus.

## PROJECT OVERVIEW
{}

## COMPLETED WORK
{}

## CURRENT TASK
{}

## ARCHITECTURAL DECISIONS (binding — do not contradict)
{}

## ARCHITECTURE
{}

## IMPORTANT FILES
{}

## RISKS
{}

## INSTRUCTIONS
1. Continue from current task without re-asking resolved questions.
2. Respect all architectural decisions listed above.
3. Match existing code style and patterns.
4. Reference important files before creating duplicates.
5. Propose next steps aligned with project direction.

## NEXT RECOMMENDED ACTION
{}
"#,
        ctx.project_overview,
        ctx.completed_work.join("\n"),
        ctx.current_task,
        ctx.important_decisions.join("\n"),
        ctx.architecture_summary,
        ctx.important_files
            .iter()
            .map(|f| format!("- {} ({})", f.path, f.reason))
            .collect::<Vec<_>>()
            .join("\n"),
        ctx.risks.join("\n"),
        ctx.next_recommended_action,
    )
}

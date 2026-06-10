use nexus_core::{HealthMetric, HealthReport, NexusResult, TaskStatus};
use nexus_memory::MemoryEngine;

pub struct HealthAnalyzer<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> HealthAnalyzer<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn analyze(&self) -> NexusResult<HealthReport> {
        let memory = self.memory.load_memory()?;
        let architecture = self.memory.load_architecture()?;
        let tasks = self.memory.load_tasks()?;
        let decisions = self.memory.load_decisions()?;

        let pending_tasks = tasks
            .tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending || t.status == TaskStatus::InProgress)
            .count();

        let doc_files = architecture
            .important_files
            .iter()
            .filter(|f| f.path.ends_with(".md") || f.path.contains("README"))
            .count();

        let dep_count = architecture.dependencies.len();
        let file_count = architecture.important_files.len().max(1);

        let technical_debt = HealthMetric {
            score: score_inverse(pending_tasks, 20),
            summary: format!("{pending_tasks} open tasks tracked"),
        };

        let dependency_risk = HealthMetric {
            score: score_inverse(dep_count, 100),
            summary: format!("{dep_count} dependencies detected"),
        };

        let documentation = HealthMetric {
            score: ((doc_files as f32 / file_count as f32) * 100.0).min(100.0),
            summary: format!("{doc_files} documentation files indexed"),
        };

        let complexity = HealthMetric {
            score: score_inverse(architecture.layers.len(), 15),
            summary: format!("{} architectural layers", architecture.layers.len()),
        };

        let maintainability = HealthMetric {
            score: {
                let decision_bonus = (decisions.decisions.len() as f32 * 2.0).min(20.0);
                let memory_freshness =
                    if memory.updated_at > chrono::Utc::now() - chrono::Duration::days(7) {
                        20.0
                    } else {
                        5.0
                    };
                (50.0 + decision_bonus + memory_freshness).min(100.0)
            },
            summary: format!(
                "{} decisions recorded, last updated {}",
                decisions.decisions.len(),
                memory.updated_at.format("%Y-%m-%d")
            ),
        };

        let overall = (technical_debt.score
            + dependency_risk.score
            + documentation.score
            + complexity.score
            + maintainability.score)
            / 5.0;

        let mut recommendations = Vec::new();
        if decisions.decisions.is_empty() {
            recommendations.push("Record architectural decisions with `nexus decide`".to_string());
        }
        if architecture.important_files.is_empty() {
            recommendations.push("Run `nexus scan` to index project structure".to_string());
        }
        if pending_tasks > 10 {
            recommendations.push("Review and prioritize open tasks".to_string());
        }
        if doc_files == 0 {
            recommendations.push("Add README and architecture documentation".to_string());
        }

        Ok(HealthReport {
            overall_score: overall,
            technical_debt,
            dependency_risk,
            documentation,
            complexity,
            maintainability,
            recommendations,
        })
    }
}

fn score_inverse(value: usize, threshold: usize) -> f32 {
    if value == 0 {
        return 100.0;
    }
    ((1.0 - (value as f32 / threshold as f32)) * 100.0).clamp(0.0, 100.0)
}

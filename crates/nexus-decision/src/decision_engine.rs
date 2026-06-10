use chrono::Utc;
use nexus_core::{
    Decision, DecisionStatus, DecisionStore, NexusResult, TimelineEvent, TimelineEventKind,
};
use nexus_memory::MemoryEngine;
use uuid::Uuid;

pub struct DecisionEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> DecisionEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn record(
        &self,
        content: impl Into<String>,
        rationale: Option<String>,
        tags: Vec<String>,
        author: Option<String>,
    ) -> NexusResult<Decision> {
        let decision = Decision {
            id: Uuid::new_v4(),
            content: content.into(),
            rationale,
            tags,
            created_at: Utc::now(),
            author,
            status: DecisionStatus::Active,
        };

        let mut store = self.memory.load_decisions()?;
        store.decisions.push(decision.clone());
        self.memory.save_decisions(&store)?;
        self.memory.persist_decision_to_db(&decision)?;

        self.memory.append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::Decision,
            title: truncate(&decision.content, 80),
            description: decision.rationale.clone(),
            timestamp: decision.created_at,
            metadata: serde_json::json!({ "decision_id": decision.id }),
        })?;

        Ok(decision)
    }

    pub fn list(&self) -> NexusResult<Vec<Decision>> {
        Ok(self.memory.load_decisions()?.decisions)
    }

    pub fn list_active(&self) -> NexusResult<Vec<Decision>> {
        Ok(self
            .memory
            .load_decisions()?
            .decisions
            .into_iter()
            .filter(|d| d.status == DecisionStatus::Active)
            .collect())
    }

    pub fn format_all(&self) -> NexusResult<String> {
        let decisions = self.list()?;
        if decisions.is_empty() {
            return Ok("No decisions recorded yet.".to_string());
        }

        let mut output = String::from("ARCHITECTURAL DECISIONS\n\n");
        for (i, d) in decisions.iter().enumerate() {
            output.push_str(&format!(
                "{}. [{}] {}\n",
                i + 1,
                d.created_at.format("%Y-%m-%d %H:%M UTC"),
                d.content
            ));
            if let Some(r) = &d.rationale {
                output.push_str(&format!("   Rationale: {r}\n"));
            }
            if !d.tags.is_empty() {
                output.push_str(&format!("   Tags: {}\n", d.tags.join(", ")));
            }
            output.push('\n');
        }
        Ok(output)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

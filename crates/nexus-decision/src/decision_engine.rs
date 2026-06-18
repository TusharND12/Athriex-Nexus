use std::collections::HashMap;

use chrono::Utc;
use nexus_core::{Decision, DecisionStatus, NexusResult, TimelineEvent, TimelineEventKind};
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
        self.record_with_supersedes(content, rationale, tags, author, None)
    }

    /// Record a decision, optionally marking a prior decision as superseded.
    ///
    /// `supersedes` matches an existing decision by id prefix (so users can pass
    /// the short id shown in `nexus decisions`). When superseding, the identical
    /// active-decision dedup is skipped so the replacement is always created.
    pub fn record_with_supersedes(
        &self,
        content: impl Into<String>,
        rationale: Option<String>,
        tags: Vec<String>,
        author: Option<String>,
        supersedes: Option<String>,
    ) -> NexusResult<Decision> {
        let content = content.into();
        let mut store = self.memory.load_decisions()?;

        let mut superseded_id: Option<Uuid> = None;
        if let Some(prefix) = supersedes
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty())
        {
            match store
                .decisions
                .iter_mut()
                .find(|d| d.id.to_string().starts_with(prefix))
            {
                Some(target) => {
                    target.status = DecisionStatus::Superseded;
                    superseded_id = Some(target.id);
                }
                None => {
                    return Err(nexus_core::NexusError::Other(format!(
                        "no decision matching id '{prefix}' to supersede"
                    )));
                }
            }
        }

        let normalized = content.trim().to_lowercase();
        if superseded_id.is_none() {
            if let Some(existing) = store.decisions.iter().find(|d| {
                d.status == DecisionStatus::Active && d.content.trim().to_lowercase() == normalized
            }) {
                return Ok(existing.clone());
            }
        }

        let decision = Decision {
            id: Uuid::new_v4(),
            content,
            rationale,
            tags,
            created_at: Utc::now(),
            author,
            status: DecisionStatus::Active,
        };

        store.decisions.push(decision.clone());
        self.memory.save_decisions(&store)?;
        // Re-sync the DB so the superseded row's status is updated too.
        self.memory.sync_stores_to_db()?;

        self.memory.append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::Decision,
            title: truncate(&decision.content, 80),
            description: decision.rationale.clone(),
            timestamp: decision.created_at,
            metadata: serde_json::json!({
                "decision_id": decision.id,
                "supersedes": superseded_id.map(|id| id.to_string()),
            }),
        })?;

        Ok(decision)
    }

    pub fn list(&self) -> NexusResult<Vec<Decision>> {
        Ok(self.memory.load_decisions()?.decisions)
    }

    pub fn list_active(&self) -> NexusResult<Vec<Decision>> {
        let mut by_content: HashMap<String, Decision> = HashMap::new();
        for decision in self
            .memory
            .load_decisions()?
            .decisions
            .into_iter()
            .filter(|d| d.status == DecisionStatus::Active)
        {
            let key = decision.content.trim().to_lowercase();
            by_content
                .entry(key)
                .and_modify(|existing| {
                    if decision.created_at > existing.created_at {
                        *existing = decision.clone();
                    }
                })
                .or_insert(decision);
        }

        let mut decisions: Vec<_> = by_content.into_values().collect();
        decisions.sort_by_key(|d| std::cmp::Reverse(d.created_at));
        Ok(decisions)
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
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let kept: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{kept}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core::NexusPaths;
    use tempfile::TempDir;

    fn engine(tmp: &TempDir) -> MemoryEngine {
        let paths = NexusPaths::from_project_root(tmp.path());
        MemoryEngine::init(paths, "test").unwrap()
    }

    #[test]
    fn truncate_does_not_panic_on_multibyte() {
        // Each '✓' is 3 bytes; byte-slicing here would previously panic.
        let s = "✓✓✓✓✓ decision text";
        let out = truncate(s, 3);
        assert!(out.ends_with('…'));
        assert_eq!(out.chars().count(), 3);
    }

    #[test]
    fn supersede_marks_prior_decision_and_creates_new() {
        let tmp = TempDir::new().unwrap();
        let mem = engine(&tmp);
        let de = DecisionEngine::new(&mem);

        let first = de.record("Use JSON storage", None, vec![], None).unwrap();
        let prefix = first.id.to_string()[..8].to_string();

        let second = de
            .record_with_supersedes("Use SQLite storage", None, vec![], None, Some(prefix))
            .unwrap();

        assert_ne!(first.id, second.id);

        // Only the new decision should be active.
        let active = de.list_active().unwrap();
        assert!(active.iter().any(|d| d.id == second.id));
        assert!(!active.iter().any(|d| d.id == first.id));

        let all = de.list().unwrap();
        let old = all.iter().find(|d| d.id == first.id).unwrap();
        assert_eq!(old.status, DecisionStatus::Superseded);
    }

    #[test]
    fn supersede_unknown_id_errors() {
        let tmp = TempDir::new().unwrap();
        let mem = engine(&tmp);
        let de = DecisionEngine::new(&mem);
        let err = de
            .record_with_supersedes("X", None, vec![], None, Some("deadbeef".to_string()))
            .unwrap_err();
        assert!(err.to_string().contains("supersede"));
    }
}

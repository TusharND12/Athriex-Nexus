use chrono::Utc;
use nexus_context::ContextEngine;
use nexus_core::{ContinuationContext, NexusResult, TimelineEvent, TimelineEventKind};
use nexus_memory::MemoryEngine;
use uuid::Uuid;

pub struct HandoffEngine<'a> {
    memory: &'a MemoryEngine,
}

impl<'a> HandoffEngine<'a> {
    pub fn new(memory: &'a MemoryEngine) -> Self {
        Self { memory }
    }

    pub fn generate(&self, compress: bool, max_tokens: usize) -> NexusResult<String> {
        let ctx = ContextEngine::new(self.memory).continue_context(compress, max_tokens)?;
        let handoff = format_handoff(&ctx);

        self.memory.append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::Handoff,
            title: "AI handoff generated".to_string(),
            description: Some(format!("~{} tokens", ctx.token_estimate)),
            timestamp: Utc::now(),
            metadata: serde_json::json!({ "compressed": compress }),
        })?;

        Ok(handoff)
    }
}

fn format_handoff(ctx: &ContinuationContext) -> String {
    format!(
        r#"<nexus-handoff version="1.0">
<!-- Athreix Nexus — Universal AI Continuation Handoff -->
<!-- Compatible: ChatGPT, Claude, Gemini, Cursor, Windsurf, Codex, open agents -->
<!-- Generated: {} -->

<system-context>
You are receiving a pre-built project continuation context from Athreix Nexus.
This context replaces the need for the developer to re-explain the project.
Treat all architectural decisions as binding constraints.
</system-context>

<continuation-prompt>
{}
</continuation-prompt>

<metadata>
  <token-estimate>{}</token-estimate>
  <compressed>{}</compressed>
  <current-task>{}</current-task>
</metadata>
</nexus-handoff>"#,
        ctx.generated_at.to_rfc3339(),
        ctx.ai_continuation_prompt,
        ctx.token_estimate,
        ctx.compressed,
        escape_xml(&ctx.current_task),
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

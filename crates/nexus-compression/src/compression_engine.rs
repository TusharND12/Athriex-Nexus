use nexus_core::ContinuationContext;

pub struct CompressionEngine;

impl CompressionEngine {
    /// Compress continuation context for AI consumption while preserving signal.
    pub fn compress(ctx: &ContinuationContext, max_tokens: usize) -> ContinuationContext {
        let mut compressed = ctx.clone();
        compressed.compressed = true;
        compressed.token_estimate = estimate_tokens(&compressed.ai_continuation_prompt);

        if compressed.token_estimate <= max_tokens {
            return compressed;
        }

        let ratio = max_tokens as f32 / compressed.token_estimate as f32;

        compressed.completed_work = truncate_list(&compressed.completed_work, ratio);
        compressed.important_decisions = truncate_list(&compressed.important_decisions, ratio);
        compressed.risks = truncate_list(&compressed.risks, ratio);
        compressed.important_files.truncate((compressed.important_files.len() as f32 * ratio) as usize);

        compressed.architecture_summary = truncate_text(&compressed.architecture_summary, ratio);
        compressed.project_overview = truncate_text(&compressed.project_overview, ratio);

        compressed.ai_continuation_prompt = Self::build_compressed_prompt(&compressed);
        compressed.token_estimate = estimate_tokens(&compressed.ai_continuation_prompt);

        compressed
    }

    fn build_compressed_prompt(ctx: &ContinuationContext) -> String {
        format!(
            r#"# ATHREIX NEXUS — COMPRESSED CONTINUATION CONTEXT

## PROJECT
{}

## CURRENT TASK
{}

## COMPLETED ({})
{}

## DECISIONS ({})
{}

## ARCHITECTURE
{}

## KEY FILES
{}

## RISKS
{}

## NEXT ACTION
{}

---
Continue development. Respect recorded decisions. Do not re-ask resolved questions.
"#,
            ctx.project_overview,
            ctx.current_task,
            ctx.completed_work.len(),
            ctx.completed_work.join("; "),
            ctx.important_decisions.len(),
            ctx.important_decisions.join("; "),
            ctx.architecture_summary,
            ctx.important_files
                .iter()
                .map(|f| format!("{} ({})", f.path, f.reason))
                .collect::<Vec<_>>()
                .join(", "),
            ctx.risks.join("; "),
            ctx.next_recommended_action,
        )
    }
}

pub fn estimate_tokens(text: &str) -> usize {
    (text.len() / 4).max(1)
}

fn truncate_list(items: &[String], ratio: f32) -> Vec<String> {
    let keep = ((items.len() as f32 * ratio).ceil() as usize).max(1).min(items.len());
    items.iter().take(keep).cloned().collect()
}

fn truncate_text(text: &str, ratio: f32) -> String {
    let max_chars = ((text.len() as f32 * ratio) as usize).max(100);
    if text.len() <= max_chars {
        text.to_string()
    } else {
        format!("{}…", &text[..max_chars.saturating_sub(1)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use nexus_core::ContinuationContext;

    fn sample_context() -> ContinuationContext {
        ContinuationContext {
            generated_at: Utc::now(),
            project_overview: "Test project".repeat(500),
            completed_work: (0..50).map(|i| format!("work item {i}")).collect(),
            current_task: "implement feature".to_string(),
            important_decisions: (0..30).map(|i| format!("decision {i}")).collect(),
            important_files: vec![],
            architecture_summary: "layered architecture".repeat(200),
            risks: vec!["risk a".into(), "risk b".into()],
            next_recommended_action: "continue coding".to_string(),
            ai_continuation_prompt: "full prompt".repeat(1000),
            compressed: false,
            token_estimate: 0,
        }
    }

    #[test]
    fn compression_reduces_token_estimate() {
        let ctx = sample_context();
        let before = estimate_tokens(&ctx.ai_continuation_prompt);
        let compressed = CompressionEngine::compress(&ctx, 500);
        assert!(compressed.compressed);
        assert!(compressed.token_estimate <= before);
    }
}

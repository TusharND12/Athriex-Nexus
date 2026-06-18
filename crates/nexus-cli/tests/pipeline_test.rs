use nexus_context::ContextEngine;
use nexus_core::NexusPaths;
use nexus_decision::DecisionEngine;
use nexus_memory::MemoryEngine;
use nexus_scanner::ScannerEngine;
use tempfile::TempDir;

#[test]
fn full_pipeline_init_scan_decide_continue() {
    let tmp = TempDir::new().unwrap();
    let paths = NexusPaths::from_project_root(tmp.path());

    let engine = MemoryEngine::init(paths.clone(), "pipeline-test").unwrap();

    // Minimal source file for scanner
    std::fs::write(
        tmp.path().join("main.rs"),
        "fn main() { println!(\"hello\"); }",
    )
    .unwrap();

    ScannerEngine::new(&engine).scan().unwrap();

    DecisionEngine::new(&engine)
        .record(
            "Use Rust for the CLI core",
            Some("Performance and cross-platform binary distribution".into()),
            vec!["architecture".into()],
            None,
        )
        .unwrap();

    let ctx = ContextEngine::new(&engine)
        .continue_context(false, 8000)
        .unwrap();

    assert!(!ctx.project_overview.is_empty());
    assert!(!ctx.ai_continuation_prompt.is_empty());
    assert!(ctx.important_decisions.iter().any(|d| d.contains("Rust")));
}

#[test]
fn continue_with_task_focuses_relevant_memory() {
    let tmp = TempDir::new().unwrap();
    let paths = NexusPaths::from_project_root(tmp.path());
    let engine = MemoryEngine::init(paths, "focus-test").unwrap();
    let de = DecisionEngine::new(&engine);

    de.record("Use Tailwind for styling the frontend", None, vec![], None)
        .unwrap();
    de.record(
        "Use SQLite for offline storage persistence",
        None,
        vec![],
        None,
    )
    .unwrap();
    de.record("Adopt rayon for parallel scanning", None, vec![], None)
        .unwrap();

    let ctx = ContextEngine::new(&engine)
        .continue_context_for(false, 8000, Some("storage persistence".into()))
        .unwrap();

    // The focus task drives the current task and ranks the matching decision first.
    assert_eq!(ctx.current_task, "storage persistence");
    assert!(
        ctx.important_decisions[0].to_lowercase().contains("sqlite"),
        "expected SQLite decision first, got: {:?}",
        ctx.important_decisions
    );
}

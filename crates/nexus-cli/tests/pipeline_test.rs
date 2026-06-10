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
    std::fs::write(tmp.path().join("main.rs"), "fn main() { println!(\"hello\"); }").unwrap();

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

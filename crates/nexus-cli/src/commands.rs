use std::fs;
use std::io::Write;

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use nexus_context::{ContextEngine, HealthAnalyzer};
use nexus_core::{AiSession, NexusPaths, TimelineEvent, TimelineEventKind};
use nexus_decision::DecisionEngine;
use nexus_handoff::HandoffEngine;
use nexus_knowledge::KnowledgeEngine;
use nexus_memory::MemoryEngine;
use nexus_nxp::NxpEngine;
use nexus_scanner::{structure::format_architecture_tree, ScannerEngine};
use nexus_search::SearchEngine;
use nexus_snapshot::SnapshotEngine;
use uuid::Uuid;

fn open_engine(paths: &NexusPaths) -> Result<MemoryEngine> {
    MemoryEngine::open(paths.clone()).map_err(|e| anyhow::anyhow!(e))
}

pub fn init(paths: &NexusPaths, name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or_else(|| {
        paths
            .project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed-project")
            .to_string()
    });

    let engine = MemoryEngine::init(paths.clone(), project_name)?;
    let nxp = NxpEngine::new(&engine);
    let nxp_path = nxp.write_to_file()?;

    println!("{} Athreix Nexus initialized", "✓".green().bold());
    println!("  Memory:   {}", engine.paths.nexus_root.display());
    println!("  Protocol: {}", nxp_path.display());
    println!();
    println!("Next steps:");
    println!("  {}  Analyze your project", "nexus scan".cyan());
    println!("  {}  Generate AI context", "nexus continue".cyan());
    Ok(())
}

pub fn scan(paths: &NexusPaths) -> Result<()> {
    let engine = open_engine(paths)?;
    println!("{} Scanning project...", "→".cyan());

    let scanner = ScannerEngine::new(&engine);
    let result = scanner.scan()?;

    let knowledge = KnowledgeEngine::new(&engine);
    knowledge.build_from_scan(&result)?;

    println!("{} Scan complete", "✓".green().bold());
    println!("  Files analyzed: {}", result.files_analyzed);
    println!(
        "  Languages: {}",
        result
            .languages
            .iter()
            .map(|l| format!("{} ({})", l.language, l.file_count))
            .collect::<Vec<_>>()
            .join(", ")
    );
    if !result.frameworks.is_empty() {
        println!("  Frameworks: {}", result.frameworks.join(", "));
    }
    Ok(())
}

pub fn r#continue(
    paths: &NexusPaths,
    compress: bool,
    max_tokens: usize,
    output: Option<std::path::PathBuf>,
    task: Option<String>,
) -> Result<()> {
    let engine = open_engine(paths)?;
    let ctx_engine = ContextEngine::new(&engine);
    let ctx = ctx_engine
        .continue_context_for(compress, max_tokens, task)
        .map_err(|e| anyhow::anyhow!(e))?;
    let formatted = ctx_engine.format_continue_output(&ctx);

    write_output(&formatted, output.as_deref())?;
    Ok(())
}

pub fn handoff(
    paths: &NexusPaths,
    compress: bool,
    max_tokens: usize,
    output: Option<std::path::PathBuf>,
) -> Result<()> {
    let engine = open_engine(paths)?;
    let handoff = HandoffEngine::new(&engine)
        .generate(compress, max_tokens)
        .map_err(|e| anyhow::anyhow!(e))?;

    write_output(&handoff, output.as_deref())?;
    Ok(())
}

pub fn snapshot(paths: &NexusPaths, label: String, description: Option<String>) -> Result<()> {
    let engine = open_engine(paths)?;
    let manifest = SnapshotEngine::new(&engine)
        .create(label, description)
        .map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "{} Snapshot created: {}",
        "✓".green().bold(),
        manifest.label
    );
    println!("  ID:        {}", manifest.id);
    println!("  Decisions: {}", manifest.decisions_count);
    println!("  Tasks:     {}", manifest.tasks_count);
    Ok(())
}

pub fn restore(paths: &NexusPaths, id: String) -> Result<()> {
    let engine = open_engine(paths)?;
    let manifest = SnapshotEngine::new(&engine)
        .restore(&id)
        .map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "{} Restored snapshot: {}",
        "✓".green().bold(),
        manifest.label
    );
    Ok(())
}

pub fn timeline(paths: &NexusPaths, limit: usize) -> Result<()> {
    let engine = open_engine(paths)?;
    let timeline = engine.load_timeline()?;

    println!("{}", "PROJECT TIMELINE".bold());
    println!();

    let events: Vec<_> = timeline.events.iter().rev().take(limit).collect();
    if events.is_empty() {
        println!("  No events recorded.");
        return Ok(());
    }

    for event in events {
        let kind = format!("{:?}", event.kind).to_lowercase();
        println!(
            "  {} {} {}",
            event
                .timestamp
                .format("%Y-%m-%d %H:%M")
                .to_string()
                .dimmed(),
            format!("[{kind}]").cyan(),
            event.title
        );
        if let Some(desc) = &event.description {
            println!("           {}", desc.dimmed());
        }
    }
    Ok(())
}

pub fn decide(
    paths: &NexusPaths,
    content: String,
    rationale: Option<String>,
    tags: Vec<String>,
    supersedes: Option<String>,
) -> Result<()> {
    let engine = open_engine(paths)?;
    let had_supersedes = supersedes.is_some();
    let decision = DecisionEngine::new(&engine)
        .record_with_supersedes(content, rationale, tags, None, supersedes)
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{} Decision recorded", "✓".green().bold());
    println!("  ID: {}", decision.id);
    println!("  {}", decision.content);
    if had_supersedes {
        println!("  {}", "(superseded prior decision)".dimmed());
    }
    Ok(())
}

pub fn decisions(paths: &NexusPaths) -> Result<()> {
    let engine = open_engine(paths)?;
    let output = DecisionEngine::new(&engine)
        .format_all()
        .map_err(|e| anyhow::anyhow!(e))?;
    print!("{output}");
    Ok(())
}

pub fn ask(paths: &NexusPaths, query: String, limit: usize) -> Result<()> {
    let engine = open_engine(paths)?;
    let output = SearchEngine::new(&engine)
        .format_answer(&query, limit)
        .map_err(|e| anyhow::anyhow!(e))?;
    print!("{output}");
    Ok(())
}

pub fn health(paths: &NexusPaths) -> Result<()> {
    let engine = open_engine(paths)?;
    let report = HealthAnalyzer::new(&engine)
        .analyze()
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", "PROJECT HEALTH".bold());
    println!();
    println!("  Overall Score: {}", score_bar(report.overall_score));
    println!();
    print_metric("Technical Debt", &report.technical_debt);
    print_metric("Dependency Risk", &report.dependency_risk);
    print_metric("Documentation", &report.documentation);
    print_metric("Complexity", &report.complexity);
    print_metric("Maintainability", &report.maintainability);

    if !report.recommendations.is_empty() {
        println!();
        println!("{}", "Recommendations:".bold());
        for rec in &report.recommendations {
            println!("  • {rec}");
        }
    }
    Ok(())
}

pub fn impact(paths: &NexusPaths, file: String) -> Result<()> {
    let engine = open_engine(paths)?;
    let report = KnowledgeEngine::new(&engine)
        .impact(&file)
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{}", "IMPACT ANALYSIS".bold());
    println!();
    if report.matched_files.is_empty() {
        println!(
            "  No file in the knowledge graph matches {}.",
            file.cyan()
        );
        println!("  Run `nexus scan` first, or check the path.");
        return Ok(());
    }

    println!("  Matched files:");
    for f in &report.matched_files {
        println!("    • {f}");
    }
    println!();
    if report.links.is_empty() {
        println!("  No connected decisions, tasks, or modules recorded yet.");
        return Ok(());
    }
    println!("  Connected nodes ({}):", report.links.len());
    for link in &report.links {
        println!(
            "    {} {} — {}",
            format!("[{}]", link.kind).cyan(),
            link.name,
            link.relation.dimmed()
        );
    }
    Ok(())
}

pub fn map(paths: &NexusPaths) -> Result<()> {
    let engine = open_engine(paths)?;
    let architecture = engine.load_architecture()?;

    println!("{}", "ARCHITECTURE MAP".bold());
    println!();
    if architecture.layers.is_empty() {
        println!("  No architecture data. Run `nexus scan` first.");
        return Ok(());
    }
    print!("{}", format_architecture_tree(&architecture.layers));
    Ok(())
}

pub fn export_nxp(paths: &NexusPaths) -> Result<()> {
    let engine = open_engine(paths)?;
    let path = NxpEngine::new(&engine)
        .write_to_file()
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{} Exported NXP protocol file", "✓".green().bold());
    println!("  {}", path.display());
    Ok(())
}

pub fn import_nxp(paths: &NexusPaths, file: Option<std::path::PathBuf>) -> Result<()> {
    let engine = open_engine(paths)?;
    let path = file.unwrap_or_else(|| paths.nxp_file());
    let doc = NxpEngine::read_from_file(&path).map_err(|e| anyhow::anyhow!(e))?;
    NxpEngine::new(&engine)
        .import(&doc)
        .map_err(|e| anyhow::anyhow!(e))?;

    println!(
        "{} Imported NXP from {}",
        "✓".green().bold(),
        path.display()
    );
    Ok(())
}

pub fn session(
    paths: &NexusPaths,
    tool: Option<String>,
    prompt: String,
    response: String,
    files: Vec<String>,
    notes: Option<String>,
) -> Result<()> {
    let engine = open_engine(paths)?;
    let session = AiSession {
        id: Uuid::new_v4(),
        tool,
        prompt,
        response,
        files_modified: files,
        timestamp: Utc::now(),
        notes,
        tags: vec![],
    };

    let path = engine
        .save_session(&session)
        .map_err(|e| anyhow::anyhow!(e))?;

    engine
        .append_timeline_event(TimelineEvent {
            id: Uuid::new_v4(),
            kind: TimelineEventKind::Session,
            title: "AI session recorded".to_string(),
            description: None,
            timestamp: session.timestamp,
            metadata: serde_json::json!({ "session_id": session.id }),
        })
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("{} Session saved", "✓".green().bold());
    println!("  {}", path.display());
    Ok(())
}

fn write_output(content: &str, output: Option<&std::path::Path>) -> Result<()> {
    match output {
        Some(path) => {
            fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
            eprintln!(
                "{} Wrote {} (~{} tokens)",
                "✓".green().bold(),
                path.display(),
                content.len() / 4
            );
        }
        None => {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(content.as_bytes())?;
        }
    }
    Ok(())
}

fn score_bar(score: f32) -> String {
    let filled = (score / 10.0).round() as usize;
    let bar: String = (0..10)
        .map(|i| if i < filled { '█' } else { '░' })
        .collect();
    format!("{bar} {:.1}/100", score)
}

fn print_metric(name: &str, metric: &nexus_core::HealthMetric) {
    println!(
        "  {:<18} {} — {}",
        name,
        score_bar(metric.score),
        metric.summary
    );
}

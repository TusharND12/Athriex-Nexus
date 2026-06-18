mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use nexus_core::NexusPaths;

#[derive(Parser)]
#[command(
    name = "nexus",
    version,
    about = "Athreix Nexus — Never explain your project to an AI twice.",
    long_about = "Local-first, offline-first persistent memory layer for software projects.\nGit stores source code. Nexus stores project knowledge."
)]
struct Cli {
    /// Project directory (defaults to current directory)
    #[arg(short, long, global = true)]
    path: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .nexus/ memory layer in the current project
    Init {
        /// Project name (defaults to directory name)
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Analyze source code, dependencies, and git history
    Scan,
    /// Reconstruct complete project understanding (flagship command)
    Continue {
        /// Compress output for smaller context windows
        #[arg(short, long)]
        compress: bool,
        /// Max tokens when compressing
        #[arg(long, default_value = "8000")]
        max_tokens: usize,
        /// Write output to file
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        /// Focus context on a specific task (ranks memory by relevance to it)
        #[arg(short, long)]
        task: Option<String>,
    },
    /// Generate universal AI continuation handoff prompt
    Handoff {
        #[arg(short, long)]
        compress: bool,
        #[arg(long, default_value = "8000")]
        max_tokens: usize,
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    /// Create a memory checkpoint
    Snapshot {
        /// Snapshot label
        label: String,
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Restore a previous memory checkpoint
    Restore {
        /// Snapshot ID prefix or label
        id: String,
    },
    /// Display chronological project evolution
    Timeline {
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Record an architectural decision
    Decide {
        /// Decision text
        content: String,
        #[arg(short, long)]
        rationale: Option<String>,
        #[arg(short, long)]
        tag: Vec<String>,
        /// Mark a prior decision (by id prefix) as superseded by this one
        #[arg(short, long)]
        supersedes: Option<String>,
    },
    /// Display all stored decisions
    Decisions,
    /// Semantic search across project memory
    Ask {
        /// Search query
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Analyze project health score
    Health,
    /// Generate architecture map
    Map,
    /// Trace what a file connects to in the knowledge graph
    Impact {
        /// File path or fragment to analyze
        file: String,
    },
    /// Export project.nxp (Nexus Protocol)
    Export,
    /// Import project.nxp
    Import {
        #[arg(short, long)]
        file: Option<std::path::PathBuf>,
    },
    /// Record an AI session
    Session {
        #[arg(short, long)]
        tool: Option<String>,
        #[arg(short, long)]
        prompt: String,
        #[arg(short, long)]
        response: String,
        #[arg(short, long)]
        file: Vec<String>,
        #[arg(short, long)]
        notes: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = match cli.path {
        Some(p) => p,
        None => std::env::current_dir()
            .map_err(|e| anyhow::anyhow!("cannot determine current directory: {e}"))?,
    };
    let paths = NexusPaths::from_project_root(&cwd);

    match cli.command {
        Commands::Init { name } => commands::init(&paths, name),
        Commands::Scan => commands::scan(&paths),
        Commands::Continue {
            compress,
            max_tokens,
            output,
            task,
        } => commands::r#continue(&paths, compress, max_tokens, output, task),
        Commands::Handoff {
            compress,
            max_tokens,
            output,
        } => commands::handoff(&paths, compress, max_tokens, output),
        Commands::Snapshot { label, description } => commands::snapshot(&paths, label, description),
        Commands::Restore { id } => commands::restore(&paths, id),
        Commands::Timeline { limit } => commands::timeline(&paths, limit),
        Commands::Decide {
            content,
            rationale,
            tag,
            supersedes,
        } => commands::decide(&paths, content, rationale, tag, supersedes),
        Commands::Decisions => commands::decisions(&paths),
        Commands::Ask { query, limit } => commands::ask(&paths, query, limit),
        Commands::Health => commands::health(&paths),
        Commands::Map => commands::map(&paths),
        Commands::Impact { file } => commands::impact(&paths, file),
        Commands::Export => commands::export_nxp(&paths),
        Commands::Import { file } => commands::import_nxp(&paths, file),
        Commands::Session {
            tool,
            prompt,
            response,
            file,
            notes,
        } => commands::session(&paths, tool, prompt, response, file, notes),
    }
}

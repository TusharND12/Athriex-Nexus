# Athreix Nexus — Complete Guide

**Tagline:** Never explain your project to an AI twice.

**Version:** 0.1.0  
**Author:** Tushar Dhokane  
**License:** MIT

This document consolidates all Athreix Nexus documentation into one reference: project overview, architecture, commands, database design, NXP protocol, installation, and roadmap.

---

## Part 1 — Project Overview

### What Is Athreix Nexus?

Athreix Nexus is a **local-first, offline-first command-line tool** that acts as a persistent memory layer for software projects. Git stores source code. Athreix Nexus stores project knowledge.

| Tool | Stores |
|------|--------|
| Git | Source code, commits, branches |
| Athreix Nexus | Architecture, decisions, progress, AI sessions, project context |

### The Problem

Developers constantly lose context when:

- Chat sessions expire or context windows fill up
- Switching between Claude, ChatGPT, Gemini, Cursor, Windsurf, VS Code, and terminals
- Moving between devices and operating systems
- Team members join mid-project
- Long-term projects evolve over months

### The Solution

Athreix Nexus continuously understands what the project is, what has been built, current progress, architecture, dependencies, technical decisions, important files, git history, pending work, previous AI sessions, and roadmap — then generates a complete **AI continuation context** that any AI system can immediately understand.

### What It Is NOT

- Not a SaaS dashboard
- Not a project management tool
- Not a note-taking app
- Not cloud-dependent
- Not a web application

### What It IS

- **Git for AI Context** — versioned, local project memory
- **Memory OS for software projects** — persistent knowledge per repository
- **Universal AI handoff infrastructure** — one context, any AI tool

### Core Philosophy

Everything works from the terminal. No dashboard. No web dependency. No cloud dependency. All project intelligence is stored locally and works offline.

---

## Part 2 — Technology Stack

| Layer | Technology |
|-------|------------|
| Core language | Rust |
| Storage | SQLite + human-readable JSON files |
| Code analysis | tree-sitter |
| Version control integration | Git via libgit2 |
| Serialization | JSON and TOML |
| Full-text search | SQLite FTS5 |
| Packaging | Cargo |
| Testing | Rust native test ecosystem |

### Supported Platforms

| Environment | Supported |
|-------------|-----------|
| Windows CMD | Yes |
| Windows PowerShell | Yes |
| Windows Terminal | Yes |
| WSL | Yes |
| Linux (Bash, Zsh, Fish) | Yes |
| macOS Terminal | Yes |

---

## Part 3 — System Architecture

### Vision

Athreix Nexus is the universal memory layer between software projects and AI systems. It eliminates context loss across chat sessions, tools, devices, and team members.

### Architectural Layers

The system follows **Clean Architecture** with four layers:

1. **CLI Layer (Interface)** — Receives user commands, displays output. Only layer that talks to the user.
2. **Engine Layer (Use Cases)** — Orchestrates business logic. Each command maps to one or more engines.
3. **Core Layer (Domain)** — Shared types, data models, path constants, and error definitions.
4. **Memory Layer (Infrastructure)** — Filesystem persistence, SQLite database, serialization.

**Dependency rule:** Inner layers never depend on outer layers. Engines depend on core and memory. Only the CLI depends on all engines.

### The 14 Engines

| Engine | Crate | Responsibility |
|--------|-------|----------------|
| CLI Layer | nexus-cli | Argument parsing, command routing, formatted output |
| Core | nexus-core | Shared types, models, paths, errors |
| Memory Engine | nexus-memory | Initialize projects, JSON persistence, SQLite, FTS indexing |
| Context Engine | nexus-context | Reconstruct full project state for `nexus continue` |
| Scanner Engine | nexus-scanner | tree-sitter code analysis, dependency detection, structure mapping |
| Git Engine | nexus-git | Branch, commits, dirty files via libgit2 |
| Decision Engine | nexus-decision | Record and list architectural decisions (ADRs) |
| Snapshot Engine | nexus-snapshot | Create and restore memory checkpoints |
| Knowledge Engine | nexus-knowledge | Build and persist the project knowledge graph |
| Search Engine | nexus-search | Full-text search across all project memory |
| Compression Engine | nexus-compression | Reduce context size for AI token limits |
| Handoff Engine | nexus-handoff | Generate universal AI handoff documents |
| NXP Engine | nexus-nxp | Export and import the portable Nexus Protocol file |

### Local Project Structure

When Nexus is initialized in any project, it creates a `.nexus/` folder:

| File / Folder | Purpose |
|---------------|---------|
| memory.json | Project identity, description, technologies, focus, risks |
| architecture.json | Layers, technologies, dependencies, important files |
| timeline.json | Chronological log of all Nexus events |
| decisions.json | Architectural decision records |
| tasks.json | Work items and their status |
| knowledge.db | SQLite database for search, graph, and session index |
| project.nxp | Portable Nexus Protocol export file |
| sessions/ | Individual AI session archive files |
| snapshots/ | Point-in-time memory checkpoints |

Your existing source code is never modified. Nexus only adds the `.nexus/` folder.

### Storage Strategy — Dual Persistence

JSON files serve as the **human-readable source of truth** — easy to read, diff, and commit to Git. SQLite provides **query acceleration** and full-text search. On every write, engines update JSON first, then sync to SQLite.

### Data Flow for `nexus continue` (Flagship Command)

1. Memory Engine loads all JSON stores and queries SQLite
2. Git Engine summarizes repository state (branch, commits, uncommitted files)
3. Decision Engine loads all active architectural decisions
4. Context Engine merges all sources into a unified continuation context
5. Compression Engine optionally reduces output for smaller AI context windows
6. CLI renders the formatted output with all sections

### Unique Internal Systems

**Context Compression Engine** — Compresses thousands of files, hundreds of commits, and hundreds of decisions into a compact context package optimized for AI consumption while preserving current task signal, binding decisions, architecture summary, and risk awareness.

**AI Session Recorder** — Stores prompt, response, files modified, timestamp, and notes. Sessions are archived in `.nexus/sessions/` and indexed in SQLite for search via `nexus ask`.

**Project Knowledge Graph** — Tracks relationships between files, functions, services, APIs, databases, tasks, decisions, modules, and packages. Edge types include: imports, calls, depends on, implements, related to, owns, and documents.

**NXP Protocol** — A portable, versioned, human-readable file format (project.nxp) containing everything required to reconstruct full project understanding on any machine.

### Security Model

- 100% local operation — no network calls ever made
- No telemetry — project knowledge never leaves the disk
- `.nexus/` can be committed to Git or gitignored per team preference
- Secrets in source code are not extracted or transmitted

### Future Plugin Architecture (Planned v0.3)

Plugins will live in `.nexus/plugins/` and support custom scanners, session importers for AI tools, and optional local embedding models. A `NexusPlugin` trait will allow extending scan and continue behavior without modifying core code.

---

## Part 4 — Complete Command Reference

### Recommended Workflow Order

init → scan → decide → map → health → continue → snapshot → handoff → export

---

### Setup Commands

#### nexus init

**What it does:** Creates the `.nexus/` memory layer in the current project directory.

**When to use:** Once per project, at the very beginning.

**Creates:** memory.json, architecture.json, timeline.json, decisions.json, tasks.json, knowledge.db, project.nxp, sessions folder, and snapshots folder.

**Importance:** Foundation command. Nothing else works without initialization.

**Options:** `--name` to set the project name (defaults to folder name).

---

#### nexus scan

**What it does:** Analyzes the entire project — source code, dependencies, folder structure, frameworks, technologies, and git history. Updates architecture.json and the knowledge graph.

**When to use:** After init, and again whenever the project changes significantly.

**Detects:** Programming languages (Rust, JavaScript, TypeScript, Python, and more), frameworks (React, Next.js, Actix, Django, etc.), entry points, important files, and dependency manifests (Cargo.toml, package.json, pyproject.toml, requirements.txt).

**Importance:** Gives Nexus real understanding of your codebase. Required for accurate `nexus continue` output.

---

### Memory Commands

#### nexus decide

**What it does:** Records an architectural decision with automatic timestamp. Persisted to decisions.json and indexed in SQLite.

**When to use:** Whenever you make a technical choice worth remembering permanently.

**Options:** `--rationale` for why the decision was made, `--tag` for categorization.

**Importance:** Decisions become binding constraints in AI continuation prompts. AI tools will not re-ask resolved questions.

**Example decisions:** "Use PostgreSQL for relational consistency", "Use Redis for session caching", "All API routes require JWT authentication".

---

#### nexus decisions

**What it does:** Displays all stored architectural decisions in chronological order with timestamps, rationale, and tags.

**When to use:** Before starting new work, to review what has already been decided.

---

#### nexus session

**What it does:** Records an AI coding session — tool name, prompt, response, files modified, timestamp, and optional notes. Saved to `.nexus/sessions/` and indexed for search.

**When to use:** After important AI coding sessions worth preserving.

**Options:** `--tool` (cursor, chatgpt, claude, etc.), `--prompt`, `--response`, `--file` (modified files), `--notes`.

**Importance:** Builds a searchable history of all AI work done on the project.

---

### Intelligence Commands

#### nexus continue — FLAGSHIP COMMAND

**What it does:** Reconstructs complete project understanding from all memory sources and generates a formatted AI continuation context.

**Output sections:**
- Project Overview
- Completed Work
- Current Task
- Important Decisions
- Important Files
- Architecture Summary
- Risks
- Next Recommended Action
- AI Continuation Prompt (ready to paste into any AI)

**When to use:** At the start of every new AI session. Paste the output into ChatGPT, Claude, Gemini, Cursor, Windsurf, or any coding agent.

**Options:** `--compress` to reduce token count, `--max-tokens` to set compression limit (default 8000), `--output` to save to a file.

**Importance:** This is the core value of the entire product. The reason Athreix Nexus exists.

---

#### nexus handoff

**What it does:** Generates a universal XML handoff document compatible with all major AI platforms.

**When to use:** Handing the project to a different AI tool, team member, or device.

**Options:** `--compress`, `--max-tokens`, `--output` to save to file.

**Importance:** Standardized portable format for cross-platform AI continuity.

---

#### nexus ask

**What it does:** Searches all project memory — decisions, sessions, tasks, and indexed content — using full-text search.

**When to use:** Quick lookup without reading all project files manually.

**Examples:** "Why are we using WebRTC?", "Which file handles authentication?", "What is the current roadmap?"

**Options:** `--limit` to control number of results (default 10).

---

#### nexus map

**What it does:** Displays the architecture tree organized by layers — Frontend, Backend, API, Database, Shared, Infrastructure, Tests, Configuration.

**When to use:** After running `nexus scan`, to understand project structure at a glance.

---

#### nexus health

**What it does:** Analyzes and scores the project across five dimensions, producing an overall health score out of 100 with recommendations.

**Metrics scored:**

| Metric | What it measures |
|--------|-----------------|
| Technical Debt | Number of open tasks |
| Dependency Risk | Number of detected dependencies |
| Documentation | README and documentation files indexed |
| Complexity | Number of architectural layers |
| Maintainability | Decisions recorded and memory freshness |

**When to use:** Periodic project quality check.

---

### Checkpoint Commands

#### nexus snapshot

**What it does:** Creates a memory checkpoint — a complete copy of all memory state at the current moment.

**When to use:** Before major refactors, releases, or experiments.

**Options:** `--description` for additional context about the checkpoint.

**Importance:** Like a Git tag, but for project knowledge rather than source code.

---

#### nexus restore

**What it does:** Restores all memory files to a previous snapshot state.

**When to use:** When you want to roll back project knowledge to an earlier point.

**Argument:** Snapshot ID prefix or label name.

---

#### nexus timeline

**What it does:** Displays chronological project evolution — all Nexus events in order.

**Tracks:** Initialization, scans, decisions, snapshots, snapshot restores, AI sessions, handoffs, milestones, and task updates.

**Options:** `--limit` to control number of events shown (default 50).

---

### Protocol Commands

#### nexus export

**What it does:** Writes or updates `project.nxp` — the portable Nexus Protocol file containing complete project knowledge.

**When to use:** Sharing project context with teammates, backing up knowledge, or moving to another machine.

---

#### nexus import

**What it does:** Imports project knowledge from a `project.nxp` file, overwriting local memory stores and re-indexing SQLite.

**When to use:** Setting up Nexus on a new machine from a teammate's export, or restoring from a backup.

**Options:** `--file` to specify a custom NXP file path (defaults to `.nexus/project.nxp`).

---

## Part 5 — NXP Protocol (Nexus Protocol v1.0.0)

### Overview

NXP is a portable, versioned, human-readable file format for complete project knowledge transfer between machines, team members, and AI systems.

- **File location:** `.nexus/project.nxp`
- **Format:** TOML envelope with embedded data payloads
- **Proposed MIME type:** application/vnd.athreix.nxp+toml

### Design Goals

1. **Portable** — A single file reconstructs full project understanding
2. **Versioned** — Protocol can evolve without breaking existing consumers
3. **Extensible** — Extensions field reserved for future capabilities
4. **Human readable** — Inspectable in any text editor
5. **AI consumable** — Direct input for LLM context loading

### NXP File Fields

| Field | Required | Description |
|-------|----------|-------------|
| protocol_version | Yes | NXP specification version (semver) |
| nexus_version | Yes | Version of the exporting Nexus binary |
| exported_at | Yes | Export timestamp in RFC3339 format |
| project_json | Yes | Project identity, focus, technologies, risks |
| architecture_json | Yes | Layers, technologies, dependencies, important files |
| decisions_json | Yes | All architectural decision records |
| tasks_json | Yes | All work items |
| timeline_json | Yes | Full event chronology |
| knowledge_graph_json | Yes | All knowledge graph nodes and edges |
| recent_sessions_json | Yes | Up to 20 most recent AI sessions |
| git_summary_json | No | Branch, commit count, recent commits, dirty files |
| extensions_json | Yes | Reserved extensibility object |

### Embedded Data — Project Memory

Contains: version, project name, description, created and updated timestamps, technologies list, current focus, completed work list, risks list, and metadata object.

### Embedded Data — Decision

Contains: unique ID, decision content, optional rationale, tags, creation timestamp, optional author, and status (active, superseded, or deprecated).

### Embedded Data — Knowledge Graph

**Node kinds:** file, function, service, api, database, task, decision, module, package

**Edge relations:** imports, calls, depends on, implements, related to, owns, documents

Each node has: ID, kind, name, optional path, and metadata. Each edge has: ID, source node, target node, relation type, and weight.

### Versioning Rules

- **Major version change:** Breaking schema changes such as field removal or type changes
- **Minor version change:** Additive fields that are backward compatible
- **Patch version change:** Documentation or clarification only

Consumers must check protocol_version before parsing, ignore unknown fields, and preserve extensions on round-trip.

### Import Process

When `nexus import` runs: parse the TOML envelope, validate protocol version compatibility (1.x), deserialize all payloads, overwrite JSON stores, re-index SQLite, and regenerate project.nxp.

### AI Consumption Guide

AI systems can load NXP directly by: reading project.nxp, parsing project data for identity and focus, loading decisions as binding constraints, using architecture and knowledge graph for code navigation, checking tasks for current work, and reviewing recent sessions for prior AI context.

### Reserved Extension Keys (v1.0)

embeddings, custom_scanners, team_id, checksum — all reserved for future versions.

---

## Part 6 — SQLite Database Design

**Database file:** `.nexus/knowledge.db`  
**Engine:** SQLite 3 with FTS5 full-text search  
**Current schema version:** 1

### Dual-Write Strategy

JSON files are the source of truth for human editing and Git diffing. SQLite provides query acceleration and full-text search. Both are created on init. On every write, JSON is updated first, then SQLite is synced.

### Tables

#### schema_meta
Tracks the current database schema version. Used for future migrations.

#### decisions
Indexed copy of all architectural decisions. Fields: ID, content, rationale, tags, creation timestamp, author, and status. Indexed by creation date.

#### tasks
Work items with status tracking. Fields: ID, title, description, status (pending, in progress, completed, cancelled), priority (low, medium, high, critical), creation and update timestamps, related files, and blocked-by references. Indexed by status.

#### timeline_events
Queryable event log mirroring timeline.json. Fields: ID, event kind, title, description, timestamp, and metadata. Indexed by timestamp.

#### knowledge_nodes
Knowledge graph vertices. Fields: ID, node kind, name, optional file path, and metadata. Indexed by kind.

#### knowledge_edges
Knowledge graph connections. Fields: ID, source node ID, target node ID, relation type, and weight. Foreign keys reference knowledge_nodes.

#### sessions
AI session search index. Full session records are stored as individual files in `.nexus/sessions/`. Fields: ID, tool name, prompt, response, files modified, timestamp, notes, and tags. Indexed by timestamp.

#### snapshots
Checkpoint registry. Archive files stored in `.nexus/snapshots/`. Fields: ID, label, creation timestamp, description, memory hash, architecture hash, decisions count, tasks count, and archive path.

#### scan_history
Historical scan results for trend analysis over time. Fields: auto-increment ID, scan timestamp, files analyzed count, and full scan result.

#### memory_fts (FTS5 Virtual Table)
Full-text search index powering `nexus ask`. Indexes content from decisions, tasks, and sessions. Uses Porter stemming for better search relevance.

### Planned Future Tables (v2)

- embeddings — for local vector search
- file_symbols — tree-sitter symbol index
- commit_links — linking decisions to specific git commit SHAs

---

## Part 7 — Installation

### Prerequisites

- Rust 1.75 or newer (via rustup)
- Git (optional, improves scan and continue output)
- C compiler (required for native dependencies: libgit2 and SQLite)

### Windows Installation

1. Install Rust via winget or rustup.rs
2. Install Visual Studio 2022 Build Tools with "Desktop development with C++" workload
3. Restart terminal after installation
4. Build the project with Cargo in release mode
5. Binary located at target/release/nexus.exe

**Common Windows issue:** If build fails with "link.exe not found", the C++ workload is not installed. Open Visual Studio Installer, modify Build Tools 2022, and enable Desktop development with C++.

### Linux, WSL, and macOS Installation

1. Install Rust via rustup
2. Install build essentials and libssl-dev (Linux/WSL)
3. Build with Cargo in release mode
4. Optionally copy binary to /usr/local/bin for global access

### Install Methods

| Method | Description |
|--------|-------------|
| Build from source | Recommended for development. Produces target/release/nexus binary. |
| Cargo install | Installs globally to ~/.cargo/bin. Run `nexus` from anywhere. |
| Direct run | Use `cargo run -p nexus-cli` without installing. |

### Shell Compatibility

PowerShell, CMD, Bash, Zsh, Fish, and WSL are all fully supported.

### Setting Up Nexus on a Project

1. Navigate to your project directory
2. Run `nexus init` to create the memory layer
3. Run `nexus scan` to analyze the codebase
4. Optionally add `.nexus/knowledge.db` to .gitignore, or commit `.nexus/` for team sharing

### Team Sharing Options

- **Option A:** Commit `.nexus/` to Git (exclude large database file if preferred)
- **Option B:** Share `project.nxp` via artifact storage or messaging
- **Option C:** Each developer runs `nexus scan` locally on their machine

### Troubleshooting

| Issue | Solution |
|-------|----------|
| link.exe not found on Windows | Install Visual Studio Build Tools with C++ workload |
| nexus command not found | Add ~/.cargo/bin to PATH, or use full binary path |
| Git errors during scan | Ensure project is a git repository, or ignore the warning |
| tree-sitter parse warnings | Non-critical; unsupported languages are skipped |
| Project not initialized error | Run `nexus init` first |
| Already initialized error | Skip init and run scan directly |

### Updating

Pull latest source, then rebuild or run `cargo install --path crates/nexus-cli --force`.

---

## Part 8 — Roadmap

### MVP v0.1.0 — Current (Complete)

**Goal:** Prove the core loop — init, scan, decide, continue.

Delivered: 14-crate Cargo workspace, .nexus initialization, SQLite and JSON dual persistence, tree-sitter scanning for Rust/JS/TS/Python, dependency detection for Cargo/npm/pip, git history integration, nexus continue, nexus handoff, decision recording, snapshot and restore, timeline, FTS5 search, health scoring, architecture map, NXP protocol v1.0, context compression engine, knowledge graph, and AI session recording.

### Phase 2 — Intelligence v0.2.0

**Goal:** Deeper code understanding and team workflows.

Planned: Session importers for Cursor, ChatGPT, and Claude export formats. Symbol-level knowledge graph with function call edges. Auto-task detection from git commits. `nexus watch` filesystem watcher for incremental scans. Decision supersession. Team merge protocol for .nexus conflicts. Shell completions. Pre-commit hook template.

### Phase 3 — Ecosystem v0.3.0

**Goal:** Plugin system and editor integration.

Planned: Plugin API with NexusPlugin trait. Custom scanner plugins. VS Code and Cursor extension. Optional local embeddings. Vector search alongside FTS5. `nexus diff` for memory comparison between snapshots. CI integration with `nexus continue --check`.

### Phase 4 — Distribution v1.0.0

**Goal:** Production-grade developer infrastructure.

Planned: Signed binaries for Windows, macOS, and Linux. Homebrew tap. winget and scoop packages. crates.io publish. NXP v2 with checksums and signing. Schema migration tooling. Performance target: scan 100k files in under 30 seconds. Comprehensive integration test suite.

### Phase 5 — Platform v2.0.0

**Goal:** Category-defining memory infrastructure.

Planned: Optional self-hosted Nexus Server (still no cloud dependency). Cross-project knowledge linking. Organization-wide decision registry. AI agent SDK in Rust and TypeScript. NXP conformance test suite. Industry partnerships with Cursor, Windsurf, and others.

### Success Metrics

| Metric | Target |
|--------|--------|
| nexus continue latency | Under 2 seconds for a 10,000-file project |
| Context compression efficiency | 70% token reduction with --compress |
| NXP round-trip fidelity | 100% field preservation |
| Offline operation | Zero network calls |
| Cross-platform CI | Windows, Linux, and macOS |

### Testing Strategy

| Layer | Approach |
|-------|----------|
| Unit tests | Model serialization, compression logic, path handling |
| Integration tests | Full init → scan → continue pipeline |
| Snapshot tests | Round-trip create and restore |
| NXP tests | Export and import fidelity |
| CLI tests | Command output validation |
| Platform CI | Matrix across Windows, Ubuntu, and macOS |

---

## Part 9 — Quick Reference

| Command | Purpose |
|---------|---------|
| nexus init | Create .nexus/ memory layer |
| nexus scan | Analyze code, dependencies, architecture |
| nexus decide | Record an architectural decision |
| nexus decisions | List all decisions |
| nexus continue | Generate full AI continuation context |
| nexus handoff | Universal AI handoff document |
| nexus ask | Search project memory |
| nexus map | Display architecture tree |
| nexus health | Project health score |
| nexus snapshot | Save memory checkpoint |
| nexus restore | Restore a checkpoint |
| nexus timeline | Project event history |
| nexus export | Write project.nxp |
| nexus import | Load project.nxp |
| nexus session | Record an AI session |

---

## Part 10 — Testing on Another Project

To use Athreix Nexus on any existing project:

1. Navigate to the target project directory
2. Run `nexus init` with an optional project name
3. Run `nexus scan` to analyze the codebase
4. Run `nexus decide` to record key architectural choices
5. Run `nexus map` and `nexus health` to review project state
6. Run `nexus continue` to generate the AI handoff context
7. Optionally run `nexus snapshot` to save a baseline checkpoint
8. Optionally run `nexus handoff` and `nexus export` for sharing

Nexus only adds a `.nexus/` folder. Your existing source code, configuration, and Git history are never modified.

---

*Athreix Nexus — Git stores source code. Nexus stores project knowledge.*

*Never explain your project to an AI twice.*

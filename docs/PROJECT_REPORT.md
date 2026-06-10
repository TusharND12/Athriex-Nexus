# Athreix Nexus — Project Report

**Tagline:** Never explain your project to an AI twice.

**Version:** 0.1.0  
**Author:** Tushar Dhokane  
**License:** MIT

---

## 1. What Is Athreix Nexus?

Athreix Nexus is a **local-first, offline-first CLI** that stores **project knowledge** — the layer Git does not cover.

| Tool | Stores |
|------|--------|
| **Git** | Source code, commits, branches |
| **Athreix Nexus** | Architecture, decisions, progress, AI sessions, project context |

### Why This Project Matters

Developers lose context constantly when:

- Chat sessions expire or context windows fill up
- Switching between Claude, ChatGPT, Cursor, Windsurf, and terminals
- Team members join mid-project
- Long-term projects evolve over months

**Athreix Nexus solves this** by continuously understanding your project and generating a complete **AI continuation context** that any AI can immediately use.

### What It Is NOT

- Not a SaaS dashboard
- Not a project management tool
- Not a note-taking app
- Not cloud-dependent

### What It IS

- **Git for AI Context** — versioned project memory
- **Memory OS** — persistent knowledge layer per repo
- **Universal AI handoff** — one prompt, any AI tool

---

## 2. Technology Stack

| Layer | Technology |
|-------|------------|
| Language | Rust |
| Storage | SQLite + JSON files |
| Code analysis | tree-sitter |
| Git integration | libgit2 (git2 crate) |
| Serialization | JSON, TOML |
| Search | SQLite FTS5 |
| Packaging | Cargo |
| Platforms | Windows, Linux, macOS, WSL |

---

## 3. System Architecture

### 3.1 High-Level Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    DEVELOPER (Terminal)                  │
└──────────────────────────┬──────────────────────────────┘
                           │
                  ┌────────▼────────┐
                  │    nexus-cli    │  ← You run commands here
                  └────────┬────────┘
                           │
     ┌─────────────────────┼─────────────────────┐
     │                     │                     │
┌────▼────┐  ┌──────▼──────┐  ┌──────▼──────┐  ┌▼────────┐
│ Context │  │   Scanner   │  │  Decision   │  │ Handoff │
│ Engine  │  │   Engine    │  │   Engine    │  │ Engine  │
└────┬────┘  └──────┬──────┘  └──────┬──────┘  └────┬────┘
     │              │                 │               │
     └──────────────┼─────────────────┼───────────────┘
                    │                 │
             ┌──────▼──────┐   ┌──────▼──────┐
             │   Memory    │   │  Knowledge  │
             │   Engine    │   │   Engine    │
             └──────┬──────┘   └──────┬──────┘
                    │                 │
             ┌──────▼─────────────────▼──────┐
             │         .nexus/ folder         │
             │  JSON stores + SQLite + NXP    │
             └───────────────────────────────┘
```

### 3.2 Crate Architecture (14 Engines)

| Crate | Engine | Responsibility |
|-------|--------|----------------|
| `nexus-cli` | CLI Layer | Parse commands, display output |
| `nexus-core` | Core | Shared types, models, paths, errors |
| `nexus-memory` | Memory Engine | Init, JSON files, SQLite, FTS index |
| `nexus-context` | Context Engine | Rebuild full project state (`continue`) |
| `nexus-scanner` | Scanner Engine | tree-sitter code analysis, deps, structure |
| `nexus-git` | Git Engine | Commits, branch, dirty files |
| `nexus-decision` | Decision Engine | Record/list architectural decisions |
| `nexus-snapshot` | Snapshot Engine | Create/restore memory checkpoints |
| `nexus-knowledge` | Knowledge Engine | Build file/task/decision graph |
| `nexus-search` | Search Engine | FTS5 search (`ask`) |
| `nexus-compression` | Compression Engine | Shrink context for token limits |
| `nexus-handoff` | Handoff Engine | Universal AI XML handoff |
| `nexus-nxp` | NXP Engine | Export/import portable protocol file |

### 3.3 Local Project Structure

Every Nexus-enabled project gets:

```
your-project/
├── src/                    ← Your code (unchanged)
├── .nexus/                 ← Nexus memory layer
│   ├── memory.json         ← Project identity, focus, risks
│   ├── architecture.json   ← Layers, technologies, important files
│   ├── timeline.json       ← Chronological event log
│   ├── decisions.json      ← Architectural decision records
│   ├── tasks.json          ← Work items
│   ├── knowledge.db        ← SQLite: search, graph, sessions
│   ├── project.nxp         ← Portable Nexus Protocol export
│   ├── sessions/           ← AI session JSON files
│   └── snapshots/          ← Memory checkpoints
└── ...
```

### 3.4 Data Flow — `nexus continue` (Flagship)

```
1. Load memory.json, decisions.json, architecture.json, tasks.json
2. Query SQLite knowledge.db for sessions and graph
3. Read git history (branch, commits, uncommitted files)
4. Merge everything into ContinuationContext
5. Optionally compress for smaller AI context windows
6. Output: PROJECT OVERVIEW + DECISIONS + ARCHITECTURE + AI PROMPT
```

---

## 4. Complete Command Reference

### How to run (Windows CMD)

```cmd
set NEXUS=T:\Athreix projects\Athreix Nexus\Athriex-Nexus\target\release\nexus.exe
%NEXUS% <command>
```

Or after `cargo install --path crates/nexus-cli`:

```cmd
nexus <command>
```

---

### 4.1 Setup Commands

#### `nexus init`

```cmd
%NEXUS% init --name "my-project"
```

| | |
|---|---|
| **What it does** | Creates `.nexus/` folder with all memory files and SQLite database |
| **When to use** | Once per project, at the very start |
| **Creates** | `memory.json`, `architecture.json`, `timeline.json`, `decisions.json`, `tasks.json`, `knowledge.db`, `project.nxp` |
| **Importance** | Foundation — nothing else works without this |

---

#### `nexus scan`

```cmd
%NEXUS% scan
```

| | |
|---|---|
| **What it does** | Analyzes source code, dependencies, folder structure, git history |
| **When to use** | After init, and again whenever the project changes significantly |
| **Detects** | Languages (Rust, JS, TS, Python), frameworks, entry points, important files |
| **Importance** | Gives Nexus real understanding of your codebase — required for accurate `continue` |

---

### 4.2 Memory Commands

#### `nexus decide`

```cmd
%NEXUS% decide "Use PostgreSQL for relational consistency"
%NEXUS% decide "Use Redis for caching" --rationale "Sub-millisecond reads required"
```

| | |
|---|---|
| **What it does** | Records an architectural decision with timestamp |
| **When to use** | Whenever you make a technical choice worth remembering |
| **Importance** | AI tools won't re-ask resolved questions — decisions are binding constraints |

---

#### `nexus decisions`

```cmd
%NEXUS% decisions
```

| | |
|---|---|
| **What it does** | Lists all recorded architectural decisions |
| **When to use** | Review what has been decided before continuing work |

---

#### `nexus session`

```cmd
%NEXUS% session --tool cursor --prompt "Add auth" --response "Added JWT middleware" --file src/auth.rs
```

| | |
|---|---|
| **What it does** | Saves an AI conversation to `.nexus/sessions/` and indexes it |
| **When to use** | After important AI coding sessions |
| **Importance** | Preserves AI work history searchable via `nexus ask` |

---

### 4.3 Intelligence Commands

#### `nexus continue` ⭐ FLAGSHIP

```cmd
%NEXUS% continue
%NEXUS% continue --compress --max-tokens 8000
%NEXUS% continue --output context.txt
```

| | |
|---|---|
| **What it does** | Reconstructs complete project understanding and outputs AI-ready context |
| **Output sections** | Project Overview, Completed Work, Current Task, Decisions, Important Files, Architecture, Risks, Next Action, AI Continuation Prompt |
| **When to use** | Starting any new AI session — paste output into ChatGPT, Claude, Cursor, etc. |
| **Importance** | **The core value of the entire product** |

---

#### `nexus handoff`

```cmd
%NEXUS% handoff
%NEXUS% handoff --output handoff.xml
```

| | |
|---|---|
| **What it does** | Generates universal XML handoff document for any AI tool |
| **When to use** | Handing project to a different AI or team member |
| **Importance** | Standardized portable format — works across all AI platforms |

---

#### `nexus ask`

```cmd
%NEXUS% ask "Why are we using SQLite?"
%NEXUS% ask "Which file handles authentication?"
```

| | |
|---|---|
| **What it does** | Searches all project memory (decisions, sessions, tasks) |
| **When to use** | Quick lookup without reading all files |
| **Importance** | Semantic memory retrieval across the whole project |

---

#### `nexus map`

```cmd
%NEXUS% map
```

| | |
|---|---|
| **What it does** | Displays architecture tree (Frontend / Backend / API / Shared layers) |
| **When to use** | After `scan` — understand project structure at a glance |

---

#### `nexus health`

```cmd
%NEXUS% health
```

| | |
|---|---|
| **What it does** | Scores project on technical debt, dependencies, documentation, complexity, maintainability |
| **When to use** | Periodic check on project quality |
| **Output** | Overall score /100 + recommendations |

---

### 4.4 Checkpoint Commands

#### `nexus snapshot`

```cmd
%NEXUS% snapshot "before-refactor"
%NEXUS% snapshot "v1-release" --description "MVP complete"
```

| | |
|---|---|
| **What it does** | Saves a checkpoint of all memory state |
| **When to use** | Before major changes, releases, or experiments |
| **Importance** | Like `git tag` but for project knowledge |

---

#### `nexus restore`

```cmd
%NEXUS% restore "before-refactor"
```

| | |
|---|---|
| **What it does** | Restores memory to a previous snapshot |
| **When to use** | When you want to roll back project knowledge |

---

#### `nexus timeline`

```cmd
%NEXUS% timeline
%NEXUS% timeline --limit 20
```

| | |
|---|---|
| **What it does** | Shows chronological log of all Nexus events |
| **Tracks** | Init, scans, decisions, snapshots, sessions, handoffs |

---

### 4.5 Protocol Commands

#### `nexus export`

```cmd
%NEXUS% export
```

| | |
|---|---|
| **What it does** | Writes/updates `project.nxp` — portable full project knowledge file |
| **When to use** | Sharing project context with teammates or other machines |

---

#### `nexus import`

```cmd
%NEXUS% import
%NEXUS% import --file path/to/project.nxp
```

| | |
|---|---|
| **What it does** | Imports project knowledge from an NXP file |
| **When to use** | Setting up Nexus on a new machine from a teammate's export |

---

## 5. Full Workflow — Start to End

### On ANY project (copy-paste for CMD)

```cmd
:: Step 0 — Go to your project
cd "C:\path\to\your-project"

:: Step 1 — Set Nexus path (adjust if needed)
set NEXUS=T:\Athreix projects\Athreix Nexus\Athriex-Nexus\target\release\nexus.exe

:: Step 2 — Initialize memory layer
%NEXUS% init --name "your-project-name"

:: Step 3 — Analyze codebase
%NEXUS% scan

:: Step 4 — Record key decisions
%NEXUS% decide "Your main architectural choice here"
%NEXUS% decide "Your second decision" --rationale "Why you chose this"

:: Step 5 — Review project state
%NEXUS% map
%NEXUS% health
%NEXUS% decisions

:: Step 6 — Generate AI context (THE MAIN OUTPUT)
%NEXUS% continue

:: Step 7 — Optional: save checkpoint and handoff
%NEXUS% snapshot "baseline"
%NEXUS% handoff --output handoff.xml
%NEXUS% export
```

### Recommended order

```
init → scan → decide → map → health → continue → snapshot → handoff
```

---

## 6. Unique Features

### Context Compression Engine
Compresses thousands of files, commits, and decisions into a token-efficient prompt:
```cmd
%NEXUS% continue --compress --max-tokens 8000
```

### AI Session Recorder
Every AI conversation can be saved and searched later:
```cmd
%NEXUS% session --tool cursor --prompt "..." --response "..."
%NEXUS% ask "What did we decide about auth?"
```

### Project Knowledge Graph
Tracks relationships between files, tasks, decisions, packages, and services internally in `knowledge.db`.

### NXP Protocol (`project.nxp`)
Human-readable TOML + JSON portable format — any future AI system can consume it directly.

---

## 7. Build & Install

### Build from source (Windows)

```cmd
cd "T:\Athreix projects\Athreix Nexus\Athriex-Nexus"
"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cargo build --release -p nexus-cli
```

### Install globally

```cmd
cargo install --path crates/nexus-cli
```

### Run tests

```cmd
cargo test --workspace
```

---

## 8. Project Health Metrics

When you run `nexus health`, these are scored:

| Metric | What it measures |
|--------|-----------------|
| Technical Debt | Open tasks count |
| Dependency Risk | Number of dependencies |
| Documentation | README and docs files indexed |
| Complexity | Number of architectural layers |
| Maintainability | Decisions recorded + memory freshness |

---

## 9. Roadmap Summary

| Version | Focus |
|---------|-------|
| **v0.1 (current)** | Core loop: init → scan → decide → continue |
| **v0.2** | Session importers (Cursor, ChatGPT exports), `nexus watch` |
| **v0.3** | Plugin API, VS Code extension, local embeddings |
| **v1.0** | crates.io publish, signed binaries, NXP v2 |
| **v2.0** | SDK, cross-project linking, optional self-hosted sync |

---

## 10. Quick Reference Card

| Command | One-line purpose |
|---------|-----------------|
| `nexus init` | Create `.nexus/` memory |
| `nexus scan` | Analyze code + architecture |
| `nexus decide` | Record a decision |
| `nexus decisions` | List all decisions |
| `nexus continue` | **Generate full AI context** |
| `nexus handoff` | Universal AI handoff XML |
| `nexus ask` | Search project memory |
| `nexus map` | Architecture tree |
| `nexus health` | Project health score |
| `nexus snapshot` | Save memory checkpoint |
| `nexus restore` | Restore checkpoint |
| `nexus timeline` | Project event history |
| `nexus export` | Write `project.nxp` |
| `nexus import` | Load `project.nxp` |
| `nexus session` | Record AI session |

---

## 11. Related Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) — Full system design
- [NXP_PROTOCOL.md](NXP_PROTOCOL.md) — Portable protocol spec
- [SQLITE_SCHEMA.md](SQLITE_SCHEMA.md) — Database schema
- [INSTALLATION.md](INSTALLATION.md) — Cross-platform install
- [ROADMAP.md](ROADMAP.md) — Phase-by-phase plan

---

*Athreix Nexus — Git stores source code. Nexus stores project knowledge.*

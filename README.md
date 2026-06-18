# Athreix Nexus

**Never explain your project to an AI twice.**

Athreix Nexus is a local-first, offline-first, cross-platform CLI that acts as a persistent memory layer for software projects.

| Git stores | Nexus stores |
|------------|--------------|
| Source code | Project knowledge |

## Quick Start

```bash
# Install Rust: https://rustup.rs
cargo install --path crates/nexus-cli

# In your project
nexus init
nexus scan
nexus decide "Use SQLite for offline-first local storage"
nexus continue
```

## Core Commands

| Command | Description |
|---------|-------------|
| `nexus init` | Create `.nexus/` memory layer |
| `nexus scan` | Analyze code, deps, git, architecture |
| `nexus continue` | **Flagship** — full AI continuation context (use `--task "…"` to focus it) |
| `nexus handoff` | Universal AI handoff prompt |
| `nexus snapshot` | Memory checkpoint |
| `nexus restore` | Restore checkpoint |
| `nexus timeline` | Project evolution chronology |
| `nexus decide` | Record architectural decision (`--supersedes <id>` to replace one) |
| `nexus decisions` | List all decisions |
| `nexus ask` | Search project memory |
| `nexus health` | Project health score |
| `nexus map` | Architecture tree |
| `nexus impact` | Trace what a file connects to in the knowledge graph |
| `nexus export` | Write `project.nxp` |
| `nexus session` | Record AI session |

## Project Layout

Every Nexus-enabled project contains:

```
.nexus/
├── memory.json          # Project identity & focus
├── architecture.json    # Layers, tech, important files
├── timeline.json        # Chronological events
├── decisions.json       # Architectural decisions
├── tasks.json           # Work tracking
├── sessions/            # AI session records
├── snapshots/           # Memory checkpoints
├── knowledge.db         # SQLite + FTS5 search + graph
└── project.nxp          # Portable Nexus Protocol export
```

## Architecture

14-engine clean architecture in Rust:

```
nexus-cli          → CLI Layer
nexus-memory       → Memory Engine
nexus-context      → Context Engine
nexus-knowledge    → Knowledge Engine
nexus-git          → Git Engine
nexus-scanner      → Scanner Engine (tree-sitter)
nexus-decision     → Decision Engine
nexus-snapshot     → Snapshot Engine
nexus-handoff      → Handoff Engine
nexus-compression  → Compression Engine
nexus-search       → Search Engine (FTS5)
nexus-nxp          → NXP Protocol Engine
nexus-core         → Shared types & models
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full system design.

## Documentation

- [Complete Guide](docs/NEXUS_COMPLETE_GUIDE.md) — All-in-one reference (no code, info only)
- [Project Report](docs/PROJECT_REPORT.md) — Full command guide, architecture, workflow
- [Architecture](docs/ARCHITECTURE.md)
- [NXP Protocol](docs/NXP_PROTOCOL.md)
- [SQLite Schema](docs/SQLITE_SCHEMA.md)
- [Installation](docs/INSTALLATION.md)
- [Roadmap](docs/ROADMAP.md)

## Philosophy

This is **not** a SaaS dashboard, PM tool, or note app.

This **is**:
- Git for AI Context
- Memory OS for software projects
- Universal AI handoff infrastructure

Everything works from the terminal. Offline. Local. Cross-platform.

## License

MIT — see [LICENSE](LICENSE)

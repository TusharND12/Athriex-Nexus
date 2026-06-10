# Athreix Nexus — System Architecture

## Vision

Athreix Nexus is the **universal memory layer** between software projects and AI systems. It solves context loss across chat sessions, tools, devices, and team members.

```
┌─────────────────────────────────────────────────────────────────┐
│                        DEVELOPER                                 │
│              (terminal, any OS, any shell)                         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │   nexus-cli     │  CLI Layer
                    │  (clap router)  │
                    └────────┬────────┘
                             │
     ┌───────────────────────┼───────────────────────┐
     │                       │                       │
┌────▼─────┐  ┌──────▼──────┐  ┌──────▼──────┐  ┌───▼────┐
│ Context  │  │  Handoff    │  │ Compression │  │ Search │
│ Engine   │  │  Engine     │  │  Engine     │  │ Engine │
└────┬─────┘  └──────┬──────┘  └──────┬──────┘  └───┬────┘
     │               │                │              │
     └───────────────┼────────────────┼──────────────┘
                     │                │
              ┌──────▼──────┐   ┌─────▼─────┐
              │   Memory    │   │ Knowledge │
              │   Engine    │   │  Engine   │
              └──────┬──────┘   └─────┬─────┘
                     │                │
     ┌───────────────┼────────────────┼───────────────┐
     │               │                │               │
┌────▼────┐   ┌──────▼──────┐  ┌─────▼─────┐  ┌─────▼─────┐
│ Scanner │   │    Git      │  │ Decision  │  │ Snapshot  │
│ Engine  │   │   Engine    │  │  Engine   │  │  Engine   │
└────┬────┘   └──────┬──────┘  └─────┬─────┘  └─────┬─────┘
     │               │                │               │
     └───────────────┼────────────────┼───────────────┘
                     │                │
              ┌──────▼────────────────▼──────┐
              │         nexus-core           │
              │   (types, models, paths)     │
              └──────────────┬───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │         .nexus/                │
              │  JSON stores + SQLite + NXP    │
              └──────────────────────────────┘
```

## Crate Breakdown

| Crate | Layer | Responsibility |
|-------|-------|----------------|
| `nexus-cli` | CLI | Argument parsing, command dispatch, user output |
| `nexus-core` | Domain | Shared types, errors, path constants, data models |
| `nexus-memory` | Infrastructure | JSON persistence, SQLite, FTS indexing, init |
| `nexus-context` | Application | Reconstruct `ContinuationContext`, health analysis |
| `nexus-handoff` | Application | Format universal AI handoff documents |
| `nexus-compression` | Application | Token-efficient context compression |
| `nexus-search` | Application | FTS5 + graph-assisted semantic search |
| `nexus-scanner` | Application | tree-sitter analysis, dep detection, structure |
| `nexus-git` | Application | Branch, commits, dirty files via libgit2 |
| `nexus-decision` | Application | ADR recording and listing |
| `nexus-snapshot` | Application | Checkpoint create/restore |
| `nexus-knowledge` | Application | Knowledge graph build and persist |
| `nexus-nxp` | Application | NXP protocol export/import |

## Data Flow: `nexus continue`

```
1. MemoryEngine loads JSON stores + SQLite
2. GitEngine summarizes repository state
3. DecisionEngine loads active ADRs
4. ContextEngine merges all sources
5. CompressionEngine optionally reduces token count
6. CLI renders formatted continuation output
```

## Storage Strategy

**Dual persistence** — optimized for human readability and machine query:

| Store | Format | Purpose |
|-------|--------|---------|
| `memory.json` | JSON | Project identity, focus, risks |
| `architecture.json` | JSON | Layers, technologies, files |
| `decisions.json` | JSON | Architectural decision records |
| `tasks.json` | JSON | Work items |
| `timeline.json` | JSON | Event chronology |
| `knowledge.db` | SQLite | Indexed search, graph, sessions |
| `sessions/*.json` | JSON | Individual AI session archives |
| `snapshots/*/ ` | JSON | Point-in-time memory copies |
| `project.nxp` | TOML | Portable full export |

## Clean Architecture Boundaries

```
┌─────────────────────────────────────────┐
│  CLI (Interface Adapters)               │
├─────────────────────────────────────────┤
│  Engines (Use Cases)                    │
│  - orchestrate domain logic             │
│  - no direct stdout in engines          │
├─────────────────────────────────────────┤
│  Core (Entities)                        │
│  - ProjectMemory, Decision, Task, etc.    │
├─────────────────────────────────────────┤
│  Memory (Infrastructure)                │
│  - filesystem, SQLite, serialization    │
└─────────────────────────────────────────┘
```

**Dependency rule**: Inner layers never depend on outer layers. Engines depend on `nexus-core` and `nexus-memory`. Only `nexus-cli` depends on all engines.

## Command Architecture

Each CLI command maps 1:1 to an engine orchestration:

```rust
// Pattern: commands.rs
fn scan(paths: &NexusPaths) -> Result<()> {
    let engine = MemoryEngine::open(paths)?;
    let result = ScannerEngine::new(&engine).scan()?;
    KnowledgeEngine::new(&engine).build_from_scan(&result)?;
    // render results
}
```

## Unique Systems

### Context Compression Engine
Reduces thousands of files/commits/decisions into token-budgeted prompts while preserving:
- Current task signal
- Binding architectural decisions
- Architecture summary
- Risk awareness

### AI Session Recorder
Sessions stored as JSON in `.nexus/sessions/` and indexed in SQLite FTS for `nexus ask`.

### Project Knowledge Graph
Nodes: File, Function, Service, API, Database, Task, Decision, Module, Package  
Edges: Imports, Calls, DependsOn, Implements, RelatedTo, Owns, Documents

### NXP Protocol
Human-readable TOML envelope with JSON payloads — portable, versioned, extensible.

## Future Plugin Architecture

```
.nexus/plugins/
├── custom-scanner.toml      # Register additional file analyzers
├── session-importers/       # Cursor, ChatGPT export parsers
└── embedders/               # Optional local embedding models
```

Plugin interface (Phase 3):
```rust
pub trait NexusPlugin {
    fn name(&self) -> &str;
    fn on_scan(&self, ctx: &ScanContext) -> NexusResult<PluginScanOutput>;
    fn on_continue(&self, ctx: &mut ContinuationContext) -> NexusResult<()>;
}
```

## Cross-Platform Support

| Environment | Support |
|-------------|---------|
| Windows CMD / PowerShell / Terminal | ✓ |
| WSL | ✓ |
| Linux (bash, zsh, fish) | ✓ |
| macOS Terminal | ✓ |

Path handling uses `std::path` throughout. SQLite bundled via `rusqlite` feature.

## Security Model

- **100% local** — no network calls
- **No telemetry** — project knowledge never leaves disk
- `.nexus/` should be committed or gitignored per team preference
- Secrets in code are not extracted or transmitted

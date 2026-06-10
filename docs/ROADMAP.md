# Athreix Nexus — Roadmap

## MVP (v0.1.0) — Current

**Goal**: Prove the core loop — init → scan → decide → continue

- [x] Cargo workspace with 14 crates
- [x] `.nexus/` initialization
- [x] SQLite + JSON dual persistence
- [x] tree-sitter scanning (Rust, JS, TS, Python)
- [x] Dependency detection (Cargo, npm, pip)
- [x] Git history integration
- [x] `nexus continue` — full continuation context
- [x] `nexus handoff` — universal AI prompt
- [x] Decision recording (ADR)
- [x] Snapshot / restore
- [x] Timeline
- [x] FTS5 search (`nexus ask`)
- [x] Health scoring
- [x] Architecture map
- [x] NXP protocol v1.0
- [x] Context compression engine
- [x] Knowledge graph
- [x] AI session recording

## Phase 2 — Intelligence (v0.2.0)

**Goal**: Deeper code understanding and team workflows

- [ ] Session importers (Cursor, ChatGPT, Claude export formats)
- [ ] Symbol-level knowledge graph (function call edges via tree-sitter)
- [ ] Auto-task detection from git commits
- [ ] `nexus watch` — filesystem watcher for incremental scans
- [ ] Decision supersession (`nexus decide --supersedes <id>`)
- [ ] Team merge protocol for `.nexus/` conflicts
- [ ] Shell completions (bash, zsh, fish, powershell)
- [ ] Pre-commit hook template

## Phase 3 — Ecosystem (v0.3.0)

**Goal**: Plugin system and editor integration

- [ ] Plugin API (`NexusPlugin` trait)
- [ ] Custom scanner plugins
- [ ] VS Code / Cursor extension (thin wrapper over CLI)
- [ ] Optional local embeddings (fastembed/llama.cpp)
- [ ] Vector search alongside FTS5
- [ ] `nexus diff` — memory diff between snapshots
- [ ] CI integration (`nexus continue --check`)

## Phase 4 — Distribution (v1.0.0)

**Goal**: Production-grade developer infrastructure

- [ ] Signed binaries (Windows, macOS, Linux)
- [ ] Homebrew tap
- [ ] winget / scoop packages
- [ ] `cargo install nexus-cli` on crates.io
- [ ] NXP v2 with checksums and signing
- [ ] Migration tooling for schema upgrades
- [ ] Performance: scan 100k files < 30s
- [ ] Comprehensive integration test suite

## Phase 5 — Platform (v2.0.0)

**Goal**: Category-defining memory infrastructure

- [ ] Nexus Server (optional self-hosted sync — still no cloud dependency)
- [ ] Cross-project knowledge linking
- [ ] Organization-wide decision registry
- [ ] AI agent SDK (`nexus-sdk` Rust + TypeScript)
- [ ] Standard test: "NXP conformance suite"
- [ ] Industry partnerships (Cursor, Windsurf, etc.)

## MVP Implementation Plan

### Week 1: Foundation
1. Workspace scaffolding ✓
2. Core models and paths ✓
3. Memory engine + SQLite ✓
4. `nexus init` ✓

### Week 2: Analysis
5. Scanner engine ✓
6. Git engine ✓
7. Knowledge graph ✓
8. `nexus scan`, `nexus map` ✓

### Week 3: Context
9. Context engine ✓
10. Compression engine ✓
11. Handoff engine ✓
12. `nexus continue`, `nexus handoff` ✓

### Week 4: Memory Operations
13. Decision engine ✓
14. Snapshot engine ✓
15. Search engine ✓
16. NXP engine ✓
17. All remaining commands ✓

### Week 5: Polish
18. Documentation ✓
19. Tests
20. Cross-platform CI
21. Release binaries

## Testing Strategy

| Layer | Approach |
|-------|----------|
| Unit | Model serialization, compression, path logic |
| Integration | Init → scan → continue pipeline in tempdir |
| Snapshot | Round-trip create/restore |
| NXP | Export/import fidelity |
| CLI | Command output golden files |
| Platform | CI matrix: windows-latest, ubuntu, macos |

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Success Metrics

| Metric | Target |
|--------|--------|
| `nexus continue` latency | < 2s for 10k file project |
| Context token efficiency | 70% reduction with `--compress` |
| NXP round-trip fidelity | 100% field preservation |
| Offline operation | Zero network calls |
| Cross-platform | 3 OS targets in CI |

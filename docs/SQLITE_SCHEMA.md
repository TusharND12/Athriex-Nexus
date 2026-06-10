# Athreix Nexus — SQLite Schema

Database: `.nexus/knowledge.db`  
Engine: SQLite 3 with FTS5  
Migration version: `1`

## schema_meta

Tracks schema version.

```sql
CREATE TABLE schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

## decisions

Indexed architectural decisions (mirrors `decisions.json`).

```sql
CREATE TABLE decisions (
    id         TEXT PRIMARY KEY,    -- UUID
    content    TEXT NOT NULL,
    rationale  TEXT,
    tags       TEXT NOT NULL DEFAULT '[]',  -- JSON array
    created_at TEXT NOT NULL,       -- RFC3339
    author     TEXT,
    status     TEXT NOT NULL DEFAULT 'active'
);
CREATE INDEX idx_decisions_created ON decisions(created_at);
```

## tasks

Work items with status tracking.

```sql
CREATE TABLE tasks (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    description   TEXT,
    status        TEXT NOT NULL DEFAULT 'pending',
    priority      TEXT NOT NULL DEFAULT 'medium',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    related_files TEXT NOT NULL DEFAULT '[]',
    blocked_by    TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX idx_tasks_status ON tasks(status);
```

## timeline_events

Queryable event log (mirrors `timeline.json`).

```sql
CREATE TABLE timeline_events (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL,
    title       TEXT NOT NULL,
    description TEXT,
    timestamp   TEXT NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_timeline_ts ON timeline_events(timestamp);
```

## knowledge_nodes

Knowledge graph vertices.

```sql
CREATE TABLE knowledge_nodes (
    id       TEXT PRIMARY KEY,
    kind     TEXT NOT NULL,
    name     TEXT NOT NULL,
    path     TEXT,
    metadata TEXT NOT NULL DEFAULT '{}'
);
CREATE INDEX idx_knowledge_kind ON knowledge_nodes(kind);
```

## knowledge_edges

Knowledge graph edges.

```sql
CREATE TABLE knowledge_edges (
    id       TEXT PRIMARY KEY,
    from_id  TEXT NOT NULL,
    to_id    TEXT NOT NULL,
    relation TEXT NOT NULL,
    weight   REAL NOT NULL DEFAULT 1.0,
    FOREIGN KEY (from_id) REFERENCES knowledge_nodes(id),
    FOREIGN KEY (to_id) REFERENCES knowledge_nodes(id)
);
```

## sessions

AI session index (full records in `sessions/*.json`).

```sql
CREATE TABLE sessions (
    id             TEXT PRIMARY KEY,
    tool           TEXT,
    prompt         TEXT NOT NULL,
    response       TEXT NOT NULL,
    files_modified TEXT NOT NULL DEFAULT '[]',
    timestamp      TEXT NOT NULL,
    notes          TEXT,
    tags           TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX idx_sessions_ts ON sessions(timestamp);
```

## snapshots

Checkpoint registry (archives in `snapshots/<uuid>/`).

```sql
CREATE TABLE snapshots (
    id                TEXT PRIMARY KEY,
    label             TEXT NOT NULL,
    created_at        TEXT NOT NULL,
    description       TEXT,
    memory_hash       TEXT NOT NULL,
    architecture_hash TEXT NOT NULL,
    decisions_count   INTEGER NOT NULL,
    tasks_count       INTEGER NOT NULL,
    archive_path      TEXT NOT NULL
);
```

## scan_history

Historical scan results for trend analysis.

```sql
CREATE TABLE scan_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    scanned_at      TEXT NOT NULL,
    files_analyzed  INTEGER NOT NULL,
    result_json     TEXT NOT NULL
);
```

## memory_fts (FTS5)

Full-text search index powering `nexus ask`.

```sql
CREATE VIRTUAL TABLE memory_fts USING fts5(
    doc_id UNINDEXED,
    source,
    content,
    tokenize = 'porter'
);
```

Indexed sources: `decision`, `task`, `session`

### Example Query

```sql
SELECT doc_id, source, snippet(memory_fts, 2, '>>', '<<', '…', 20), rank
FROM memory_fts
WHERE memory_fts MATCH '"authentication" OR "auth"'
ORDER BY rank
LIMIT 10;
```

## Dual-Write Strategy

JSON files are the **source of truth** for human editing and git diffing.  
SQLite provides **query acceleration** and FTS. On init, both are created. On write, engines update JSON first, then sync to SQLite.

## Future Migrations (v2 planned)

- `embeddings` table for local vector search
- `file_symbols` table for tree-sitter symbol index
- `commit_links` table linking decisions to git SHAs

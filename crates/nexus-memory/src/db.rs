use nexus_core::NexusResult;
use rusqlite::Connection;

pub const SCHEMA_VERSION: i32 = 1;

pub const MIGRATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS decisions (
    id         TEXT PRIMARY KEY,
    content    TEXT NOT NULL,
    rationale  TEXT,
    tags       TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    author     TEXT,
    status     TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS tasks (
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

CREATE TABLE IF NOT EXISTS timeline_events (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL,
    title       TEXT NOT NULL,
    description TEXT,
    timestamp   TEXT NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS knowledge_nodes (
    id       TEXT PRIMARY KEY,
    kind     TEXT NOT NULL,
    name     TEXT NOT NULL,
    path     TEXT,
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS knowledge_edges (
    id       TEXT PRIMARY KEY,
    from_id  TEXT NOT NULL,
    to_id    TEXT NOT NULL,
    relation TEXT NOT NULL,
    weight   REAL NOT NULL DEFAULT 1.0,
    FOREIGN KEY (from_id) REFERENCES knowledge_nodes(id),
    FOREIGN KEY (to_id) REFERENCES knowledge_nodes(id)
);

CREATE TABLE IF NOT EXISTS sessions (
    id             TEXT PRIMARY KEY,
    tool           TEXT,
    prompt         TEXT NOT NULL,
    response       TEXT NOT NULL,
    files_modified TEXT NOT NULL DEFAULT '[]',
    timestamp      TEXT NOT NULL,
    notes          TEXT,
    tags           TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS snapshots (
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

CREATE TABLE IF NOT EXISTS scan_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    scanned_at      TEXT NOT NULL,
    files_analyzed  INTEGER NOT NULL,
    result_json     TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    doc_id UNINDEXED,
    source,
    content,
    tokenize = 'porter'
);

CREATE INDEX IF NOT EXISTS idx_decisions_created ON decisions(created_at);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_timeline_ts ON timeline_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_knowledge_kind ON knowledge_nodes(kind);
CREATE INDEX IF NOT EXISTS idx_sessions_ts ON sessions(timestamp);
"#;

pub fn open_and_migrate(path: &std::path::Path) -> NexusResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn =
        Connection::open(path).map_err(|e| nexus_core::NexusError::Database(e.to_string()))?;
    // WAL allows concurrent readers with a single writer; busy_timeout makes
    // brief lock contention between nexus processes block-and-retry instead of
    // failing immediately. synchronous=NORMAL is durable under WAL.
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA busy_timeout = 5000;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(|e| nexus_core::NexusError::Database(e.to_string()))?;
    conn.execute_batch(MIGRATIONS)
        .map_err(|e| nexus_core::NexusError::Database(e.to_string()))?;
    let version: i32 = conn
        .query_row(
            "SELECT value FROM schema_meta WHERE key = 'version'",
            [],
            |row| row.get::<_, String>(0).map(|v| v.parse().unwrap_or(0)),
        )
        .unwrap_or(0);
    if version < SCHEMA_VERSION {
        conn.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES ('version', ?1)",
            [SCHEMA_VERSION.to_string()],
        )
        .map_err(|e| nexus_core::NexusError::Database(e.to_string()))?;
    }
    Ok(conn)
}

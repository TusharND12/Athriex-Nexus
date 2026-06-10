# NXP — Nexus Protocol Specification v1.0.0

## Overview

**NXP (Nexus Protocol)** is a portable, versioned, human-readable file format for complete project knowledge transfer between machines, team members, and AI systems.

File: `.nexus/project.nxp`  
Format: TOML envelope + embedded JSON payloads  
MIME (proposed): `application/vnd.athreix.nxp+toml`

## Design Goals

1. **Portable** — single file reconstructs full project understanding
2. **Versioned** — protocol evolution without breaking consumers
3. **Extensible** — `extensions` field for future capabilities
4. **Human readable** — inspectable in any text editor
5. **AI consumable** — direct input for LLM context loading

## File Structure

```toml
protocol_version = "1.0.0"
nexus_version = "0.1.0"
exported_at = "2026-06-10T12:00:00Z"

# JSON-encoded payloads (string fields)
project_json = '{"project_name":"my-app",...}'
architecture_json = '{"layers":[...],...}'
decisions_json = '{"decisions":[...]}'
tasks_json = '{"tasks":[...]}'
timeline_json = '{"events":[...]}'
knowledge_graph_json = '{"nodes":[...],"edges":[...]}'
recent_sessions_json = '[...]'
git_summary_json = '{"branch":"main",...}'
extensions_json = '{}'
```

## Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `protocol_version` | string (semver) | yes | NXP spec version |
| `nexus_version` | string (semver) | yes | Exporting Nexus binary version |
| `exported_at` | RFC3339 timestamp | yes | Export time |
| `project_json` | JSON string | yes | `ProjectMemory` object |
| `architecture_json` | JSON string | yes | `Architecture` object |
| `decisions_json` | JSON string | yes | `DecisionStore` object |
| `tasks_json` | JSON string | yes | `TaskStore` object |
| `timeline_json` | JSON string | yes | `Timeline` object |
| `knowledge_graph_json` | JSON string | yes | `KnowledgeGraph` object |
| `recent_sessions_json` | JSON string | yes | Array of `AiSession` (max 20) |
| `git_summary_json` | JSON string | no | `GitSummary` or null |
| `extensions_json` | JSON string | yes | Extensibility object |

## Embedded Schema: ProjectMemory

```json
{
  "version": "0.1.0",
  "project_name": "string",
  "description": "string",
  "created_at": "RFC3339",
  "updated_at": "RFC3339",
  "technologies": ["string"],
  "current_focus": "string",
  "completed_work": ["string"],
  "risks": ["string"],
  "metadata": {}
}
```

## Embedded Schema: Decision

```json
{
  "id": "uuid",
  "content": "string",
  "rationale": "string|null",
  "tags": ["string"],
  "created_at": "RFC3339",
  "author": "string|null",
  "status": "active|superseded|deprecated"
}
```

## Embedded Schema: KnowledgeGraph

```json
{
  "nodes": [{
    "id": "uuid",
    "kind": "file|function|service|api|database|task|decision|module|package",
    "name": "string",
    "path": "string|null",
    "metadata": {}
  }],
  "edges": [{
    "id": "uuid",
    "from": "uuid",
    "to": "uuid",
    "relation": "imports|calls|depends_on|implements|related_to|owns|documents",
    "weight": 0.0
  }]
}
```

## Versioning Rules

- **Major**: Breaking schema changes (field removal, type changes)
- **Minor**: Additive fields (backward compatible)
- **Patch**: Documentation/clarification only

Consumers MUST:
1. Check `protocol_version` before parsing
2. Ignore unknown fields in JSON payloads
3. Preserve `extensions_json` round-trip

## Import Semantics

`nexus import` performs:
1. Parse TOML envelope
2. Validate `protocol_version` compatibility (1.x)
3. Deserialize JSON payloads
4. Overwrite JSON stores
5. Re-index SQLite (decisions, tasks, graph)
6. Regenerate `project.nxp`

## AI Consumption Guide

AI systems can load NXP directly:

```
1. Read project.nxp
2. Parse project_json for identity and focus
3. Load decisions_json as binding constraints
4. Use architecture_json + knowledge_graph_json for code navigation
5. Check tasks_json for current work
6. Review recent_sessions_json for prior AI context
```

## Extensions (v1.0 reserved keys)

```json
{
  "embeddings": null,
  "custom_scanners": [],
  "team_id": null,
  "checksum": null
}
```

## Example Minimal NXP

```toml
protocol_version = "1.0.0"
nexus_version = "0.1.0"
exported_at = "2026-06-10T10:00:00Z"
project_json = "{\"version\":\"0.1.0\",\"project_name\":\"demo\",\"description\":\"\",\"created_at\":\"2026-06-10T10:00:00Z\",\"updated_at\":\"2026-06-10T10:00:00Z\",\"technologies\":[],\"current_focus\":\"\",\"completed_work\":[],\"risks\":[],\"metadata\":{}}"
architecture_json = "{\"version\":\"0.1.0\",\"updated_at\":\"2026-06-10T10:00:00Z\",\"layers\":[],\"technologies\":[],\"dependencies\":[],\"entry_points\":[],\"important_files\":[]}"
decisions_json = "{\"version\":\"0.1.0\",\"decisions\":[]}"
tasks_json = "{\"version\":\"0.1.0\",\"tasks\":[]}"
timeline_json = "{\"version\":\"0.1.0\",\"events\":[]}"
knowledge_graph_json = "{\"nodes\":[],\"edges\":[]}"
recent_sessions_json = "[]"
git_summary_json = "null"
extensions_json = "{}"
```

# @athreix/nexus

**Never explain your project to an AI twice.**

Athreix Nexus is a local-first, offline-first, cross-platform CLI that acts as a
persistent memory layer for software projects. Git stores your source code;
Nexus stores your project knowledge — decisions, architecture, timeline, and the
context an AI assistant needs to continue your work.

## Install

```bash
# Run instantly, no install
npx @athreix/nexus --help

# Or install globally
npm install -g @athreix/nexus
nexus --help
```

On install, the correct prebuilt binary for your platform is downloaded from the
matching [GitHub Release](https://github.com/TusharND12/Athriex-Nexus/releases)
and checksum-verified. Supported platforms: Windows x64, macOS x64, macOS arm64,
Linux x64.

## Quick start

```bash
nexus init
nexus scan
nexus decide "Use SQLite for offline-first local storage"
nexus continue --task "implement the search index"
```

## Common commands

| Command | Description |
|---------|-------------|
| `nexus init` | Create the `.nexus/` memory layer |
| `nexus scan` | Analyze code, deps, git, architecture |
| `nexus continue` | Reconstruct full AI continuation context (`--task` to focus) |
| `nexus handoff` | Universal AI handoff prompt |
| `nexus decide` | Record an architectural decision (`--supersedes` to replace one) |
| `nexus impact` | Trace what a file connects to in the knowledge graph |
| `nexus ask` | Search project memory |
| `nexus health` | Project health score |

Run `nexus --help` for the full command list.

## Notes

- **Offline / air-gapped installs:** set `ATHREIX_NEXUS_SKIP_DOWNLOAD=1` to skip
  the binary download, or build from source with
  `cargo install --git https://github.com/TusharND12/Athriex-Nexus nexus-cli`.
- The tool itself makes **zero network calls** at runtime — only the npm
  installer fetches the binary.

## License

MIT — see the [repository](https://github.com/TusharND12/Athriex-Nexus).

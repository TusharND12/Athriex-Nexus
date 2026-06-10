# Athreix Nexus — Installation

## Prerequisites

- **Rust 1.75+** — [https://rustup.rs](https://rustup.rs)
- **Git** (optional, for git integration)
- **C compiler** (for libgit2/sqlite native deps)

### Windows

```powershell
# Install Rust
winget install Rustlang.Rustup

# Restart terminal, then:
rustup default stable

# Build Nexus
cd Athriex-Nexus
cargo build --release

# Binary at: target\release\nexus.exe
```

### Linux / WSL / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

cd Athriex-Nexus
cargo build --release
sudo cp target/release/nexus /usr/local/bin/  # optional
```

## Install Methods

### From Source (recommended for development)

```bash
cargo install --path crates/nexus-cli
```

### Direct Run

```bash
cargo run --release -p nexus-cli -- init
cargo run --release -p nexus-cli -- continue
```

## Verify Installation

```bash
nexus --version
nexus init --name my-project
nexus scan
nexus continue
```

## Shell Compatibility

| Shell | Status |
|-------|--------|
| PowerShell | ✓ |
| CMD | ✓ |
| Bash | ✓ |
| Zsh | ✓ |
| Fish | ✓ |
| WSL | ✓ |

## Project Setup

```bash
cd your-project
nexus init
echo ".nexus/knowledge.db" >> .gitignore  # optional — or commit .nexus/ for team sharing
nexus scan
```

## Team Sharing

**Option A**: Commit `.nexus/` (minus large db if preferred)  
**Option B**: Share `project.nxp` via artifact storage  
**Option C**: Each dev runs `nexus scan` locally

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `link.exe not found` (Windows) | Install Visual Studio Build Tools |
| `nexus: command not found` | Add `~/.cargo/bin` to PATH |
| Git errors on scan | Ensure project is a git repo or ignore |
| tree-sitter parse warnings | Non-critical; unsupported langs skipped |

## Updating

```bash
cd Athriex-Nexus
git pull
cargo install --path crates/nexus-cli --force
```

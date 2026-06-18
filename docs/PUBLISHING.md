# Publishing Athreix Nexus

Athreix Nexus ships in two layers:

1. **GitHub Releases** — prebuilt binaries for Windows / macOS / Linux.
2. **npm (`@athreix/nexus`)** — a thin wrapper that downloads the matching
   binary on install. The published npm package contains only JavaScript.

Both are produced automatically by [`.github/workflows/release.yml`](../.github/workflows/release.yml)
when you push a version tag.

## One-time setup

### 1. npm account & scope
- Create an account at https://www.npmjs.com.
- Create the **`athreix`** organization (Settings → Add Organization). This is
  free for public packages and gives you the `@athreix/*` scope.
  - Prefer no org? Switch `npm/package.json` `"name"` to `"athreix-nexus"`
    (unscoped) and drop the `publishConfig.access` requirement.

### 2. npm automation token → GitHub secret
- npm → Access Tokens → **Generate New Token** → *Automation*.
- In GitHub: repo → Settings → Secrets and variables → Actions → **New repository secret**
  - Name: `NPM_TOKEN`
  - Value: the token

## Cutting a release

1. **Bump the version** in both places (keep them in sync):
   - `Cargo.toml` → `[workspace.package] version`
   - `npm/package.json` → `"version"`
2. Commit and push to `main`.
3. Tag and push the tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
4. The `Release` workflow then:
   - builds the four platform binaries and uploads them (+ `.sha256`) to the
     GitHub Release for the tag, then
   - publishes `@athreix/nexus` to npm at the tag's version.

> Order matters: the npm `publish-npm` job runs **after** the binaries job, so the
> binaries exist before anyone can `npm install` that version (the installer
> downloads them from the GitHub Release).

## Verifying

```bash
npx @athreix/nexus@latest --version
npm install -g @athreix/nexus && nexus --help
```

## Manual publish (fallback, no CI)

```bash
# 1. Create the GitHub Release + upload the 4 binaries and .sha256 files yourself
#    (asset names must match npm/lib/resolve.js).
# 2. Then:
cd npm
npm login
npm version --no-git-tag-version 0.1.0
npm publish --access public
```

## Supported platforms

| npm key        | Rust target                  | Asset                   |
|----------------|------------------------------|-------------------------|
| `win32-x64`    | `x86_64-pc-windows-msvc`     | `nexus-win32-x64.exe`   |
| `darwin-x64`   | `x86_64-apple-darwin`        | `nexus-darwin-x64`      |
| `darwin-arm64` | `aarch64-apple-darwin`       | `nexus-darwin-arm64`    |
| `linux-x64`    | `x86_64-unknown-linux-gnu`   | `nexus-linux-x64`       |

Other platforms fall back to `cargo install --git <repo> nexus-cli`.

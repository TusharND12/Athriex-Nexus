#!/usr/bin/env node
'use strict';

// Downloads the prebuilt Athreix Nexus binary for the current platform from the
// matching GitHub Release, verifies its SHA-256 checksum, and installs it into
// the package's bin/ directory. Uses only Node built-ins (no dependencies).

const fs = require('fs');
const path = require('path');
const https = require('https');
const crypto = require('crypto');

const { target, binDir } = require('../lib/resolve');
const pkg = require('../package.json');

const REPO = 'TusharND12/Athriex-Nexus';
const VERSION = pkg.version;

function download(url, dest, redirects = 0) {
  return new Promise((resolve, reject) => {
    if (redirects > 5) {
      return reject(new Error('too many redirects for ' + url));
    }
    const file = fs.createWriteStream(dest);
    const req = https.get(
      url,
      { headers: { 'User-Agent': 'athreix-nexus-installer' } },
      (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          file.close();
          fs.rmSync(dest, { force: true });
          return download(res.headers.location, dest, redirects + 1).then(resolve, reject);
        }
        if (res.statusCode !== 200) {
          file.close();
          fs.rmSync(dest, { force: true });
          return reject(new Error('HTTP ' + res.statusCode + ' for ' + url));
        }
        res.pipe(file);
        file.on('finish', () => file.close(() => resolve()));
      }
    );
    req.on('error', (err) => {
      fs.rmSync(dest, { force: true });
      reject(err);
    });
  });
}

function sha256(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

async function main() {
  if (process.env.ATHREIX_NEXUS_SKIP_DOWNLOAD) {
    console.log('Athreix Nexus: skipping binary download (ATHREIX_NEXUS_SKIP_DOWNLOAD set).');
    return;
  }

  let t;
  try {
    t = target();
  } catch (e) {
    // Unsupported platform: warn but don't fail the whole install — the launcher
    // will print actionable guidance if the user tries to run `nexus`.
    console.warn(e.message);
    return;
  }

  const dir = binDir();
  fs.mkdirSync(dir, { recursive: true });

  const base = `https://github.com/${REPO}/releases/download/v${VERSION}`;
  const assetUrl = `${base}/${t.asset}`;
  const shaUrl = `${assetUrl}.sha256`;
  const outPath = path.join(dir, t.bin);
  const tmp = outPath + '.download';

  console.log(`Athreix Nexus: downloading ${t.asset} (v${VERSION})...`);
  await download(assetUrl, tmp);

  // Checksum verification: if the .sha256 sidecar is missing we warn and proceed,
  // but a genuine mismatch is always fatal.
  let expected = null;
  const shaTmp = tmp + '.sha256';
  try {
    await download(shaUrl, shaTmp);
    expected = fs.readFileSync(shaTmp, 'utf8').trim().split(/\s+/)[0];
    fs.rmSync(shaTmp, { force: true });
  } catch (e) {
    console.warn(`Athreix Nexus: checksum file unavailable (${e.message}); skipping verification.`);
  }

  if (expected) {
    const actual = sha256(tmp);
    if (expected.toLowerCase() !== actual.toLowerCase()) {
      fs.rmSync(tmp, { force: true });
      throw new Error(
        `checksum mismatch for ${t.asset}: expected ${expected}, got ${actual}`
      );
    }
    console.log('Athreix Nexus: checksum verified.');
  }

  fs.renameSync(tmp, outPath);
  if (process.platform !== 'win32') {
    fs.chmodSync(outPath, 0o755);
  }
  console.log(`Athreix Nexus: installed binary to ${outPath}`);
}

main().catch((err) => {
  console.error('Athreix Nexus: install failed — ' + err.message);
  console.error('You can build from source instead:');
  console.error('  cargo install --git https://github.com/TusharND12/Athriex-Nexus nexus-cli');
  process.exit(1);
});

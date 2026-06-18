'use strict';

// Single source of truth for the platform -> release-asset mapping.
// Shared by the postinstall downloader and the runtime launcher so they can
// never disagree about which file to fetch / execute.

const path = require('path');

// Keys are `${process.platform}-${process.arch}`.
const SUPPORTED = {
  'win32-x64': { asset: 'nexus-win32-x64.exe', bin: 'nexus.exe' },
  'darwin-x64': { asset: 'nexus-darwin-x64', bin: 'nexus' },
  'darwin-arm64': { asset: 'nexus-darwin-arm64', bin: 'nexus' },
  'linux-x64': { asset: 'nexus-linux-x64', bin: 'nexus' },
};

function platformKey() {
  return `${process.platform}-${process.arch}`;
}

function target() {
  const key = platformKey();
  const t = SUPPORTED[key];
  if (!t) {
    const supported = Object.keys(SUPPORTED).join(', ');
    const err = new Error(
      `Athreix Nexus: unsupported platform "${key}".\n` +
        `Supported: ${supported}.\n` +
        `Build from source instead:\n` +
        `  cargo install --git https://github.com/TusharND12/Athriex-Nexus nexus-cli`
    );
    err.unsupported = true;
    throw err;
  }
  return t;
}

function binDir() {
  return path.join(__dirname, '..', 'bin');
}

function binaryPath() {
  return path.join(binDir(), target().bin);
}

module.exports = { SUPPORTED, platformKey, target, binDir, binaryPath };

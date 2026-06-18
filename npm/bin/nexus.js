#!/usr/bin/env node
'use strict';

// Thin launcher: locate the platform binary that postinstall downloaded and
// hand off to it, forwarding args, stdio, and the exit code unchanged.

const fs = require('fs');
const { spawnSync } = require('child_process');
const { binaryPath } = require('../lib/resolve');

let bin;
try {
  bin = binaryPath();
} catch (e) {
  console.error(e.message);
  process.exit(1);
}

if (!fs.existsSync(bin)) {
  console.error('Athreix Nexus: binary not found at ' + bin);
  console.error('The install step may have been skipped (e.g. --ignore-scripts).');
  console.error('Re-run the installer with:');
  console.error('  node ' + require.resolve('../scripts/postinstall.js'));
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });

if (result.error) {
  console.error('Athreix Nexus: failed to launch binary — ' + result.error.message);
  process.exit(1);
}

// spawnSync sets status to null when the child was killed by a signal.
process.exit(result.status === null ? 1 : result.status);

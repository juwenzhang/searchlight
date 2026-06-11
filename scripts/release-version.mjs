#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const semverPattern = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/;

function usage() {
  console.error(`Usage:
  node scripts/release-version.mjs sync [vX.Y.Z]
  node scripts/release-version.mjs check [vX.Y.Z]

Tag can also come from RELEASE_VERSION or GITHUB_REF_NAME.`);
}

function fail(message) {
  console.error(`[release-version] ${message}`);
  process.exit(1);
}

function resolveTag(args) {
  const positional = args.find((arg) => !arg.startsWith('--'));
  return positional ?? process.env.RELEASE_VERSION ?? process.env.GITHUB_REF_NAME ?? '';
}

function parseVersion(tag) {
  const value = tag.trim();
  const match = semverPattern.exec(value);
  if (!match) {
    fail(`Invalid release tag "${tag}". Expected vX.Y.Z or X.Y.Z, with optional prerelease/build metadata.`);
  }

  const [, major, minor, patch, prerelease, build] = match;
  const version = `${major}.${minor}.${patch}${prerelease ? `-${prerelease}` : ''}${build ? `+${build}` : ''}`;
  return { tag: value.startsWith('v') ? value : `v${value}`, version };
}

function packagePath(relativePath) {
  return resolve(rootDir, relativePath);
}

function readJson(relativePath) {
  return JSON.parse(readFileSync(packagePath(relativePath), 'utf8'));
}

function writeJson(relativePath, value) {
  writeFileSync(packagePath(relativePath), `${JSON.stringify(value, null, 2)}\n`);
}

function readCargoVersion() {
  const cargoPath = packagePath('Cargo.toml');
  const content = readFileSync(cargoPath, 'utf8');
  const match = /^version\s*=\s*"([^"]+)"/m.exec(content);
  if (!match) fail('Cargo.toml is missing [package] version.');
  return match[1];
}

function writeCargoVersion(version) {
  const cargoPath = packagePath('Cargo.toml');
  const content = readFileSync(cargoPath, 'utf8');
  const next = content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  writeFileSync(cargoPath, next);
}

function peerRange(version) {
  return `^${version}`;
}

function sync(version) {
  writeCargoVersion(version);

  const reactHooks = readJson('packages/react-hooks/package.json');
  reactHooks.version = version;
  reactHooks.peerDependencies ??= {};
  reactHooks.peerDependencies['@luhanxin/searchlight'] = peerRange(version);
  reactHooks.devDependencies ??= {};
  reactHooks.devDependencies['@luhanxin/searchlight'] = 'workspace:^';
  writeJson('packages/react-hooks/package.json', reactHooks);

  const pkgPath = packagePath('pkg/package.json');
  if (existsSync(pkgPath)) {
    const wasmPackage = readJson('pkg/package.json');
    wasmPackage.version = version;
    writeJson('pkg/package.json', wasmPackage);
  }
}

function assertEqual(label, actual, expected) {
  if (actual !== expected) {
    fail(`${label} mismatch: expected ${expected}, got ${actual}`);
  }
}

function check(version) {
  assertEqual('Cargo.toml version', readCargoVersion(), version);

  const reactHooks = readJson('packages/react-hooks/package.json');
  assertEqual('packages/react-hooks/package.json version', reactHooks.version, version);
  assertEqual(
    'packages/react-hooks peer dependency @luhanxin/searchlight',
    reactHooks.peerDependencies?.['@luhanxin/searchlight'],
    peerRange(version),
  );

  const pkgPath = packagePath('pkg/package.json');
  if (existsSync(pkgPath)) {
    const wasmPackage = readJson('pkg/package.json');
    assertEqual('pkg/package.json version', wasmPackage.version, version);
  }
}

const [command, ...args] = process.argv.slice(2);
if (!command || !['sync', 'check'].includes(command)) {
  usage();
  process.exit(1);
}

const { tag, version } = parseVersion(resolveTag(args));

if (command === 'sync') {
  sync(version);
}

check(version);
console.log(`[release-version] ${command} ok: ${tag} -> ${version}`);

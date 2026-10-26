#!/usr/bin/env node
// Stage downloaded CI bundles for a release, generate latest.json and SHA256SUMS.
import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pkg = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
const repo = 'AsaanKaroobar/Asaan-Launcher';

function option(name, fallback) {
  const equals = process.argv.find((value) => value.startsWith(name + '='));
  if (equals) return equals.slice(name.length + 1);
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

const version = option('--version', pkg.version);
const tag = option('--tag', version);
const source = path.resolve(option('--source', path.join(root, '.artifacts', 'release-source')));
const output = path.resolve(option('--output', path.join(root, '.artifacts', 'release-staged', version)));
const notesFile = option('--notes-file', null);
const notes = notesFile
  ? (await fs.readFile(path.resolve(notesFile), 'utf8')).trim()
  : option('--notes', 'See the release notes on GitHub.');

async function walk(directory) {
  const found = [];
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) found.push(...await walk(full));
    else if (entry.isFile()) found.push(full);
  }
  return found;
}

function matches(files, endings) {
  return files.filter((file) => endings.some((ending) => file.endsWith(ending)));
}
function pick(files, endings, description) {
  const found = matches(files, endings);
  if (found.length !== 1) {
    throw new Error(`Expected exactly one ${description}, found ${found.length}: ${found.join(', ') || 'none'}`);
  }
  return found[0];
}
function optional(files, endings, description) {
  const found = matches(files, endings);
  if (found.length > 1) {
    throw new Error(`Expected at most one ${description}, found ${found.length}: ${found.join(', ')}`);
  }
  return found[0] ?? null;
}

const files = await walk(source);
const deb = pick(files, ['.deb'], 'Linux Debian package');
const rawAppImage = pick(files, ['.AppImage'], 'Linux AppImage');
const dmg = pick(files, ['.dmg'], 'macOS disk image');
const macUpdater = pick(files, ['.app.tar.gz'], 'macOS updater archive');
const macSignature = pick(files, ['.app.tar.gz.sig'], 'macOS updater signature');
const exe = pick(files, ['.exe'], 'Windows NSIS installer');

// Tauri signs raw bundles for deb/rpm/AppImage/NSIS in newer releases, and
// creates tarballs/zips for older or platform-specific update paths. Prefer the
// raw signature when present, otherwise use the archive signature.
const rawAppImageSignature = optional(files, ['.AppImage.sig'], 'Linux AppImage signature');
const appImageArchive = optional(files, ['.AppImage.tar.gz'], 'Linux updater archive');
const appImageArchiveSignature = optional(files, ['.AppImage.tar.gz.sig'], 'Linux updater archive signature');
const rawExeSignature = optional(files, ['.exe.sig'], 'Windows installer signature');
const winArchive = optional(files, ['.nsis.zip'], 'Windows NSIS updater archive') ?? optional(files, ['.zip'], 'Windows updater archive');
const winArchiveSignature = optional(files, ['.nsis.zip.sig'], 'Windows NSIS updater signature') ?? optional(files, ['.zip.sig'], 'Windows updater signature');

const linuxUpdater = rawAppImageSignature
  ? { file: rawAppImage, signature: rawAppImageSignature, name: `Asaaan-Launcher_${version}_linux_amd64.AppImage`, signatureName: `Asaaan-Launcher_${version}_linux_amd64.AppImage.sig` }
  : appImageArchive && appImageArchiveSignature
    ? { file: appImageArchive, signature: appImageArchiveSignature, name: `Asaaan-Launcher_${version}_linux_amd64.AppImage.tar.gz`, signatureName: `Asaaan-Launcher_${version}_linux_amd64.AppImage.tar.gz.sig` }
    : null;
if (!linuxUpdater) throw new Error('Linux updater requires .AppImage.sig or .AppImage.tar.gz.sig.');

const windowsUpdater = rawExeSignature
  ? { file: exe, signature: rawExeSignature, name: `Asaaan-Launcher_${version}_windows_x64.exe`, signatureName: `Asaaan-Launcher_${version}_windows_x64.exe.sig` }
  : winArchive && winArchiveSignature
    ? { file: winArchive, signature: winArchiveSignature, name: `Asaaan-Launcher_${version}_windows_x64.nsis.zip`, signatureName: `Asaaan-Launcher_${version}_windows_x64.nsis.zip.sig` }
    : null;
if (!windowsUpdater) throw new Error('Windows updater requires .exe.sig or .nsis.zip.sig.');

const names = {
  deb: `Asaaan-Launcher_${version}_linux_amd64.deb`,
  rawAppImage: `Asaaan-Launcher_${version}_linux_amd64.AppImage`,
  dmg: `Asaaan-Launcher_${version}_macos_aarch64.dmg`,
  macUpdater: `Asaaan-Launcher_${version}_macos_aarch64.app.tar.gz`,
  macSignature: `Asaaan-Launcher_${version}_macos_aarch64.app.tar.gz.sig`,
  exe: `Asaaan-Launcher_${version}_windows_x64.exe`,
};

const copies = [
  [deb, names.deb],
  [rawAppImage, names.rawAppImage],
  [dmg, names.dmg],
  [macUpdater, names.macUpdater],
  [macSignature, names.macSignature],
  [exe, names.exe],
  [linuxUpdater.file, linuxUpdater.name],
  [linuxUpdater.signature, linuxUpdater.signatureName],
  [windowsUpdater.file, windowsUpdater.name],
  [windowsUpdater.signature, windowsUpdater.signatureName],
].filter(([from, name]) => from && !(name === names.rawAppImage && from !== rawAppImage));

const signatures = {
  linux: (await fs.readFile(linuxUpdater.signature, 'utf8')).trim(),
  macos: (await fs.readFile(macSignature, 'utf8')).trim(),
  windows: (await fs.readFile(windowsUpdater.signature, 'utf8')).trim(),
};
if (!signatures.linux || !signatures.macos || !signatures.windows) {
  throw new Error('One or more updater signature files are empty.');
}

const base = `https://github.com/${repo}/releases/download/${encodeURIComponent(tag)}`;
const latest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms: {
    'linux-x86_64': {
      signature: signatures.linux,
      url: `${base}/${linuxUpdater.name}`,
    },
    'darwin-aarch64': {
      signature: signatures.macos,
      url: `${base}/${names.macUpdater}`,
    },
    'windows-x86_64': {
      signature: signatures.windows,
      url: `${base}/${windowsUpdater.name}`,
    },
  },
};

await fs.rm(output, { recursive: true, force: true });
await fs.mkdir(output, { recursive: true });
for (const [from, name] of copies) {
  await fs.copyFile(from, path.join(output, name));
}
await fs.writeFile(path.join(output, 'latest.json'), JSON.stringify(latest, null, 2) + '\n');

const staged = (await fs.readdir(output)).filter((name) => name !== 'SHA256SUMS').sort();
const sums = [];
for (const name of staged) {
  const bytes = await fs.readFile(path.join(output, name));
  sums.push(createHash('sha256').update(bytes).digest('hex') + '  ' + name);
}
await fs.writeFile(path.join(output, 'SHA256SUMS'), sums.join('\n') + '\n');
console.log(`Staged Asaaan Launcher ${version} in ${output}`);
for (const name of staged) console.log(`  ${name}`);

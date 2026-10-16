import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';

const run = promisify(execFile);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const deployRoot = path.join(root, 'deployments/linux');
const current = path.join(deployRoot, 'current');
const command = process.argv[2];
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');

async function install() {
  const pkg = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
  const bundle = path.join(root, 'src-tauri/target/release/bundle/deb');
  const candidates = (await fs.readdir(bundle)).filter((file) => file.endsWith('.deb')).sort();
  if (candidates.length !== 1) throw new Error('Expected exactly one Debian package. Clean stale bundles and rebuild before installing.');
  const file = path.join(bundle, candidates[0]);
  const fields = (await run('dpkg-deb', ['--field', file, 'Package', 'Version', 'Architecture'])).stdout;
  const field = (name) => fields.match(new RegExp('^' + name + ': (.+)$', 'm'))?.[1];
  if (field('Package') !== 'asaaan-launcher' || field('Version') !== pkg.version) throw new Error('The package identity/version does not match this project.');
  const architecture = (await run('dpkg', ['--print-architecture'])).stdout.trim();
  if (field('Architecture') !== architecture) throw new Error('The package does not match this machine architecture.');
  const sha256 = digest(await fs.readFile(file));
  const version = pkg.version + '-' + sha256.slice(0, 12);
  const destination = path.join(deployRoot, 'versions', version);
  await fs.mkdir(path.dirname(destination), { recursive: true });
  let present = false;
  try { await fs.access(destination); present = true; } catch (error) { if (error.code !== 'ENOENT') throw error; }
  if (!present) {
    const staging = await fs.mkdtemp(path.join(deployRoot, '.staging-'));
    try {
      await run('dpkg-deb', ['--extract', file, staging]);
      const binary = path.join(staging, 'usr/bin/asaaan-launcher');
      await fs.access(binary, fs.constants.X_OK);
      await fs.writeFile(path.join(staging, 'installation.json'), JSON.stringify({ checkpoint: 'CP001', version: pkg.version, package: candidates[0], packageSha256: sha256, binarySha256: digest(await fs.readFile(binary)), architecture, installedAt: new Date().toISOString() }, null, 2) + '\n');
      await fs.rename(staging, destination);
    } finally { await fs.rm(staging, { recursive: true, force: true }); }
  }
  const temporaryLink = path.join(deployRoot, '.current-' + process.pid);
  await fs.symlink(path.relative(deployRoot, destination), temporaryLink);
  await fs.rename(temporaryLink, current);
  const shortcut = '[Desktop Entry]\nType=Application\nName=Asaaan Launcher\nComment=Asaaan Launcher desktop foundation\nExec=\"' + path.join(current, 'usr/bin/asaaan-launcher') + '\"\nIcon=' + path.join(root, 'src-tauri/icons/128x128.png') + '\nTerminal=false\nCategories=Utility;\n';
  await fs.writeFile(path.join(deployRoot, 'Asaaan Launcher.desktop'), shortcut, { mode: 0o755 });
  console.log('Installed Asaaan Launcher locally at ' + current);
  console.log('Package SHA-256: ' + sha256);
  console.log('Run npm run smoke:local, then npm run start:local.');
}

async function installedBinary() {
  const binary = path.join(current, 'usr/bin/asaaan-launcher');
  const manifest = JSON.parse(await fs.readFile(path.join(current, 'installation.json'), 'utf8'));
  if (digest(await fs.readFile(binary)) !== manifest.binarySha256) throw new Error('Installed binary changed. Reinstall the validated package.');
  return binary;
}

async function smoke() {
  const binary = await installedBinary();
  const result = await run(binary, ['--smoke-test'], { timeout: 30000, maxBuffer: 1024 * 1024 });
  if (!result.stdout.includes('ASAAAN_NATIVE_SMOKE_OK')) throw new Error('Native process exited without frontend readiness evidence.\n' + result.stdout + result.stderr);
  console.log(result.stdout.trim());
  if (result.stderr.trim()) console.error(result.stderr.trim());
  const evidence = { checkpoint: 'CP001', result: 'passed', kind: 'installed native frontend/Rust round trip', at: new Date().toISOString(), installation: JSON.parse(await fs.readFile(path.join(current, 'installation.json'), 'utf8')) };
  await fs.mkdir(path.join(root, '.artifacts'), { recursive: true });
  await fs.writeFile(path.join(root, '.artifacts/native-smoke.json'), JSON.stringify(evidence, null, 2) + '\n');
}

async function start() {
  const binary = await installedBinary();
  const child = spawn(binary, [], { stdio: 'inherit' });
  child.on('error', (error) => { console.error(error.message); process.exitCode = 1; });
  child.on('exit', (code) => { process.exitCode = code ?? 1; });
}

try {
  if (process.platform !== 'linux') throw new Error('This local deployment script currently supports Linux Debian packages only.');
  if (command === 'install') await install();
  else if (command === 'smoke') await smoke();
  else if (command === 'run') await start();
  else throw new Error('Usage: node scripts/local-deploy.mjs install|smoke|run');
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}

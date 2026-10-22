// Native Wayland/Hyprland checks for the standalone picker and Settings window.
// Uses a live Hyprland session and only targets the launcher's own windows.
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';

const run = promisify(execFile);
const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const pkg = JSON.parse(await fs.readFile(path.join(root, 'package.json'), 'utf8'));
const artifacts = path.join(root, '.artifacts/wayland-hyprland');
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

if (process.env.XDG_SESSION_TYPE !== 'wayland' || !process.env.HYPRLAND_INSTANCE_SIGNATURE) {
  console.log('Skipped: a live Hyprland Wayland session is required.');
  process.exit(0);
}
const existing = await run('pgrep', ['-x', 'asaaan-launcher']).catch(() => null);
if (existing) {
  console.log('Skipped: an Asaaan Launcher resident is already running; stop it before this live-session test.');
  process.exit(0);
}

const installed = path.join(root, 'deployments/linux/current');
const explicit = process.argv.indexOf('--binary');
if (explicit >= 0 && !process.argv[explicit + 1]) throw new Error('--binary requires a path');
const binary = explicit >= 0 ? path.resolve(process.argv[explicit + 1]) : path.join(installed, 'usr/bin/asaaan-launcher');
await fs.access(binary, fs.constants.X_OK);
const manifest = explicit < 0 ? JSON.parse(await fs.readFile(path.join(installed, 'installation.json'), 'utf8')) : null;
if (manifest && (manifest.version !== pkg.version || manifest.checkpoint !== pkg.checkpoint)) {
  throw new Error(`Installed package is ${manifest.version}/${manifest.checkpoint}; expected ${pkg.version}/${pkg.checkpoint}. Reinstall before running this test.`);
}
const binarySha256 = createHash('sha256').update(await fs.readFile(binary)).digest('hex');
if (manifest && manifest.binarySha256 !== binarySha256) throw new Error('Installed binary hash does not match its installation manifest.');

const hypr = async (...args) => JSON.parse((await run('hyprctl', ['-j', ...args], { timeout: 5000 })).stdout);
const send = async (address, key) => {
  const result = await run('hyprctl', ['dispatch', 'sendshortcut', `,${key},address:${address}`], { timeout: 5000 });
  if (result.stdout.trim() !== 'ok') throw new Error(`Hyprland sendshortcut ${key} failed: ${result.stdout}`);
};
async function until(check, message, timeout = 20000) {
  const deadline = Date.now() + timeout;
  while (!(await check())) {
    if (Date.now() >= deadline) throw new Error(message);
    await delay(50);
  }
}
function expectedCenter(monitor, width, height) {
  const scale = monitor.scale > 0 ? monitor.scale : 1;
  const reserved = monitor.reserved ?? [0, 0, 0, 0];
  return [
    monitor.x + reserved[0] + Math.max(0, Math.round((monitor.width / scale - reserved[0] - reserved[2] - width) / 2)),
    monitor.y + reserved[1] + Math.max(0, Math.round((monitor.height / scale - reserved[1] - reserved[3] - height) / 2)),
  ];
}
async function geometry(pid, title = null) {
  const client = (await hypr('clients')).find((item) => item.pid === pid && (!title || title.test(item.title)));
  if (!client) return null;
  const monitor = (await hypr('monitors')).find((item) => item.id === client.monitor);
  const expected = expectedCenter(monitor, client.size[0], client.size[1]);
  return { client, monitor, expected };
}

const scratch = await fs.mkdtemp(path.join(os.tmpdir(), 'asaaan-wayland-windows-'));
const env = { ...process.env, XDG_CONFIG_HOME: scratch, XDG_DATA_HOME: scratch };
const stages = [];
let picker = null;
let resident = null;
let pickerLog = '';
let residentLog = '';

try {
  await fs.mkdir(artifacts, { recursive: true });
  picker = spawn(binary, ['--pick', '--prompt', 'Choose a workspace'], { stdio: ['pipe', 'pipe', 'pipe'], env });
  let pickerOut = '';
  picker.stdout.on('data', (data) => { pickerOut += data; });
  picker.stderr.on('data', (data) => { pickerLog += data; });
  picker.stdin.end('Development\nDesign\nDocuments\n');
  await until(async () => (await geometry(picker.pid)) !== null, 'Picker window missing: ' + pickerLog);
  await until(async () => {
    const current = await geometry(picker.pid);
    return current && current.client.mapped && current.client.size[0] === 750 && current.client.size[1] === 474 && current.client.floating;
  }, 'Picker did not settle');
  const pickerGeometry = await geometry(picker.pid);
  await until(async () => {
    const current = await geometry(picker.pid);
    return current && Math.abs(current.client.at[0] - current.expected[0]) <= 2 && Math.abs(current.client.at[1] - current.expected[1]) <= 2;
  }, 'Picker did not center on the focused monitor');
  await until(async () => (await hypr('activewindow')).address === pickerGeometry.client.address, 'Picker did not receive focus');
  await delay(800);
  for (const key of ['D', 'E', 'V']) await send(pickerGeometry.client.address, key);
  await delay(400);
  await send(pickerGeometry.client.address, 'Return');
  await until(() => picker.exitCode !== null, 'Picker did not choose a result: ' + pickerLog, 5000);
  if (pickerOut !== 'Development\n') throw new Error('Picker output mismatch: ' + JSON.stringify(pickerOut));
  if (pickerGeometry.client.xwayland !== false) throw new Error('Picker opened through XWayland');
  stages.push('standalone picker is native Wayland, floating, 750x474, centered, focused and keyboard-selectable');
  const pickerEvidence = {
    class: pickerGeometry.client.class,
    xwayland: pickerGeometry.client.xwayland,
    floating: pickerGeometry.client.floating,
    at: pickerGeometry.client.at,
    size: pickerGeometry.client.size,
    monitor: pickerGeometry.client.monitor,
    expectedAt: pickerGeometry.expected,
    stdout: pickerOut,
  };
  picker = null;

  resident = spawn(binary, ['--settings'], { stdio: ['ignore', 'pipe', 'pipe'], env });
  resident.stdout.on('data', (data) => { residentLog += data; });
  resident.stderr.on('data', (data) => { residentLog += data; });
  const settingsWindow = /Settings/;
  const findSettings = async () => {
    const client = (await hypr('clients')).find((item) => item.class === 'asaaan-launcher' && settingsWindow.test(item.title));
    if (!client) return null;
    const monitor = (await hypr('monitors')).find((item) => item.id === client.monitor);
    return { client, monitor, expected: expectedCenter(monitor, client.size[0], client.size[1]) };
  };
  await until(async () => (await findSettings()) !== null, 'Settings window missing: ' + residentLog);
  await until(async () => {
    const current = await findSettings();
    return current && current.client.mapped && current.client.size[0] === 960 && current.client.size[1] === 640 && current.client.floating;
  }, 'Settings did not settle');
  const settingsGeometry = await findSettings();
  await until(async () => {
    const current = await findSettings();
    return current && Math.abs(current.client.at[0] - current.expected[0]) <= 2 && Math.abs(current.client.at[1] - current.expected[1]) <= 2;
  }, 'Settings did not center on the focused monitor');
  await until(async () => (await hypr('activewindow')).address === settingsGeometry.client.address, 'Settings did not receive focus');
  if (settingsGeometry.client.xwayland !== false) throw new Error('Settings opened through XWayland');
  await run(binary, ['--quit'], { env, timeout: 5000 });
  await until(() => resident.exitCode !== null, 'Resident did not quit after Settings', 5000);
  await until(async () => (await findSettings()) === null, 'Settings window survived quit');
  stages.push('resident-owned Settings is native Wayland, floating, 960x640, centered, focused and cleaned up');
  resident = null;

  const evidence = {
    checkpoint: pkg.checkpoint,
    result: 'passed',
    kind: 'installed native Wayland picker and Settings window placement/focus',
    at: new Date().toISOString(),
    binary,
    binarySha256,
    installation: manifest,
    environment: {
      sessionType: process.env.XDG_SESSION_TYPE,
      desktop: process.env.XDG_CURRENT_DESKTOP,
      hyprland: (await run('hyprctl', ['version'])).stdout.split('\n')[0],
    },
    stages,
    picker: pickerEvidence,
    settings: {
      class: settingsGeometry.client.class,
      title: settingsGeometry.client.title,
      xwayland: settingsGeometry.client.xwayland,
      floating: settingsGeometry.client.floating,
      at: settingsGeometry.client.at,
      size: settingsGeometry.client.size,
      monitor: settingsGeometry.client.monitor,
      expectedAt: settingsGeometry.expected,
      quit: 'passed',
    },
  };
  await fs.writeFile(path.join(artifacts, 'windows.json'), JSON.stringify(evidence, null, 2) + '\n');
  console.log(JSON.stringify({ result: 'passed', stages, picker: evidence.picker, settings: evidence.settings, binarySha256 }, null, 2));
} catch (error) {
  await fs.mkdir(artifacts, { recursive: true }).catch(() => {});
  await fs.writeFile(path.join(artifacts, 'windows-failure.json'), JSON.stringify({ checkpoint: pkg.checkpoint, result: 'failed', error: String(error?.stack ?? error), pickerLog, residentLog, stages }, null, 2) + '\n').catch(() => {});
  throw error;
} finally {
  if (picker && picker.exitCode === null) picker.kill();
  if (resident && resident.exitCode === null) resident.kill();
  await fs.rm(scratch, { recursive: true, force: true }).catch(() => {});
}

// Native Wayland/Hyprland acceptance for the installed launcher.
// Uses this live Hyprland session; never injects keys into another window.
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
const manifest = explicit < 0
  ? JSON.parse(await fs.readFile(path.join(installed, 'installation.json'), 'utf8'))
  : null;
if (manifest && (manifest.version !== pkg.version || manifest.checkpoint !== pkg.checkpoint)) {
  throw new Error(`Installed package is ${manifest.version}/${manifest.checkpoint}; expected ${pkg.version}/${pkg.checkpoint}. Reinstall before running this test.`);
}
const binarySha256 = createHash('sha256').update(await fs.readFile(binary)).digest('hex');
if (manifest && manifest.binarySha256 !== binarySha256) throw new Error('Installed binary hash does not match its installation manifest.');

const hypr = async (...args) => JSON.parse((await run('hyprctl', ['-j', ...args], { timeout: 5000 })).stdout);
const hyprDispatch = async (dispatcher, argument) => {
  const result = await run('hyprctl', ['dispatch', dispatcher, argument], { timeout: 5000 });
  if (result.stdout.trim() !== 'ok') throw new Error(`Hyprland ${dispatcher} failed: ${result.stdout}${result.stderr}`);
};
const a11y = async (pid) => JSON.parse((await run('/usr/bin/python3', [path.join(root, 'scripts/wayland-search-state.py'), String(pid)], { timeout: 15000, maxBuffer: 4 * 1024 * 1024 })).stdout);

function expectedCenter(monitor, width, height) {
  const scale = monitor.scale > 0 ? monitor.scale : 1;
  const reserved = monitor.reserved ?? [0, 0, 0, 0];
  const usableWidth = monitor.width / scale - reserved[0] - reserved[2];
  const usableHeight = monitor.height / scale - reserved[1] - reserved[3];
  return [
    monitor.x + reserved[0] + Math.max(0, Math.round((usableWidth - width) / 2)),
    monitor.y + reserved[1] + Math.max(0, Math.round((usableHeight - height) / 2)),
  ];
}

const scratch = await fs.mkdtemp(path.join(os.tmpdir(), 'asaaan-wayland-'));
const env = { ...process.env, XDG_CONFIG_HOME: scratch, XDG_DATA_HOME: scratch };
const stages = [];
let log = '';
let resident;
let screenshot = null;
let searchScreenshot = null;
let failure = null;

async function until(check, message, timeout = 20000) {
  const deadline = Date.now() + timeout;
  while (!(await check())) {
    if (Date.now() >= deadline) throw new Error(message + '\n--- resident log ---\n' + log);
    await delay(50);
  }
}
async function panelClient(pid) {
  const clients = await hypr('clients');
  return clients.find((client) => client.pid === pid);
}
async function shown(mark) {
  await until(() => log.slice(mark).includes('visible: true'), 'Launcher did not present');
  const pid = Number([...log.matchAll(/ASAAAN_LIFECYCLE Started\((\d+)\)/g)].at(-1)?.[1]);
  if (!pid) throw new Error('Resident did not report a panel PID');
  return pid;
}
async function hidden(mark) {
  await until(() => log.slice(mark).includes('visible: false'), 'Launcher did not hide');
}
async function toggle(args = ['--toggle']) {
  const mark = log.length;
  await run(binary, args, { env, timeout: 10000 });
  return mark;
}

try {
  await fs.mkdir(artifacts, { recursive: true });
  const before = await hypr('activewindow');
  resident = spawn(binary, ['--background'], { stdio: ['ignore', 'pipe', 'pipe'], env });
  resident.stdout.on('data', (data) => { log += data; });
  resident.stderr.on('data', (data) => { log += data; });
  await until(() => log.includes('ASAAAN_SHORTCUT'), 'Wayland shortcut status did not initialize');
  const shortcut = log.split('\n').find((line) => line.includes('ASAAAN_SHORTCUT')) ?? '';
  if (!shortcut.includes('Wayland')) throw new Error('Wayland shortcut status was not reported: ' + shortcut);
  stages.push('resident starts on Wayland without an X11 shortcut grab');

  await toggle();
  let pid = await shown(0);
  const initial = await panelClient(pid);
  const settled = async () => {
    const candidate = await panelClient(pid);
    return candidate?.mapped === true && candidate.size[0] === 750 && candidate.size[1] === 474 ? candidate : null;
  };
  await until(async () => (await settled()) !== null, 'Hyprland did not settle the launcher to 750x474');
  const client = await settled();
  if (client.class !== 'asaaan-launcher') throw new Error('Unexpected launcher class: ' + client.class);
  if (client.xwayland !== false) throw new Error('Launcher opened through XWayland');
  if (client.floating !== true) throw new Error('Launcher is not floating on Hyprland');
  const focusedMonitor = (await hypr('monitors')).find((monitor) => monitor.focused);
  if (!focusedMonitor) throw new Error('No focused Hyprland monitor');
  if (client.monitor !== focusedMonitor.id) throw new Error(`Launcher opened on monitor ${client.monitor}, expected focused monitor ${focusedMonitor.id}`);
  const expected = expectedCenter(focusedMonitor, client.size[0], client.size[1]);
  await until(async () => {
    const candidate = await panelClient(pid);
    return candidate && Math.abs(candidate.at[0] - expected[0]) <= 2 && Math.abs(candidate.at[1] - expected[1]) <= 2;
  }, `Launcher did not settle at the centered position ${expected}`);
  await until(async () => (await hypr('activewindow')).address === client.address, 'Launcher did not receive Wayland focus', 5000);
  const active = await hypr('activewindow');
  if (active.address !== client.address) throw new Error('Launcher did not receive Wayland focus');
  stages.push('native Wayland surface (xwayland=false), floating, 750x474, centered on the focused monitor, focused');

  await until(async () => {
    const state = await a11y(pid);
    return state.search === '' && state.items.some((name) => /Calculator/.test(name));
  }, 'Search results did not become visible');
  screenshot = path.join(artifacts, 'wayland-hyprland.png');
  await run('grim', ['-g', `${client.at[0]},${client.at[1]} ${client.size[0]}x${client.size[1]}`, screenshot]);
  const screenshotSize = (await fs.stat(screenshot)).size;
  if (screenshotSize < 10000) throw new Error('Launcher screenshot is unexpectedly small');
  stages.push('application search results render and survive the first hidden-created presentation');

  await hyprDispatch('focuswindow', `address:${client.address}`);
  await delay(150);
  const typingActive = await hypr('activewindow');
  if (typingActive.address !== client.address) throw new Error('Launcher lost Wayland focus before the keyboard test');
  for (const key of ['C', 'A', 'L']) await hyprDispatch('sendshortcut', `,${key},address:${client.address}`);
  await until(async () => {
    const state = await a11y(pid);
    return state.search === 'cal' && state.items.some((name) => /Calculator|Calendar/.test(name));
  }, 'Keyboard input did not reach the search field');
  searchScreenshot = path.join(artifacts, 'wayland-hyprland-search.png');
  await run('grim', ['-g', `${client.at[0]},${client.at[1]} ${client.size[0]}x${client.size[1]}`, searchScreenshot]);
  if ((await fs.stat(searchScreenshot)).size < 10000) throw new Error('Filtered-search screenshot is unexpectedly small');
  stages.push('physical-style Hyprland key delivery types "cal" and filters results');

  const escapeMark = log.length;
  await hyprDispatch('sendshortcut', `,Escape,address:${client.address}`);
  await until(async () => (await a11y(pid)).search === '', 'Escape did not clear the search field');
  await hyprDispatch('sendshortcut', `,Escape,address:${client.address}`);
  await hidden(escapeMark);
  await delay(250);
  const hiddenClient = await panelClient(pid);
  if (hiddenClient && hiddenClient.mapped) throw new Error('Launcher remained mapped after Escape');
  stages.push('Escape clears then hides through Wayland key delivery');

  const reopenMark = await toggle();
  const reopenedPid = await shown(reopenMark);
  await until(async () => (await panelClient(reopenedPid))?.mapped === true, 'Hyprland did not remap the launcher');
  const reopened = await panelClient(reopenedPid);
  if (reopenedPid !== pid) throw new Error(`Grace-period toggle did not reuse the panel process: first=${pid}, reopened=${reopenedPid}`);
  const reopenedActive = await hypr('activewindow');
  if (reopenedActive.address !== reopened.address) throw new Error('Reused launcher did not regain focus');
  stages.push('grace-period --toggle reuses the same panel process and regains focus');

  const finishMark = await toggle();
  await hidden(finishMark);
  await delay(250);
  const focusAfter = await hypr('activewindow');
  const focusRestored = before.address ? focusAfter.address === before.address : null;
  if (before.address && !focusRestored) throw new Error('Focus did not return to the previous window: ' + before.address);
  await toggle(['--quit']);
  await until(() => resident.exitCode !== null, 'Resident did not quit');
  if (await panelClient(pid)) throw new Error('Launcher window survived resident quit');
  stages.push('toggle hides, focus returns to the previous window, and quit removes the panel');

  const evidence = {
    checkpoint: pkg.checkpoint,
    result: 'passed',
    kind: 'installed native Wayland/Hyprland resident, window placement, focus and keyboard',
    at: new Date().toISOString(),
    environment: {
      sessionType: process.env.XDG_SESSION_TYPE,
      desktop: process.env.XDG_CURRENT_DESKTOP,
      hyprland: (await run('hyprctl', ['version'])).stdout.split('\n')[0],
      socket: process.env.HYPRLAND_INSTANCE_SIGNATURE,
    },
    binary,
    binarySha256,
    installation: manifest,
    stdout: log,
    stages,
    initialClient: initial ? { address: initial.address, at: initial.at, size: initial.size } : null,
    beforeFocus: before.address,
    afterFocus: focusAfter.address,
    focusRestored,
    client: {
      address: client.address,
      reopenedAddress: reopened.address,
      class: client.class,
      title: client.title,
      xwayland: client.xwayland,
      floating: client.floating,
      at: client.at,
      size: client.size,
      monitor: client.monitor,
    },
    monitor: {
      id: focusedMonitor.id,
      name: focusedMonitor.name,
      x: focusedMonitor.x,
      y: focusedMonitor.y,
      width: focusedMonitor.width,
      height: focusedMonitor.height,
      scale: focusedMonitor.scale,
      reserved: focusedMonitor.reserved,
    },
    expectedAt: expected,
    screenshots: [screenshot, searchScreenshot],
  };
  await fs.writeFile(path.join(artifacts, 'evidence.json'), JSON.stringify(evidence, null, 2) + '\n');
  console.log(JSON.stringify({ result: 'passed', stages, client: evidence.client, expectedAt: expected, binarySha256, focusRestored }, null, 2));
} catch (error) {
  failure = error;
  const evidence = { checkpoint: pkg.checkpoint, result: 'failed', at: new Date().toISOString(), error: String(error?.stack ?? error), stdout: log, stages };
  await fs.mkdir(artifacts, { recursive: true }).catch(() => {});
  await fs.writeFile(path.join(artifacts, 'failure.json'), JSON.stringify(evidence, null, 2) + '\n').catch(() => {});
  throw error;
} finally {
  if (resident && resident.exitCode === null) resident.kill();
  await fs.rm(scratch, { recursive: true, force: true }).catch(() => {});
}

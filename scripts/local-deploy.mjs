import os from 'node:os';
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
const pkg = JSON.parse(await fs.readFile(path.join(root, "package.json"), "utf8"));
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');

async function install() {
  const bundle = path.join(root, 'src-tauri/target/release/bundle/deb');
  const candidates = (await fs.readdir(bundle)).filter((file) => file.endsWith('.deb') && file.includes('_' + pkg.version + '_')).sort();
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
      await fs.writeFile(path.join(staging, 'installation.json'), JSON.stringify({ checkpoint: pkg.checkpoint, version: pkg.version, package: candidates[0], packageSha256: sha256, binarySha256: digest(await fs.readFile(binary)), architecture, installedAt: new Date().toISOString() }, null, 2) + '\n');
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
  if (manifest.version !== pkg.version || manifest.checkpoint !== pkg.checkpoint) throw new Error('The installed package is from another checkpoint. Run npm run install:local.');
  if (digest(await fs.readFile(binary)) !== manifest.binarySha256) throw new Error('Installed binary changed. Reinstall the validated package.');
  return binary;
}

async function smoke() {
  const binary = await installedBinary();
  const result = await run(binary, ['--smoke-test'], { timeout: 30000, maxBuffer: 1024 * 1024 });
  if (!result.stdout.includes('ASAAAN_NATIVE_SMOKE_OK ' + pkg.checkpoint)) throw new Error('Native process exited without frontend readiness evidence.\n' + result.stdout + result.stderr);
  console.log(result.stdout.trim());
  if (result.stderr.trim()) console.error(result.stderr.trim());
  const evidence = { checkpoint: pkg.checkpoint, result: 'passed', kind: 'installed native frontend/Tauri/Rust cal search with ordered result IDs', at: new Date().toISOString(), installation: JSON.parse(await fs.readFile(path.join(current, 'installation.json'), 'utf8')) };
  await fs.mkdir(path.join(root, '.artifacts'), { recursive: true });
  await fs.writeFile(path.join(root, '.artifacts/native-smoke.json'), JSON.stringify(evidence, null, 2) + '\n');
}

async function verifyPanelEof(binary) {
  await new Promise((resolve,reject)=>{
    const child=spawn(binary,['--panel'],{stdio:['pipe','pipe','pipe']});let buffer='';let shown=false;let diagnostic='';
    const timer=setTimeout(()=>{child.kill();reject(Error('Native panel did not exit after private pipe EOF. '+diagnostic));},20000);
    child.stderr.on('data',d=>diagnostic+=d);
    child.on('error',error=>{clearTimeout(timer);reject(error);});
    child.stdin.on('error',error=>{child.kill();clearTimeout(timer);reject(error);});
    child.stdout.on('data',d=>{
      buffer+=d;if(buffer.length>4*1024*1024){child.kill();reject(Error('Oversized native panel packet'));return;}
      let newline;while((newline=buffer.indexOf('\n'))>=0){const line=buffer.slice(0,newline);buffer=buffer.slice(newline+1);let packet;
        try{packet=JSON.parse(line);}catch(error){child.kill();reject(error);return;}
        if(packet.type==='ready')child.stdin.write(JSON.stringify({type:'request',request:'show',sequence:1})+'\n');
        if(packet.type==='presented'&&packet.visible===true){shown=true;child.stdin.end();}
      }
    });
    child.on('exit',code=>{clearTimeout(timer);if(code===0&&shown)resolve();else reject(Error('Native EOF test failed: '+code+' '+diagnostic));});
  });
}

async function residentSmoke() {
  const binary = await installedBinary();
  const result = await run(binary, ['--resident-smoke-test'], {timeout:60000,maxBuffer:1024*1024});
  if (!result.stdout.includes('ASAAAN_RESIDENT_SMOKE_OK CP007 cycles=3')) throw Error('Resident cycle verification failed. '+result.stdout+result.stderr);
  const pids=[...result.stderr.matchAll(/ASAAAN_LIFECYCLE Started\((\d+)\)/g)].map(m=>Number(m[1]));
  if(pids.length!==3 || new Set(pids).size!==3) throw Error('Expected three distinct panel processes');
  for(const pid of pids) { try { process.kill(pid,0); } catch(e) {if(e.code==='ESRCH')continue;throw e;} throw Error('Panel process remains after dismissal: '+pid); }
  await verifyPanelEof(binary);
  console.log(result.stdout.trim()+' nativePanelEof=passed');
  console.error(result.stderr.trim());
  await fs.writeFile(path.join(root,'.artifacts/resident-smoke.json'),JSON.stringify({checkpoint:pkg.checkpoint,result:'passed',cycles:3,pids,allReaped:true,nativePanelEof:true,at:new Date().toISOString(),installation:JSON.parse(await fs.readFile(path.join(current,'installation.json'),'utf8'))},null,2)+'\n');
}

async function applicationSmoke() {
  const binary=await installedBinary();
  const scratch=await fs.mkdtemp(path.join(os.tmpdir(),'asaaan cp006-'));
  try {
    const data=path.join(scratch,'data');const system=path.join(scratch,'system');const marker=path.join(scratch,'marker.json');const probe=path.join(scratch,'probe.mjs');
    await fs.mkdir(path.join(data,'applications'),{recursive:true});await fs.mkdir(system,{recursive:true});
    await fs.writeFile(probe,'import fs from "node:fs/promises"; console.log("Application output is isolated from panel IPC"); await fs.writeFile(process.argv[2],JSON.stringify(process.argv.slice(3)));\n');
    const desktop=path.join(data,'applications/asaaan-checkpoint-probe.desktop');
    const text='[Desktop Entry]\nType=Application\nName=Asaaan Checkpoint Probe\nExec="'+process.execPath+'" "'+probe+'" "'+marker+'" "literal ; safe" %c %% %k %U\n';
    await fs.writeFile(desktop,text);
    const result=await run(binary,['--application-smoke-test'],{timeout:30000,maxBuffer:1024*1024,env:{...process.env,XDG_DATA_HOME:data,XDG_DATA_DIRS:system,XDG_CONFIG_HOME:path.join(scratch,'config'),XDG_CURRENT_DESKTOP:'AsaaanTest',LC_ALL:'C.UTF-8',LANGUAGE:'en'}});
    if(!result.stdout.includes('ASAAAN_APPLICATION_SMOKE_OK '+pkg.checkpoint))throw Error('Installed UI did not confirm a native application launch: '+result.stdout+result.stderr);
    const deadline=Date.now()+5000;let recorded;
    while(Date.now()<deadline){try{recorded=JSON.parse(await fs.readFile(marker,'utf8'));break;}catch(error){if(error.code!=='ENOENT')throw error;await new Promise(resolve=>setTimeout(resolve,25));}}
    if(JSON.stringify(recorded)!==JSON.stringify(['literal ; safe','Asaaan Checkpoint Probe','%',desktop]))throw Error('Native desktop Exec expansion/launch probe failed: '+JSON.stringify(recorded));
    const evidence={checkpoint:pkg.checkpoint,result:'passed',kind:'installed React selection to Rust catalog ID to native GIO launch; isolated desktop fixture',arguments:recorded,at:new Date().toISOString(),installation:JSON.parse(await fs.readFile(path.join(current,'installation.json'),'utf8'))};
    await fs.writeFile(path.join(root,'.artifacts/application-smoke.json'),JSON.stringify(evidence,null,2)+'\n');console.log(result.stdout.trim());
  } finally {await fs.rm(scratch,{recursive:true,force:true});}
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
  else if (command === 'resident-smoke') await residentSmoke();
  else if (command === 'application-smoke') await applicationSmoke();
  else if (command === 'run') await start();
  else throw new Error('Usage: node scripts/local-deploy.mjs install|smoke|resident-smoke|application-smoke|run');
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}

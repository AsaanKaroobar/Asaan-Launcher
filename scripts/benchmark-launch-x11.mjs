// Repeatable installed launch benchmark. Run only through the isolated X11 harness.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import os from 'node:os';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
const run=promisify(execFile);
if(path.basename(process.env.XDG_RUNTIME_DIR??'')!=='runtime'||!path.basename(path.dirname(process.env.XDG_RUNTIME_DIR??'')).startsWith('asaaan-x11-'))throw Error('Run npm run benchmark:launch; a private X11 session is required');
const root=new URL('../',import.meta.url);
const pkg=JSON.parse(await fs.readFile(new URL('package.json',root),'utf8'));
const option=name=>process.argv.includes(name)?process.argv[process.argv.indexOf(name)+1]:undefined;
const binary=option('--binary')??new URL('deployments/linux/current/usr/bin/asaaan-launcher',root).pathname;
const count=Number(option('--samples')??12);
if(!Number.isInteger(count)||count<5||count>100)throw Error('Use 5–100 samples');
const output=new URL(option('--output')??'.artifacts/launch-benchmark.json',root);
const reuseDelayMs=Number(option('--reuse-delay-ms')??2000);
if(!Number.isFinite(reuseDelayMs)||reuseDelayMs<0||reuseDelayMs>30000)throw Error('Invalid reopen delay');
const python=new URL('scripts/x11-fixture.py',root).pathname;
function processExists(pid){process.kill(pid,0);}
const delay=ms=>new Promise(r=>setTimeout(r,ms));
let resident,fixture;let log='';const memory=[];let freshnessVerified=false;
async function footprint(label){const tree=[{pid:resident.pid},...await descendants(resident.pid)];let rssKiB=0,pssKiB=0;const pids=[],unreadable=[];for(const row of tree){try{const text=await fs.readFile('/proc/'+row.pid+'/smaps_rollup','utf8');rssKiB+=Number(text.match(/^Rss:\s+(\d+)/m)[1]);pssKiB+=Number(text.match(/^Pss:\s+(\d+)/m)[1]);pids.push(row.pid);}catch(e){if(e.code==='EACCES'||e.code==='EPERM')unreadable.push(row.pid);else if(e.code!=='ENOENT'&&e.code!=='ESRCH')throw e;}}memory.push({label,pids,unreadable,complete:unreadable.length===0,rssKiB,pssKiB});}
async function until(check,message,ms=15000){const deadline=Date.now()+ms;while(!(await check())){if(Date.now()>deadline)throw Error(message+'\n'+log);await delay(25);}}
async function descendants(pid){const rows=(await run('ps',['-eo','pid=,ppid=,comm='])).stdout.trim().split('\n').map(line=>{const [id,parent,...name]=line.trim().split(/\s+/);return {pid:Number(id),parent:Number(parent),name:name.join(' ')};});const found=new Set([pid]);let changed=true;while(changed){changed=false;for(const row of rows)if(found.has(row.parent)&&!found.has(row.pid)){found.add(row.pid);changed=true;}}return rows.filter(row=>row.pid!==pid&&found.has(row.pid));}
async function awaitPanelFocus(pid){
 const tree=(await run('xwininfo',['-root','-tree'])).stdout;const line=tree.split('\n').find(line=>line.includes('"Asaaan Launcher"'));if(!line)throw Error('Native panel window missing');
 const id=line.trim().split(' ')[0];const owner=(await run('xprop',['-id',id,'_NET_WM_PID'])).stdout;if(!owner.includes('= '+pid))throw Error('Native focus target belongs to another process: '+owner);
 await until(async()=>JSON.parse((await run('python3',[python,'ancestors'])).stdout).includes(Number(id)),'Desktop did not activate the launcher');
}
async function searchState(){return JSON.parse((await run('/usr/bin/python3',[new URL('scripts/x11-search-state.py',root).pathname])).stdout);}
async function key(...keys){await run('python3',[python,'keys',...keys]);}
async function nativeRows(pid){return JSON.parse((await run('/usr/bin/python3',[new URL('scripts/launch-native-state.py',root).pathname,String(pid)])).stdout);}
const catalogProbe=path.join(process.env.XDG_DATA_HOME,'applications','asaaan-reuse-probe.desktop');
async function writeProbe(name){await fs.mkdir(path.dirname(catalogProbe),{recursive:true});await fs.writeFile(catalogProbe,'[Desktop Entry]\nType=Application\nName='+name+'\nExec=/usr/bin/true\nTerminal=false\n');}
const installation=JSON.parse(await fs.readFile(path.resolve(path.dirname(binary),'../../installation.json'),'utf8'));
if(createHash('sha256').update(await fs.readFile(binary)).digest('hex')!==installation.binarySha256)throw Error('Installed binary differs from its manifest');
try{
 await writeProbe('Rfp Original Probe');
 fixture=spawn('python3',[python,'fixture'],{stdio:['ignore','pipe','inherit']});let fixtureId='';fixture.stdout.on('data',d=>fixtureId+=d);await until(()=>fixtureId.trim(),'Focus fixture did not start');
 resident=spawn(binary,['--background'],{stdio:['ignore','pipe','pipe']});resident.stderr.on('data',d=>log+=d);await until(()=>log.includes('ASAAAN_SHORTCUT'),'Shortcut setup missing');if(!log.includes('ASAAAN_SHORTCUT Ctrl+Alt+Space'))throw Error('Shortcut could not register');
 await delay(300);if(log.includes('Started('))throw Error('Background resident allocated a panel');
 const cycles=[];
 for(let cycle=1;cycle<=count;cycle++){
  const start=performance.now();const offset=log.length;await key('Control_L','Alt_L','space');
  await until(()=>log.slice(offset).includes('visible: true'),'Global shortcut did not show panel');
  const displayed=performance.now();let pid=Number(log.slice(offset).match(/Started\((\d+)\)/)[1]);await awaitPanelFocus(pid);const opened=performance.now();const coldPid=pid;const uiProcesses=await descendants(pid);
  await until(async()=>{const state=JSON.parse((await run('/usr/bin/python3',[new URL('scripts/actions-native-state.py',root).pathname,'state',String(pid)])).stdout);return state.query===''&&state.selected.length>0;},'Initial application results were not rendered');
  const resultsObservedMs=performance.now()-start;
  if(cycle===1){const duplicate=await run(binary,[],{timeout:5000});if(duplicate.stderr.includes('Started('))throw Error('Second resident spawned another panel');}
  await awaitPanelFocus(pid);
  if(cycle===1&&process.argv.includes('--screenshot')){const destination=process.argv[process.argv.indexOf('--screenshot')+1];await delay(500);const tree=(await run('xwininfo',['-root','-tree'])).stdout;const id=tree.split('\n').find(line=>line.includes('"Asaaan Launcher"')).trim().split(' ')[0];await run('import',['-window','root',destination]);}
  await delay(300);
  await key('z');await until(async()=>(await searchState()).some(field=>field.text==='z'),'Cold search field did not accept input');
  await key('Escape');await until(async()=>(await searchState()).some(field=>field.text===''),'Escape did not clear input');
  const hideOffset=log.length;await key('Control_L','Alt_L','space');await until(()=>log.slice(hideOffset).includes('visible: false'),'Warm hide did not complete');await delay(reuseDelayMs);if(cycle===1)await footprint('hidden-before-reopen');
  const warmStart=performance.now();const warmOffset=log.length;await key('Control_L','Alt_L','space');await until(()=>log.slice(warmOffset).includes('visible: true'),'Warm reopen did not complete');const restarted=log.slice(warmOffset).match(/Started\((\d+)\)/);const reused=!restarted;if(restarted){pid=Number(restarted[1]);uiProcesses.push(...await descendants(pid));}await awaitPanelFocus(pid);const reopenFocusMs=performance.now()-warmStart;
  if(process.argv.includes('--expect-reuse')&&!reused)throw Error('Reopen allocated a fresh panel inside the reuse window');
  if(cycle===1&&process.argv.includes('--expect-reuse')){
   await run('python3',[python,'text','rfp']);
   await until(async()=>{const state=await nativeRows(pid);return state.query==='rfp'&&state.rows.some(row=>row.includes('Rfp Original Probe'));},'Probe query did not render');
   const hiddenOffset=log.length;await key('Control_L','Alt_L','space');await until(()=>log.slice(hiddenOffset).includes('visible: false'),'Probe hide did not complete');
   await writeProbe('Rfp Updated Probe');await delay(2000);
   const probeOffset=log.length;await key('Control_L','Alt_L','space');await until(()=>log.slice(probeOffset).includes('visible: true'),'Probe reopen did not complete');
   if(log.slice(probeOffset).includes('Started('))throw Error('Freshness check respawned the panel');
   await awaitPanelFocus(pid);await until(async()=>{const state=await nativeRows(pid);return state.query==='rfp'&&state.rows.some(row=>row.includes('Rfp Updated Probe'))&&!state.rows.some(row=>row.includes('Rfp Original Probe'));},'Reused panel did not refresh its native catalog',1500);
   await key('Escape');await until(async()=>(await searchState()).some(field=>field.text===''),'Probe query did not clear');freshnessVerified=true;
  }
  await delay(100);
  const dismissalStart=performance.now();const dismissOffset=log.length;await key('Escape');await until(()=>log.slice(dismissOffset).includes('visible: false'),'Escape did not hide panel');await until(()=>log.slice(offset).includes('Exited('+pid+')'),'Hidden panel was not reaped');const reapedAfterHideMs=performance.now()-dismissalStart;
  for(const process of uiProcesses){await until(()=>{try{processExists(process.pid);return false;}catch(e){if(e.code==='ESRCH')return true;throw e;}},'Renderer/helper survived panel dismissal: '+process.pid);}
  if((await descendants(resident.pid)).length)throw Error('Idle resident retains child processes');
  if(cycle===1)await footprint('disposed');
  const focus=(await run('python3',[python,'focus'])).stdout.trim();
  cycles.push({cycle,coldPid,pid,reused,reuseDelayMs,reapedAfterHideMs,uiProcesses,allUiProcessesExited:true,displayReadyMs:displayed-start,coldPresentationMs:opened-start,resultsObservedMs,reopenFocusMs,focusRestored:focus===fixtureId.trim()});
 }
 const percentile=(key,p)=>{const sorted=cycles.map(c=>c[key]).sort((a,b)=>a-b);return sorted[Math.ceil(sorted.length*p)-1];};

 const evidence={binary,installation,freshnessVerified,memory,memoryMetric:"Sum of readable process PSS for resident and descendants (check complete/unreadable); summed RSS double-counts shared pages. Linux metrics are not macOS physical footprint.",environment:{kernel:os.release(),cpu:os.cpus()[0]?.model,node:process.version,softwareGL:process.env.LIBGL_ALWAYS_SOFTWARE,eglVendor:process.env.__EGL_VENDOR_LIBRARY_FILENAMES,glxVendor:process.env.__GLX_VENDOR_LIBRARY_NAME,compositingDisabled:process.env.WEBKIT_DISABLE_COMPOSITING_MODE,trace:process.env.ASAAAN_TRACE_STARTUP==='1'},metric:'Milliseconds from shortcut test-adapter invocation to DOM presentation / verified OS focus. Includes adapter and observation overhead. Each cold sample starts after all panel descendants exit; filesystem caches are not flushed.',summary:{samples:count,displayMedianMs:percentile('displayReadyMs',.5),displayP95Ms:percentile('displayReadyMs',.95),focusMedianMs:percentile('coldPresentationMs',.5),focusP95Ms:percentile('coldPresentationMs',.95),reopenFocusMedianMs:percentile('reopenFocusMs',.5),reopenFocusP95Ms:percentile('reopenFocusMs',.95),reusedSamples:cycles.filter(c=>c.reused).length,reuseDelayMs,resultsObservedMedianMs:percentile('resultsObservedMs',.5),resultsObservedP95Ms:percentile('resultsObservedMs',.95)},checkpoint:pkg.checkpoint,at:new Date().toISOString(),globalShortcut:'Ctrl+Alt+Space',cycles,log};
 await fs.mkdir(new URL('.artifacts/',root),{recursive:true});await fs.writeFile(output,JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify(evidence,null,2));
 if(cycles.some(c=>!c.focusRestored))throw Error('Previous-window focus restoration failed');
 await run(binary,['--quit'],{timeout:5000});await until(()=>resident.exitCode!==null,'Quit did not stop resident');
}catch(error){
 const tree=(await run('xwininfo',['-root','-tree'])).stdout;const line=tree.split('\n').find(line=>line.includes('"Asaaan Launcher"'));
 console.error('Native window:',line,'focus:',(await run('python3',[python,'focus'])).stdout.trim(),'keys:',(await run('python3',[python,'state'])).stdout.trim());
 if(line){const id=line.trim().split(' ')[0];await run('import',['-window','root',new URL('.artifacts/resident-failure.png',root).pathname]);}
 await fs.writeFile(new URL('.artifacts/resident-x11-failure.log',root),log);
 throw error;
}finally{if(resident&&resident.exitCode===null)resident.kill();fixture?.kill();}

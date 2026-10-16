// Explicit X11 desktop check. Uses a dedicated focus fixture and scoped key events.
import fs from 'node:fs/promises';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
const run=promisify(execFile);
const root=new URL('../',import.meta.url);
const binary=new URL('deployments/linux/current/usr/bin/asaaan-launcher',root).pathname;
const python=new URL('scripts/x11-fixture.py',root).pathname;
function processExists(pid){process.kill(pid,0);}
const delay=ms=>new Promise(r=>setTimeout(r,ms));
let resident,fixture;let log='';const samples=[];
async function until(check,message,ms=15000){const deadline=Date.now()+ms;while(!(await check())){if(Date.now()>deadline)throw Error(message+'\n'+log);await delay(25);}}
async function descendants(pid){const rows=(await run('ps',['-eo','pid=,ppid=,comm='])).stdout.trim().split('\n').map(line=>{const [id,parent,...name]=line.trim().split(/\s+/);return {pid:Number(id),parent:Number(parent),name:name.join(' ')};});const found=new Set([pid]);let changed=true;while(changed){changed=false;for(const row of rows)if(found.has(row.parent)&&!found.has(row.pid)){found.add(row.pid);changed=true;}}return rows.filter(row=>row.pid!==pid&&found.has(row.pid));}
async function awaitPanelFocus(pid){
 const tree=(await run('xwininfo',['-root','-tree'])).stdout;const line=tree.split('\n').find(line=>line.includes('"Asaaan Launcher"'));if(!line)throw Error('Native panel window missing');
 const id=line.trim().split(' ')[0];const owner=(await run('xprop',['-id',id,'_NET_WM_PID'])).stdout;if(!owner.includes('= '+pid))throw Error('Native focus target belongs to another process: '+owner);
 await until(async()=>JSON.parse((await run('python3',[python,'ancestors'])).stdout).includes(Number(id)),'Desktop did not activate the launcher');
}
async function key(...keys){await run('python3',[python,'keys',...keys]);}
async function memory(label){const s=await fs.readFile('/proc/'+resident.pid+'/smaps_rollup','utf8');const pick=k=>Number(s.match(new RegExp('^'+k+':\\s+(\\d+)','m'))[1]);samples.push({label,rssKiB:pick('Rss'),pssKiB:pick('Pss')});}
try{
 fixture=spawn('python3',[python,'fixture'],{stdio:['ignore','pipe','inherit']});let fixtureId='';fixture.stdout.on('data',d=>fixtureId+=d);await until(()=>fixtureId.trim(),'Focus fixture did not start');
 resident=spawn(binary,['--background'],{stdio:['ignore','pipe','pipe']});resident.stderr.on('data',d=>log+=d);await until(()=>log.includes('ASAAAN_SHORTCUT'),'Shortcut setup missing');if(!log.includes('ASAAAN_SHORTCUT Ctrl+Alt+Space'))throw Error('Shortcut could not register');
 await delay(300);await memory('cold');if(log.includes('Started('))throw Error('Background resident allocated a panel');
 const cycles=[];
 for(let cycle=1;cycle<=3;cycle++){
  const start=Date.now();const offset=log.length;await key('Control_L','Alt_L','space');
  await until(()=>log.slice(offset).includes('visible: true'),'Global shortcut did not show panel');
  const displayed=Date.now();const pid=Number(log.slice(offset).match(/Started\((\d+)\)/)[1]);await awaitPanelFocus(pid);const opened=Date.now();const uiProcesses=await descendants(pid);
  if(cycle===1){const duplicate=await run(binary,[],{timeout:5000});if(duplicate.stderr.includes('Started('))throw Error('Second resident spawned another panel');}
  await awaitPanelFocus(pid);
  if(cycle===2){const toggleOffset=log.length;await key('Control_L','Alt_L','space');await until(()=>log.slice(toggleOffset).includes('visible: false'),'Shortcut did not toggle hidden');const reopenOffset=log.length;await key('Control_L','Alt_L','space');await until(()=>log.slice(reopenOffset).includes('visible: true'),'Shortcut did not reopen during grace');if(log.slice(toggleOffset).includes('Started('))throw Error('Grace-period reopen allocated another panel');await awaitPanelFocus(pid);}
  const dismissOffset=log.length;await key('Escape');await until(()=>log.slice(dismissOffset).includes('visible: false'),'Escape did not hide panel');await until(()=>log.slice(offset).includes('Exited('+pid+')'),'Hidden panel was not reaped');
  for(const process of uiProcesses){await until(()=>{try{processExists(process.pid);return false;}catch(e){if(e.code==='ESRCH')return true;throw e;}},'Renderer/helper survived panel dismissal: '+process.pid);}
  if((await descendants(resident.pid)).length)throw Error('Idle resident retains child processes');
  const focus=(await run('python3',[python,'focus'])).stdout.trim();
  cycles.push({cycle,pid,uiProcesses,allUiProcessesExited:true,displayReadyMs:displayed-start,coldPresentationMs:opened-start,focusRestored:focus===fixtureId.trim()});await memory('dismissed-'+cycle);
 }
 for(let i=0;i<12;i++){await delay(5000);await memory('settled-'+(i+1));}
 const evidence={checkpoint:'CP003',at:new Date().toISOString(),globalShortcut:'Ctrl+Alt+Space',cycles,samples,targetMiB:30,targetMet:samples.every(s=>s.pssKiB<=30*1024),metric:'Linux PSS; RSS recorded separately, not equivalent to macOS physical footprint',log};
 await fs.mkdir(new URL('.artifacts/',root),{recursive:true});await fs.writeFile(new URL('.artifacts/resident-x11.json',root),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify(evidence,null,2));
 if(cycles.some(c=>!c.focusRestored))throw Error('Previous-window focus restoration failed');
 await run(binary,['--quit'],{timeout:5000});await until(()=>resident.exitCode!==null,'Quit did not stop resident');
}catch(error){
 const tree=(await run('xwininfo',['-root','-tree'])).stdout;const line=tree.split('\n').find(line=>line.includes('"Asaaan Launcher"'));
 console.error('Native window:',line,'focus:',(await run('python3',[python,'focus'])).stdout.trim(),'keys:',(await run('python3',[python,'state'])).stdout.trim());
 if(line){const id=line.trim().split(' ')[0];await run('import',['-window',id,new URL('.artifacts/resident-failure.png',root).pathname]);}
 throw error;
}finally{if(resident&&resident.exitCode===null)resident.kill();fixture?.kill();}

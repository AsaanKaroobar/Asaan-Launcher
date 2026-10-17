// Installed normal UI, verified owned multi-window app, no production test bypass.
import fs from 'node:fs/promises';import path from 'node:path';import {spawn,execFile} from 'node:child_process';import {promisify} from 'node:util';import {createHash} from 'node:crypto';
const run=promisify(execFile),root=new URL('../',import.meta.url),binary=new URL('deployments/linux/current/usr/bin/asaaan-launcher',root).pathname,python=new URL('scripts/x11-fixture.py',root).pathname,observer=new URL('scripts/actions-native-state.py',root).pathname;
if(!process.env.XDG_RUNTIME_DIR?.includes('asaaan-x11-'))throw Error('Use the isolated X11 harness');
const folder=path.join(process.env.XDG_CACHE_HOME,'process probe 日本語 ; literal'),data=path.join(folder,'data'),system=path.join(folder,'system'),desktop=path.join(data,'applications/org.asaaan.ProcessProbe.desktop'),storage=path.join(data,'com.asaaan.launcher/personalization.json'),mode=path.join(folder,'mode'),calls=path.join(folder,'calls.jsonl'),id='app:desktop:org.asaaan.ProcessProbe.desktop';
const env={...process.env,XDG_DATA_HOME:data,XDG_DATA_DIRS:system};
const delay=ms=>new Promise(r=>setTimeout(r,ms)),stages=[];let resident,fixture,windows=[],log='';
async function until(check,message,ms=15000){const end=Date.now()+ms;while(!(await check())){if(Date.now()>end)throw Error(message+'\n'+log);await delay(40);}}
async function state(){return JSON.parse((await run('/usr/bin/python3',[observer,'ui',String(panelPid())],{env,timeout:4000,maxBuffer:1024*1024})).stdout)}
async function key(...keys){await run('python3',[python,'keys',...keys],{env})}
async function prop(w,name,type,value){await run('xprop',['-id',String(w),'-f',name,type,'-set',name,value],{env})}
async function setID(value){for(const w of windows)await prop(w,'_GTK_APPLICATION_ID','8u',value)}
async function menu(){await key('Control_L','k');await until(async()=>(await state()).menu.includes('Open'),'Menu missing');await until(async()=>(await state()).text.some(t=>t.includes('Hide minimizes')||t.includes('identity')||t.includes('executable')||t.includes('remote')||t.includes('changed')),'Identity check not completed');return (await state()).menu}
async function dismiss(){await key('Escape')}
async function select(label){const snap=await state();const index=snap.menu.indexOf(label);if(index<0)throw Error('Missing native action '+label);await key('Home');for(let i=0;i<index;i++)await key('Down');await key('Return')}
async function reopen(){await run(binary,[],{env,timeout:5000});await until(async()=>(await state()).query!==null,'Panel missing');await delay(350)}
async function query(){await key('Control_L','a');await key('BackSpace');await run('python3',[python,'text','process'],{env});await until(async()=>(await state()).query==='process','Query missing');await delay(250)}
async function requests(){try{return(await fs.readFile(calls,'utf8')).trim().split('\n').filter(Boolean).map(JSON.parse)}catch(e){if(e.code==='ENOENT')return[];throw e}}
async function startFixture(){
 let ready='';fixture=spawn('/usr/bin/python3',[new URL('scripts/process-fixture.py',root).pathname,calls,mode],{env,stdio:['ignore','pipe','pipe']});
 fixture.stdout.on('data',d=>ready+=d);fixture.stderr.on('data',d=>log+=d);
 await until(()=>ready.includes(']'),'App fixture not ready');windows=JSON.parse(ready.trim());tracked.add(fixture.pid);livePid=fixture.pid;await delay(350);
}
async function assertNoProcess(reason){const names=await menu();if(names.some(n=>['Quit','Hide','Restart','Force Quit…'].includes(n)))throw Error(reason+' unexpectedly offered controls');await dismiss()}
async function stop(child){if(child&&child.exitCode===null&&child.signalCode===null){child.kill();await until(()=>child.exitCode!==null||child.signalCode!==null,'Fixture teardown stalled');}child?.stdout?.destroy();child?.stderr?.destroy()}
function record(stage){stages.push(stage);console.log('Passed: '+stage)}
const tracked=new Set();let livePid=null;
async function alive(pid){try{const text=await fs.readFile('/proc/'+pid+'/stat','utf8');return text.slice(text.lastIndexOf(')')+2).split(' ')[0]!=='Z'}catch{return false}}
async function starts(){try{return(await fs.readFile(calls+'.starts','utf8')).trim().split('\n').filter(Boolean).map(JSON.parse)}catch(e){if(e.code==='ENOENT')return[];throw e}}
function panelPid(){return Number([...log.matchAll(/Started\((\d+)\)/g)].at(-1)?.[1])}
const nativeObserver=new URL('scripts/settings-native-state.py',root).pathname;
async function dialogState(pid){return JSON.parse((await run('/usr/bin/python3',[nativeObserver,'state',String(pid)],{env,timeout:5000,maxBuffer:2e6})).stdout)}
async function dialogClick(pid,label){await run('/usr/bin/python3',[nativeObserver,'click-exact',String(pid),label],{env,timeout:5000,maxBuffer:2e6})}
async function fresh(value){await fs.writeFile(mode,value);await startFixture();livePid=fixture.pid;tracked.add(livePid);await setID('org.asaaan.ProcessProbe');await reopen();await query()}
async function stopCurrent(){await fs.writeFile(mode,'exit');if(livePid)await until(async()=>!await alive(livePid),'Owned fixture did not exit');await stop(fixture)}
try{
 const installation=JSON.parse(await fs.readFile(new URL('deployments/linux/current/installation.json',root),'utf8'));
 if(installation.checkpoint!=='CP013'||createHash('sha256').update(await fs.readFile(binary)).digest('hex')!==installation.binarySha256)throw Error('Install CP013 first');
 await fs.mkdir(path.dirname(desktop),{recursive:true});await fs.mkdir(system,{recursive:true});await fs.symlink('/usr/share/mime',path.join(system,'mime'));await fs.symlink('/usr/share/icons',path.join(system,'icons'));await fs.mkdir(path.dirname(storage),{recursive:true});
 const probe=new URL('scripts/process-fixture.py',root).pathname;
 const entry='[Desktop Entry]\nType=Application\nName=Process Probe\nExec=/usr/bin/python3 "'+probe+'" "'+calls+'" "'+mode+'" org.asaaan.ProcessProbe\n';
 await fs.writeFile(desktop,entry);await fs.writeFile(mode,'refuse');
 await fs.writeFile(storage,JSON.stringify({version:2,favorites:[id],records:{[id]:{count:4,lastUsed:123}},queries:['keep'],rememberSearches:false,aliases:{},hiddenResults:{}}));const before=await fs.readFile(storage);
 resident=spawn(binary,[],{env,stdio:['ignore','ignore','pipe']});resident.stderr.on('data',d=>log+=d);await until(()=>log.includes('visible: true'),'Launcher missing');await delay(350);
 await fresh('refuse');await menu();const original=fixture.pid;await select('Force Quit…');const panel=panelPid();
 await until(async()=>(await dialogState(panel)).some(n=>n.name.includes("You will lose any changes you haven't saved.")),'Native unsaved-work warning missing');
 if(!log.includes('visible: false'))throw Error('Force Quit did not dismiss launcher before confirmation');
 await delay(1000);if(!await alive(panel))throw Error('Confirmation lost its foreground work lease');
 await key('Return');await until(async()=>!(await dialogState(panel)).some(n=>n.name.includes("You will lose")),'Default Cancel did not close native confirmation');
 if(!await alive(original)||(await requests()).length)throw Error('Default Cancel affected app');await reopen();await query();record('native confirmation warns about unsaved work; default Enter cancels; pending dialog retains worker');
 await menu();await select('Force Quit…');const escapePanel=panelPid();await until(async()=>(await dialogState(escapePanel)).some(n=>n.name.includes("You will lose")),'Second native confirmation missing');await key('Escape');
 await until(async()=>!(await dialogState(escapePanel)).some(n=>n.name.includes("You will lose")),'Escape did not cancel confirmation');
 if(!await alive(original))throw Error('Escape cancellation terminated app');record('Escape cancels without termination');
 await stopCurrent();await fresh('hang');const hung=fixture.pid;const closeCount=(await requests()).length;await menu();await select('Force Quit…');const forcePanel=panelPid();
 await until(async()=>(await dialogState(forcePanel)).some(n=>n.name.includes("You will lose")),'Force confirmation missing');await dialogClick(forcePanel,'Force Quit');
 await until(()=>fixture.signalCode==='SIGKILL','Confirmed force did not terminate unresponsive owned app');if((await requests()).length!==closeCount)throw Error('Force sent graceful window close');
 if(await alive(hung))throw Error('Forced process remains alive');record('confirmed Force Quit terminates the hung verified process through its stable handle');
 await fresh('refuse');await menu();await select('Force Quit…');const stalePanel=panelPid();await until(async()=>(await dialogState(stalePanel)).some(n=>n.name.includes("You will lose")),'Stale confirmation missing');
 await stopCurrent();await fs.writeFile(mode,'refuse');await startFixture();await setID('org.asaaan.ProcessProbe');const replacement=fixture.pid;
 const approvalOffset=log.length;const replacementCalls=(await requests()).length;await dialogClick(stalePanel,'Force Quit');
 await until(()=>log.slice(approvalOffset).includes('accepted=true')&&log.slice(approvalOffset).includes('revalidating'),'Native approval did not reach revalidation');
 await until(async()=>!await alive(stalePanel),'Rejected hidden operation did not complete and dispose normally');
 if((await requests()).length!==replacementCalls)throw Error('Stale confirmation sent a close request to replacement');await reopen();await query();
 if(!await alive(replacement))throw Error('Confirmation affected replacement instance');record('confirmation revalidates instance after approval; replacement survives');
 await query();await menu();await stopCurrent();await fs.writeFile(mode,'refuse');await startFixture();await setID('org.asaaan.ProcessProbe');const staleRestart=fixture.pid;const staleCount=(await starts()).length;
 await select('Restart');await until(async()=>(await state()).text.some(s=>s.includes('instance changed')),'Restart did not reject stale menu');
 if(!await alive(staleRestart)||(await starts()).length!==staleCount)throw Error('Stale Restart affected replacement');record('stale Restart does not close or open a replacement instance');
 await stopCurrent();await fresh('refuse');const refused=fixture.pid;const startCount=(await starts()).length;
 await menu();await select('Restart');await until(async()=>(await state()).text.some(s=>s.includes('Restarting Process Probe')),'Restart progress missing');
 await until(async()=>(await state()).text.some(s=>s.includes('still running')&&s.includes('not restarted')),'Refused Restart deadline missing',16000);
 if(!await alive(refused)||(await starts()).length!==startCount||(await state()).query!=='process')throw Error('Refused Restart escalated, relaunched, or changed query');record('refused graceful restart times out without escalation or reopening and preserves query');
 await fs.writeFile(mode,'background');const backgroundCalls=(await requests()).length;await menu();await select('Restart');const busyPanel=panelPid();
 await until(async()=>(await requests()).length===backgroundCalls+2,'Background fixture did not close windows');await key('Escape');await key('Escape');await delay(1100);
 if(!await alive(busyPanel)||!await alive(refused))throw Error('Pending restart or background process was reaped');await reopen();
 await until(async()=>(await state()).text.some(s=>s.includes('still running')&&s.includes('not restarted')),'Background process was incorrectly treated as exited',16000);
 if((await starts()).length!==startCount)throw Error('Closed windows caused early reopen');record('closed windows with a live background process do not count as termination; hidden restart retains work lease');
 await stopCurrent();await fresh('delayed');const delayedPid=fixture.pid;const delayedCount=(await starts()).length;const delayedCalls=(await requests()).length;
 await menu();await select('Restart');await until(async()=>(await requests()).length===delayedCalls+2,'Delayed fixture did not receive graceful requests');await delay(500);
 if(!await alive(delayedPid)||(await starts()).length!==delayedCount)throw Error('Restart reopened before actual process exit');
 await until(async()=>(await starts()).length===delayedCount+1,'Restart did not reopen after exit');const opened=(await starts()).at(-1);tracked.add(opened.pid);livePid=opened.pid;
 if(opened.pid===delayedPid||await alive(delayedPid)||!await alive(opened.pid))throw Error('Restart did not replace exited process exactly once');
 await reopen();await query();let offered=await menu();if(!offered.includes('Restart')||!offered.includes('Force Quit…'))throw Error('Reopened app did not retain verified identity');await dismiss();
 stages.push('delayed process exit is awaited before exactly one native GIO reopen; reopened instance is identifiable');
 await stopCurrent();await fresh('delayed');const changedCount=(await starts()).length;const changedCalls=(await requests()).length;await menu();await select('Restart');
 await until(async()=>(await requests()).length===changedCalls+2,'Changed-entry fixture did not receive close');await fs.writeFile(desktop,entry.replace(/Exec=.*/,'Exec=/usr/bin/true'));
 await until(async()=>(await state()).text.some(s=>s.includes('desktop entry changed')),'Edited desktop entry did not stop restart');if((await starts()).length!==changedCount)throw Error('Changed desktop entry was reopened');await fs.writeFile(desktop,entry);
 stages.push('desktop entry changes while waiting prevent reopening');
 if(!(await fs.readFile(storage)).equals(before))throw Error('Restart/Force Quit changed preferences or usage');record('restart and force paths preserve exact preference/history bytes');
 await fresh('refuse');const pendingTarget=fixture.pid;await menu();await select('Force Quit…');const pendingPanel=panelPid();
 await until(async()=>(await dialogState(pendingPanel)).some(n=>n.name.includes("You will lose")),'Pending confirmation missing before resident quit');
 await run(binary,['--quit'],{env,timeout:5000});await until(()=>resident.exitCode===0,'Resident did not quit');await until(async()=>!await alive(pendingPanel),'Pending confirmation worker outlived resident');
 if(!await alive(pendingTarget))throw Error('Resident quit terminated an unapproved target');record('resident quit closes a pending confirmation without terminating its unapproved target');
 await fs.mkdir(new URL('.artifacts/',root),{recursive:true});await fs.writeFile(new URL('.artifacts/process-controls-smoke.json',root),JSON.stringify({checkpoint:'CP013',binarySha256:installation.binarySha256,stages},null,2)+'\n');console.log('CP013 Restart/Force Quit passed '+stages.length+' stages: '+stages.join('; '));
}catch(error){await fs.mkdir(new URL('.artifacts/',root),{recursive:true});await fs.writeFile(new URL('.artifacts/process-controls-failure.log',root),String(error)+'\n'+log);throw error}
finally{
 await fs.writeFile(mode,'exit').catch(()=>{});await delay(200);
 for(const pid of tracked){if(!await alive(pid))continue;try{const vars=(await fs.readFile('/proc/'+pid+'/environ','utf8')).split('\0');const args=await fs.readFile('/proc/'+pid+'/cmdline','utf8');if(vars.includes('XDG_RUNTIME_DIR='+process.env.XDG_RUNTIME_DIR)&&args.includes(calls)&&args.includes('process-fixture.py'))process.kill(pid,'SIGTERM')}catch{}}
 await stop(fixture);if(resident&&resident.exitCode===null&&resident.signalCode===null){try{await run(binary,['--quit'],{env,timeout:5000})}catch{}}await stop(resident);
}

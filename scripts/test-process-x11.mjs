// Installed normal UI, verified owned multi-window app, no production test bypass.
import fs from 'node:fs/promises';import path from 'node:path';import {spawn,execFile} from 'node:child_process';import {promisify} from 'node:util';import {createHash} from 'node:crypto';
const run=promisify(execFile),root=new URL('../',import.meta.url),binary=new URL('deployments/linux/current/usr/bin/asaaan-launcher',root).pathname,python=new URL('scripts/x11-fixture.py',root).pathname,observer=new URL('scripts/actions-native-state.py',root).pathname;
if(!process.env.XDG_RUNTIME_DIR?.includes('asaaan-x11-'))throw Error('Use the isolated X11 harness');
const folder=path.join(process.env.XDG_CACHE_HOME,'process probe 日本語 ; literal'),data=path.join(folder,'data'),system=path.join(folder,'system'),desktop=path.join(data,'applications/org.asaaan.ProcessProbe.desktop'),storage=path.join(data,'com.asaaan.launcher/personalization.json'),mode=path.join(folder,'mode'),calls=path.join(folder,'calls.jsonl'),id='app:desktop:org.asaaan.ProcessProbe.desktop';
const env={...process.env,XDG_DATA_HOME:data,XDG_DATA_DIRS:system};
const delay=ms=>new Promise(r=>setTimeout(r,ms)),stages=[];let resident,fixture,windows=[],log='';
async function until(check,message,ms=15000){const end=Date.now()+ms;while(!(await check())){if(Date.now()>end)throw Error(message+'\n'+log);await delay(40);}}
async function state(){return JSON.parse((await run('/usr/bin/python3',[observer,'ui'],{env,timeout:4000,maxBuffer:1024*1024})).stdout)}
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
 await until(()=>ready.includes(']'),'App fixture not ready');windows=JSON.parse(ready.trim());await delay(350);
}
async function assertNoProcess(reason){const names=await menu();if(names.some(n=>['Quit','Hide','Restart','Force Quit…'].includes(n)))throw Error(reason+' unexpectedly offered controls');await dismiss()}
async function stop(child){if(child&&child.exitCode===null&&child.signalCode===null){child.kill();await until(()=>child.exitCode!==null||child.signalCode!==null,'Fixture teardown stalled');}child?.stdout?.destroy();child?.stderr?.destroy()}
try{
 const installation=JSON.parse(await fs.readFile(new URL('deployments/linux/current/installation.json',root),'utf8'));
 if(installation.checkpoint!=='CP014'||createHash('sha256').update(await fs.readFile(binary)).digest('hex')!==installation.binarySha256)throw Error('Install CP014 first');
 await fs.mkdir(path.dirname(desktop),{recursive:true});await fs.mkdir(system,{recursive:true});await fs.mkdir(path.dirname(storage),{recursive:true});
 const entry='[Desktop Entry]\nType=Application\nName=Process Probe\nExec=/usr/bin/python3\n';
 await fs.writeFile(desktop,entry);await fs.writeFile(mode,'refuse');
 await fs.writeFile(storage,JSON.stringify({version:2,favorites:[id],records:{[id]:{count:4,lastUsed:123}},queries:['keep'],rememberSearches:false,aliases:{},hiddenResults:{}}));
 const before=await fs.readFile(storage);
 resident=spawn(binary,[],{env,stdio:['ignore','ignore','pipe']});resident.stderr.on('data',d=>log+=d);await until(()=>log.includes('visible: true'),'Launcher missing');await delay(350);await query();
 await assertNoProcess('Stopped app');stages.push('stopped app offers no running controls');
 await startFixture();await reopen();await query();await assertNoProcess('Same title and WM_CLASS without desktop ID');stages.push('same name and WM_CLASS cannot select a running app');
 await setID('org.asaaan.ProcessProbe');
 for(const w of windows)await prop(w,'WM_CLIENT_MACHINE','8s','remote.invalid');
 await assertNoProcess('Remote identity');stages.push('remote window identity rejected');
 const hostname=(await fs.readFile('/proc/sys/kernel/hostname','utf8')).trim();
 for(const w of windows)await prop(w,'WM_CLIENT_MACHINE','8s',hostname);
 for(const w of windows)await prop(w,'_KDE_NET_WM_DESKTOP_FILE','8u','org.other.App');
 await assertNoProcess('Conflicting desktop IDs');for(const w of windows)await run('xprop',['-id',String(w),'-remove','_KDE_NET_WM_DESKTOP_FILE'],{env});stages.push('conflicting native identities rejected');
 await fs.writeFile(desktop,entry.replace('Exec=/usr/bin/python3','Exec=/usr/bin/true'));
 await assertNoProcess('Executable mismatch');await fs.writeFile(desktop,entry);stages.push('desktop executable mismatch rejected before delegation');
 for(const w of windows)await run('xprop',['-id',String(w),'-remove','WM_PROTOCOLS'],{env});
 let unsupported=await menu();if(unsupported.includes('Quit')||!unsupported.includes('Hide'))throw Error('Unsupported graceful protocol offered Quit');await dismiss();
 for(const w of windows)await prop(w,'WM_PROTOCOLS','32a','WM_DELETE_WINDOW');stages.push('missing graceful close protocol omits Quit without disabling Hide');
 let names=await menu();if(!names.includes('Hide')||!names.includes('Quit')||!names.includes('Force Quit…')||!names.includes('Restart'))throw Error('Incorrect verified process actions');
 await select('Hide');await until(()=>log.includes('visible: false'),'Hide did not dismiss launcher');
 await until(async()=>{for(const w of windows){const p=(await run('xprop',['-id',String(w),'_NET_WM_STATE'],{env})).stdout;if(!p.includes('_NET_WM_STATE_HIDDEN'))return false}return true},'Hide did not minimize every window');
 if(fixture.exitCode!==null||fixture.signalCode!==null)throw Error('Hide terminated fixture');stages.push('verified multi-window Hide minimizes both and dismisses launcher');
 await reopen();await query();names=await menu();if(names.includes('Hide')||!names.includes('Quit'))throw Error('Hidden app offered Hide or lost Quit');await select('Quit');
 await until(async()=>(await requests()).length===2,'Graceful requests did not reach both windows');
 await delay(700);if(fixture.exitCode!==null||fixture.signalCode!==null)throw Error('Refused Quit terminated app');
 if((await state()).query!=='process')throw Error('Quit changed query');await until(async()=>(await state()).text.some(s=>s.includes('Quitting Process Probe')),'Quit acknowledgement missing');
 stages.push('hidden app omits Hide; refused graceful Quit preserves process, windows and query');
 await fs.writeFile(mode,'accept');await menu();await select('Quit');await until(()=>fixture.exitCode===0,'Accepted Quit did not close fixture naturally');
 await assertNoProcess('Exited app');stages.push('accepted graceful Quit exits naturally; fresh menu removes process controls');
 await fs.writeFile(mode,'refuse');await startFixture();await setID('org.asaaan.ProcessProbe');await reopen();await query();names=await menu();if(!names.includes('Quit'))throw Error('Restarted fixture not detected');
 await stop(fixture);await select('Quit');await until(async()=>(await state()).text.some(s=>s.includes('no longer running')),'Stale menu action did not revalidate');
 stages.push('stale menu rechecks native identity and reports exited app');
 await startFixture();await setID('org.asaaan.ProcessProbe');await reopen();await query();await menu();await stop(fixture);await startFixture();await setID('org.asaaan.ProcessProbe');
 const requestCount=(await requests()).length;await select('Quit');await until(async()=>(await state()).text.some(s=>s.includes('instance changed')),'Replacement instance was not refused');
 if((await requests()).length!==requestCount||fixture.exitCode!==null||fixture.signalCode!==null)throw Error('Stale menu affected replacement app');stages.push('replacement instance is protected from an old menu token');
 if(!(await fs.readFile(storage)).equals(before))throw Error('Process controls changed preferences/history');stages.push('preferences, favorites and usage bytes unchanged');
 await run(binary,['--quit'],{env,timeout:5000});await until(()=>resident.exitCode===0,'Resident did not quit');
 await fs.mkdir(new URL('.artifacts/',root),{recursive:true});await fs.writeFile(new URL('.artifacts/process-smoke.json',root),JSON.stringify({checkpoint:'CP014',binarySha256:installation.binarySha256,stages},null,2)+'\n');console.log('CP014 process actions passed '+stages.length+' stages: '+stages.join('; '));
}catch(error){await fs.mkdir(new URL('.artifacts/',root),{recursive:true});await fs.writeFile(new URL('.artifacts/process-failure.log',root),String(error)+'\n'+log);throw error}
finally{await stop(fixture);if(resident&&resident.exitCode===null&&resident.signalCode===null){try{await run(binary,['--quit'],{env,timeout:5000})}catch{}}await stop(resident)}

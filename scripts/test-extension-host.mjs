// Tests the installed CP013 executable with its bundled runtime; no GUI/test-only IPC bypass.
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=path.resolve(import.meta.dirname,'..');
const pkg=JSON.parse(await fs.readFile(path.join(root,'package.json'),'utf8'));
const install=path.join(root,'deployments/linux/current');
const binary=path.join(install,'usr/bin/asaaan-launcher');
const manifest=JSON.parse(await fs.readFile(path.join(install,'installation.json'),'utf8'));
const hash=b=>createHash('sha256').update(b).digest('hex');
assert.equal(manifest.version,pkg.version);assert.equal(manifest.checkpoint,pkg.checkpoint);
assert.equal(hash(await fs.readFile(binary)),manifest.binarySha256);
const scratch=await fs.mkdtemp(path.join(os.tmpdir(),'asaaan-extension-π spaces-'));
const owned=[];const stages=[];
const env={...process.env,HOME:scratch,XDG_DATA_HOME:path.join(scratch,'data'),XDG_CONFIG_HOME:path.join(scratch,'config'),XDG_CACHE_HOME:path.join(scratch,'cache'),PATH:path.join(scratch,'bin')+':/usr/bin:/bin',DISPLAY:'',WAYLAND_DISPLAY:'',NODE_OPTIONS:'',BUN_OPTIONS:''};
await fs.mkdir(path.join(scratch,'bin'));
await fs.writeFile(path.join(scratch,'bin/bun'),'#!/bin/sh\ntouch "'+scratch+'/wrong-bun"\nexit 99\n',{mode:0o755});
const personal=path.join(scratch,'data/com.asaaan.launcher/personalization.json');
await fs.mkdir(path.dirname(personal),{recursive:true});await fs.writeFile(personal,'{"fixture":"unchanged"}\n');
function all(n,type,out=[]){if(n?.type===type)out.push(n);for(const c of n?.children??[])all(c,type,out);return out;}
function start(extension,command,exe=binary,args=['--extension-check',extension,command]){
 const child=spawn(exe,args,{env,cwd:scratch,stdio:['pipe','pipe','pipe']});
 const state={child,messages:[],stderr:'',closed:false,code:null,parseError:null};
 owned.push(state);let buffered='';
 child.stdout.on('data',b=>{buffered+=b;let at;while((at=buffered.indexOf('\n'))>=0){const line=buffered.slice(0,at);buffered=buffered.slice(at+1);try{state.messages.push(JSON.parse(line));}catch(e){state.parseError=e;}}});
 child.stderr.on('data',b=>state.stderr+=b);child.stdin.on('error',()=>{});
 child.on('error',e=>{state.parseError=e;state.closed=true;});
 child.on('close',code=>{state.closed=true;state.code=code;if(buffered)state.parseError=Error('Partial output at EOF');});
 state.send=v=>child.stdin.write(JSON.stringify(v)+'\n');return state;
}
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function until(state,predicate,timeout=10000){
 const end=Date.now()+timeout;
 do{if(state.parseError)throw state.parseError;const value=predicate(state);if(value)return value;await pause(20);}while(Date.now()<end);
 throw Error('Timed out '+JSON.stringify(state.messages).slice(-4000)+'\n'+state.stderr.slice(-2000));
}
const message=(s,type,p=()=>true,timeout)=>until(s,s=>s.messages.find(m=>m.type===type&&p(m)),timeout);
async function closed(s,code=0){await until(s,s=>s.closed,5000);assert.equal(s.code,code,s.stderr);assert.equal(s.parseError,null);}
async function absent(pid){
 if(!pid)return;
 for(let i=0;i<100;i++){
  try{const stat=await fs.readFile('/proc/'+pid+'/stat','utf8');if(stat.slice(stat.lastIndexOf(')')+2).startsWith('Z'))return;}
  catch(e){if(e.code==='ENOENT')return;throw e;}await pause(20);
 }throw Error('Owned fixture process survived: '+pid);
}
async function done(name){stages.push(name);console.log('PASS '+name);}
try{
 const runtime=path.join(install,'usr/lib/Asaaan Launcher/extension-runtime');
 const info=JSON.parse(await fs.readFile(path.join(runtime,'runtime.json'),'utf8'));
 assert.equal(info.bun,'1.4.3');assert.equal(info.sha256,hash(await fs.readFile(path.join(runtime,'bun'))));
 await done('bundled pinned Bun integrity');
 const s=start('hello','planets');const pid=(await message(s,'started')).pid;s.send({type:'ping'});await message(s,'pong');
 let tree=(await message(s,'render')).tree;
 assert.deepEqual(all(tree,'List.Item').map(n=>n.props.title),['Mercury','Venus','Earth','Mars','Jupiter','Saturn','Uranus','Neptune']);
 const action=all(tree,'Action').find(n=>n.props.title==='Add Favorite');assert(action?.handlers.includes('onAction'));
 const count=s.messages.filter(m=>m.type==='render').length;s.send({type:'event',id:action.id,prop:'onAction',args:[]});
 await message(s,'toast',m=>m.title==='Favorites updated');
 await until(s,s=>s.messages.filter(m=>m.type==='render').length>count);
 const beforeFull=s.messages.filter(m=>m.type==='render').length;
 s.send({type:'fullRender'});await until(s,s=>s.messages.filter(m=>m.type==='render').length>beforeFull);
 const beforePing=s.messages.filter(m=>m.type==='pong').length;s.send({type:'ping'});await until(s,s=>s.messages.filter(m=>m.type==='pong').length>beforePing);
 s.child.stdin.end();await closed(s);await absent(pid);
 await done('real planets render, event, reference update, full render, ping and EOF');
 const t=start('host-probe','transport');const tpid=(await message(t,'started')).pid;
 const details=JSON.parse((await message(t,'hud',m=>m.title?.startsWith('{"bun"'))).title);
 assert.equal(details.bun,'1.4.3');assert.equal(details.supportPath,path.join(scratch,'data/com.asaaan.launcher/extensions-data/host-probe'));
 const request=await message(t,'request');assert.equal(request.method,'clipboard.read');
 await message(t,'hud',m=>m.title.includes('unavailable in checkpoint 13'));
 await message(t,'ended',m=>m.success);await closed(t);await absent(tpid);
 assert(t.stderr.includes('fixture console log'));assert(t.stderr.includes('\x07fixture raw stdout'));
 await done('protocol stdout isolation and correlated unsupported request');
 const throws=start('diagnostics','throws');await message(throws,'error',m=>m.fatal===true);throws.child.stdin.end();await closed(throws);
 await absent((await message(throws,'started')).pid);await done('fatal extension error and cleanup');
 const rejects=start('diagnostics','rejects');await message(rejects,'error',m=>m.fatal===false);
 rejects.send({type:'ping'});await message(rejects,'pong');rejects.child.stdin.end();await closed(rejects);
 await absent((await message(rejects,'started')).pid);await done('rejected promise stays nonfatal and responsive');
 const exits=start('diagnostics','exits');await message(exits,'ended',m=>m.code===3&&!m.success);await closed(exits);await absent((await message(exits,'started')).pid);
 await done('abnormal process exit distinguished from normal completion');
 const hangs=start('diagnostics','hangs');const hpid=(await message(hangs,'started')).pid;await message(hangs,'render');
 await message(hangs,'unresponsive',()=>true,16000);
 const at=Date.now();hangs.child.stdin.end();await closed(hangs);assert(Date.now()-at<3500);await absent(hpid);
 await done('watchdog detects blocked event loop; bounded EOF force cleanup');
 for(const reason of ['eof','cancel']){
  const child=start('host-probe','child');const parent=(await message(child,'started')).pid;
  const descendant=JSON.parse((await message(child,'hud',m=>m.title?.startsWith('{"child"'))).title).child;
  // Allow the owned child to install its SIGTERM refusal, then exercise the host-group escalation.
  await pause(150);
  if(reason==='eof')child.child.stdin.end();else child.child.kill('SIGTERM');
  await closed(child);await absent(parent);await absent(descendant);
  await done('owned descendant cleanup on '+reason);
 }
 const flood=start('host-probe','flood');const end=await message(flood,'ended');await closed(flood);
 assert(end.logBytes<=20000);assert(flood.stderr.endsWith('end-of-bounded-log\n'));
 await done('stderr retention bounded without protocol contamination');
 const oversized=start('hello','planets');const opid=(await message(oversized,'started')).pid;
 oversized.child.stdin.write('x'.repeat(65539)+'\n');await closed(oversized,69);await absent(opid);
 await done('oversized input refuses and cleans host');
 for(const args of [['--extension-check','hello','planets','--search','x'],['--choose','--extension-check','hello','planets'],['--extension-check','../escape','planets']]){
  const bad=start('', '',binary,args);await closed(bad,64);assert.equal(bad.messages.length,0);
 }
 const unknown=start('hello','missing');await closed(unknown,69);assert.equal(unknown.messages.length,0);
 await done('exclusive modes, path validation and missing command errors');
 const badRoot=path.join(scratch,'broken');await fs.mkdir(path.join(badRoot,'usr/bin'),{recursive:true});
 const badBinary=path.join(badRoot,'usr/bin/asaaan-launcher');await fs.copyFile(binary,badBinary);await fs.chmod(badBinary,0o755);
 const missing=start('hello','planets',badBinary);await closed(missing,69);assert(missing.stderr.includes('runtime is missing'));
 await fs.mkdir(path.join(badRoot,'usr/lib/Asaaan Launcher/extension-runtime/runtime'),{recursive:true});
 await fs.writeFile(path.join(badRoot,'usr/lib/Asaaan Launcher/extension-runtime/runtime/host.ts'),'');
 await fs.mkdir(path.join(badRoot,'usr/lib/Asaaan Launcher/extension-runtime/extensions/hello'),{recursive:true});
 await fs.copyFile(path.join(runtime,'extensions/hello/package.json'),path.join(badRoot,'usr/lib/Asaaan Launcher/extension-runtime/extensions/hello/package.json'));
 const missingBun=start('hello','planets',badBinary);await closed(missingBun,69);assert(missingBun.stderr.includes("couldn't start"));
 await assert.rejects(fs.access(path.join(scratch,'wrong-bun')));
 await done('missing resources and Bun fail without developer PATH fallback');
 assert.equal(await fs.readFile(personal,'utf8'),'{"fixture":"unchanged"}\n');
 await done('personalization stays byte-identical');
 const evidence={checkpoint:pkg.checkpoint,version:pkg.version,result:'passed',binarySha256:manifest.binarySha256,packageSha256:manifest.packageSha256,bun:info,stages,platform:'Linux x64, installed headless native host; no native macOS/Windows qualification',at:new Date().toISOString()};
 await fs.mkdir(path.join(root,'.artifacts/checkpoint-013'),{recursive:true});
 await fs.writeFile(path.join(root,'.artifacts/checkpoint-013/extension-host.json'),JSON.stringify(evidence,null,2)+'\n');
}catch(e){await fs.mkdir(path.join(root,'.artifacts/checkpoint-013'),{recursive:true});await fs.writeFile(path.join(root,'.artifacts/checkpoint-013/extension-host-failure.json'),JSON.stringify(owned.map(s=>({pid:s.child.pid,code:s.code,messages:s.messages,stderr:s.stderr.slice(-8000)})),null,2));throw e;}
finally{
 for(const state of owned){if(!state.closed){state.child.kill('SIGTERM');await until(state,s=>s.closed,5000).catch(()=>state.child.kill('SIGKILL'));}}
 await fs.rm(scratch,{recursive:true,force:true});
}

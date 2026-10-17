// Installed standalone picker; only a private X11/DBus/XDG session is permitted.
import fs from 'node:fs/promises';import path from 'node:path';import {fileURLToPath} from 'node:url';import {spawn,execFile} from 'node:child_process';import {promisify} from 'node:util';import {createHash} from 'node:crypto';import assert from 'node:assert/strict';
const run=promisify(execFile),root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..'),binary=path.join(root,'deployments/linux/current/usr/bin/asaaan-launcher'),fixture=path.join(root,'scripts/x11-fixture.py');
if(!process.env.XDG_RUNTIME_DIR?.includes('asaaan-x11-'))throw Error('Use isolated harness');
const delay=ms=>new Promise(r=>setTimeout(r,ms));const stages=[];let active,focus,resident,residentLog='';
async function until(check,message,ms=15000){const end=Date.now()+ms;while(!(await check())){if(Date.now()>end)throw Error(message);await delay(50);}}
async function state(){return JSON.parse((await run('/usr/bin/python3',[path.join(root,'scripts/picker-native-state.py')],{timeout:5000,maxBuffer:1024*1024})).stdout);}
async function key(...keys){await run('python3',[fixture,'keys',...keys]);}
async function text(value){await run('python3',[fixture,'text',value]);}
function start(args,input,env=process.env){const child=spawn(binary,args,{env,stdio:['pipe','pipe','pipe']});const item={child,out:'',err:'',done:false,code:null};child.stdout.on('data',d=>item.out+=d);child.stderr.on('data',d=>item.err+=d);child.on('close',code=>{item.code=code;item.done=true;});child.stdin.end(input);return item;}
async function result(item,code,out){await until(()=>item.done,'Picker did not exit: '+item.err);assert.equal(item.code,code,item.err);assert.equal(item.out,out);assert.ok(!item.err.includes('ASAAAN_LIFECYCLE'),'Picker started resident');}
async function open(args,input){active=start(['--pick','--prompt','Pick a line',...args],input);await until(async()=>{if(active.done)throw Error('Picker exited early: '+active.code+' '+active.err);return (await state()).query!==null;},'Picker field missing');await delay(200);return active;}
async function tree(){const pids=[];for(const pid of await fs.readdir('/proc')){if(!/^\d+$/.test(pid))continue;try{const vars=(await fs.readFile('/proc/'+pid+'/environ','utf8')).split('\0');if(vars.includes('XDG_RUNTIME_DIR='+process.env.XDG_RUNTIME_DIR)){const cmd=(await fs.readFile('/proc/'+pid+'/cmdline','utf8')).split('\0');if(cmd[0]===binary)pids.push({pid:Number(pid),cmd});}}catch{}}return pids;}
async function files(folder){try{const out={};for(const d of await fs.readdir(folder,{withFileTypes:true})){const full=path.join(folder,d.name);if(d.isDirectory())Object.assign(out,await files(full));else if(d.isFile())out[full]=createHash('sha256').update(await fs.readFile(full)).digest('hex');}return out;}catch(e){if(e.code==='ENOENT')return{};throw e;}}
try{
 const installation=JSON.parse(await fs.readFile(path.join(root,'deployments/linux/current/installation.json'),'utf8'));assert.equal(installation.checkpoint,'CP008');assert.equal(createHash('sha256').update(await fs.readFile(binary)).digest('hex'),installation.binarySha256);
 const noGui={...process.env,DISPLAY:'',WAYLAND_DISPLAY:'',DBUS_SESSION_BUS_ADDRESS:'',ASAAAN_PICK_AUTO:'1'};
 for(const [args,input,code,out] of [
  [[], 'alpha\n\n   \nbeta\tdetail\n',0,'alpha\n'],
  [[], '  two words  \n',0,'  two words  \n'],
  [['--query','be','--index'],'alpha\n\n   \nbeta\tdetail\n',0,'3\n'],
  [['--query','nav'],'142\tnav rework\nnav\tsomething else',0,'nav\tsomething else\n'],
  [['--query','xyz'],'alpha\nbeta',1,''],
  [[], '\n\n  \n',1,''],
  [[], Buffer.from([0xe9,10]),0,'é\n'],
  [[], '日本語 ; literal\tcolumn\n',0,'日本語 ; literal\tcolumn\n'],
  [['--query','beta','--index'],'alpha\r\nbeta\r\n',0,'1\n']
 ]){const item=start(['--pick',...args],input,noGui);await result(item,code,out);}stages.push('headless text/index/tabs/blank positions/CRLF/Latin-1/Unicode/no-match and empty exit contracts');
 for(const args of [['--index'],['--index=true'],['--pick=true'],['--pick','--query'],['--pick','--panel'],['--pick','--pick']]){const item=start(args,'alpha\n',noGui);await result(item,64,'');}stages.push('invalid arguments exit 64 without UI');
 // A real terminal on stdin is a usage error even in automatic mode.
 const tty=await run('/usr/bin/python3',['-c','import os,pty,subprocess,sys; m,s=pty.openpty(); p=subprocess.run([sys.argv[1],"--pick"],stdin=s,stdout=subprocess.PIPE,stderr=subprocess.PIPE); os.close(s);os.close(m); print(p.returncode); assert p.returncode==64 and p.stdout==b""',binary],{env:noGui});assert.equal(tty.stdout.trim(),'64');stages.push('terminal stdin rejected with 64');
 const tempBefore=(await fs.readdir('/tmp')).filter(x=>x.startsWith('asaaan-picker-')).sort();
 const before=await files(path.join(process.env.XDG_DATA_HOME,'com.asaaan.launcher'));
 let ready='';focus=spawn('python3',[fixture,'fixture'],{stdio:['ignore','pipe','inherit']});focus.stdout.on('data',d=>ready+=d);await until(()=>ready,'Focus fixture not ready');
 await open(['--query','be'],'alpha\n\n   \nbeta\tdetail 日本語 ; literal\n');await text('ta');await until(async()=>(await state()).query==='beta','Starting query caret did not append');await key('Return');await result(active,0,'beta\tdetail 日本語 ; literal\n');stages.push('installed window: initial filter, append at caret, full Unicode/tab line output');
 await open(['--index'],'alpha\n\n   \nbeta\n');await key('End');await key('Return');await result(active,0,'3\n');stages.push('keyboard End chooses original blank-inclusive index');
 await open(['--query','xyz'],'alpha\nbeta\n');await key('Return');await delay(250);assert.equal(active.done,false);await key('Control_L','a');await text('alpha');await until(async()=>(await state()).query==='alpha','Correction missing');await delay(100);await key('Return');await result(active,0,'alpha\n');stages.push('Return with zero matches stays open; query correction succeeds');
 await open(['--query','be'],'alpha\nbeta\n');await key('Escape');await result(active,1,'');stages.push('Escape cancels a nonempty query without stdout');
 await open([],'alpha\nbeta\n');let focusReady='';const other=spawn('python3',[fixture,'fixture'],{stdio:['ignore','pipe','inherit']});other.stdout.on('data',d=>focusReady+=d);await until(()=>focusReady,'Second focus fixture missing');await result(active,1,'');other.kill();other.stdout.destroy();stages.push('native focus loss cancels');
 assert.deepEqual(await files(path.join(process.env.XDG_DATA_HOME,'com.asaaan.launcher')),before);assert.deepEqual(await tree(),[]);assert.deepEqual((await fs.readdir('/tmp')).filter(x=>x.startsWith('asaaan-picker-')).sort(),tempBefore);stages.push('no application data writes or resident/panel processes after picker exits');
 // Coexist with the normal resident; the picker must not forward stdin or activation into it.
 resident=spawn(binary,[],{stdio:['ignore','ignore','pipe']});resident.stderr.on('data',d=>residentLog+=d);await until(()=>residentLog.includes('visible: true'),'Resident failed to start');await open([],'standalone\n');await key('Return');await result(active,0,'standalone\n');assert.equal(resident.exitCode,null);assert.ok((await tree()).some(p=>p.pid===resident.pid));await run(binary,['--quit'],{timeout:5000});await until(()=>resident.exitCode!==null||resident.signalCode!==null,'Resident did not quit');stages.push('standalone picker coexists with running resident');
 const evidence={checkpoint:'CP008',result:'passed',stages,installation,at:new Date().toISOString()};await fs.mkdir(path.join(root,'.artifacts'),{recursive:true});await fs.writeFile(path.join(root,'.artifacts/picker-smoke.json'),JSON.stringify(evidence,null,2)+'\n');console.log('ASAAAN_PICKER_SMOKE_OK CP008 '+stages.length+' stages');
}catch(e){await fs.writeFile(path.join(root,'.artifacts/picker-failure.json'),JSON.stringify({message:String(e),stages,output:active?.out,error:active?.err,state:await state().catch(()=>null)},null,2));throw e;}finally{for(const c of [active?.child,focus,resident]){if(c?.exitCode===null&&c?.signalCode===null)c.kill();c?.stdout?.destroy();c?.stderr?.destroy();}}

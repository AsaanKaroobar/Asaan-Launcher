// Installed native UI, isolated catalog, resident-only persistence and restarts.
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
const run=promisify(execFile),root=new URL('../',import.meta.url),binary=new URL('deployments/linux/current/usr/bin/asaaan-launcher',root).pathname;
if(!process.env.XDG_RUNTIME_DIR?.includes('asaaan-x11-'))throw Error('Run through the isolated X11 harness');
const python=new URL('scripts/x11-fixture.py',root).pathname;
const data=process.env.XDG_DATA_HOME,storage=path.join(data,'com.asaaan.launcher/personalization.json');
const id='app:desktop:asaaan-personalization-probe.desktop';
const folder=path.join(process.env.XDG_CACHE_HOME,'probe'),marker=path.join(folder,'opened.json'),probe=path.join(folder,'probe.mjs');
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function until(check,message){const end=Date.now()+15000;while(!(await check())){if(Date.now()>end)throw Error(message+'\n'+log);await delay(30);}}
async function saved(){try{return JSON.parse(await fs.readFile(storage,'utf8'));}catch(e){if(e.code==='ENOENT')return null;throw e;}}
async function key(...keys){await run('python3',[python,'keys',...keys]);}
async function tabs(count){for(let index=0;index<count;index++)await key('Tab');}
async function text(value){await run('python3',[python,'text',value]);}
let resident,fixture,log='';const cycles=[];
async function stop(){if(resident?.exitCode===null){await run(binary,['--quit']);await until(()=>resident.exitCode!==null,'Resident did not quit');}}
try{
 const installation=JSON.parse(await fs.readFile(new URL('deployments/linux/current/installation.json',root),'utf8'));
 if(installation.checkpoint!=='CP007'||createHash('sha256').update(await fs.readFile(binary)).digest('hex')!==installation.binarySha256)throw Error('Install the validated CP007 package first');
 await fs.mkdir(folder,{recursive:true});await fs.mkdir(path.join(data,'applications'),{recursive:true});
 await fs.writeFile(probe,'import fs from "node:fs/promises"; await fs.writeFile(process.argv[2],JSON.stringify({opened:true,args:process.argv.slice(3)})); console.log("Probe output stays outside panel protocol");');
 await fs.writeFile(path.join(data,'applications/asaaan-personalization-probe.desktop'),'[Desktop Entry]\nType=Application\nName=Personalization Probe\nExec="'+process.execPath+'" "'+probe+'" "'+marker+'" "literal ; safe"\n');
 fixture=spawn('python3',[python,'fixture'],{stdio:['ignore','pipe','inherit']});let ready='';fixture.stdout.on('data',data=>ready+=data);await until(()=>ready,'Focus fixture not ready');
 for(let cycle=1;cycle<=4;cycle++){
  log='';await fs.rm(marker,{force:true});resident=spawn(binary,[],{stdio:['ignore','ignore','pipe']});resident.stderr.on('data',data=>log+=data);
  await until(()=>log.includes('visible: true'),'Native launcher did not present');
  await delay(350);
  if(cycle===2){await key('Up');}else await text('personalization');
  await delay(250);
  if(cycle<=2){await key('Control_L','Shift_L','f');await until(async()=>{const state=await saved();return state&&state.favorites.includes(id)===(cycle===1);},'Native favorite change was not saved');}
  if(cycle===3){
   // Input -> Clear -> History. Open the popover, then Refresh -> Customize -> Close -> checkbox.
   await tabs(2);await key('Return');await tabs(4);await key('space');
   await until(async()=>(await saved())?.rememberSearches===false,'History opt-out was not persisted');await delay(150);await key('Escape');
   await tabs(2);await key('Return');await tabs(6);await key('Return');
   await until(async()=>(await saved())?.queries.length===0,'History clear was not persisted');await key('Escape');
  }
  await key('Return');
  await until(async()=>{const state=await saved();return state?.records[id]?.count===cycle;},'Successful launch use count not persisted');
  await until(async()=>{try{return JSON.parse(await fs.readFile(marker,'utf8')).opened;}catch{return false;}},'Native probe did not launch');
  const state=await saved();if(JSON.stringify(state.queries)!==JSON.stringify(cycle<3?['personalization']:[]))throw Error('Query deduplication/opt-out mismatch');
  if(cycle>=3&&state.rememberSearches!==false)throw Error('History opt-out did not survive restart');
  const opened=JSON.parse(await fs.readFile(marker,'utf8'));if(JSON.stringify(opened.args)!==JSON.stringify(['literal ; safe']))throw Error('Native launch arguments changed');
  await until(()=>log.includes('Exited('),'Foreground did not exit after persisted launch');
  if(log.includes('personalization"')||log.includes('PersonalizationRequest'))throw Error('Personalization payload leaked into lifecycle logs');
  cycles.push({cycle,pid:resident.pid,count:state.records[id].count,favorite:state.favorites.includes(id),queries:state.queries,rememberSearches:state.rememberSearches,foregroundReaped:true});
  await stop();
 }
 const evidence={checkpoint:'CP007',result:'passed',kind:'installed normal React UI to resident persistence; private X11/XDG fixture',cycles,installation,at:new Date().toISOString()};
 await fs.writeFile(new URL('.artifacts/personalization-smoke.json',root),JSON.stringify(evidence,null,2)+'\n');console.log('ASAAAN_PERSONALIZATION_SMOKE_OK CP007 restarts=3 launches=4 favorites=passed recall=passed optOut=passed clear=passed');
}finally{await stop().catch(()=>resident?.kill());fixture?.kill();}

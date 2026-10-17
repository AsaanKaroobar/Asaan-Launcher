// Isolate native keyboard tests from the user's desktop, D-Bus session and preferences.
import fs from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
const run=promisify(execFile);const here=fileURLToPath(import.meta.url);const root=path.resolve(path.dirname(here),'..');
const delay=ms=>new Promise(resolve=>setTimeout(resolve,ms));
async function wait(check,message){const end=Date.now()+15000;while(!(await check())){if(Date.now()>end)throw Error(message);await delay(50);}}
async function execute(exe,args,options={}){const child=spawn(exe,args,{stdio:'inherit',...options});await new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',code=>code===0?resolve():reject(Error(exe+' exited '+code)));});}
if(process.argv.includes('--session')){
 const wm=spawn('kwin_x11',['--replace'],{stdio:['ignore','ignore','pipe']});let log='';wm.stderr.on('data',data=>log+=data);
 try{
  await wait(async()=>{if(wm.exitCode!==null)throw Error('Isolated window manager failed: '+log);try{return (await run('xprop',['-root','_NET_SUPPORTING_WM_CHECK'])).stdout.includes('window id # 0x');}catch{return false;}},'Window manager did not become ready: '+log);
  if(process.argv.includes('--process-only'))await execute(process.execPath,[path.join(root,'scripts/test-process-x11.mjs')],{cwd:root});
  else if(process.argv.includes('--settings-only'))await execute(process.execPath,[path.join(root,'scripts/test-settings-x11.mjs')],{cwd:root});
  else if(process.argv.includes('--picker-only'))await execute(process.execPath,[path.join(root,'scripts/test-picker-x11.mjs')],{cwd:root});
  else if(process.argv.includes('--actions-only'))await execute(process.execPath,[path.join(root,'scripts/test-actions-x11.mjs')],{cwd:root});
  else if(process.argv.includes('--customization-only'))await execute(process.execPath,[path.join(root,'scripts/test-customization-x11.mjs')],{cwd:root});
  else if(process.argv.includes('--personalization-only'))await execute(process.execPath,[path.join(root,'scripts/test-personalization-x11.mjs')],{cwd:root});
  else if(process.argv.includes('--smoke-only')){for(const command of ['smoke','application-smoke','resident-smoke'])await execute(process.execPath,[path.join(root,'scripts/local-deploy.mjs'),command],{cwd:root});}
  else await execute(process.execPath,[path.join(root,'scripts/test-resident-x11.mjs'),'--functional-only'],{cwd:root});
 }finally{wm.kill();await delay(100);if(wm.exitCode===null)wm.kill('SIGKILL');}
}else{
 const scratch=await fs.mkdtemp(path.join(os.tmpdir(),'asaaan-x11-'));let display='';
 const xserver=spawn('Xvfb',['-displayfd','3','-screen','0','1280x800x24','-nolisten','tcp','-ac'],{stdio:['ignore','ignore','pipe','pipe']});xserver.stdio[3].on('data',data=>display+=data);let log='';xserver.stderr.on('data',data=>log+=data);
 try{
  await wait(()=>{if(xserver.exitCode!==null)throw Error('Xvfb failed: '+log);return display.trim();},'Xvfb did not allocate a display');
  for(const folder of ['runtime','config','data','cache'])await fs.mkdir(path.join(scratch,folder),{mode:0o700});
  const env={...process.env,DISPLAY:':'+display.trim(),XAUTHORITY:'',XDG_SESSION_TYPE:'x11',XDG_CURRENT_DESKTOP:'KDE',XDG_RUNTIME_DIR:path.join(scratch,'runtime'),XDG_CONFIG_HOME:path.join(scratch,'config'),XDG_DATA_HOME:path.join(scratch,'data'),XDG_CACHE_HOME:path.join(scratch,'cache'),KWIN_COMPOSE:'N',LIBGL_ALWAYS_SOFTWARE:'1',WEBKIT_DISABLE_DMABUF_RENDERER:'1',WEBKIT_DISABLE_COMPOSITING_MODE:'1',GTK_IM_MODULE:'xim',QT_IM_MODULE:'compose',XMODIFIERS:'@im=none',IBUS_ADDRESS:''};
  await execute('dbus-run-session',['--',process.execPath,here,'--session',...process.argv.slice(2)],{cwd:root,env});
  }finally{
  xserver.kill();
  // Portal helpers can outlive their private bus and hold a FUSE mount in runtime/doc.
  for(const pid of await fs.readdir('/proc')){if(!/^\d+$/.test(pid))continue;try{const vars=(await fs.readFile('/proc/'+pid+'/environ','utf8')).split('\0');if(vars.includes('XDG_RUNTIME_DIR='+path.join(scratch,'runtime')))process.kill(Number(pid),'SIGTERM');}catch{}}
  await delay(200);
  try{await run('fusermount3',['-u',path.join(scratch,'runtime/doc')]);}catch{}
  await fs.rm(scratch,{recursive:true,force:true,maxRetries:5,retryDelay:100}).catch(error=>console.error('Private session cleanup:',error.message));
 }

}

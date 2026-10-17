import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const run=promisify(execFile);
const bun=path.join(root,'node_modules','bun','bin','bun.exe');
const executable=await fs.realpath(bun);
if((await run(executable,['--version'])).stdout.trim()!=='1.4.3')throw Error('Expected pinned Bun 1.4.3; run npm ci.');
const env={...process.env,PATH:path.dirname(executable)+path.delimiter+process.env.PATH};
const mode=process.argv[2];
if(mode==='test'){
 // Subprocess tests use process.execPath, so a developer Bun on PATH cannot replace the pinned runtime.
 const child=spawn(executable,['--config=runtime/bunfig.toml','test','./runtime'],{cwd:root,env,stdio:'inherit'});
 child.on('exit',(code)=>process.exit(code??1));
}else if(mode==='prepare'){
 await run(executable,['install','--frozen-lockfile'],{cwd:path.join(root,'runtime'),env});
 const destination=path.join(root,'.runtime-bundle');
 await fs.rm(destination,{recursive:true,force:true});
 await fs.mkdir(destination,{recursive:true});
 const sources=(await fs.readdir(path.join(root,'runtime'))).filter(n=>!['tests','survey.ts','smoke.ts','testing.ts','bunfig.toml'].includes(n));
 for(const name of sources)await fs.cp(path.join(root,'runtime',name),path.join(destination,'runtime',name),{recursive:true,dereference:true});
 for(const name of ['hello','diagnostics','host-probe'])await fs.cp(path.join(root,'extensions',name),path.join(destination,'extensions',name),{recursive:true});
 await fs.cp(path.join(root,'LICENSES'),path.join(destination,'LICENSES'),{recursive:true});
 await fs.copyFile(path.join(root,'NOTICE.md'),path.join(destination,'NOTICE.md'));
 const target=path.join(destination,process.platform==='win32'?'bun.exe':'bun');
 await fs.copyFile(executable,target);await fs.chmod(target,0o755);
 const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
 await fs.writeFile(path.join(destination,'runtime.json'),JSON.stringify({bun:'1.4.3',platform:process.platform,architecture:process.arch,sha256:digest(await fs.readFile(target))},null,2)+'\n');
 console.log('Prepared bundled Bun 1.4.3 and frozen runtime dependencies for '+process.platform+'/'+process.arch);
}else throw Error('Usage: node scripts/runtime.mjs test|prepare');

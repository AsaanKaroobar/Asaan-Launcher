import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root=new URL('../',import.meta.url);
const read=p=>fs.readFile(new URL(p,root),'utf8');
const hash=b=>createHash('sha256').update(b).digest('hex');
const prefix='tests/reference/extensions/';
const provenance=JSON.parse(await read(prefix+'provenance.json'));
if(provenance.commit!=='5b26e9d7a0240243251d7421bee5b90da5f52b9b')throw Error('Extension reference changed');
for(const entry of provenance.sources){
 const bytes=await fs.readFile(new URL(prefix+entry.file,root));
 if(hash(bytes)!==entry.sha256)throw Error('Frozen extension source changed: '+entry.source);
 if(entry.portedSha256&&hash(await fs.readFile(new URL(entry.source,root)))!==entry.portedSha256)throw Error('Imported source changed without updating its patch record: '+entry.source);
 if(entry.source.startsWith('runtime/tests/')){
  let expected=bytes.toString();
  if(entry.source.endsWith('bridge.test.ts')||entry.source.endsWith('cache.test.ts')){
   expected=expected.replaceAll('["bun",','[process.execPath,')
    .replaceAll('env: { ...process.env, HOME: home }','env: { ...process.env, HOME: home, ASAAAN_EXTENSION_DATA: path.join(home, "data") }')
    .replace('path.join(home, "Library/Application Support/Floe/Data/flush-sample/cache-default.json")','path.join(home, "data/flush-sample/cache-default.json")');
  }
  if(await read(entry.source)!==expected)throw Error('Original assertions changed: '+entry.source);
 }
}
const map=JSON.parse(await read(prefix+'test-map.json'));
const ids=new Set();
for(const row of map){
 if(ids.has(row.id))throw Error('Duplicate reference ID '+row.id);ids.add(row.id);
 if(!['unported','ported-unverified','passing','partial'].includes(row.status))throw Error('Unknown status');
 if(!row.target){if(row.status!=='unported'||!row.note)throw Error('Unmapped declaration lacks explanation');continue;}
 const text=await read(row.target);
 if(row.target.endsWith('.rs')&&!text.includes('fn '+row.test+'('))throw Error('Missing Rust equivalent '+row.test);
 if(row.target.endsWith('.ts')&&!text.includes(row.name))throw Error('Missing imported test '+row.name);
}
const expected=JSON.parse(await read(prefix+'inventory.json'));
if(JSON.stringify(map.map(r=>({id:r.id,name:r.name,source:r.source})))!==JSON.stringify(expected))throw Error('Original declarations disappeared or changed');
console.log('Verified '+provenance.sources.length+' frozen extension sources and '+map.length+' declarations ('+map.filter(r=>r.target).length+' mapped, remainder explicitly deferred). Runtime assertions are preserved; native UI/API parity remains partial.');

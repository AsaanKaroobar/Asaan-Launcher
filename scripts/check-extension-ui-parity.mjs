import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root=new URL('../',import.meta.url),prefix='tests/reference/extension-ui/';
const read=p=>fs.readFile(new URL(p,root),'utf8');
const hash=b=>createHash('sha256').update(b).digest('hex');
const provenance=JSON.parse(await read(prefix+'provenance.json'));
if(provenance.commit!=='5b26e9d7a0240243251d7421bee5b90da5f52b9b')throw Error('Unexpected List reference revision');
for(const entry of provenance.sources)if(hash(await fs.readFile(new URL(prefix+entry.file,root)))!==entry.sha256)throw Error('Frozen List reference changed: '+entry.source);
const inventory=JSON.parse(await read(prefix+'inventory.json')),map=JSON.parse(await read(prefix+'test-map.json'));
if(JSON.stringify(map.map(({id,suite,name,source})=>({id,suite,name,source})))!==JSON.stringify(inventory))throw Error('Reference declarations disappeared');
const found=[];
for(const entry of provenance.sources.filter(e=>e.source.includes('/Tests/')||e.source.startsWith('Tests/'))){
 let suite='',pending=false;
 for(const line of (await read(prefix+entry.file)).split('\n')){
  const s=line.match(/^struct (\w+)/);if(s)suite=s[1];
  if(line.includes('@Test'))pending=true;
  const f=line.match(/func (\w+)\(/);
  if(pending&&f){found.push({id:suite+'.'+f[1],suite,name:f[1],source:entry.source});pending=false;}
 }
}
if(JSON.stringify(found)!==JSON.stringify(inventory))throw Error('Inventory differs from frozen Swift declarations');
for(const row of map){
 if(!['passing','partial','deferred'].includes(row.status)||!row.note)throw Error('Unexplained mapping');
 if(!row.target){if(row.status!=='deferred')throw Error('Missing equivalent');continue;}
 const source=await read(row.target);
 if(!source.includes(row.target.endsWith('.rs')?'fn '+row.test+'(':'it("'+row.test+'"'))throw Error('Missing equivalent: '+row.id);
}
const original=await fs.readFile(new URL('tests/reference/extensions/frozen/extensions/hello/src/planets.tsx',root));
if(hash(original)!==hash(await fs.readFile(new URL('extensions/list-lab/src/planets.tsx',root))))throw Error('Original Planets fixture changed');
console.log('Verified '+provenance.sources.length+' frozen List sources and '+map.length+' Swift declarations: '+map.filter(r=>r.status==='passing').length+' equivalent, '+map.filter(r=>r.status==='partial').length+' partial, '+map.filter(r=>r.status==='deferred').length+' deferred. Full extension UI parity is not claimed.');

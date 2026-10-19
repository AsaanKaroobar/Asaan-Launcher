import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root=new URL('../',import.meta.url);
const provenance=JSON.parse(await fs.readFile(new URL('tests/reference/lifecycle/provenance.json',root),'utf8'));
const map=JSON.parse(await fs.readFile(new URL('tests/reference/lifecycle/test-map.json',root),'utf8'));
let count=0;
for(const source of provenance.sources){
 const bytes=await fs.readFile(new URL('tests/reference/lifecycle/'+source.file,root));
 if(createHash('sha256').update(bytes).digest('hex')!==source.sha256)throw Error('Reference changed: '+source.file);
 if(source.file.endsWith('Tests.swift'))for(const match of bytes.toString().matchAll(/@Test\b[\s\S]*?\bfunc\s+(\w+)/g)){
  count++;const entry=map.find(t=>t.source===source.file&&t.function===match[1]);
  if(!entry)throw Error('Unmapped lifecycle case: '+match[1]);
  if(entry.status==='ported'){const target=await fs.readFile(new URL(entry.target,root),'utf8');if(!new RegExp('fn\\s+'+entry.targetFunction+'\\s*\\(').test(target))throw Error('Missing equivalent: '+entry.targetFunction);}
  else if(!entry.reason)throw Error('Missing deferred reason');
 }
}
if(count!==map.length)throw Error('Unexpected lifecycle mapping count');
console.log('Verified lifecycle reference hashes and '+count+' mappings ('+map.filter(t=>t.status==='ported').length+' ported, '+map.filter(t=>t.status==='deferred').length+' deferred).');

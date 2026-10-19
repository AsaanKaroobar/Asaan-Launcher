import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root=new URL('../',import.meta.url);
const read=path=>fs.readFile(new URL(path,root),'utf8');
const provenance=JSON.parse(await read('tests/reference/customization/provenance.json'));
const map=JSON.parse(await read('tests/reference/customization/test-map.json'));let count=0;
for(const entry of provenance.sources){const bytes=await fs.readFile(new URL('tests/reference/customization/'+entry.file,root));if(createHash('sha256').update(bytes).digest('hex')!==entry.sha256)throw Error('Changed customization reference: '+entry.file);
 if(entry.file.endsWith('Tests.swift'))for(const match of bytes.toString().matchAll(/@Test\b[\s\S]*?\bfunc\s+(\w+)/g)){count++;const row=map.find(r=>r.source===entry.file&&r.function===match[1]);if(!row)throw Error('Unmapped customization test: '+match[1]);if(row.target){const source=await read(row.target);if(row.kind==='rust'?!new RegExp('fn\\s+'+row.test+'\\s*\\(').test(source):!source.includes('it('+JSON.stringify(row.test)))throw Error('Missing equivalent: '+row.test);}if(row.status!=='ported'&&!row.reason)throw Error('Missing deferred scope explanation');}}
if(count!==map.length)throw Error('Unexpected customization mapping count');console.log('Verified '+count+' customization mappings: '+['ported','partial','deferred'].map(state=>map.filter(r=>r.status===state).length+' '+state).join(', '));

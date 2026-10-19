import fs from 'node:fs/promises';import {createHash} from 'node:crypto';
const root=new URL('../',import.meta.url),read=p=>fs.readFile(new URL(p,root),'utf8');
const provenance=JSON.parse(await read('tests/reference/search-cli/provenance.json'));
const map=JSON.parse(await read('tests/reference/search-cli/test-map.json'));
let count=0;
for(const entry of provenance.sources){const bytes=await fs.readFile(new URL('tests/reference/search-cli/'+entry.file,root));if(createHash('sha256').update(bytes).digest('hex')!==entry.sha256)throw Error('Changed search CLI reference: '+entry.file);if(entry.file.endsWith('Tests.swift'))for(const match of bytes.toString().matchAll(/@Test\b[\s\S]*?\bfunc\s+(\w+)/g)){count++;const row=map.find(r=>r.source===entry.file&&r.function===match[1]);if(!row)throw Error('Unmapped ranking test '+match[1]);if(!new RegExp('fn\\s+'+row.test+'\\s*\\(').test(await read(row.target)))throw Error('Missing ranking equivalent '+row.test);}}
if(count!==map.length)throw Error('Unexpected ranking map count');
const contracts=JSON.parse(await read('tests/reference/search-cli/contracts.json')),command=await read('tests/reference/search-cli/SearchCommand.swift');
if(!command.includes('model.results.prefix(8)'))throw Error('Reference CLI output limit changed');
for(const row of contracts){if(!command.includes('func '+row.function+'('))throw Error('Missing reference command');if(!new RegExp('fn\\s+'+row.test+'\\s*\\(').test(await read(row.target)))throw Error('Missing CLI test '+row.test);}
console.log('Verified '+count+' reused ranking declarations and '+contracts.length+' scoped CLI contracts; non-application providers remain deferred.');

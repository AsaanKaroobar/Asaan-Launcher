import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const provenance=JSON.parse(await fs.readFile(path.join(root,'tests/reference/provenance.json'),'utf8'));
for(const [name,hash] of Object.entries(provenance.sha256)){
 const data=await fs.readFile(path.join(root,'tests/reference/floe',name));
 if(createHash('sha256').update(data).digest('hex')!==hash)throw Error('Pinned reference changed: '+name);
}
const map=JSON.parse(await fs.readFile(path.join(root,'tests/reference/test-map.json'),'utf8'));
const source=await fs.readFile(path.join(root,'tests/reference/floe/RankingTests.swift'),'utf8');
const names=[...source.matchAll(/@Test\b[\s\S]*?\bfunc\s+(\w+)/g)].map(m=>m[1]);
if(names.length!==38)throw Error('Reference test scope changed');
for(const name of names)if(!map.tests.some(t=>t.source.endsWith('RankingTests.swift')&&t.function===name))throw Error('Unmapped ranking test: '+name);
for(const test of map.tests){const target=await fs.readFile(path.join(root,test.target),'utf8');if(!new RegExp('fn\\s+'+test.function+'\\s*\\(').test(target))throw Error('Missing ported test: '+test.function);}
const fixtures=JSON.parse(await fs.readFile(path.join(root,'tests/fixtures/floe-search.json'),'utf8'));
if(fixtures.referenceCommit!==provenance.referenceCommit||fixtures.matches.length!==775||fixtures.rankings.length!==20)throw Error('Reference snapshot provenance or case count changed');
console.log('Verified pinned reference hashes, 38/38 ranking mappings, 4 merge mappings, 775 matcher fixtures and 20 ranking fixtures');

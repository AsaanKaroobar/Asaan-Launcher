import fs from 'node:fs/promises';import {createHash} from 'node:crypto';
const root=new URL('../',import.meta.url),read=p=>fs.readFile(new URL(p,root),'utf8');
const provenance=JSON.parse(await read('tests/reference/process-actions/provenance.json'));
for(const entry of provenance.sources){
 const bytes=await fs.readFile(new URL('tests/reference/process-actions/'+entry.file,root));
 if(createHash('sha256').update(bytes).digest('hex')!==entry.sha256)throw Error('Changed process-action reference '+entry.file);
}
const contracts=JSON.parse(await read('tests/reference/process-actions/contracts.json'));
const source=await read('tests/reference/process-actions/ItemActions.swift');
for(const c of contracts){
 if(!source.includes(c.reference))throw Error('Reference contract missing '+c.reference);
 if(!new RegExp('fn\\s+'+c.test+'\\s*\\(').test(await read(c.target)))throw Error('Missing equivalent '+c.test);
}
const tests=await read('tests/reference/process-actions/ItemActionTests.swift');
if(!tests.includes('func anAppThatIsNotRunningOffersNoQuitAndASystemAppNoTrash('))throw Error('Original non-running test missing');
const map=JSON.parse(await read('tests/reference/process-actions/test-map.json'));
for(const row of map){if(!tests.includes('func '+row.function+'(')||!new RegExp('fn\\s+'+row.test+'\\s*\\(').test(await read(row.target)))throw Error('Missing original test mapping '+row.function);}
const native=await read('scripts/test-process-x11.mjs');
for(const token of ['Refused Quit terminated app','Hide did not minimize every window','Stale menu action did not revalidate','Process controls changed preferences/history'])if(!native.includes(token))throw Error('Missing installed contract '+token);
for(const token of ['Default Cancel affected app','Refused Restart escalated','Closed windows caused early reopen','Confirmation affected replacement instance','Changed desktop entry was reopened'])if(!(await read('scripts/test-process-controls-x11.mjs')).includes(token))throw Error('Missing checkpoint 12 installed contract '+token);
console.log('Verified '+contracts.length+' scoped process contracts/boundaries and partial original non-running declaration; Restart/Force Quit require native qualification on each OS; Trash remains deferred.');

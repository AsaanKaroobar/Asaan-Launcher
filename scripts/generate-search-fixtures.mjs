import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
const run=promisify(execFile);
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const folder=path.join(root,'tests/reference/floe');
const manifest=JSON.parse(await fs.readFile(path.join(root,'tests/reference/provenance.json'),'utf8'));
const hash=s=>createHash('sha256').update(s).digest('hex');
const sources={};
for(const [file,expected] of Object.entries(manifest.sha256)){
  const text=await fs.readFile(path.join(folder,file),'utf8');
  if(hash(text)!==expected)throw Error('Reference source hash changed: '+file);
  sources[file]=text;
}
const candidates=['Safari','Activity Monitor','Visual Studio Code','Google Chrome','ColorSync Utility','System Settings','Café Crème','Über Notes','foo_bar-baz.swift','Sources/Floe/Ranking.swift','A-B-C','xaxbxc ab-c','','Privacy & Security','1Password 7','iPhone Mirroring','ÉCOLE','a\r\nb'];
const queries=['','s','saf','SAF','mon','vsc','tivi','sfr','gchr','cs','abc','é','cré','uber','über','fbb','rank','floe/r','bar-b','y m','1p','7','pm','ecole','école','ac','b','a b','safarii','&'];
const matches=queries.flatMap(query=>candidates.map(candidate=>({query,candidate})));
const extras=[['abc','xaxbxc ab-c'],['os','ColorSync'],['os','Colorsync'],['oé','ColorÉcole'],['cé','Crème brûlée'],['saf','Safari'],['gchr','Google Chrome'],['é','Cafe\u0301'],['e\u0301','Café'],['b','a\r\nb'],['💻','a👩‍💻💻'],['👩','👩‍💻 Notes'],['👩‍💻','👩‍💻 Notes'],['🇵🇰','🇵🇰 Notes'],['s','Straße'],['ss','Straße'],['οσ','ΟΣ'],['ος','ΟΣ'],['i','İstanbul'],['i\u0307','İstanbul'],['é','École'],['1','1️⃣ Notes'],['e','Café'],['ab','a👨‍👩‍👧‍👦b'],['af','a🇵🇰f']];
for(const [query,candidate] of extras)matches.push({query,candidate});
// Extra crossed Unicode cases characterize the language boundary independently of Rust.
for(const query of ['','a','é','e\u0301','👩','👩‍💻','🇵🇰','ΟΣ','ος','i','i\u0307','éx','ab','aé','éa'])for(const candidate of ['','a','Cafe\u0301','Café','👩‍💻 Notes','a🇵🇰b','a👨‍👩‍👧‍👦b','ΟΣ','İstanbul','ColorÉcole','éx','Éx','a\r\nb','aéa'])matches.push({query,candidate});
const app=title=>({id:title,title,isApp:true,keywords:[]});
const cmd=(title,keywords=[])=>({...app(title),isApp:false,keywords});
const items=[cmd('Hacker News',['Hacker News']),cmd('Browse Planets',['hello']),app('Calculator'),app('Calendar'),app('Notes'),cmd('Search Menu Bar Items'),cmd('Floe Settings')];
const base={query:'',items,searchOnly:[],favorites:[],aliases:{},usage:{},limit:40};
const rankings=[];
function scenario(name,mode,changes={}){rankings.push({...base,...changes,name,mode});}
scenario('browse default','browse');scenario('favorites suggestions','browse',{favorites:['Notes'],usage:{Calendar:5,'Hacker News':2}});scenario('missing favorite','browse',{favorites:['Calendar','Notes','Gone'],usage:{Calendar:9}});scenario('stable suggestions','browse',{usage:Object.fromEntries(items.map(i=>[i.id,1]))});
scenario('unused cal','search',{query:'cal'});scenario('used cal','search',{query:'cal',usage:{Calculator:6}});scenario('favorite cal','search',{query:'cal',favorites:['Calculator']});scenario('exact alias beats capped usage','search',{query:'c',aliases:{'Hacker News':'c'},usage:{Calculator:1000000}});
const many=Array.from({length:80},(_,i)=>app('App '+(i+1)));
scenario('stable top five','browse',{items:many,usage:Object.fromEntries(many.map((i,n)=>[i.id,n===40?2:1]))});
for(const limit of [0,3,40])scenario('limit '+limit,'search',{items:many,query:'app',limit});
scenario('search only invisible','browse',{items:[app('Safari')],searchOnly:[cmd('Displays'),cmd('Sound')]});scenario('search only favorite','browse',{items:[app('Safari')],searchOnly:[cmd('Displays'),cmd('Sound')],favorites:['Displays']});scenario('search only suggestion','browse',{items:[app('Safari')],searchOnly:[cmd('Displays')],usage:{Displays:1}});
scenario('keyword less than title','search',{items:[cmd('Show Installed',['brew']),cmd('Upgrade',['brew']),app('Brewfile')],query:'brew'});scenario('keyword refuses scattered','search',{items:[cmd('Show Installed',['brew']),app('Brewfile')],query:'bw'});
scenario('manifest keyword','search',{items:[cmd('Search Items',['vault','1p','login']),app('Login Items')],query:'login'});
scenario('duplicate known id first wins','browse',{items:[app('Calendar'),{...app('Calendar'),title:'Another Calendar'}],favorites:['Calendar']});
scenario('unicode alias whole character','search',{items:[app('Notes')],query:'👩',aliases:{Notes:'👩‍💻'}});
const input={matches,rankings};
const temp=await fs.mkdtemp(path.join(os.tmpdir(),'asaaan-swift-oracle-'));
try{
  const declaration='import Foundation\nnonisolated enum Fuzzy { static func score(_ q:String,_ c:String)->Int? { evaluate(q,c,wantPositions:false)?.score }; static func tierScore(_ q:String,_ c:String)->Int? { evaluate(q,c,wantPositions:false,scattered:false)?.score } }\n';
  let ranking=sources['Ranking.swift'].slice(sources['Ranking.swift'].indexOf('enum Ranking {'),sources['Ranking.swift'].indexOf('\nextension Collection'));
  ranking=ranking.replace(/String\(localized:\s*"([^"]*)",\s*bundle:\s*\.floe(?:,\s*comment:\s*"[^"]*")?\)/g,(_,text)=>JSON.stringify(text));
  const harness=await fs.readFile(path.join(root,'tests/reference/Oracle.swift'),'utf8');
  const main=path.join(temp,'main.swift');await fs.writeFile(main,declaration+sources['Fuzzy.swift']+'\n'+ranking+'\n'+harness);
  const compiler=process.env.SWIFTC||'swiftc';const version=(await run(compiler,['--version'])).stdout.trim();
  await run(compiler,['-O','-swift-version','6','-module-cache-path',path.join(temp,'cache'),main,'-o',path.join(temp,'oracle')],{timeout:60000,maxBuffer:2**20});
  const result=await new Promise((resolve,reject)=>{
    import('node:child_process').then(({spawn})=>{const child=spawn(path.join(temp,'oracle'));let stdout='',stderr='';child.stdout.on('data',d=>stdout+=d);child.stderr.on('data',d=>stderr+=d);child.on('error',reject);child.on('close',code=>code===0?resolve(stdout):reject(Error(stderr)));child.stdin.end(JSON.stringify(input));});
  });
  const output={referenceCommit:manifest.referenceCommit,compiler:version,scope:'Pure matcher and ranking bodies; domain/Algorithms/localization adapters only. Not a macOS UI or full app test.',...JSON.parse(result)};
  const target=path.join(root,'tests/fixtures/floe-search.json');const encoded=JSON.stringify(output,null,2)+'\n';
  if(process.argv.includes('--check')){const old=JSON.parse(await fs.readFile(target,'utf8'));if(JSON.stringify(old.matches)!==JSON.stringify(output.matches)||JSON.stringify(old.rankings)!==JSON.stringify(output.rankings))throw Error('Swift reference outputs differ from frozen fixtures');console.log('Swift reference fixtures verified');}
  else{await fs.mkdir(path.dirname(target),{recursive:true});await fs.writeFile(target,encoded);console.log('Wrote '+output.matches.length+' matcher and '+output.rankings.length+' ranking cases from '+version);}
}finally{await fs.rm(temp,{recursive:true,force:true});}

import {applyTheme} from "./theme";
import {useCallback,useEffect,useRef,useState} from "react";
import {closeSettings,confirmSettingsShown,connectSettings,readSettings,reportSettingsPage,saveSettings,type SettingsPage,type SettingsView} from "./settings-native";
import type {PersonalizationChange} from "./native";
declare global {interface Window {__ASAAAN_SETTINGS__?:boolean}}
const message=(error:unknown)=>error instanceof Error?error.message:String(error);
export default function Settings(){
 const [page,setPage]=useState<SettingsPage>("general");
 const [view,setView]=useState<SettingsView|null>(null);
 const [error,setError]=useState("");
 const [status,setStatus]=useState("");
 const [busy,setBusy]=useState(false);
 const [filter,setFilter]=useState("");
 const [editor,setEditor]=useState<{id:string;name:string;draft:string}|null>(null);
 const title=useRef<HTMLHeadingElement>(null),epoch=useRef(0),saving=useRef(false),alive=useRef(true);
 const refresh=useCallback(async()=>{
  if(saving.current)return;
  const request=++epoch.current;
  try{const next=await readSettings();if(alive.current&&epoch.current===request){setView(next);setError("");}}
  catch(e){if(alive.current&&epoch.current===request)setError(message(e));}
 },[]);
 useEffect(()=>{alive.current=true;void refresh();const timer=setInterval(()=>{void refresh();},5000);const focus=()=>{void refresh();};window.addEventListener("focus",focus);return()=>{alive.current=false;epoch.current++;clearInterval(timer);window.removeEventListener("focus",focus);};},[refresh]);
 const selectPage=useCallback((next:SettingsPage)=>{setPage(next);void reportSettingsPage(next).catch(e=>setError(message(e)));},[]);
 useEffect(()=>{
  let disposed=false;let stop:(()=>void)|undefined;
  void connectSettings(sequence=>{title.current?.focus();void confirmSettingsShown(sequence,document.activeElement===title.current).catch(e=>setError(message(e)));},selectPage).then(unsubscribe=>{if(disposed)unsubscribe();else stop=unsubscribe;},e=>setError(message(e)));
  return()=>{disposed=true;stop?.();};
 },[selectPage]);
 async function close(){try{await closeSettings();}catch(e){setError(message(e));}}
 useEffect(()=>{const key=(event:KeyboardEvent)=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==="w"){event.preventDefault();void closeSettings().catch(e=>setError(message(e)));}};window.addEventListener("keydown",key);return()=>window.removeEventListener("keydown",key);},[]);
 async function change(operation:PersonalizationChange){
  if(saving.current)return false;
  saving.current=true;epoch.current++;setBusy(true);setError("");setStatus("");
  try{const snapshot=await saveSettings(operation);if(alive.current){setView(old=>old?{...old,snapshot}:old);setStatus("Saved");}return true;}
  catch(e){if(alive.current)setError(message(e));return false;}
  finally{saving.current=false;if(alive.current)setBusy(false);}
 }
 useEffect(()=>{if(view)applyTheme(view.snapshot.theme);},[view]);
 const snapshot=view?.snapshot,aliases=snapshot?.aliases??{},hidden=snapshot?.hiddenResults??{};
 const applications=view?.applications.filter(a=>(a.name+" "+a.id+" "+(aliases[a.id]??"")).toLocaleLowerCase().includes(filter.toLocaleLowerCase()))??[];
 return <main className="settings-shell" aria-labelledby="settings-title">
  <aside className="settings-sidebar"><span className="wordmark">asaaan<span aria-hidden="true">.</span></span><span className="settings-caption">Launcher settings</span><nav aria-label="Settings pages">{(["general","applications"] as const).map(name=><button key={name} aria-current={page===name?"page":undefined} onClick={()=>selectPage(name)}>{name==="general"?"General":"Applications"}</button>)}</nav><p>Preferences stay on this device.</p></aside>
  <section className="settings-detail"><header className="settings-header"><div><h1 id="settings-title" ref={title} tabIndex={-1}>{page==="general"?"General":"Applications"}</h1><p>{page==="general"?"Search history and local preferences":"Aliases and visibility in search"}</p></div><button onClick={()=>{void close();}} aria-label="Close settings">×</button></header>
   <div className="settings-content" aria-busy={busy}>
    {error&&<div role="alert" className="settings-error"><p>{error}</p><button disabled={busy} onClick={()=>{void refresh();}}>Reload settings</button></div>}
    {view?.warning&&<p role="status" className="settings-warning">{view.warning}</p>}
    {!view&&!error&&<p role="status">Loading settings…</p>}
    {snapshot&&page==="general"&&<>
     <section className="settings-card"><h2>Appearance</h2><label className="theme-choice">Launcher theme<select aria-label="Launcher theme" value={snapshot.theme??"floe"} disabled={busy} onChange={e=>{void change({kind:"theme",theme:e.target.value==="legacy"?"legacy":"floe"});}}><option value="floe">Floe</option><option value="legacy">Legacy</option></select></label><p className="settings-note">Floe uses the reference launcher layout. Legacy keeps the original Asaaan design.</p></section>
     <section className="settings-card" aria-labelledby="history-heading"><h2 id="history-heading">Search history</h2><label className="settings-toggle"><span><strong>Remember searches</strong><small>Save searches locally when you open an application.</small></span><input type="checkbox" checked={snapshot.rememberSearches} disabled={busy} onChange={e=>{void change({kind:"rememberSearches",enabled:e.target.checked});}}/></label><p className="settings-note">Turning this off keeps existing history. Application usage and favorites are managed separately.</p><div className="settings-card-footer"><span>{snapshot.queries.length} recent searches</span><button disabled={busy||!snapshot.queries.length} onClick={()=>{void change({kind:"clearQueries"});}}>Clear search history</button></div>{snapshot.queries.length>0&&<ul className="settings-history">{[...snapshot.queries].reverse().map(query=><li key={query}>{query}</li>)}</ul>}</section>
     <section className="settings-card"><h2>Application preferences</h2><p>Set short aliases for applications and choose which applications appear in search.</p><button onClick={()=>selectPage("applications")}>Manage applications</button></section>
    </>}
    {snapshot&&page==="applications"&&<>
     <label className="settings-filter">Filter applications<input type="search" value={filter} onChange={e=>setFilter(e.target.value)} placeholder="Name, alias or application ID"/></label>
     <p className="settings-note">Hiding an application removes it from Favorites. Restoring it keeps its alias and usage.</p>
     <ul className="settings-apps" aria-label="Application preferences">{applications.map(application=>{const isHidden=Object.prototype.hasOwnProperty.call(hidden,application.id);return <li key={application.id}>
      <div className="settings-app-name"><strong>{application.name}</strong><small>{application.id}</small><div>{!application.installed&&<span className="settings-badge">Not installed</span>}{isHidden&&<span className="settings-badge">Hidden</span>}{aliases[application.id]&&<span className="alias-chip">{aliases[application.id]}</span>}</div></div>
      {editor?.id===application.id?<form className="settings-alias" onSubmit={e=>{e.preventDefault();void change({kind:"alias",id:editor.id,alias:editor.draft}).then(saved=>{if(saved)setEditor(null);});}}><label>Alias for {editor.name}<input autoFocus maxLength={1024} value={editor.draft} disabled={busy} onChange={e=>setEditor({...editor,draft:e.target.value})}/></label><div><button disabled={busy} type="submit">Save alias</button><button disabled={busy} type="button" onClick={()=>setEditor(null)}>Cancel</button></div></form>:<div className="settings-app-actions">{application.installed&&<button disabled={busy} onClick={()=>setEditor({id:application.id,name:application.name,draft:aliases[application.id]??""})}>{aliases[application.id]?"Edit alias":"Set alias"}</button>}{aliases[application.id]&&<button disabled={busy} onClick={()=>{void change({kind:"alias",id:application.id,alias:""});}}>Remove alias</button>}{(application.installed||isHidden)&&<button disabled={busy} onClick={()=>{void change({kind:isHidden?"restore":"hide",id:application.id});}}>{isHidden?"Restore":"Hide"}</button>}</div>}
     </li>;})}</ul>{!applications.length&&<p>No applications match this filter.</p>}
    </>}
   </div><footer className="settings-footer"><span role="status">{busy?"Saving…":status}</span><span>Asaaan Launcher · Settings</span></footer>
  </section>
 </main>;
}

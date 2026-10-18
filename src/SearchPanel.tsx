import ExtensionPanel from "./ExtensionPanel";
import {extensionExchange,type ExtensionView} from "./extension-native";
import {ApplicationActions,AliasEditor,Customization} from "./ApplicationActions";
import ApplicationIcon from "./ApplicationIcon";
import { useEffect, useLayoutEffect, useId, useRef, useState } from "react";
import { applicationProcessOptions, type ApplicationActionDescriptor, runApplicationAction, type ApplicationAction, searchApplications, launchApplication, reportApplicationLaunched, confirmPanelPresented, hidePanel, onPanelShown, reportSearchReady, searchReferenceCatalog, changePersonalization, type PersonalizationChange, type Personalization, type SearchResult } from "./native";

type SearchState = { status: "loading" } | { status: "loaded"; query: string; rows: SearchResult[]; total?: number; commandTotal?:number; warning?: string | null } | { status: "error"; message: string };

export default function SearchPanel({ smokeTest = false, referenceCatalog = false, applicationSmoke = false }: { smokeTest?: boolean; referenceCatalog?: boolean; applicationSmoke?: boolean }) {
  const isReference = referenceCatalog || smokeTest;
  const[extensionView,setExtensionView]=useState<ExtensionView|null>(null);
  const extensionOpen=useRef(false);extensionOpen.current=Boolean(extensionView);
  useEffect(()=>{if(isReference||applicationSmoke)return;let cancelled=false;void extensionExchange({kind:"read"}).then(reply=>{if(!cancelled&&reply.active)setExtensionView(reply.active);}).catch(()=>{});return()=>{cancelled=true;};},[isReference,applicationSmoke]);
  const normalize=(value:Personalization)=>({...value,aliases:value.aliases??{},hiddenResults:value.hiddenResults??{}});
  const [preferences,setPreferences]=useState(()=>normalize({favorites:[],queries:[],rememberSearches:true,aliases:{},hiddenResults:{}}));
  const [actionTarget,setActionTarget]=useState<SearchResult|null>(null);
  const [processOptions,setProcessOptions]=useState<{id:string;actions:ApplicationActionDescriptor[];notice:string|null}|null>(null);
  useEffect(()=>{
    setProcessOptions(null);
    if(!actionTarget||isReference||actionTarget.item.kind==="command")return;
    let cancelled=false;
    void applicationProcessOptions(actionTarget.item.id).then(value=>{
      if(!cancelled)setProcessOptions({id:actionTarget.item.id,...value});
    }).catch(()=>{if(!cancelled)setProcessOptions({id:actionTarget.item.id,actions:[],notice:"Running-app controls could not be checked."});});
    return()=>{cancelled=true;};
  },[actionTarget,isReference]);
  const [aliasTarget,setAliasTarget]=useState<SearchResult|null>(null);
  const [customizing,setCustomizing]=useState(false);
  const [historyOpen,setHistoryOpen]=useState(false);
  const [saving,setSaving]=useState(false);
  const changing=useRef(false);
  const preferencesGeneration=useRef(0);
  const preserveSelection=useRef<string|null>(null);
  const recall=useRef({index:-1,last:""});
  const [query, setQuery] = useState(applicationSmoke ? "Asaaan Checkpoint Probe" : smokeTest ? "cal" : "");
  const [selection, setSelection] = useState(0);
  const [attempt, setAttempt] = useState(0);
  const [poll, setPoll] = useState(0);
  const forceRefresh = useRef(false);
  const launching = useRef(false);
  const acting=useRef(false);
  const [actingKind,setActingKind]=useState<ApplicationAction|null>(null);
  const [actingName,setActingName]=useState<string|null>(null);
  const [actionFeedback,setActionFeedback]=useState<string|null>(null);
  const smokeLaunched = useRef(false);
  const [launchingName, setLaunchingName] = useState<string | null>(null);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [state, setState] = useState<SearchState>({ status: "loading" });
  useEffect(() => { if (isReference || applicationSmoke || state.status !== "loaded") return; const timer = window.setInterval(() => setPoll(n=>n+1),5000); return () => window.clearInterval(timer); },[isReference,applicationSmoke,state.status]);
  useEffect(() => { setSelection(0); setLaunchError(null);setActionFeedback(null); },[query]);
  const inputRef = useRef<HTMLInputElement>(null);
  const lastPresentation=useRef<number|null>(null);
  const selectedIdentity=useRef<string|null>(null);
  useEffect(() => {
    const focus = () => inputRef.current?.focus(); let disposed = false; let unlisten: (() => void) | undefined;
    window.addEventListener("focus", focus);
    void onPanelShown((sequence) => {
      const reopening=lastPresentation.current!==null&&lastPresentation.current!==sequence;
      lastPresentation.current=sequence;
      if(!isReference&&!applicationSmoke){if(reopening&&!extensionOpen.current){preserveSelection.current=selectedIdentity.current;forceRefresh.current=true;setPoll(n=>n+1);}void extensionExchange({kind:"read"}).then(reply=>{if(!disposed){if(!reply.active&&extensionOpen.current)setQuery("");setExtensionView(reply.active);}}).catch(()=>{});}
      setActionFeedback(null);setActionTarget(null);setAliasTarget(null);setCustomizing(false);setHistoryOpen(false);
      // WebKit can retain DOM focus while its editable native focus is lost on hide.
      // Reacquire editing focus on every presentation, including grace-period reuse.
      inputRef.current?.blur();focus();
      void confirmPanelPresented(sequence, document.activeElement === inputRef.current).catch((error: unknown) => { if (!disposed) setState({status: "error", message: String(error)}); });
    },sequence=>{void hidePanel(sequence).catch((error:unknown)=>{if(!disposed)setState({status:"error",message:String(error)});});}).then((stop) => { if (disposed) stop(); else unlisten = stop; }).catch((error: unknown) => { if (!disposed) setState({status: "error", message: String(error)}); });
    return () => { disposed = true; window.removeEventListener("focus", focus); unlisten?.(); };
  }, []);
  const listId = useId();
  const selected = useRef<HTMLLIElement>(null);

  useEffect(() => {
    let cancelled = false;
    const generation=preferencesGeneration.current;
    setState(previous => previous.status === "loaded" && previous.query === query ? previous : {status: "loading"});
    const refresh = forceRefresh.current; forceRefresh.current = false;
    const pending = isReference ? searchReferenceCatalog(query).then(rows=>({rows,total:rows.length,warning:null,favorites:[],queries:[],rememberSearches:true,aliases:{},hiddenResults:{}})) : searchApplications(query,refresh);
    pending.then((reply) => {
      if (!cancelled && generation===preferencesGeneration.current) {
        setState({ status: "loaded", query, rows: reply.rows, total: reply.total, commandTotal:"commandTotal" in reply?Number(reply.commandTotal):0, warning: reply.warning });
        if (!isReference) setPreferences(normalize({favorites:reply.favorites??[],queries:reply.queries??[],rememberSearches:reply.rememberSearches??true,aliases:reply.aliases,hiddenResults:reply.hiddenResults}));
        const id=preserveSelection.current;preserveSelection.current=null;
        setSelection(previous=>{const index=id?reply.rows.findIndex(row=>row.item.id===id):-1;return index>=0?index:Math.min(previous,Math.max(0,reply.rows.length-1));});
      }
    }).catch((error: unknown) => {
      if (!cancelled && generation===preferencesGeneration.current) setState({ status: "error", message: error instanceof Error ? error.message : "Search could not be completed." });
    });
    return () => { cancelled = true; };
  }, [query, attempt, poll, isReference]);

  useEffect(() => {
    let cancelled = false;
    if (isReference && state.status === "loaded" && state.query === query) {
      void reportSearchReady(query, state.rows.map((r) => r.item.id)).catch((error: unknown) => {
        if (!cancelled) setState({ status: "error", message: error instanceof Error ? error.message : "Native search could not be confirmed." });
      });
    }
    return () => { cancelled = true; };
  }, [state, query, isReference]);

  useEffect(() => { selected.current?.scrollIntoView?.({ block: "nearest" }); }, [selection]);

  const rows = state.status === "loaded" && state.query === query ? state.rows.filter(row=>!Object.prototype.hasOwnProperty.call(preferences.hiddenResults,row.item.id)) : [];
  selectedIdentity.current=rows[selection]?.item.id??null;
  useEffect(()=>{setSelection(value=>Math.min(value,Math.max(0,rows.length-1)));},[rows.length]);
  async function open(row: SearchResult | undefined) {
    if (!row || isReference || launching.current || changing.current || acting.current) return;
    launching.current = true; setLaunchingName(row.item.title); setLaunchError(null);
    try { if(row.item.kind==="command"){const reply=await extensionExchange({kind:"start",id:row.item.id,query});if(!reply.active)throw Error("The extension did not start.");setExtensionView(reply.active);return;} const outcome=await launchApplication(row.item.id,query); if(outcome?.warning)setLaunchError(outcome.warning); if (applicationSmoke) await reportApplicationLaunched(row.item.id); }
    catch (error: unknown) { setLaunchError(error instanceof Error ? error.message : typeof error==="string"?error:"The application could not be opened."); }
    finally { launching.current = false; setLaunchingName(null); }
  }
  async function metadataAction(row:SearchResult,action:ApplicationAction,token?:string){
    if(isReference||acting.current||launching.current||changing.current)return;
    acting.current=true;setActingKind(action);setActingName(row.item.title);setActionFeedback(null);setLaunchError(null);setActionTarget(null);inputRef.current?.focus();
    try{const outcome=await (token===undefined?runApplicationAction(row.item.id,action):runApplicationAction(row.item.id,action,token));if(outcome.dismissed){setQuery("");recall.current={index:-1,last:""};}else setActionFeedback(outcome.message);}
    catch(error:unknown){setLaunchError(error instanceof Error?error.message:typeof error==="string"?error:"The application action failed.");}
    finally{acting.current=false;setActingName(null);setActingKind(null);}
  }
  async function change(operation:PersonalizationChange) {
    if(changing.current||isReference||acting.current)return false;changing.current=true;preferencesGeneration.current++;setSaving(true);setLaunchError(null);
    preserveSelection.current=operation.kind==="favorite"||operation.kind==="alias"?operation.id:rows[selection]?.item.id??null;
    try{const next=await changePersonalization(operation);setPreferences(normalize(next));return true;}
    catch(error:unknown){preserveSelection.current=null;setLaunchError(error instanceof Error?error.message:"Could not save the change.");return false;}
    finally{preferencesGeneration.current++;setPoll(n=>n+1);changing.current=false;setSaving(false);if(operation.kind==="favorite")inputRef.current?.focus();}
  }
  function favorite(row=rows[selection]){if(row)void change({kind:"favorite",id:row.item.id,enabled:!preferences.favorites.includes(row.item.id)});}
  useEffect(() => {
    if (applicationSmoke && !smokeLaunched.current && rows.length === 1 && rows[0].item.id === "app:desktop:asaaan-checkpoint-probe.desktop") {smokeLaunched.current = true; void open(rows[0]);}
  },[applicationSmoke,state]);
  useLayoutEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (event.isComposing || event.defaultPrevented || extensionOpen.current) return;
      const activeQuery=inputRef.current?.value??query;
      const primary=(event.ctrlKey||event.metaKey)&&!event.altKey;
      if(aliasTarget||customizing){if(saving&&event.key==="Tab")event.preventDefault();if(event.key==="Escape"&&!saving){event.preventDefault();setAliasTarget(null);setCustomizing(false);inputRef.current?.focus();}return;}
      if(actionTarget){
        if(event.key==="Escape"||(primary&&!event.shiftKey&&event.key.toLowerCase()==="k")){event.preventDefault();setActionTarget(null);inputRef.current?.focus();}
        if(primary&&event.shiftKey&&event.key.toLowerCase()==="f"){event.preventDefault();favorite(actionTarget);setActionTarget(null);}
        return;
      }
      if(!isReference&&primary&&!event.shiftKey&&event.key===","){event.preventDefault();setCustomizing(true);setHistoryOpen(false);setLaunchError(null);return;}
      if(!isReference&&primary&&!event.shiftKey&&event.key.toLowerCase()==="k"){event.preventDefault();if(!acting.current&&state.status==="loaded"&&state.query===activeQuery&&rows[selection]){setActionTarget(rows[selection]);setHistoryOpen(false);setLaunchError(null);}return;}
      if(historyOpen){if(event.key==="Escape"){event.preventDefault();setHistoryOpen(false);inputRef.current?.focus();}return;}
      if (!isReference&&event.key.toLowerCase()==="f"&&event.shiftKey&&(event.ctrlKey||event.metaKey)&&!event.altKey){event.preventDefault();favorite();return;}
      if (!isReference&&event.key==="ArrowUp"&&selection===0&&preferences.queries.length&&(activeQuery===""||activeQuery===recall.current.last)){
        event.preventDefault();recall.current.index=Math.min(recall.current.index+1,preferences.queries.length-1);
        const next=preferences.queries[preferences.queries.length-1-recall.current.index];recall.current.last=next;setQuery(next);return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        if (activeQuery) {recall.current={index:-1,last:""};setQuery("");} else void hidePanel().catch((error: unknown) => setState({status:"error",message:error instanceof Error ? error.message : "Could not hide the launcher."}));
        return;
      }
      if (!rows.length || (event.target instanceof HTMLElement && event.target.closest("button,a"))) return;
      if (event.key === "Enter") { event.preventDefault(); if(state.status==="loaded"&&state.query===activeQuery)void open(rows[selection]); }
      if (event.key === "ArrowDown") { event.preventDefault(); setSelection(value=>Math.min(rows.length-1,value+1)); }
      if (event.key === "ArrowUp") { event.preventDefault(); setSelection(value=>Math.max(0,value-1)); }
    };
    window.addEventListener("keydown",key);
    return () => window.removeEventListener("keydown",key);
  },[query,selection,state,isReference,applicationSmoke,historyOpen,preferences,actionTarget,aliasTarget,customizing,saving]);
  if(extensionView)return <ExtensionPanel key={extensionView.token} initial={extensionView} inputRef={inputRef} onClose={()=>{setExtensionView(null);setQuery("");setPoll(n=>n+1);requestAnimationFrame(()=>inputRef.current?.focus());}}/>;
  return (
    <section className="search-panel" aria-label={isReference ? "Reference catalog search" : "Installed applications"}>
      <div className="search-field">
        <span aria-hidden="true" className="search-symbol">⌕</span>
        <input ref={inputRef} autoFocus type="text" role="combobox" aria-label={isReference ? "Search reference catalog" : "Search applications"} aria-controls={listId} aria-expanded={rows.length > 0} aria-autocomplete="list" aria-activedescendant={rows.length ? listId + "-" + selection : undefined} value={query} maxLength={1024} placeholder={isReference ? "Search the reference catalog…" : "Search applications and commands…"} autoComplete="off" spellCheck={false}
          onChange={(event) => {recall.current={index:-1,last:""};setQuery(event.target.value);}}
          />
        {query && <button className="clear-query" aria-label="Clear search" onClick={() => {recall.current={index:-1,last:""};setQuery("");}}>Clear</button>}
      </div>
      <div className="catalog-notice"><span>{isReference ? "Reference catalog" : state.status === "loaded" ? state.total+" applications"+(state.commandTotal?" · "+state.commandTotal+" commands":"") : "Applications"}</span>{isReference ? <span>Sample items only. Nothing launches.</span> : <span className="catalog-tools"><button className="refresh-apps" onClick={()=>setHistoryOpen(open=>!open)} aria-expanded={historyOpen}>History</button><button className="refresh-apps" onClick={()=>{forceRefresh.current=true;setAttempt(n=>n+1);}}>Refresh applications</button><button className="refresh-apps" onClick={()=>{setCustomizing(true);setHistoryOpen(false);setActionTarget(null);setLaunchError(null);}}>Customize</button></span>}</div>
      {historyOpen&&!isReference&&<aside role="dialog" aria-label="Search history" className="history-popover">
        <div className="history-heading"><strong>Recent searches</strong><button onClick={()=>{setHistoryOpen(false);inputRef.current?.focus();}} aria-label="Close search history">×</button></div>
        <label><input type="checkbox" checked={preferences.rememberSearches} disabled={saving} onChange={event=>void change({kind:"rememberSearches",enabled:event.target.checked})}/> Remember searches</label>
        <p>Searches are saved locally when an application opens. Turning this off keeps existing history.</p>
        <ul>{[...preferences.queries].reverse().map(value=><li key={value}><button onClick={()=>{recall.current={index:-1,last:""};setQuery(value);setHistoryOpen(false);inputRef.current?.focus();}}>{value}</button></li>)}</ul>
        {!preferences.queries.length&&<p>No recent searches.</p>}
        <button disabled={saving||!preferences.queries.length} onClick={()=>void change({kind:"clearQueries"})}>Clear search history</button>
      </aside>}
      {(aliasTarget||customizing)&&<div className="customization-backdrop" aria-hidden="true" onMouseDown={()=>{if(!saving){setAliasTarget(null);setCustomizing(false);inputRef.current?.focus();}}}/>}
      {actionTarget&&<ApplicationActions id={listId+"-actions"} title={actionTarget.item.title} onClose={()=>{setActionTarget(null);inputRef.current?.focus();}} actions={[
        {label:"Open",run:()=>{const target=actionTarget;setActionTarget(null);void open(target);}},
        {label:preferences.aliases[actionTarget.item.id]?"Edit Alias…":"Set Alias…",run:()=>{setAliasTarget(actionTarget);setActionTarget(null);setLaunchError(null);}},
        ...(processOptions?.id===actionTarget.item.id?processOptions.actions:[]).map(action=>({label:action.label,run:()=>void metadataAction(actionTarget,action.kind,action.token)})),
        ...(actionTarget.actions??[]).map(action=>({label:action.label,run:()=>void metadataAction(actionTarget,action.kind)})),
        {label:preferences.favorites.includes(actionTarget.item.id)?"Remove from Favorites":"Add to Favorites",disabled:saving,run:()=>{favorite(actionTarget);setActionTarget(null);}},
        {label:"Hide from Search",disabled:saving,run:()=>{void change({kind:"hide",id:actionTarget.item.id});setActionTarget(null);inputRef.current?.focus();}},
      ]}/>}
      {actionTarget&&processOptions?.id===actionTarget.item.id&&processOptions.notice&&<p className="launch-feedback" role="status">{processOptions.notice}</p>}
      {aliasTarget&&<AliasEditor key={aliasTarget.item.id} title={aliasTarget.item.title} initial={preferences.aliases[aliasTarget.item.id]??""} saving={saving} error={launchError} onSave={alias=>change({kind:"alias",id:aliasTarget.item.id,alias})} onClose={()=>{setAliasTarget(null);inputRef.current?.focus();}}/>}
      {customizing&&<Customization hidden={preferences.hiddenResults} saving={saving} error={launchError} onRestore={id=>change({kind:"restore",id})} onClose={()=>{setCustomizing(false);inputRef.current?.focus();}}/>}
      {actingName&&<p className="launch-feedback" role="status">{actingKind==="restart"?"Restarting ":"Working on "}{actingName}…</p>}
      {actionFeedback&&<p className="launch-feedback" role="status">{actionFeedback}</p>}
      {launchingName && <p className="launch-feedback" role="status">Opening {launchingName}…</p>}
      {launchError && !aliasTarget && !customizing && <p className="launch-feedback error" role="alert">{launchError}</p>}
      {state.status === "loaded" && state.warning && <p className="launch-feedback" role="status">{state.warning}</p>}
      <div className="search-results">
        {state.status === "loading" && <p className="search-message" aria-live="polite">Searching…</p>}
        {state.status === "error" && <div className="search-message"><p className="error" role="alert">{state.message}</p><button onClick={() => setAttempt((n) => n + 1)}>Retry search</button></div>}
        {state.status === "loaded" && !rows.length && <p className="search-message" aria-live="polite">{!isReference && !query ? (Object.keys(preferences.hiddenResults).length?"No visible applications. Restore hidden applications in Customize.":"No applications found. Refresh after installing an application.") : "No matches for “"+query+"”"}</p>}
        <ul id={listId} role="listbox" aria-label="Search results">
          {rows.map((row, index) => (
            <li key={row.item.id} id={listId + "-" + index} role="option" aria-selected={selection === index} ref={selection === index ? selected : undefined} onMouseDown={(event) => event.preventDefault()} onClick={() => { setSelection(index); inputRef.current?.focus(); }} onContextMenu={event=>{event.preventDefault();setSelection(index);setActionTarget(row);setHistoryOpen(false);setLaunchError(null);}} onDoubleClick={() => { void open(row); }} className={selection === index ? "result selected" : "result"}>
              {isReference ? <span className="result-icon" aria-hidden="true">{row.item.title[0]}</span> : row.item.kind==="command"?<span className="result-icon" aria-hidden="true">⌘</span>:<ApplicationIcon id={row.item.id} title={row.item.title} />}
              <span className="result-title">{row.item.title}</span>{!isReference&&preferences.favorites.includes(row.item.id)&&<span className="favorite-marker" aria-label="Favorite">★</span>}
              {!isReference&&preferences.aliases[row.item.id]&&<span className="alias-chip" aria-label="Alias">{preferences.aliases[row.item.id]}</span>}
              <span className="result-kind">{row.origin ?? row.section ?? (row.item.kind === "application" ? "Application" : "Command")}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="search-help"><span>{isReference ? <>Try <kbd>cal</kbd>, <kbd>hn</kbd> or <kbd>vsc</kbd></> : <><kbd>Enter</kbd> to open <button className="favorite-action" disabled={!rows.length||saving||Boolean(actingName)} onMouseDown={event=>event.preventDefault()} onClick={()=>favorite()}>{preferences.favorites.includes(rows[selection]?.item.id)?"Remove from Favorites":"Add to Favorites"}</button><button className="favorite-action" aria-haspopup="menu" aria-expanded={Boolean(actionTarget)} aria-controls={actionTarget?listId+"-actions":undefined} disabled={!rows[selection]||saving||Boolean(actingName)} onMouseDown={event=>event.preventDefault()} onClick={()=>{setActionTarget(rows[selection]);setHistoryOpen(false);setLaunchError(null);}}>Actions</button></>}</span><span><kbd>↑</kbd> <kbd>↓</kbd> select <kbd>Esc</kbd> clear / hide</span></div>
    </section>
  );
}

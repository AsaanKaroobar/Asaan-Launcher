import ApplicationIcon from "./ApplicationIcon";
import { useEffect, useId, useRef, useState } from "react";
import { searchApplications, launchApplication, reportApplicationLaunched, confirmPanelPresented, hidePanel, onPanelShown, reportSearchReady, searchReferenceCatalog, type SearchResult } from "./native";

type SearchState = { status: "loading" } | { status: "loaded"; query: string; rows: SearchResult[]; total?: number; warning?: string | null } | { status: "error"; message: string };

export default function SearchPanel({ smokeTest = false, referenceCatalog = false, applicationSmoke = false }: { smokeTest?: boolean; referenceCatalog?: boolean; applicationSmoke?: boolean }) {
  const isReference = referenceCatalog || smokeTest;
  const [query, setQuery] = useState(applicationSmoke ? "Asaaan Checkpoint Probe" : smokeTest ? "cal" : "");
  const [selection, setSelection] = useState(0);
  const [attempt, setAttempt] = useState(0);
  const [poll, setPoll] = useState(0);
  const forceRefresh = useRef(false);
  const launching = useRef(false);
  const smokeLaunched = useRef(false);
  const [launchingName, setLaunchingName] = useState<string | null>(null);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [state, setState] = useState<SearchState>({ status: "loading" });
  useEffect(() => { if (isReference || applicationSmoke || state.status !== "loaded") return; const timer = window.setInterval(() => setPoll(n=>n+1),5000); return () => window.clearInterval(timer); },[isReference,applicationSmoke,state.status]);
  useEffect(() => { setSelection(0); setLaunchError(null); },[query]);
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const focus = () => inputRef.current?.focus(); let disposed = false; let unlisten: (() => void) | undefined;
    window.addEventListener("focus", focus);
    void onPanelShown((sequence) => {
      focus();
      void confirmPanelPresented(sequence, document.activeElement === inputRef.current).catch((error: unknown) => { if (!disposed) setState({status: "error", message: String(error)}); });
    }).then((stop) => { if (disposed) stop(); else unlisten = stop; }).catch((error: unknown) => { if (!disposed) setState({status: "error", message: String(error)}); });
    return () => { disposed = true; window.removeEventListener("focus", focus); unlisten?.(); };
  }, []);
  const listId = useId();
  const selected = useRef<HTMLLIElement>(null);

  useEffect(() => {
    let cancelled = false;
    setState(previous => previous.status === "loaded" && previous.query === query ? previous : {status: "loading"});
    const refresh = forceRefresh.current; forceRefresh.current = false;
    const pending = isReference ? searchReferenceCatalog(query).then(rows=>({rows,total:rows.length,warning:null})) : searchApplications(query,refresh);
    pending.then((reply) => {
      if (!cancelled) {setState({ status: "loaded", query, rows: reply.rows, total: reply.total, warning: reply.warning }); setSelection(previous=>Math.min(previous,Math.max(0,reply.rows.length-1)));}
    }).catch((error: unknown) => {
      if (!cancelled) setState({ status: "error", message: error instanceof Error ? error.message : "Search could not be completed." });
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

  const rows = state.status === "loaded" && state.query === query ? state.rows : [];
  async function open(row: SearchResult | undefined) {
    if (!row || isReference || launching.current) return;
    launching.current = true; setLaunchingName(row.item.title); setLaunchError(null);
    try { await launchApplication(row.item.id); if (applicationSmoke) await reportApplicationLaunched(row.item.id); }
    catch (error: unknown) { setLaunchError(error instanceof Error ? error.message : "The application could not be opened."); }
    finally { launching.current = false; setLaunchingName(null); }
  }
  useEffect(() => {
    if (applicationSmoke && !smokeLaunched.current && rows.length === 1 && rows[0].item.id === "app:desktop:asaaan-checkpoint-probe.desktop") {smokeLaunched.current = true; void open(rows[0]);}
  },[applicationSmoke,state]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (event.isComposing || event.defaultPrevented) return;
      if (event.key === "Escape") {
        event.preventDefault();
        if (query) setQuery(""); else void hidePanel().catch((error: unknown) => setState({status:"error",message:error instanceof Error ? error.message : "Could not hide the launcher."}));
        return;
      }
      if (!rows.length || (event.target instanceof HTMLElement && event.target.closest("button,a"))) return;
      if (event.key === "Enter") { event.preventDefault(); void open(rows[selection]); }
      if (event.key === "ArrowDown") { event.preventDefault(); setSelection(value=>Math.min(rows.length-1,value+1)); }
      if (event.key === "ArrowUp") { event.preventDefault(); setSelection(value=>Math.max(0,value-1)); }
    };
    window.addEventListener("keydown",key);
    return () => window.removeEventListener("keydown",key);
  },[query,selection,state,isReference,applicationSmoke]);
  return (
    <section className="search-panel" aria-label={isReference ? "Reference catalog search" : "Installed applications"}>
      <div className="search-field">
        <span aria-hidden="true" className="search-symbol">⌕</span>
        <input ref={inputRef} autoFocus type="text" role="combobox" aria-label={isReference ? "Search reference catalog" : "Search applications"} aria-controls={listId} aria-expanded={rows.length > 0} aria-autocomplete="list" aria-activedescendant={rows.length ? listId + "-" + selection : undefined} value={query} placeholder={isReference ? "Search the reference catalog…" : "Search applications…"} autoComplete="off" spellCheck={false}
          onChange={(event) => setQuery(event.target.value)}
          />
        {query && <button className="clear-query" aria-label="Clear search" onClick={() => setQuery("")}>Clear</button>}
      </div>
      <div className="catalog-notice"><span>{isReference ? "Reference catalog" : state.status === "loaded" ? state.total+" applications" : "Applications"}</span>{isReference ? <span>Sample items only. Nothing launches.</span> : <button className="refresh-apps" onClick={()=>{forceRefresh.current=true;setAttempt(n=>n+1);}}>Refresh applications</button>}</div>
      {launchingName && <p className="launch-feedback" role="status">Opening {launchingName}…</p>}
      {launchError && <p className="launch-feedback error" role="alert">{launchError}</p>}
      {state.status === "loaded" && state.warning && <p className="launch-feedback" role="status">{state.warning}</p>}
      <div className="search-results">
        {state.status === "loading" && <p className="search-message" aria-live="polite">Searching…</p>}
        {state.status === "error" && <div className="search-message"><p className="error" role="alert">{state.message}</p><button onClick={() => setAttempt((n) => n + 1)}>Retry search</button></div>}
        {state.status === "loaded" && !rows.length && <p className="search-message" aria-live="polite">{!isReference && !query ? "No applications found. Refresh after installing an application." : "No matches for “"+query+"”"}</p>}
        <ul id={listId} role="listbox" aria-label="Search results">
          {rows.map((row, index) => (
            <li key={row.item.id} id={listId + "-" + index} role="option" aria-selected={selection === index} ref={selection === index ? selected : undefined} onMouseDown={(event) => event.preventDefault()} onClick={() => { setSelection(index); inputRef.current?.focus(); }} onDoubleClick={() => { void open(row); }} className={selection === index ? "result selected" : "result"}>
              {isReference ? <span className="result-icon" aria-hidden="true">{row.item.title[0]}</span> : <ApplicationIcon id={row.item.id} title={row.item.title} />}
              <span className="result-title">{row.item.title}</span>
              <span className="result-kind">{row.origin ?? row.section ?? (row.item.kind === "application" ? "Application" : "Command")}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="search-help"><span>{isReference ? <>Try <kbd>cal</kbd>, <kbd>hn</kbd> or <kbd>vsc</kbd></> : <><kbd>Enter</kbd> or double-click to open</>}</span><span><kbd>↑</kbd> <kbd>↓</kbd> select <kbd>Esc</kbd> clear / hide</span></div>
    </section>
  );
}

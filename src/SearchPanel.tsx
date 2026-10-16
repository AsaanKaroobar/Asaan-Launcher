import { useEffect, useId, useRef, useState } from "react";
import { reportSearchReady, searchReferenceCatalog, type SearchResult } from "./native";

type SearchState = { status: "loading" } | { status: "loaded"; query: string; rows: SearchResult[] } | { status: "error"; message: string };

export default function SearchPanel({ smokeTest = false }: { smokeTest?: boolean }) {
  const [query, setQuery] = useState(smokeTest ? "cal" : "");
  const [selection, setSelection] = useState(0);
  const [attempt, setAttempt] = useState(0);
  const [state, setState] = useState<SearchState>({ status: "loading" });
  const listId = useId();
  const selected = useRef<HTMLLIElement>(null);

  useEffect(() => {
    let cancelled = false;
    setState({ status: "loading" });
    setSelection(0);
    searchReferenceCatalog(query).then((rows) => {
      if (!cancelled) setState({ status: "loaded", query, rows });
    }).catch((error: unknown) => {
      if (!cancelled) setState({ status: "error", message: error instanceof Error ? error.message : "Search could not be completed." });
    });
    return () => { cancelled = true; };
  }, [query, attempt]);

  useEffect(() => {
    let cancelled = false;
    if (state.status === "loaded" && state.query === query) {
      void reportSearchReady(query, state.rows.map((r) => r.item.id)).catch((error: unknown) => {
        if (!cancelled) setState({ status: "error", message: error instanceof Error ? error.message : "Native search could not be confirmed." });
      });
    }
    return () => { cancelled = true; };
  }, [state, query]);

  useEffect(() => { selected.current?.scrollIntoView?.({ block: "nearest" }); }, [selection]);

  const rows = state.status === "loaded" && state.query === query ? state.rows : [];
  return (
    <section className="search-panel" aria-label="Reference catalog search">
      <div className="search-field">
        <span aria-hidden="true" className="search-symbol">⌕</span>
        <input autoFocus type="text" role="combobox" aria-label="Search reference catalog" aria-controls={listId} aria-expanded={rows.length > 0} aria-autocomplete="list" aria-activedescendant={rows.length ? listId + "-" + selection : undefined} value={query} placeholder="Search the reference catalog…" autoComplete="off" spellCheck={false}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.nativeEvent.isComposing) return;
            if (event.key === "Escape") { event.preventDefault(); setQuery(""); return; }
            if (!rows.length) return;
            if (event.key === "ArrowDown") { event.preventDefault(); setSelection((s) => Math.min(rows.length - 1, s + 1)); }
            if (event.key === "ArrowUp") { event.preventDefault(); setSelection((s) => Math.max(0, s - 1)); }
          }} />
        {query && <button className="clear-query" aria-label="Clear search" onClick={() => setQuery("")}>Clear</button>}
      </div>
      <div className="catalog-notice"><span>Reference catalog</span><span>Sample items only. Nothing launches.</span></div>
      <div className="search-results">
        {state.status === "loading" && <p className="search-message" aria-live="polite">Searching…</p>}
        {state.status === "error" && <div className="search-message"><p className="error" role="alert">{state.message}</p><button onClick={() => setAttempt((n) => n + 1)}>Retry search</button></div>}
        {state.status === "loaded" && !rows.length && <p className="search-message" aria-live="polite">No matches for “{query}”</p>}
        <ul id={listId} role="listbox" aria-label="Search results">
          {rows.map((row, index) => (
            <li key={row.item.id} id={listId + "-" + index} role="option" aria-selected={selection === index} ref={selection === index ? selected : undefined} onClick={() => setSelection(index)} className={selection === index ? "result selected" : "result"}>
              <span className="result-icon" aria-hidden="true">{row.item.title[0]}</span>
              <span className="result-title">{row.item.title}</span>
              <span className="result-kind">{row.section ?? (row.item.kind === "application" ? "Application" : "Command")}</span>
            </li>
          ))}
        </ul>
      </div>
      <div className="search-help"><span>Try <kbd>cal</kbd>, <kbd>hn</kbd> or <kbd>vsc</kbd></span><span><kbd>↑</kbd> <kbd>↓</kbd> select <kbd>Esc</kbd> clear</span></div>
    </section>
  );
}

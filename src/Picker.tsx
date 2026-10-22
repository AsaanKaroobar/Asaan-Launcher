import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
export interface PickItem {index:number;text:string;title:string;detail:string|null}
export interface PickView {prompt:string;query:string;rows:PickItem[];total:number;count:string}
declare global {interface Window {__ASAAAN_PICKER__?:boolean}}
export default function Picker(){
  const [view,setView]=useState<PickView|null>(null),[query,setQuery]=useState<string|null>(null),[selection,setSelection]=useState(0),[error,setError]=useState("");
  const field=useRef<HTMLInputElement>(null), finishing=useRef(false), composing=useRef(false), initial=useRef(true), liveQuery=useRef("");
  useEffect(()=>{let cancelled=false; invoke<PickView>("picker_view",{query}).then(next=>{if(!cancelled){setView(next);setSelection(0);liveQuery.current=next.query;}},e=>{if(!cancelled)setError(String(e));}); return()=>{cancelled=true;};},[query]);
  useLayoutEffect(()=>{if(!view||!initial.current)return;initial.current=false;field.current?.focus();field.current?.setSelectionRange(view.query.length,view.query.length);const retry=window.setTimeout(()=>field.current?.focus(),200);return()=>window.clearTimeout(retry);},[view]);
  useEffect(()=>{const focus=()=>field.current?.focus();window.addEventListener("focus",focus);return()=>window.removeEventListener("focus",focus);},[]);
  useEffect(()=>{document.getElementById("pick-"+view?.rows[selection]?.index)?.scrollIntoView?.({block:"nearest"});},[view,selection]);
  async function finish(item?:PickItem){if(finishing.current||item&&view?.query!==liveQuery.current)return;finishing.current=true;try{await invoke("picker_finish",{index:item?.index??null,query:liveQuery.current});}catch(e){finishing.current=false;setError(String(e));}}
  const ready=view?.query===liveQuery.current;
  return <main className="launcher picker" aria-labelledby="picker-title" onKeyDown={e=>{
      if(composing.current||e.nativeEvent.isComposing||e.keyCode===229)return;
      if(e.key==="Escape"){e.preventDefault();void finish();return;}
      if(e.key==="Enter"){e.preventDefault();if(ready&&view?.rows[selection])void finish(view.rows[selection]);return;}
      const delta:Record<string,number>={ArrowDown:1,ArrowUp:-1,PageDown:9,PageUp:-9,Home:-Number.MAX_SAFE_INTEGER,End:Number.MAX_SAFE_INTEGER};
      if(e.key in delta){e.preventDefault();setSelection(s=>Math.max(0,Math.min((view?.rows.length??0)-1,s+delta[e.key])));}
    }}><header className="topbar"><span className="wordmark">asaaan<span aria-hidden="true">.</span></span><h1 id="picker-title" className="product-type">Choose</h1></header><section className="search-panel">
    <label className="search-field"><span className="search-symbol" aria-hidden="true">⌕</span><input ref={field} aria-label={view?.prompt??"Search…"} placeholder={view?.prompt??"Search…"} value={query??view?.query??""} autoComplete="off" spellCheck={false} role="combobox" aria-controls="pick-results" aria-expanded="true" aria-activedescendant={ready&&view?.rows[selection]?"pick-"+view.rows[selection].index:undefined} onCompositionStart={()=>{composing.current=true;}} onCompositionEnd={()=>{composing.current=false;}} onChange={e=>{liveQuery.current=e.target.value;setQuery(e.target.value);setSelection(0);}}/></label>
    <div className="search-results" id="pick-results" role="listbox" aria-label="Input lines">{view?.rows.map((item,i)=><div id={"pick-"+item.index} key={item.index} role="option" aria-label={item.title+(item.detail?" "+item.detail:"")} aria-selected={i===selection} className={"result"+(i===selection?" selected":"")} onMouseDown={e=>e.preventDefault()} onClick={()=>setSelection(i)} onDoubleClick={()=>{void finish(item);}}><span className="result-title">{item.title}</span>{item.detail!==null&&<span className="result-kind">{item.detail}</span>}</div>)}{view?.rows.length===0&&<p className="search-message">Nothing matches. Try fewer letters.</p>}</div>
    {error&&<p role="alert" className="error launch-feedback">{error}</p>}
    <div className="search-help"><span role="status">{view?.count??"Loading…"}</span><span><button onClick={()=>{void finish();}}>Cancel <kbd>esc</kbd></button> {view?.rows[selection]&&<button disabled={!ready} onClick={()=>{void finish(view.rows[selection]);}}>Choose <kbd>↵</kbd></button>}</span></div>
  </section><footer><span>Asaaan Launcher</span><span>Picker</span></footer></main>;
}

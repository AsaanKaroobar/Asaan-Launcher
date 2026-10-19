import {Fragment,useEffect,useLayoutEffect,useId,useRef,useState,type RefObject} from "react";
import {extensionExchange,type ExtensionView,type ExtensionSnapshot,type ExtensionAction,type ExtensionOperation} from "./extension-native";
export const flattenActions=(entries:ExtensionAction[]):ExtensionAction[]=>entries.flatMap(e=>e.submenu?flattenActions(e.children):[e]);
export function menuEntries(entries:ExtensionAction[],query:string):ExtensionAction[]{
 return query?flattenActions(entries).filter(e=>e.title.toLowerCase().includes(query.toLowerCase())):entries;
}
export function matchesShortcut(value:unknown,event:Pick<KeyboardEvent,"key"|"metaKey"|"ctrlKey"|"altKey"|"shiftKey">,mac=typeof navigator!=="undefined"&&/Mac/.test(navigator.platform)):boolean{
 if(!value||typeof value!=="object")return false;
 const shortcut=value as {key?:unknown;modifiers?:unknown};
 if(typeof shortcut.key!=="string"||!Array.isArray(shortcut.modifiers)||!shortcut.modifiers.every(m=>typeof m==="string"))return false;
 const modifiers=new Set(shortcut.modifiers);
 const command=modifiers.has("cmd")||modifiers.has("command");
 const ctrl=modifiers.has("ctrl")||modifiers.has("control")||(!mac&&command);
 const meta=modifiers.has("meta")||(mac&&command);
 const alt=modifiers.has("alt")||modifiers.has("opt")||modifiers.has("option");
 const normalize=(key:string)=>key==="return"?"enter":key.toLowerCase();
 return normalize(shortcut.key)===normalize(event.key)&&event.ctrlKey===ctrl&&event.metaKey===meta&&event.altKey===alt&&event.shiftKey===modifiers.has("shift");
}
export function ExtensionIcon({value,title}:{value:string|null;title:string}){
 const kind=(value??"").split(":").pop()?.toLowerCase();
 if(kind==="globe")return <span className="extension-icon" role="img" aria-label="Globe"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="12" cy="12" r="9"/><ellipse cx="12" cy="12" rx="4" ry="9"/><path d="M3 12h18M5 6.5h14M5 17.5h14"/></svg></span>;
 if(kind==="star")return <span className="extension-icon marked" role="img" aria-label="Star"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="m12 3 2.8 5.7 6.3.9-4.5 4.4 1 6.2-5.6-3-5.6 3 1-6.2L3 9.6l6.2-.9Z"/></svg></span>;
 return <span className="extension-icon" aria-hidden="true">{Array.from(title)[0]??"⌘"}</span>;
}
export default function ExtensionPanel({initial,inputRef,onClose}:{initial:ExtensionView;inputRef:RefObject<HTMLInputElement|null>;onClose:()=>void}){
 const[view,setView]=useState(initial);const current=useRef(initial);
 const[draft,setDraft]=useState(initial.query);const draftRef=useRef(initial.query);
 const[selected,setSelected]=useState(initial.selected);const selectedRef=useRef(initial.selected);
 const[error,setError]=useState<string|null>(null);
 const[menu,setMenu]=useState(false);const[menuQuery,setMenuQuery]=useState("");const[path,setPath]=useState<number[]>([]);
 const[menuSelection,setMenuSelection]=useState(0);const menuSelectionRef=useRef(0);
 const menuInput=useRef<HTMLInputElement>(null);const selectedElement=useRef<HTMLLIElement>(null);
 const id=useId();const alive=useRef(true);const pending=useRef(0);const intent=useRef(0);
 const chain=useRef<Promise<void>>(Promise.resolve());const actionPending=useRef(false);
 const closing=useRef(false);const onCloseRef=useRef(onClose);onCloseRef.current=onClose;
 const menuRef=useRef<HTMLElement>(null);
 function apply(reply:ExtensionSnapshot,latest:boolean){
  if(!alive.current)return;
  const next=reply.active;
  if(!next){closing.current=true;onCloseRef.current();return;}
  if(next.token!==current.current.token&&next.commandId!==current.current.commandId){closing.current=true;onCloseRef.current();return;}
  if(next.token!==current.current.token||next.revision>=current.current.revision){
   const screenChanged=next.screen!==current.current.screen||next.token!==current.current.token;
   current.current=next;
   if(latest){
    setView(next);setDraft(next.query);draftRef.current=next.query;setSelected(next.selected);selectedRef.current=next.selected;
    if(screenChanged){setMenu(false);setPath([]);setMenuQuery("");inputRef.current?.focus();}
   }
  }
 }
 function queue(build:ExtensionOperation|((snapshot:ExtensionView)=>ExtensionOperation|null),after?:(snapshot:ExtensionView)=>void):Promise<void>{
  const sequence=++intent.current;pending.current++;
  const work=chain.current.then(async()=>{
   if(!alive.current||closing.current)return;
   const operation=typeof build==="function"?build(current.current):build;if(!operation)return;
   const reply=await extensionExchange(operation);
   apply(reply,sequence===intent.current);
   if(alive.current&&sequence===intent.current&&reply.active)after?.(reply.active);
  }).catch((error:unknown)=>{if(alive.current){setError(error instanceof Error?error.message:String(error));}}).finally(()=>{pending.current--;});
  chain.current=work;return work;
 }
 useEffect(()=>{
  alive.current=true;inputRef.current?.focus();
  let inFlight=false;
  const poll=async()=>{
   if(inFlight||pending.current||closing.current)return;inFlight=true;const sequence=intent.current;
   try{const reply=await extensionExchange({kind:"read"});if(alive.current&&sequence===intent.current)apply(reply,true);}
   catch(error:unknown){if(alive.current)setError(error instanceof Error?error.message:String(error));}
   finally{inFlight=false;}
  };
  const timer=window.setInterval(()=>void poll(),150);
  return()=>{alive.current=false;window.clearInterval(timer);};
 },[]);
 useEffect(()=>{if(menu)menuInput.current?.focus();else inputRef.current?.focus();},[menu]);
 useEffect(()=>{selectedElement.current?.scrollIntoView?.({block:"nearest"});},[selected]);
 let container=view.actions;
 for(const part of path){const found=container.find(e=>e.submenu&&e.id===part);if(!found){container=[];break;}container=found.children;}
 const entries=menuEntries(container,menuQuery);
 const entriesRef=useRef(entries);entriesRef.current=entries;
 useEffect(()=>{
  let container=current.current.actions;for(const part of path){const found=container.find(e=>e.submenu&&e.id===part);if(!found){setPath([]);setMenuSelection(0);menuSelectionRef.current=0;return;}container=found.children;}
  const next=Math.min(menuSelectionRef.current,Math.max(0,entries.length-1));menuSelectionRef.current=next;setMenuSelection(next);
 },[view.revision,entries.length,path]);
 useEffect(()=>{
  if(!menu)return;const outside=(event:PointerEvent)=>{if(!menuRef.current?.contains(event.target as Node)){setMenu(false);setPath([]);setMenuQuery("");}};document.addEventListener("pointerdown",outside);return()=>document.removeEventListener("pointerdown",outside);
 },[menu]);
 function search(text:string){setError(null);setDraft(text);draftRef.current=text;setSelected(0);selectedRef.current=0;setMenu(false);void queue(v=>({kind:"search",token:v.token,text}));}
 function select(index:number){
  const query=inputRef.current?.value??draftRef.current;
  if(current.current.query!==query||!view.rows[index])return;
  setSelected(index);selectedRef.current=index;
  void queue({kind:"select",token:view.token,id:view.rows[index].id,query});
 }
 function activate(index=0,shortcut?:KeyboardEvent){
  if(actionPending.current||error)return;
  const query=inputRef.current?.value??draftRef.current;
  const rowId=view.rows[selectedRef.current]?.id??null;
  actionPending.current=true;
  setMenu(false);setPath([]);setMenuQuery("");
  void queue(v=>{
   if(v.query!==query||(v.rows[v.selected]?.id??null)!==rowId)return null;
   const actions=flattenActions(v.actions);
   const action=shortcut?actions.find(a=>matchesShortcut(a.shortcut,shortcut)):actions[index];
   return action?{kind:"run",token:v.token,id:action.id,revision:action.revision,query}:null;
  }).finally(()=>{actionPending.current=false;});
 }
 function run(entry:ExtensionAction){
  if(entry.submenu){
   setPath(parts=>[...parts,entry.id]);setMenuQuery("");setMenuSelection(0);menuSelectionRef.current=0;
   void queue({kind:"openMenu",token:view.token,id:entry.id,revision:entry.revision,query:draftRef.current});return;
  }
  if(actionPending.current)return;actionPending.current=true;
  setMenu(false);setPath([]);setMenuQuery("");
  void queue({kind:"run",token:view.token,id:entry.id,revision:entry.revision,query:draftRef.current}).finally(()=>{actionPending.current=false;inputRef.current?.focus();});
 }
 function back(){setError(null);setMenu(false);void queue(v=>({kind:error?"stop":"back",token:v.token}));}
 function retry(){setError(null);setMenu(false);void queue(v=>({kind:"retry",token:v.token}));}
 function openMenu(){if(menu){setMenu(false);return;}void queue({kind:"read"},()=>{setMenu(true);setPath([]);setMenuQuery("");setMenuSelection(0);menuSelectionRef.current=0;});}
 useLayoutEffect(()=>{
  const key=(event:KeyboardEvent)=>{
   if(event.isComposing||event.defaultPrevented)return;
   const primary=(event.ctrlKey||event.metaKey)&&!event.altKey;
   if(view.failure||error){
    if(event.key==="Escape"){event.preventDefault();back();}
    else if(primary&&event.key.toLowerCase()==="r"){event.preventDefault();retry();}
    return;
   }
   const modified=event.ctrlKey||event.metaKey||event.altKey||event.shiftKey;
   const declared=modified&&flattenActions(current.current.actions).some(a=>matchesShortcut(a.shortcut,event));
   if(declared){event.preventDefault();activate(0,event);return;}
   if(primary&&event.key.toLowerCase()==="k"){event.preventDefault();openMenu();return;}
   if(menu){
    if(event.key==="Escape"||event.key==="ArrowLeft"){
     event.preventDefault();
     if(menuQuery)setMenuQuery("");else if(path.length)setPath(parts=>parts.slice(0,-1));else setMenu(false);
     menuSelectionRef.current=0;setMenuSelection(0);return;
    }
    if(event.key==="ArrowDown"||event.key==="ArrowUp"){
     event.preventDefault();const next=Math.max(0,Math.min(entriesRef.current.length-1,menuSelectionRef.current+(event.key==="ArrowDown"?1:-1)));menuSelectionRef.current=next;setMenuSelection(next);return;
    }
    if(event.key==="Enter"||event.key==="ArrowRight"){
     const entry=entriesRef.current[menuSelectionRef.current];if(entry&&(event.key==="Enter"||entry.submenu)){event.preventDefault();run(entry);}return;
    }
    return;
   }
   if(event.key==="Escape"||(primary&&event.key==="[")){event.preventDefault();back();return;}
   if(event.key==="Enter"){event.preventDefault();activate(primary&&!event.shiftKey?1:0);return;}
   if(event.target instanceof HTMLElement&&event.target.closest("button"))return;
   const delta=event.key==="ArrowDown"?1:event.key==="ArrowUp"?-1:event.key==="PageDown"?10:event.key==="PageUp"?-10:event.key==="Home"?-Number.MAX_SAFE_INTEGER:event.key==="End"?Number.MAX_SAFE_INTEGER:0;
   if(delta){event.preventDefault();select(Math.max(0,Math.min(view.rows.length-1,selectedRef.current+delta)));}
  };
  window.addEventListener("keydown",key);return()=>window.removeEventListener("keydown",key);
 },[view,draft,menu,path,menuQuery,error]);
 const ready=view.query===draft;
 const rows=ready?view.rows:[];
 const listLike=view.kind==="List"||view.kind==="Grid";
 const failure=view.failure?.message??error;
 const primary=flattenActions(view.actions)[0];
 return <section className="search-panel extension-panel" aria-label={view.title} aria-busy={view.loading}>
  <div className="search-field"><button className="extension-back" aria-label="Back to launcher" onClick={back}>‹</button><input ref={inputRef} autoFocus readOnly={Boolean(failure)} role="combobox" aria-label="Search extension" aria-autocomplete="list" aria-expanded={rows.length>0} aria-controls={id} aria-activedescendant={rows[selected]?id+"-"+rows[selected].id:undefined} value={draft} placeholder={view.placeholder} autoComplete="off" spellCheck={false} maxLength={1024} onChange={event=>search(event.target.value)}/>{draft&&<button className="clear-query" aria-label="Clear extension search" onClick={()=>search("")}>Clear</button>}</div>
  {view.loading&&<div role="progressbar" aria-label="Loading extension" className="extension-loading"/>}
  <div className="catalog-notice"><span>{view.title}</span><span>{listLike?rows.length+" results":"Extension"}</span></div>
  <div className="search-results">
   {failure?<div className="search-message extension-failure"><p role="alert" className="error">{failure}</p><button onClick={retry}>Retry extension</button><button onClick={()=>{setError(null);void queue(v=>({kind:"stop",token:v.token}));}}>Return to search</button></div>
   :view.kind&&!listLike?<div className="search-message"><p>{view.kind} views are not available in this checkpoint.</p><button onClick={back}>Back to list</button></div>
   :<><ul id={id} role="listbox" aria-label="Extension results">{rows.map((row,index)=><Fragment key={row.id}>{row.section&&(index===0||rows[index-1].section!==row.section)&&<li role="presentation" className="extension-section">{row.section}</li>}<li id={id+"-"+row.id} role="option" aria-label={[row.title,row.subtitle,...row.accessories,row.icon?.split(":").pop()?.toLowerCase()==="star"?"Star":null].filter(Boolean).join(", ")} aria-selected={selected===index} ref={selected===index?selectedElement:undefined} className={selected===index?"result selected":"result"} onMouseDown={event=>event.preventDefault()} onClick={()=>{select(index);inputRef.current?.focus();}} onDoubleClick={()=>{select(index);activate();}}>
    <ExtensionIcon value={row.icon} title={row.title}/><span className="result-title">{row.title}</span>{row.subtitle&&<span className="extension-subtitle">{row.subtitle}</span>}<span className="extension-accessories">{row.accessories.map((text,i)=><span key={i}>{text}</span>)}</span>
   </li></Fragment>)}</ul>{!rows.length&&!view.loading&&<div className="search-message"><strong>{view.emptyTitle}</strong>{view.emptyDescription&&<p>{view.emptyDescription}</p>}</div>}</>}
  </div>
  {menu&&!failure&&<aside ref={menuRef} className="extension-actions" role="menu" aria-label="Extension actions"><div className="extension-action-search">{path.length>0&&<button aria-label="Back in actions" onClick={()=>{setPath(parts=>parts.slice(0,-1));setMenuQuery("");}}>‹</button>}<input ref={menuInput} aria-label="Search extension actions" placeholder="Search actions…" value={menuQuery} onChange={event=>{setMenuQuery(event.target.value);setMenuSelection(0);menuSelectionRef.current=0;}}/></div>{entries.map((entry,index)=><div key={entry.id}>{entry.section&&(!index||entries[index-1].section!==entry.section)&&<span className="extension-menu-section">{entry.section}</span>}<button role="menuitem" aria-label={entry.title} className={index===menuSelection?"selected":""} onMouseEnter={()=>{setMenuSelection(index);menuSelectionRef.current=index;}} onClick={()=>run(entry)}>{entry.title}{entry.submenu&&<span aria-hidden="true">›</span>}</button></div>)}{!entries.length&&<p>No matching actions</p>}</aside>}
  <div className="search-help extension-footer"><span role="status" className={view.toast?.style==="failure"?"error":""}>{view.toast?<>{view.toast.title}{view.toast.message&&" · "+view.toast.message}</>:view.title}</span><span><button className="favorite-action" disabled={!primary||Boolean(failure)||!ready} onClick={()=>activate()}>{primary?.title??"No action"} <kbd>↵</kbd></button><button className="favorite-action" aria-haspopup="menu" aria-expanded={menu} disabled={!view.actions.length||Boolean(failure)||!ready} onClick={openMenu}>Actions</button></span></div>
 </section>;
}

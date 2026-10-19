import {useEffect,useRef,useState,type KeyboardEvent} from "react";
function trap(event:KeyboardEvent<HTMLElement>){if(event.key!=="Tab")return;event.preventDefault();const fields=Array.from(event.currentTarget.querySelectorAll<HTMLElement>("button:not(:disabled),input:not(:disabled)"));if(!fields.length)return;const index=fields.indexOf(document.activeElement as HTMLElement);fields[(index+(event.shiftKey?-1:1)+fields.length)%fields.length].focus();}
export interface MenuAction {label:string;run:()=>void;disabled?:boolean}
export function ApplicationActions({id,title,actions,onClose}:{id:string;title:string;actions:MenuAction[];onClose:()=>void}){
 const menu=useRef<HTMLElement>(null);
 useEffect(()=>{const outside=(event:PointerEvent)=>{if(!menu.current?.contains(event.target as Node))onClose();};document.addEventListener("pointerdown",outside);return()=>document.removeEventListener("pointerdown",outside);},[onClose]);
 const [selection,setSelection]=useState(0);const buttons=useRef<Array<HTMLButtonElement|null>>([]);
 useEffect(()=>{buttons.current[selection]?.focus();},[selection]);
 return <aside id={id} ref={menu} role="menu" aria-label={title+" actions"} className="actions-popover" onKeyDown={event=>{
  const move=(value:number)=>{event.preventDefault();event.stopPropagation();buttons.current[value]?.focus();setSelection(value);};
  const focused=buttons.current.findIndex(button=>button===document.activeElement);
  const current=focused<0?selection:focused;
  if(event.key==="ArrowDown")move((current+1)%actions.length);
  if(event.key==="ArrowUp")move((current+actions.length-1)%actions.length);
  if(event.key==="Home")move(0);if(event.key==="End")move(actions.length-1);
  if(event.key==="Escape"){event.preventDefault();event.stopPropagation();onClose();}
  if(event.key==="Tab")onClose();
 }}><strong>{title}</strong>{actions.map((action,index)=><button key={action.label} role="menuitem" ref={node=>{buttons.current[index]=node;}} disabled={action.disabled} onFocus={()=>setSelection(index)} onClick={action.run}>{action.label}</button>)}</aside>;
}
export function AliasEditor({title,initial,saving,error,onSave,onClose}:{title:string;initial:string;saving:boolean;error:string|null;onSave:(alias:string)=>Promise<boolean>;onClose:()=>void}){
 const [draft,setDraft]=useState(initial);const field=useRef<HTMLInputElement>(null);useEffect(()=>{field.current?.focus();field.current?.select();},[]);useEffect(()=>{if(!saving)field.current?.focus();},[saving]);
 async function save(value:string){if(!saving&&await onSave(value))onClose();}
 return <aside role="dialog" aria-modal="true" onKeyDown={trap} aria-label={"Alias for "+title} className="customization-popover"><form onSubmit={event=>{event.preventDefault();void save(draft);}} onKeyDown={event=>{if(event.key==="Enter"&&event.nativeEvent.isComposing)event.preventDefault();}}>
  <h2>Alias for {title}</h2><label htmlFor="application-alias">Alias</label><input ref={field} id="application-alias" value={draft} maxLength={256} disabled={saving} onChange={event=>setDraft(event.target.value)} autoComplete="off" spellCheck={false}/>
  <p>An exact alias takes priority in search. Leave it empty to remove the alias.</p>{error&&<p role="alert" className="error">{error}</p>}
  <div className="customization-buttons"><button type="button" disabled={saving} onClick={()=>void save("")}>Clear Alias</button><button type="button" disabled={saving} onClick={onClose}>Cancel</button><button type="submit" disabled={saving}>Save Alias</button></div>
 </form></aside>;
}
export function Customization({hidden,saving,error,onRestore,onClose}:{hidden:Record<string,string>;saving:boolean;error:string|null;onRestore:(id:string)=>Promise<boolean>;onClose:()=>void}){
 const close=useRef<HTMLButtonElement>(null);useEffect(()=>{close.current?.focus();},[]);
 const entries=Object.entries(hidden).sort(([id,a],[other,b])=>a.localeCompare(b,undefined,{sensitivity:"base"})||id.localeCompare(other));
 return <aside role="dialog" aria-modal="true" onKeyDown={trap} aria-label="Customize search" className="customization-popover"><div className="history-heading"><h2>Customize search</h2><button ref={close} onClick={onClose} aria-label="Close customization">×</button></div>
 <p>Select an application and open Actions to set an alias or hide it from search.</p><h3>Hidden applications</h3>{!entries.length&&<p>No hidden applications.</p>}
 <ul>{entries.map(([id,title])=><li key={id}><span title={id}>{title}</span><button aria-label={"Show "+title+" again"} disabled={saving} onClick={async()=>{await onRestore(id);close.current?.focus();}}>Show Again</button></li>)}</ul>
 <p>Showing an application again keeps its alias and usage. It does not add it back to Favorites.</p>{error&&<p role="alert" className="error">{error}</p>}
 </aside>;
}

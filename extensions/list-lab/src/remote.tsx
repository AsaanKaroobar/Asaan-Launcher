import {Action,ActionPanel,List,showToast} from "@raycast/api";
import {useEffect,useState} from "react";
export default function Command(){
 const[query,setQuery]=useState(""),[items,setItems]=useState<string[]>([]),[loading,setLoading]=useState(true);
 useEffect(()=>{setLoading(true);const timer=setTimeout(()=>{setItems(["Mercury","Venus","Earth","Mars"].filter(n=>n.toLowerCase().includes(query.toLowerCase())));setLoading(false);},250);return()=>clearTimeout(timer);},[query]);
 return <List filtering={false} isLoading={loading} onSearchTextChange={setQuery} searchBarPlaceholder="Ask the extension to search…">
 <List.EmptyView title="Nothing here" description="Try Mercury." actions={<ActionPanel><Action title="Reset Search" onAction={()=>setQuery("")}/></ActionPanel>}/>
 {items.map(title=><List.Item key={title} title={title} actions={<ActionPanel><Action title={"Choose "+title} onAction={()=>showToast({title:"Remote "+title})}/></ActionPanel>}/>)}
 </List>;
}

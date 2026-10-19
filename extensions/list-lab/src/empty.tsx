import {Action,ActionPanel,List} from "@raycast/api";
import {useState} from "react";
export default function Command(){
 const[ready,setReady]=useState(false);
 return <List><List.EmptyView title="An empty extension list" description="The empty view's action can change the list." actions={<ActionPanel><Action title="Load Example" onAction={()=>setReady(true)}/></ActionPanel>}/>
 {ready&&<List.Item title="Ready"/>}</List>;
}

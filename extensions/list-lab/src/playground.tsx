import {Action,ActionPanel,Clipboard,Icon,List,showToast,Toast} from "@raycast/api";
import {useState} from "react";
const planets=[{name:"Mercury",subtitle:"0.39 AU",keywords:["rocky","inner"],moons:0},{name:"Venus",subtitle:"0.72 AU",keywords:["rocky"],moons:0},{name:"Mars",subtitle:"1.52 AU",keywords:["rocky"],moons:2},{name:"Jupiter",subtitle:"5.2 AU",keywords:["gas"],moons:95}];
export default function Command(){
 const[marked,setMarked]=useState<string[]>([]);
 return <List searchBarPlaceholder="Search title, subtitle or keywords…">
 {["Rocky","Gas giant"].map((section,index)=><List.Section key={section} title={section}>{planets.filter((_,i)=>index===0?i<3:i===3).map(p=>
 <List.Item key={p.name} title={p.name} subtitle={p.subtitle} keywords={p.keywords} icon={marked.includes(p.name)?Icon.Star:Icon.Globe} accessories={[{text:p.moons+" moons"}]} actions={<ActionPanel>
 <Action title={"Choose "+p.name} onAction={()=>showToast({style:Toast.Style.Success,title:"Selected "+p.name})}/>
 <Action title="Copy Name" onAction={async()=>{await Clipboard.copy(p.name+" ; literal ✓");await showToast({style:Toast.Style.Success,title:"Copied "+p.name});}}/>
 <ActionPanel.Section title="Organize"><ActionPanel.Submenu title="More" onOpen={()=>showToast({style:Toast.Style.Success,title:"More actions"})}>
 <Action title={marked.includes(p.name)?"Remove Mark":"Mark Favorite"} shortcut={{modifiers:["shift"],key:"return"}} onAction={()=>setMarked(a=>a.includes(p.name)?a.filter(n=>n!==p.name):[...a,p.name])}/>
 </ActionPanel.Submenu></ActionPanel.Section></ActionPanel>}/>
 )}</List.Section>)}
 </List>;
}

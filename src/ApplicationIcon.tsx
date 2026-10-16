import {useEffect,useState} from "react";
import {applicationIcon} from "./native";
export default function ApplicationIcon({id,title}:{id:string;title:string}){
 const [image,setImage]=useState<string|null>(null);
 useEffect(()=>{let cancelled=false;void applicationIcon(id).then(value=>{if(!cancelled)setImage(value);}).catch(()=>{});return()=>{cancelled=true;};},[id]);
 return <span className="result-icon" aria-hidden="true">{image?<img src={image} alt="" onError={()=>setImage(null)}/>:title[0]}</span>;
}

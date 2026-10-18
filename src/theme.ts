import {useEffect,useState} from "react";
import type {Theme} from "./native";
export function applyTheme(theme:Theme="floe") {
 document.documentElement.dataset.theme=theme;
 window.dispatchEvent(new Event("asaaan-theme"));
}
export function useTheme(){
 const read=():Theme=>document.documentElement.dataset.theme==="legacy"?"legacy":"floe";
 const [theme,setTheme]=useState(read);
 useEffect(()=>{const update=()=>setTheme(read());window.addEventListener("asaaan-theme",update);return()=>window.removeEventListener("asaaan-theme",update);},[]);
 return theme;
}

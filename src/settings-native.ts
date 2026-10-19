import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import type {Personalization,PersonalizationChange} from "./native";
export type SettingsPage="general"|"applications";
export interface SettingsApplication {id:string;name:string;installed:boolean}
export interface SettingsView {snapshot:Personalization;applications:SettingsApplication[];warning:string|null}
export const readSettings=()=>invoke<SettingsView>("read_settings");
export const saveSettings=(operation:PersonalizationChange)=>invoke<Personalization>("change_personalization",{operation});
export const openSettings=(page:SettingsPage|null=null)=>invoke<void>("open_settings",{page});
export const closeSettings=()=>invoke<void>("close_settings");
export const reportSettingsPage=(page:SettingsPage)=>invoke<void>("settings_page_changed",{page});
export async function connectSettings(shown:(sequence:number)=>void,page:(value:SettingsPage)=>void){
 const stops=await Promise.all([listen<number>("panel-shown",e=>shown(e.payload)),listen<SettingsPage>("settings-page",e=>page(e.payload))]);
 try{await invoke("frontend_ready");}catch(error){stops.forEach(stop=>stop());throw error;}
 return ()=>stops.forEach(stop=>stop());
}
export const confirmSettingsShown=(sequence:number,focused:boolean)=>invoke<void>("panel_presented",{sequence,focused});

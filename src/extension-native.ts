import {invoke,isTauri} from "@tauri-apps/api/core";
export interface ExtensionEntry{id:string;title:string;extension:string;keywords:string[]}
export interface ExtensionAction{id:number;revision:number;title:string;section:string|null;submenu:boolean;children:ExtensionAction[];shortcut:unknown}
export interface ExtensionRow{id:number;revision:number;title:string;subtitle:string|null;section:string|null;icon:string|null;accessories:string[]}
export interface ExtensionView{
 token:number;revision:number;commandId:string;title:string;screen:number|null;kind:string|null;query:string;placeholder:string;loading:boolean;selected:number;
 rows:ExtensionRow[];actions:ExtensionAction[];emptyTitle:string;emptyDescription:string|null;failure:{kind:string;message:string}|null;
 toast:{id:number;style:string;title:string;message:string|null}|null;
}
export interface ExtensionSnapshot{active:ExtensionView|null;commands:ExtensionEntry[]}
export type ExtensionOperation={kind:"read"}|{kind:"start";id:string;query:string}|{kind:"search";token:number;text:string}|{kind:"select";token:number;id:number;query:string}|{kind:"run"|"openMenu";token:number;id:number;revision:number;query:string}|{kind:"back"|"stop"|"retry";token:number};
export async function extensionExchange(operation:ExtensionOperation):Promise<ExtensionSnapshot>{
 if(!isTauri())return {active:null,commands:[]};
 return invoke("extension_exchange",{operation});
}

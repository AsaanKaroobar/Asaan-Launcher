import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri, invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
  architecture: string;
  smokeTest?: boolean;
  applicationSmoke?: boolean;
  shortcut?: string;
}

export async function loadAppInfo(): Promise<AppInfo | null> {
  if (!isTauri()) return null;
  const info = await invoke<AppInfo>("get_app_info");
  if (info.name !== "Asaaan Launcher" || !info.version || !info.platform || !info.architecture) {
    throw new Error("The native application returned an invalid identity.");
  }
  return info;
}

let panelListenerReady: Promise<unknown> = Promise.resolve();
export async function reportFrontendReady(): Promise<void> {
  if (isTauri()) { await panelListenerReady; await invoke("frontend_ready"); }
}

export type ApplicationAction = "reveal" | "copyPath" | "copyName" | "copyIdentifier" | "hide" | "quit";
export interface ApplicationActionDescriptor {kind:ApplicationAction;label:string;token?:string}
export async function runApplicationAction(id:string,action:ApplicationAction,token?:string):Promise<{message:string|null;dismissed:boolean}>{return invoke(action==="hide"||action==="quit"?"application_process_action":"application_action",{request:token===undefined?{id,action}:{id,action,token}});}
export interface SearchResult {
  actions?:ApplicationActionDescriptor[];
  item: { id: string; title: string; kind: "application" | "command"; keywords: string[] };
  section: string | null;
  origin?: string | null;
}

export async function searchReferenceCatalog(query: string): Promise<SearchResult[]> {
  if (!isTauri()) throw new Error("Search is available in the native desktop app.");
  return invoke<SearchResult[]>("search_reference_catalog", { query });
}

export async function reportSearchReady(query: string, resultIds: string[]): Promise<void> {
  if (isTauri()) await invoke("search_ready", { query, resultIds });
}

export async function hidePanel(sequence?:number): Promise<void> {
  if (isTauri()) await invoke("hide_panel", sequence === undefined ? undefined : {sequence});
}

export async function onPanelShown(focus: (sequence: number) => void, hideRequested?: (sequence:number)=>void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  const subscription = Promise.all([listen<number>("panel-shown", (event) => focus(event.payload)),listen<number>("panel-hide-requested",event=>hideRequested?.(event.payload))]).then(stops=>()=>stops.forEach(stop=>stop()));
  panelListenerReady = subscription;
  return subscription;
}
export async function confirmPanelPresented(sequence: number, focused: boolean): Promise<void> {
  if (isTauri()) await invoke("panel_presented", {sequence, focused});
}

export interface Personalization { favorites: string[]; queries: string[]; rememberSearches: boolean; aliases?:Record<string,string>;hiddenResults?:Record<string,string> }
export type PersonalizationChange = {kind:"favorite";id:string;enabled:boolean}|{kind:"clearQueries"}|{kind:"rememberSearches";enabled:boolean}|{kind:"alias";id:string;alias:string}|{kind:"hide";id:string}|{kind:"restore";id:string};
export async function changePersonalization(operation:PersonalizationChange):Promise<Personalization>{return invoke("change_personalization",{operation});}
export interface ApplicationSearch { rows: SearchResult[]; total: number; generation: number; warning: string | null; favorites?:string[];queries?:string[];rememberSearches?:boolean;aliases?:Record<string,string>;hiddenResults?:Record<string,string> }
export async function searchApplications(query: string, refresh = false): Promise<ApplicationSearch> {
  if (!isTauri()) throw new Error("Applications are available in the native desktop app.");
  return invoke("search_applications", {query, refresh});
}
export async function launchApplication(id: string,query=""): Promise<{warning:string|null}|void> { return invoke("launch_application", {id,query}); }
export async function applicationIcon(id: string): Promise<string | null> { return invoke("application_icon", {id}); }
export async function reportApplicationLaunched(id: string): Promise<void> { return invoke("application_launch_ready", {id}); }

export async function applicationProcessOptions(id:string):Promise<{actions:ApplicationActionDescriptor[];notice:string|null}>{return invoke("application_process_options",{id});}

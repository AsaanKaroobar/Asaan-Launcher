import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri, invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
  architecture: string;
  smokeTest?: boolean;
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

export interface SearchResult {
  item: { id: string; title: string; kind: "application" | "command"; keywords: string[] };
  section: string | null;
}

export async function searchReferenceCatalog(query: string): Promise<SearchResult[]> {
  if (!isTauri()) throw new Error("Search is available in the native desktop app.");
  return invoke<SearchResult[]>("search_reference_catalog", { query });
}

export async function reportSearchReady(query: string, resultIds: string[]): Promise<void> {
  if (isTauri()) await invoke("search_ready", { query, resultIds });
}

export async function hidePanel(): Promise<void> {
  if (isTauri()) await invoke("hide_panel");
}

export async function onPanelShown(focus: (sequence: number) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  const subscription = listen<number>("panel-shown", (event) => focus(event.payload));
  panelListenerReady = subscription;
  return subscription;
}
export async function confirmPanelPresented(sequence: number, focused: boolean): Promise<void> {
  if (isTauri()) await invoke("panel_presented", {sequence, focused});
}

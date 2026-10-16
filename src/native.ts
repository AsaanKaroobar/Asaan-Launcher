import { isTauri, invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
  architecture: string;
  smokeTest?: boolean;
}

export async function loadAppInfo(): Promise<AppInfo | null> {
  if (!isTauri()) return null;
  const info = await invoke<AppInfo>("get_app_info");
  if (info.name !== "Asaaan Launcher" || !info.version || !info.platform || !info.architecture) {
    throw new Error("The native application returned an invalid identity.");
  }
  return info;
}

export async function reportFrontendReady(): Promise<void> {
  if (isTauri()) await invoke("frontend_ready");
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

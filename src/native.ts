import { isTauri, invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
  architecture: string;
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

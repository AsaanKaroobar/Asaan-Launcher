// Asaaan Launcher runtime data is independent of Floe's live stores.
import os from "node:os";
import path from "node:path";
export function extensionDataRoot(env = process.env): string {
  if (env.ASAAAN_EXTENSION_DATA) return path.resolve(env.ASAAAN_EXTENSION_DATA);
  const base = process.platform === "win32"
    ? env.APPDATA || path.join(os.homedir(), "AppData", "Roaming")
    : process.platform === "darwin"
      ? path.join(os.homedir(), "Library", "Application Support")
      : env.XDG_DATA_HOME || path.join(os.homedir(), ".local", "share");
  return path.join(base, "com.asaaan.launcher", "extensions-data");
}
export function safeName(name: unknown): name is string {
  return typeof name === "string" && /^[a-zA-Z0-9_-]{1,200}$/.test(name);
}

import { beforeEach, expect, it, vi } from "vitest";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { loadAppInfo } from "./native";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), isTauri: vi.fn() }));
beforeEach(() => vi.resetAllMocks());
it("does not invoke native commands from a browser preview", async () => {
  vi.mocked(isTauri).mockReturnValue(false);
  expect(await loadAppInfo()).toBeNull();
  expect(invoke).not.toHaveBeenCalled();
});
it("rejects an incompatible native identity instead of showing ready", async () => {
  vi.mocked(isTauri).mockReturnValue(true);
  vi.mocked(invoke).mockResolvedValue({ name: "Another application", version: "0.1.0", platform: "linux", architecture: "x86_64" });
  await expect(loadAppInfo()).rejects.toThrow("invalid identity");
});

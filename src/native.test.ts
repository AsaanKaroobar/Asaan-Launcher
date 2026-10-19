import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { beforeEach, expect, it, vi } from "vitest";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { applicationProcessOptions,runApplicationAction,onPanelShown, reportFrontendReady, loadAppInfo } from "./native";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), isTauri: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
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

it("waits for native presentation subscription before announcing frontend readiness", async () => {
 vi.mocked(isTauri).mockReturnValue(true);vi.mocked(invoke).mockResolvedValue(undefined);
 let finish!:(stop:UnlistenFn)=>void;vi.mocked(listen).mockReturnValue(new Promise(resolve=>{finish=resolve;}));
 const subscription=onPanelShown(()=>{});const ready=reportFrontendReady();expect(invoke).not.toHaveBeenCalled();
 finish(()=>{});await subscription;await ready;expect(invoke).toHaveBeenCalledExactlyOnceWith("frontend_ready");
});

it("routes only graceful controls to the process boundary with an opaque catalog ID",async()=>{
 vi.mocked(invoke).mockResolvedValue({message:null,dismissed:true});
 await runApplicationAction("app:notes","hide","instance-1");
 expect(invoke).toHaveBeenLastCalledWith("application_process_action",{request:{id:"app:notes",action:"hide",token:"instance-1"}});
 await runApplicationAction("app:notes","quit");
 expect(invoke).toHaveBeenLastCalledWith("application_process_action",{request:{id:"app:notes",action:"quit"}});
 await runApplicationAction("app:notes","copyPath");
 expect(invoke).toHaveBeenLastCalledWith("application_action",{request:{id:"app:notes",action:"copyPath"}});
 await applicationProcessOptions("app:notes");
 expect(invoke).toHaveBeenLastCalledWith("application_process_options",{id:"app:notes"});
});

it("Restart and Force Quit use only catalog ID and instance token at the native boundary",async()=>{
 vi.mocked(invoke).mockResolvedValue({message:null,dismissed:true});
 for(const action of ["restart","forceQuit"] as const){
  await runApplicationAction("app:notes",action,"original");
  expect(invoke).toHaveBeenLastCalledWith("application_process_action",{request:{id:"app:notes",action,token:"original"}});
 }
});

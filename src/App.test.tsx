import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import { searchApplications, onPanelShown, hidePanel, loadAppInfo, reportFrontendReady, searchReferenceCatalog, reportSearchReady } from "./native";

vi.mock("./native", () => ({ applicationIcon: vi.fn().mockResolvedValue(null), searchApplications: vi.fn().mockResolvedValue({rows:[],total:0,generation:1,warning:null}), launchApplication: vi.fn().mockResolvedValue(undefined), reportApplicationLaunched: vi.fn().mockResolvedValue(undefined), confirmPanelPresented: vi.fn().mockResolvedValue(undefined), onPanelShown: vi.fn().mockResolvedValue(() => {}), hidePanel: vi.fn().mockResolvedValue(undefined), loadAppInfo: vi.fn(), reportFrontendReady: vi.fn(), searchReferenceCatalog: vi.fn().mockResolvedValue([]), reportSearchReady: vi.fn().mockResolvedValue(undefined) }));
const info = { name: "Asaaan Launcher", version: "0.1.0", platform: "linux", architecture: "x86_64" };
beforeEach(() => { vi.resetAllMocks(); vi.mocked(searchApplications).mockResolvedValue({rows:[],total:0,generation:1,warning:null}); vi.mocked(onPanelShown).mockResolvedValue(() => {}); vi.mocked(hidePanel).mockResolvedValue(); vi.mocked(reportFrontendReady).mockResolvedValue(); vi.mocked(searchReferenceCatalog).mockResolvedValue([]); vi.mocked(reportSearchReady).mockResolvedValue(); });

describe("checkpoint 001 native connection", () => {
  it("waits for the native reply before displaying readiness and confirms a mounted frontend", async () => {
    let resolve!: (value: typeof info) => void;
    vi.mocked(loadAppInfo).mockReturnValue(new Promise((done) => { resolve = done; }));
    render(<App />);
    expect(screen.getByRole("heading", { name: "Asaaan Launcher" })).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Connecting to your desktop");
    expect(reportFrontendReady).not.toHaveBeenCalled();
    await act(async () => resolve(info));
    expect(screen.getByRole("status")).toHaveTextContent("Native connection ready");
    expect(screen.getByRole("status")).toHaveTextContent("Linux");
    await waitFor(() => expect(reportFrontendReady).toHaveBeenCalledOnce());
  });

  it("labels a browser preview without claiming native success", async () => {
    vi.mocked(loadAppInfo).mockResolvedValue(null);
    render(<App />);
    expect(await screen.findByText("Browser preview")).toBeInTheDocument();
    expect(reportFrontendReady).not.toHaveBeenCalled();
  });

  it("shows connection errors and reconnects using the retry action", async () => {
    vi.mocked(loadAppInfo).mockRejectedValueOnce(new Error("Native process unavailable")).mockResolvedValueOnce(info);
    render(<App />);
    expect(await screen.findByText("Native process unavailable")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText(/Native connection ready/)).toBeInTheDocument();
    expect(loadAppInfo).toHaveBeenCalledTimes(2);
  });

  it("ignores a late response after the view unmounts", async () => {
    let resolve!: (value: typeof info) => void;
    vi.mocked(loadAppInfo).mockReturnValue(new Promise((done) => { resolve = done; }));
    const view = render(<App />);
    view.unmount();
    await act(async () => resolve(info));
    expect(reportFrontendReady).not.toHaveBeenCalled();
  });

  it("reports a failed native readiness acknowledgement", async () => {
    vi.mocked(loadAppInfo).mockResolvedValue(info);
    vi.mocked(reportFrontendReady).mockRejectedValue(new Error("Connection closed"));
    render(<App />);
    expect(await screen.findByText("Connection closed")).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent("Connection unavailable");
  });
});

it("shows the native shortcut status and hides from the header control", async () => {
 vi.mocked(loadAppInfo).mockResolvedValue({...info,shortcut:"Shortcut unavailable · use the tray"});render(<App />);
 expect(await screen.findByText("Shortcut unavailable · use the tray")).toBeInTheDocument();
 await userEvent.click(screen.getByRole("button",{name:"Hide launcher"}));expect(hidePanel).toHaveBeenCalledOnce();
});
it("reports a failed native hide instead of silently dismissing", async () => {
 vi.mocked(loadAppInfo).mockResolvedValue(info);vi.mocked(hidePanel).mockRejectedValue(new Error("Panel unavailable"));render(<App />);
 await userEvent.click(await screen.findByRole("button",{name:"Hide launcher"}));expect(await screen.findByText(/Panel unavailable/)).toBeInTheDocument();
});

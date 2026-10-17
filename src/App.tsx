import Picker from "./Picker";
import SearchPanel from "./SearchPanel";
import { useEffect, useState } from "react";
import { hidePanel, loadAppInfo, reportFrontendReady, type AppInfo } from "./native";
import "./App.css";

type Connection =
  | { state: "connecting" }
  | { state: "connected"; info: AppInfo }
  | { state: "preview" }
  | { state: "error"; message: string };

const platformNames: Record<string, string> = { linux: "Linux", macos: "macOS", windows: "Windows" };

export default function App() { return window.__ASAAAN_PICKER__ ? <Picker/> : <Launcher/>; }

function Launcher() {
  const [connection, setConnection] = useState<Connection>({ state: "connecting" });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let cancelled = false;
    setConnection({ state: "connecting" });
    loadAppInfo().then(
      (info) => {
        if (!cancelled) setConnection(info ? { state: "connected", info } : { state: "preview" });
      },
      (error: unknown) => {
        if (!cancelled) setConnection({ state: "error", message: error instanceof Error ? error.message : "The native connection could not be established." });
      },
    );
    return () => { cancelled = true; };
  }, [attempt]);

  useEffect(() => {
    if (connection.state === "connected") {
      void reportFrontendReady().catch((error: unknown) => {
        setConnection({ state: "error", message: error instanceof Error ? error.message : "The native connection could not be confirmed." });
      });
    }
  }, [connection.state]);

  return (
    <main className="launcher" aria-labelledby="app-title">
      <header className="topbar">
        <span className="wordmark">asaaan<span aria-hidden="true">.</span></span>
        <span className="product-type">Launcher</span>
        {connection.state === "connected" && <><span className="shortcut-hint">{connection.info.shortcut ?? "Ctrl+Alt+Space"}</span><button className="hide-panel" aria-label="Hide launcher" onClick={() => { void hidePanel().catch((error: unknown) => setConnection({ state: "error", message: String(error) })); }}>×</button></>}
      </header>
      {connection.state === "connected" ? <><h1 id="app-title" className="sr-only">Asaaan Launcher</h1><SearchPanel smokeTest={connection.info.smokeTest} applicationSmoke={connection.info.applicationSmoke} /></> : <section className="welcome">
        <span className="app-mark" aria-hidden="true">a</span>
        <h1 id="app-title">Asaaan Launcher</h1>
        <p className="intro">A little less friction.<br />A little more flow.</p>
        <div className="connection" role="status" aria-live="polite">
          {connection.state === "connecting" && <><span className="status-label">Connecting to your desktop</span><span className="status-detail">Starting the native connection…</span></>}

          {connection.state === "preview" && <><span className="status-label">Browser preview</span><span className="status-detail">Open the desktop app to connect to your system.</span></>}
          {connection.state === "error" && <><span className="status-label error">Connection unavailable</span><span className="status-detail">{connection.message}</span><button onClick={() => setAttempt((value) => value + 1)}>Try again</button></>}
        </div>
      </section>}
      <footer><span role={connection.state === "connected" ? "status" : undefined}>{connection.state === "connected" ? "Native connection ready · " + (platformNames[connection.info.platform] ?? connection.info.platform) : "Desktop connection"}</span><span>{connection.state === "connected" ? connection.info.version : "Asaaan Launcher"}</span></footer>
    </main>
  );
}

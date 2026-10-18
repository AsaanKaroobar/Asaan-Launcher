import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import SearchPanel from "./SearchPanel";
import { confirmPanelPresented, hidePanel, onPanelShown, reportSearchReady, searchReferenceCatalog, type SearchResult } from "./native";
vi.mock("./native",()=>({readLauncherTheme:vi.fn().mockResolvedValue("floe"),applicationProcessOptions:vi.fn().mockResolvedValue({actions:[],notice:null}), applicationIcon: vi.fn().mockResolvedValue(null), searchApplications: vi.fn().mockResolvedValue({rows:[],total:0,generation:1,warning:null}), launchApplication: vi.fn().mockResolvedValue(undefined), reportApplicationLaunched: vi.fn().mockResolvedValue(undefined), confirmPanelPresented: vi.fn().mockResolvedValue(undefined), onPanelShown: vi.fn(), hidePanel: vi.fn().mockResolvedValue(undefined), searchReferenceCatalog: vi.fn(), reportSearchReady: vi.fn() }));
const row = (title: string): SearchResult => ({ item: { id: title, title, kind: "application", keywords: [] }, section: null });
beforeEach(() => { vi.resetAllMocks(); vi.mocked(confirmPanelPresented).mockResolvedValue(); vi.mocked(hidePanel).mockResolvedValue(); vi.mocked(onPanelShown).mockResolvedValue(() => {}); vi.mocked(reportSearchReady).mockResolvedValue(); });

it("renders native results in their returned order and bounds arrow selection", async () => {
  vi.mocked(searchReferenceCatalog).mockResolvedValue([row("Calendar"), row("Calculator")]);
  render(<SearchPanel referenceCatalog />);
  const options = await screen.findAllByRole("option");
  expect(options.map((n) => n.textContent)).toEqual([expect.stringContaining("Calendar"), expect.stringContaining("Calculator")]);
  const input = screen.getByRole("combobox");
  fireEvent.keyDown(input, { key: "ArrowUp" }); expect(options[0]).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(input, { key: "ArrowDown" }); fireEvent.keyDown(input, { key: "ArrowDown" });
  expect(options[1]).toHaveAttribute("aria-selected", "true");
  expect(input).toHaveAttribute("aria-activedescendant", options[1].id);
  expect(screen.getByText("Sample items only. Nothing launches.")).toBeInTheDocument();
});
it("does not replace a newer query with a delayed earlier response", async () => {
  let finish!: (rows: SearchResult[]) => void;
  vi.mocked(searchReferenceCatalog).mockReturnValueOnce(new Promise((resolve) => { finish = resolve; })).mockResolvedValueOnce([row("Calendar")]);
  render(<SearchPanel referenceCatalog />);
  fireEvent.change(screen.getByRole("combobox"), { target: { value: "cal" } });
  expect(await screen.findByText("Calendar")).toBeInTheDocument();
  await act(async () => finish([row("Safari")]));
  expect(screen.queryByText("Safari")).not.toBeInTheDocument();
  expect(searchReferenceCatalog).toHaveBeenLastCalledWith("cal");
  expect(reportSearchReady).not.toHaveBeenCalledWith("cal", ["Safari"]);
});
it("shows native errors and retries the same query", async () => {
  vi.mocked(searchReferenceCatalog).mockRejectedValueOnce(new Error("Search failed")).mockResolvedValueOnce([row("Notes")]);
  render(<SearchPanel referenceCatalog />);
  expect(await screen.findByRole("alert")).toHaveTextContent("Search failed");
  fireEvent.click(screen.getByRole("button", { name: "Retry search" }));
  expect(await screen.findByText("Notes")).toBeInTheDocument();
  expect(searchReferenceCatalog).toHaveBeenNthCalledWith(2, "");
});
it("shows an empty result and clears the query with Escape", async () => {
  vi.mocked(searchReferenceCatalog).mockImplementation(async (query) => query ? [] : [row("Notes")]);
  render(<SearchPanel referenceCatalog />); await screen.findByText("Notes");
  const input=screen.getByRole("combobox");fireEvent.change(input,{target:{value:"zzzz"}});
  expect(await screen.findByText("No matches for “zzzz”")).toBeInTheDocument();
  expect(input).not.toHaveAttribute("aria-activedescendant");
  fireEvent.keyDown(input,{key:"Escape"});expect(await screen.findByText("Notes")).toBeInTheDocument();expect(input).toHaveValue("");
});
it("does not treat IME composition keys as launcher commands", async () => {
  vi.mocked(searchReferenceCatalog).mockResolvedValue([]);
  render(<SearchPanel referenceCatalog />);const input=screen.getByRole("combobox");fireEvent.change(input,{target:{value:"日本"}});
  fireEvent.keyDown(input,{key:"Escape",isComposing:true});expect(input).toHaveValue("日本");
  await waitFor(()=>expect(searchReferenceCatalog).toHaveBeenLastCalledWith("日本"));
});
it("only acknowledges the completed cal search during the installed smoke test", async () => {
  vi.mocked(searchReferenceCatalog).mockResolvedValue([row("Calendar"),row("Calculator")]);render(<SearchPanel smokeTest />);
  await waitFor(()=>expect(reportSearchReady).toHaveBeenCalledWith("cal",["Calendar","Calculator"]));
});
it("ignores pending results when the search screen is removed", async () => {
  let finish!: (rows:SearchResult[])=>void;vi.mocked(searchReferenceCatalog).mockReturnValue(new Promise(resolve=>{finish=resolve;}));
  const view=render(<SearchPanel referenceCatalog />);view.unmount();await act(async()=>finish([row("Notes")]));expect(reportSearchReady).not.toHaveBeenCalled();
});

it("clears text before hiding and preserves IME composition", async () => {
  vi.mocked(searchReferenceCatalog).mockResolvedValue([]); render(<SearchPanel referenceCatalog />);
  const input=screen.getByRole("combobox"); fireEvent.change(input,{target:{value:"cal"}});
  fireEvent.keyDown(input,{key:"Escape"}); expect(input).toHaveValue(""); expect(hidePanel).not.toHaveBeenCalled();
  fireEvent.keyDown(input,{key:"Escape",isComposing:true}); expect(hidePanel).not.toHaveBeenCalled();
  fireEvent.keyDown(input,{key:"Escape"}); await waitFor(()=>expect(hidePanel).toHaveBeenCalledOnce());
});
it("returns keyboard focus to search when the panel is shown again", async () => {
  vi.mocked(searchReferenceCatalog).mockResolvedValue([]); render(<SearchPanel referenceCatalog />); await screen.findByText('No matches for “”');
  screen.getByRole("combobox").blur(); fireEvent(window,new Event("focus")); expect(screen.getByRole("combobox")).toHaveFocus();
});

it("focuses the search on native panel presentation and removes its subscription", async () => {
 const stop=vi.fn();let focus!:(sequence:number)=>void;vi.mocked(onPanelShown).mockImplementation(async (callback)=>{focus=callback;return stop;});
 vi.mocked(searchReferenceCatalog).mockResolvedValue([]);const view=render(<SearchPanel referenceCatalog />);await waitFor(()=>expect(onPanelShown).toHaveBeenCalled());
 const input=screen.getByRole("combobox");input.blur();act(()=>focus(42));expect(input).toHaveFocus();expect(confirmPanelPresented).toHaveBeenCalledWith(42,true);view.unmount();expect(stop).toHaveBeenCalledOnce();
});

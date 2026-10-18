import {act,fireEvent,render,screen,waitFor} from "@testing-library/react";
import {beforeEach,expect,it,vi} from "vitest";
import SearchPanel from "./SearchPanel";
import {extensionExchange,type ExtensionSnapshot,type ExtensionView} from "./extension-native";
import {launchApplication,searchApplications,applicationProcessOptions,onPanelShown,readLauncherTheme} from "./native";
vi.mock("./native",()=>({readLauncherTheme:vi.fn().mockResolvedValue("floe"),applicationProcessOptions:vi.fn().mockResolvedValue({actions:[],notice:null}),confirmPanelPresented:vi.fn().mockResolvedValue(undefined),hidePanel:vi.fn().mockResolvedValue(undefined),onPanelShown:vi.fn().mockResolvedValue(()=>{}),reportSearchReady:vi.fn(),searchReferenceCatalog:vi.fn(),applicationIcon:vi.fn().mockResolvedValue(null),searchApplications:vi.fn(),launchApplication:vi.fn(),reportApplicationLaunched:vi.fn(),changePersonalization:vi.fn()}));
vi.mock("./extension-native",()=>({extensionExchange:vi.fn()}));
const view:ExtensionView={token:1,revision:1,commandId:"command:list-lab/planets",title:"Browse Planets",screen:1,kind:"List",query:"mars",placeholder:"Search planets",loading:false,selected:0,rows:[{id:5,revision:1,title:"Mars",subtitle:null,section:null,icon:null,accessories:[]}],actions:[],emptyTitle:"No results",emptyDescription:null,failure:null,toast:null};
const snapshot=(active:ExtensionView|null):ExtensionSnapshot=>({active,commands:[]});
beforeEach(()=>{vi.clearAllMocks();vi.mocked(extensionExchange).mockResolvedValue(snapshot(null));vi.mocked(searchApplications).mockResolvedValue({rows:[{item:{id:view.commandId,title:view.title,kind:"command",keywords:[]},section:null}],total:0,commandTotal:1,generation:1,warning:null});});
it("opens a command through typed extension RPC and suppresses duplicate Enter while pending",async()=>{
 let finish!:(value:ExtensionSnapshot)=>void;
 vi.mocked(extensionExchange).mockImplementation(op=>op.kind==="start"?new Promise(resolve=>{finish=resolve;}):Promise.resolve(snapshot(null)));
 render(<SearchPanel/>);await screen.findByText("Browse Planets");const input=screen.getByRole("combobox");
 fireEvent.change(input,{target:{value:"planets"}});await waitFor(()=>expect(searchApplications).toHaveBeenLastCalledWith("planets",false));await screen.findByText("Browse Planets");
 fireEvent.keyDown(input,{key:"Enter"});fireEvent.keyDown(input,{key:"Enter"});
 expect(vi.mocked(extensionExchange).mock.calls.filter(([op])=>op.kind==="start")).toEqual([[{kind:"start",id:view.commandId,query:"planets"}]]);
 expect(launchApplication).not.toHaveBeenCalled();
 await act(async()=>finish(snapshot(view)));expect(screen.getByRole("combobox",{name:"Search extension"})).toHaveValue("mars");
});
it("a recreated panel restores the resident session without another launch",async()=>{
 vi.mocked(extensionExchange).mockResolvedValue(snapshot(view));render(<SearchPanel/>);
 expect(await screen.findByRole("combobox",{name:"Search extension"})).toHaveValue("mars");
 expect(vi.mocked(extensionExchange).mock.calls.every(([op])=>op.kind==="read")).toBe(true);expect(launchApplication).not.toHaveBeenCalled();
});
it("shows a native string activation error and keeps the root query for retry",async()=>{
 vi.mocked(extensionExchange).mockImplementation(async op=>{if(op.kind==="start")throw "Invalid launch history";return snapshot(null);});
 render(<SearchPanel/>);await screen.findByText("Browse Planets");const input=screen.getByRole("combobox");fireEvent.keyDown(input,{key:"Enter"});
 expect(await screen.findByRole("alert")).toHaveTextContent("Invalid launch history");expect(input).toBeInTheDocument();expect(launchApplication).not.toHaveBeenCalled();
});
it("command actions do not request application process controls",async()=>{
 render(<SearchPanel/>);await screen.findByText("Browse Planets");fireEvent.keyDown(screen.getByRole("combobox"),{key:"k",ctrlKey:true});
 await screen.findByRole("menu");expect(applicationProcessOptions).not.toHaveBeenCalled();
});

it("retained extensions pick up the resident theme on presentation without restarting",async()=>{
 vi.mocked(extensionExchange).mockResolvedValue(snapshot(view));vi.mocked(readLauncherTheme).mockResolvedValue("legacy");
 render(<SearchPanel/>);await screen.findByRole("combobox",{name:"Search extension"});
 await act(async()=>vi.mocked(onPanelShown).mock.calls[0][0](2));
 await waitFor(()=>expect(document.documentElement.dataset.theme).toBe("legacy"));
 expect(readLauncherTheme).toHaveBeenCalledOnce();expect(screen.getByRole("combobox",{name:"Search extension"})).toHaveValue("mars");
 expect(vi.mocked(extensionExchange).mock.calls.every(([op])=>op.kind==="read")).toBe(true);
});

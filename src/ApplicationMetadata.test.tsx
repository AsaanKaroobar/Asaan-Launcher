import {act,fireEvent,render,screen,waitFor,within} from "@testing-library/react";
import {beforeEach,expect,it,vi} from "vitest";
import SearchPanel from "./SearchPanel";
import {applicationProcessOptions,runApplicationAction,searchApplications,launchApplication,changePersonalization,type SearchResult} from "./native";
vi.mock("./native",()=>({applicationProcessOptions:vi.fn().mockResolvedValue({actions:[],notice:null}),runApplicationAction:vi.fn(),confirmPanelPresented:vi.fn().mockResolvedValue(undefined),hidePanel:vi.fn().mockResolvedValue(undefined),onPanelShown:vi.fn().mockResolvedValue(()=>{}),reportSearchReady:vi.fn(),searchReferenceCatalog:vi.fn(),applicationIcon:vi.fn().mockResolvedValue(null),searchApplications:vi.fn(),launchApplication:vi.fn(),reportApplicationLaunched:vi.fn(),changePersonalization:vi.fn()}));
const row=(id="app:notes"):SearchResult=>({item:{id,title:"Native Notes",kind:"application",keywords:[]},section:null,actions:[{kind:"reveal",label:"Reveal in File Manager"},{kind:"copyPath",label:"Copy Path"},{kind:"copyName",label:"Copy Name"},{kind:"copyIdentifier",label:"Copy Desktop Entry ID"}]});
const reply=(rows=[row()])=>({rows,total:rows.length,generation:1,warning:null,favorites:[],queries:[],rememberSearches:true,aliases:{},hiddenResults:{}});
beforeEach(()=>{vi.clearAllMocks();vi.mocked(applicationProcessOptions).mockResolvedValue({actions:[],notice:null});vi.mocked(searchApplications).mockResolvedValue(reply());vi.mocked(runApplicationAction).mockResolvedValue({message:"Copied Path",dismissed:false});});
async function menu(){fireEvent.keyDown(screen.getByRole("combobox"),{key:"k",ctrlKey:true});return screen.findByRole("menu");}
it("native app menu keeps Open first and favorites last without Copy File or process controls",async()=>{render(<SearchPanel/>);await screen.findByText("Native Notes");const names=within(await menu()).getAllByRole("menuitem").map(n=>n.textContent);expect(names).toEqual(["Open","Set Alias…","Reveal in File Manager","Copy Path","Copy Name","Copy Desktop Entry ID","Add to Favorites","Hide from Search"]);});
it("copy uses only the selected opaque ID and preserves query without activation history",async()=>{render(<SearchPanel/>);await screen.findByText("Native Notes");const input=screen.getByRole("combobox");fireEvent.change(input,{target:{value:"notes"}});await screen.findByText("Native Notes");fireEvent.click(within(await menu()).getByRole("menuitem",{name:"Copy Path"}));await screen.findByText("Copied Path");expect(runApplicationAction).toHaveBeenCalledExactlyOnceWith("app:notes","copyPath");expect(input).toHaveValue("notes");expect(input).toHaveFocus();expect(launchApplication).not.toHaveBeenCalled();expect(changePersonalization).not.toHaveBeenCalled();});
it("copy name and identifier dispatch their declared native operations",async()=>{render(<SearchPanel/>);await screen.findByText("Native Notes");for(const [label,kind] of [["Copy Name","copyName"],["Copy Desktop Entry ID","copyIdentifier"]]){fireEvent.click(within(await menu()).getByRole("menuitem",{name:label}));await waitFor(()=>expect(runApplicationAction).toHaveBeenLastCalledWith("app:notes",kind));await waitFor(()=>expect(screen.getByRole("button",{name:"Actions"})).toBeEnabled());}});
it("reveal resets search only after a successful native acknowledgement",async()=>{let finish!:(v:{message:string|null;dismissed:boolean})=>void;vi.mocked(runApplicationAction).mockReturnValue(new Promise(r=>finish=r));render(<SearchPanel/>);await screen.findByText("Native Notes");const input=screen.getByRole("combobox");fireEvent.change(input,{target:{value:"notes"}});await screen.findByText("Native Notes");fireEvent.click(within(await menu()).getByRole("menuitem",{name:"Reveal in File Manager"}));expect(input).toHaveValue("notes");await act(async()=>finish({message:null,dismissed:true}));expect(input).toHaveValue("");expect(launchApplication).not.toHaveBeenCalled();});
it("failed native action keeps search usable and reports the error without claiming a copy",async()=>{vi.mocked(runApplicationAction).mockRejectedValue(new Error("Provider unavailable"));render(<SearchPanel/>);await screen.findByText("Native Notes");const input=screen.getByRole("combobox");fireEvent.change(input,{target:{value:"notes"}});await screen.findByText("Native Notes");fireEvent.click(within(await menu()).getByRole("menuitem",{name:"Reveal in File Manager"}));expect(await screen.findByRole("alert")).toHaveTextContent("Provider unavailable");expect(input).toHaveValue("notes");expect(screen.queryByText("Copied Path")).not.toBeInTheDocument();});
it("platform identity capabilities omit filesystem actions for Windows identities",async()=>{const win=row();win.actions=[{kind:"copyName",label:"Copy Name"},{kind:"copyIdentifier",label:"Copy Application ID"}];vi.mocked(searchApplications).mockResolvedValue(reply([win]));render(<SearchPanel/>);await screen.findByText("Native Notes");const m=await menu();expect(within(m).getByRole("menuitem",{name:"Copy Application ID"})).toBeInTheDocument();expect(within(m).queryByRole("menuitem",{name:"Copy Path"})).not.toBeInTheDocument();expect(within(m).queryByRole("menuitem",{name:"Reveal in File Manager"})).not.toBeInTheDocument();});
it("a pending copy blocks duplicate actions and Enter without changing personalization",async()=>{let finish!:(v:{message:string|null;dismissed:boolean})=>void;vi.mocked(runApplicationAction).mockReturnValue(new Promise(r=>finish=r));render(<SearchPanel/>);await screen.findByText("Native Notes");fireEvent.click(within(await menu()).getByRole("menuitem",{name:"Copy Path"}));fireEvent.keyDown(screen.getByRole("combobox"),{key:"Enter"});fireEvent.keyDown(screen.getByRole("combobox"),{key:"k",ctrlKey:true});fireEvent.keyDown(screen.getByRole("combobox"),{key:"f",ctrlKey:true,shiftKey:true});expect(screen.queryByRole("menu")).not.toBeInTheDocument();expect(runApplicationAction).toHaveBeenCalledTimes(1);expect(launchApplication).not.toHaveBeenCalled();expect(changePersonalization).not.toHaveBeenCalled();await act(async()=>finish({message:"Copied Path",dismissed:false}));});
it("a menu's captured ID survives selection and catalog result changes",async()=>{render(<SearchPanel/>);await screen.findByText("Native Notes");const m=await menu();fireEvent.change(screen.getByRole("combobox"),{target:{value:"other"}});vi.mocked(searchApplications).mockResolvedValue(reply([row("app:other")]));fireEvent.click(within(m).getByRole("menuitem",{name:"Copy Path"}));await waitFor(()=>expect(runApplicationAction).toHaveBeenCalledExactlyOnceWith("app:notes","copyPath"));});

it("native string rejections retain the actual refusal reason",async()=>{vi.mocked(runApplicationAction).mockRejectedValue("Application was removed. Refresh the catalog.");render(<SearchPanel/>);await screen.findByText("Native Notes");fireEvent.click(within(await menu()).getByRole("menuitem",{name:"Reveal in File Manager"}));expect(await screen.findByRole("alert")).toHaveTextContent("Application was removed. Refresh the catalog.");expect(launchApplication).not.toHaveBeenCalled();});

it("verified running controls appear before metadata with Open first and personal actions last",async()=>{
 vi.mocked(applicationProcessOptions).mockResolvedValue({actions:[{kind:"hide",label:"Hide"},{kind:"quit",label:"Quit"}],notice:"Graceful close only"});
 render(<SearchPanel/>);await screen.findByText("Native Notes");await menu();
 await screen.findByRole("menuitem",{name:"Quit"});
 const names=within(screen.getByRole("menu")).getAllByRole("menuitem").map(n=>n.textContent);
 expect(names.slice(0,5)).toEqual(["Open","Set Alias…","Hide","Quit","Reveal in File Manager"]);
 expect(names.slice(-2)).toEqual(["Add to Favorites","Hide from Search"]);
 expect(screen.queryByRole("menuitem",{name:/Force|Restart/})).not.toBeInTheDocument();
 expect(applicationProcessOptions).toHaveBeenCalledExactlyOnceWith("app:notes");
 expect(screen.getByRole("status")).toHaveTextContent("Graceful close only");
});
it("Quit preserves query and reports request acknowledgement without recording usage",async()=>{
 vi.mocked(applicationProcessOptions).mockResolvedValue({actions:[{kind:"quit",label:"Quit",token:"instance-1"}],notice:null});
 vi.mocked(runApplicationAction).mockResolvedValue({message:"Quitting Native Notes",dismissed:false});
 render(<SearchPanel/>);await screen.findByText("Native Notes");fireEvent.change(screen.getByRole("combobox"),{target:{value:"notes"}});
 await screen.findByText("Native Notes");await menu();fireEvent.click(await screen.findByRole("menuitem",{name:"Quit"}));
 await screen.findByText("Quitting Native Notes");expect(screen.getByRole("combobox")).toHaveValue("notes");
 expect(runApplicationAction).toHaveBeenCalledExactlyOnceWith("app:notes","quit","instance-1");
 expect(changePersonalization).not.toHaveBeenCalled();expect(launchApplication).not.toHaveBeenCalled();
});
it("Hide clears query only after native success and does not open the app",async()=>{
 vi.mocked(applicationProcessOptions).mockResolvedValue({actions:[{kind:"hide",label:"Hide"}],notice:null});
 vi.mocked(runApplicationAction).mockResolvedValue({message:null,dismissed:true});
 render(<SearchPanel/>);await screen.findByText("Native Notes");fireEvent.change(screen.getByRole("combobox"),{target:{value:"notes"}});
 await screen.findByText("Native Notes");await menu();fireEvent.click(await screen.findByRole("menuitem",{name:"Hide"}));
 await waitFor(()=>expect(screen.getByRole("combobox")).toHaveValue(""));
 expect(runApplicationAction).toHaveBeenCalledExactlyOnceWith("app:notes","hide");expect(launchApplication).not.toHaveBeenCalled();
});
it("stale capability replies cannot attach Quit to a different menu or reopen a closed one",async()=>{
 let finish!:(v:{actions:{kind:"quit";label:string}[];notice:null})=>void;
 vi.mocked(applicationProcessOptions).mockReturnValueOnce(new Promise(r=>finish=r));
 render(<SearchPanel/>);await screen.findByText("Native Notes");await menu();fireEvent.keyDown(screen.getByRole("menu"),{key:"Escape"});
 vi.mocked(searchApplications).mockResolvedValue(reply([row("app:other")]));fireEvent.change(screen.getByRole("combobox"),{target:{value:"other"}});
 await waitFor(()=>expect(searchApplications).toHaveBeenLastCalledWith("other",false));await menu();
 await act(async()=>finish({actions:[{kind:"quit",label:"Quit"}],notice:null}));
 expect(screen.queryByRole("menuitem",{name:"Quit"})).not.toBeInTheDocument();
});
it("identity refusal omits process controls and leaves metadata usable",async()=>{
 vi.mocked(applicationProcessOptions).mockResolvedValue({actions:[],notice:"Application executable does not match its desktop entry"});
 render(<SearchPanel/>);await screen.findByText("Native Notes");await menu();
 await screen.findByText("Application executable does not match its desktop entry");
 expect(screen.queryByRole("menuitem",{name:"Quit"})).not.toBeInTheDocument();
 fireEvent.click(screen.getByRole("menuitem",{name:"Copy Name"}));await screen.findByText("Copied Path");
});
it("late process rows preserve keyboard navigation from the focused metadata action",async()=>{
 let finish!:(v:{actions:{kind:"quit";label:string}[];notice:null})=>void;
 vi.mocked(applicationProcessOptions).mockReturnValueOnce(new Promise(r=>finish=r));
 render(<SearchPanel/>);await screen.findByText("Native Notes");await menu();
 screen.getByRole("menuitem",{name:"Copy Path"}).focus();
 await act(async()=>finish({actions:[{kind:"quit",label:"Quit"}],notice:null}));
 expect(screen.getByRole("menuitem",{name:"Copy Path"})).toHaveFocus();
 fireEvent.keyDown(screen.getByRole("menu"),{key:"ArrowDown"});
 expect(screen.getByRole("menuitem",{name:"Copy Name"})).toHaveFocus();
});

// Scoped observable equivalents of Floe ActionStateTests and launcher session keys.
import {act,fireEvent,render,screen,waitFor} from "@testing-library/react";
import {beforeEach,expect,it,vi} from "vitest";
import {createRef} from "react";
import ExtensionPanel,{flattenActions,matchesShortcut,menuEntries} from "./ExtensionPanel";
import {extensionExchange,type ExtensionAction,type ExtensionSnapshot,type ExtensionView} from "./extension-native";
vi.mock("./extension-native",()=>({extensionExchange:vi.fn()}));
const a=(title:string,id:number,extra:Partial<ExtensionAction>={}):ExtensionAction=>({id,revision:id+100,title,submenu:false,section:null,children:[],shortcut:null,...extra});
const actions=[a("Open",1),a("Copy Title",2,{section:"Copy"}),a("Copy Link",3,{section:"Copy"}),a("Untitled Section Action",4,{section:""}),a("Set Priority",5,{submenu:true,children:[a("High",6),a("Low",7)]})];
const base=():ExtensionView=>({token:1,revision:1,commandId:"command:list-lab/playground",title:"Playground",screen:50,kind:"List",query:"",placeholder:"Search…",loading:false,selected:0,rows:[{id:10,revision:10,title:"Mercury",subtitle:"0.39 AU",section:"Rocky",icon:"icon:Globe",accessories:["0 moons"]},{id:20,revision:20,title:"Jupiter",subtitle:"5.2 AU",section:"Gas giant",icon:"icon:Globe",accessories:["95 moons"]}],actions,failure:null,toast:null,emptyTitle:"No Results",emptyDescription:null});
let state:ExtensionView;
beforeEach(()=>{
 vi.clearAllMocks();state=base();
 vi.mocked(extensionExchange).mockImplementation(async op=>{
  if(op.kind==="search")state={...state,revision:state.revision+1,query:op.text,selected:0,rows:op.text==="none"?[]:state.rows};
  if(op.kind==="select")state={...state,revision:state.revision+1,selected:state.rows.findIndex(r=>r.id===op.id)};
  if(op.kind==="back"||op.kind==="stop")return {active:null,commands:[]};
  if(op.kind==="retry")state={...base(),token:2};
  return {active:state,commands:[]};
 });
});
function show(view=state){const close=vi.fn();const input=createRef<HTMLInputElement>();render(<ExtensionPanel initial={view} inputRef={input} onClose={close}/>);return{close,input};}
it("a query searches every action including inside submenus",()=>{
 expect(menuEntries(actions,"COPY").map(a=>a.id)).toEqual([2,3]);expect(menuEntries(actions,"high").map(a=>a.id)).toEqual([6]);expect(menuEntries(actions,"nothing")).toEqual([]);expect(flattenActions(actions).map(a=>a.id)).toEqual([1,2,3,4,6,7]);
});
it("renders section headings, subtitles and accessories as text",()=>{
 state.rows[1].icon="icon:Star";show();expect(screen.getByRole("option",{name:"Jupiter, 5.2 AU, 95 moons, Star"})).toBeVisible();expect(screen.getAllByRole("img",{name:"Globe"})).toHaveLength(1);expect(screen.getAllByRole("option")).toHaveLength(2);expect(screen.getByText("Rocky")).toBeVisible();expect(screen.getByText("5.2 AU")).toBeVisible();expect(screen.getByText("95 moons")).toBeVisible();
});
it("search and a rapid selection then Return use the acknowledged current row",async()=>{
 show();const input=screen.getByRole("combobox");fireEvent.keyDown(input,{key:"ArrowDown"});fireEvent.keyDown(input,{key:"Enter"});
 await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"select",token:1,id:20,query:""}));
 await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"run",token:1,id:1,revision:101,query:""}));
});
it("Return runs the first action and primary Return runs the second",async()=>{
 show();const input=screen.getByRole("combobox");fireEvent.keyDown(input,{key:"Enter"});await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"run",token:1,id:1,revision:101,query:""}));
 fireEvent.keyDown(input,{key:"Enter",ctrlKey:true});await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"run",token:1,id:2,revision:102,query:""}));
});
it("a declared modified Return wins while the action menu is open",async()=>{
 state.actions=[a("Main",1),a("Specific",2,{shortcut:{modifiers:["shift"],key:"return"}})];show();
 fireEvent.click(screen.getByRole("button",{name:"Actions"}));await screen.findByRole("menu");
 fireEvent.keyDown(screen.getByRole("textbox",{name:"Search extension actions"}),{key:"Enter",shiftKey:true});
 await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"run",token:1,id:2,revision:102,query:""}));
 expect(screen.queryByRole("menu")).not.toBeInTheDocument();
});
it("menu search includes nested actions and submenu Back is independent of the List",async()=>{
 show();fireEvent.click(screen.getByRole("button",{name:"Actions"}));await screen.findByRole("menu");fireEvent.click(screen.getByRole("menuitem",{name:"Set Priority"}));
 expect(await screen.findByRole("menuitem",{name:"High"})).toBeVisible();expect(screen.queryByRole("menuitem",{name:"Copy Title"})).not.toBeInTheDocument();
 await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"openMenu",token:1,id:5,revision:105,query:""}));
 fireEvent.keyDown(screen.getByRole("textbox",{name:"Search extension actions"}),{key:"Escape"});expect(await screen.findByRole("menuitem",{name:"Open"})).toBeVisible();
 fireEvent.change(screen.getByRole("textbox",{name:"Search extension actions"}),{target:{value:"high"}});expect(screen.getByRole("menuitem",{name:"High"})).toBeVisible();
});
it("Escape pops the command even with nonempty search text",async()=>{
 state.query="mars";show();fireEvent.keyDown(screen.getByRole("combobox"),{key:"Escape"});await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"back",token:1}));
});
it("ignores composing keys and sends no arbitrary event props",()=>{
 show();fireEvent.keyDown(screen.getByRole("combobox"),{key:"Enter",isComposing:true});expect(extensionExchange).not.toHaveBeenCalled();
});
it("does not run a previous query's action when the live DOM input changed",async()=>{
 show();const input=screen.getByRole("combobox") as HTMLInputElement;input.value="typed before React";fireEvent.keyDown(input,{key:"Enter"});await act(async()=>{});expect(extensionExchange).not.toHaveBeenCalledWith(expect.objectContaining({kind:"run"}));
});
it("discards a delayed poll after a new search intent",async()=>{
 let finish!:(reply:ExtensionSnapshot)=>void;vi.mocked(extensionExchange).mockImplementation(op=>op.kind==="read"?new Promise(r=>{finish=r;}):Promise.resolve({active:{...state,query:"none",rows:[],revision:2},commands:[]}));
 show();await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"read"}));
 fireEvent.change(screen.getByRole("combobox"),{target:{value:"none"}});await screen.findByText("No Results");
 await act(async()=>finish({active:base(),commands:[]}));expect(screen.getByRole("combobox")).toHaveValue("none");expect(screen.queryByRole("option")).not.toBeInTheDocument();
});
it("failure exposes Retry and Return; retry changes the session token",async()=>{
 state.failure={kind:"crashed",message:"Unexpected exit 3"};show();expect(screen.getByRole("alert")).toHaveTextContent("Unexpected exit 3");
 fireEvent.click(screen.getByRole("button",{name:"Retry extension"}));await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"retry",token:1}));await screen.findByRole("option",{name:/Mercury/});
 fireEvent.keyDown(screen.getByRole("combobox"),{key:"Enter"});await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith(expect.objectContaining({kind:"run",token:2})));
});
it("unsupported views show a scoped placeholder with a native Back action",async()=>{
 state.kind="Detail";show();expect(screen.getByText("Detail views are not available in this checkpoint.")).toBeVisible();
 fireEvent.click(screen.getByRole("button",{name:"Back to list"}));await waitFor(()=>expect(extensionExchange).toHaveBeenCalledWith({kind:"back",token:1}));
});
it("loading suppresses the empty placeholder until the producer finishes",()=>{
 state.rows=[];state.loading=true;show();expect(screen.getByRole("progressbar")).toHaveAccessibleName("Loading extension");expect(screen.queryByText("No Results")).not.toBeInTheDocument();
});
it("text from the extension is never executed as markup",()=>{
 state.rows[0].title="<img src=x onerror=alert(1)>";show();expect(screen.getByText("<img src=x onerror=alert(1)>")).toBeVisible();expect(document.querySelector("img")).toBeNull();
});
it("shortcut modifiers match exactly and command maps to the platform primary key",()=>{
 const key={key:"Enter",ctrlKey:true,metaKey:false,shiftKey:false,altKey:false};
 expect(matchesShortcut({key:"return",modifiers:["cmd"]},key,false)).toBe(true);
 expect(matchesShortcut({key:"return",modifiers:["cmd"]},key,true)).toBe(false);
 expect(matchesShortcut({key:"return",modifiers:["cmd"]},{...key,ctrlKey:false,metaKey:true},true)).toBe(true);
 expect(matchesShortcut({key:"return",modifiers:["cmd"]},{...key,altKey:true},false)).toBe(false);
});

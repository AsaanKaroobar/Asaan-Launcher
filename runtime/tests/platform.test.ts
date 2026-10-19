import {describe,expect,test} from "bun:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {place,programOf} from "../access";
import {createLocalStorage,closeLocalStorage} from "../local-storage";
describe("cross-platform access paths",()=>{
 test("Windows drive paths retain their volume and summarize native home",()=>{
  expect(place("C:\\Users\\Dani\\Documents\\a.txt",[],"C:\\Users\\Dani")).toBe("~/Documents");
  expect(place("c:\\users\\dani\\Documents\\a.txt",[],"C:\\Users\\Dani")).toBe("~/Documents");
  expect(place("D:\\Programs\\Tools\\app.exe",[],"C:\\Users\\Dani")).toBe("D:/Programs/Tools");
 });
 test("Windows own folders and node_modules are omitted with component boundaries",()=>{
  const own=["C:\\Extensions\\sample"];
  expect(place("c:\\extensions\\sample\\asset.png",own)).toBeUndefined();
  expect(place("C:\\Extensions\\sample-other\\asset.png",own,"C:\\Users\\Dani")).toBe("C:/Extensions/sample-other");
  expect(place("C:\\project\\node_modules\\pkg\\index.js",[])).toBeUndefined();
 });
 test("UNC paths retain server/share identity",()=>{
  expect(place("\\\\server\\share\\folder\\deeper\\file.txt",[],"C:\\Users\\Dani")).toBe("//server/share/folder/deeper");
 });
 test("Windows executable names are recognized on every host",()=>{
  expect(programOf("C:\\tools\\git.exe status")).toBe("git.exe");
 });
});
describe("SQLite owner teardown",()=>{
 test("a closed storage releases its file for rename and can reopen without losing values",async()=>{
  const folder=fs.mkdtempSync(path.join(os.tmpdir(),"asaaan-storage-close-"));
  const file=path.join(folder,"local-storage.sqlite");
  const storage=createLocalStorage(()=>file);
  try {
   await storage.setItem("kept",{count:2});
   await closeLocalStorage(storage);
   fs.renameSync(file,file+".moved");
   fs.renameSync(file+".moved",file);
   expect(await storage.getItem("kept")).toEqual({count:2});
   await closeLocalStorage(storage);
   await closeLocalStorage(storage);
  } finally {
   await closeLocalStorage(storage);
   fs.rmSync(folder,{recursive:true,force:true});
  }
 });
});

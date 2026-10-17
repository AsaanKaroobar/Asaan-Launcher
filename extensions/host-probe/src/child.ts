import { showHUD } from "@raycast/api";
export default async function Command() {
  const child = Bun.spawn([process.execPath, "-e", 'process.on("SIGTERM",()=>{});setInterval(()=>{},1000)'], {stdin:"ignore",stdout:"ignore",stderr:"ignore"});
  await showHUD(JSON.stringify({child:child.pid}));
  await new Promise(()=>{});
}

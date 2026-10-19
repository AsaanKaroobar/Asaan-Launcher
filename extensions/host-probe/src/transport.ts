import { Clipboard, environment, showHUD } from "@raycast/api";
export default async function Command(props: {arguments: Record<string, unknown>}) {
  console.log("fixture console log");
  process.stdout.write("\x07fixture raw stdout\n");
  await showHUD(JSON.stringify({bun:Bun.version,supportPath:environment.supportPath,arguments:props.arguments}));
  try { await Clipboard.readText(); throw new Error("Native request was unexpectedly accepted"); }
  catch (error) { await showHUD(String(error)); }
}

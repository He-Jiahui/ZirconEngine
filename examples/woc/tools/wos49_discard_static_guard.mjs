import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
const catalog = read("scripts", "woc_game", "src", "generated", "m5_content_catalog.zr");
const inventory = read("scripts", "woc_game", "src", "progression", "inventory_vendor_state.zr");
const payloads = read("scripts", "woc_game", "src", "protocol", "command_payloads.zr");
const commands = read("scripts", "woc_game", "src", "protocol", "commands.zr");
const native = read("native", "crates", "woc_protocol", "src", "command_payload.rs");
const nativeLib = read("native", "crates", "woc_protocol", "src", "lib.rs");
const nativeTests = read("native", "crates", "woc_protocol", "tests", "command_payloads.rs");
const contracts = JSON.parse(read("contracts", "command_payloads.json"));
const content = JSON.parse(read("contracts", "m5_content.json"));

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "current WOS writer version is missing");
requireText(
  world,
  /schemaVersion != <uint>46 &&\s*schemaVersion != <uint>47 &&\s*schemaVersion != <uint>48 &&\s*schemaVersion != <uint>49 &&\s*schemaVersion != <uint>50 &&\s*schemaVersion != <uint>51 &&\s*schemaVersion != <uint>52 &&\s*schemaVersion != <uint>53 &&\s*schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS46-WOS55 decoder admission is missing",
);
requireText(world, /var discardCommand = payloads\.discardItemCommandId\(true\);/, "discard command id is not loaded");
requireText(
  world,
  /commandId == discardCommand[\s\S]*?applyM5DiscardCommand/,
  "discard command is not dispatched",
);
requireText(
  world,
  /applyM5DiscardCommand[\s\S]*?presence == <uint>0 && trailing != <uint>1[\s\S]*?presence == <uint>1 && trailing != <uint>5[\s\S]*?m5InventoryItemCodeFromPayload[\s\S]*?flag\("item", itemIndex, "noDiscard"\)[\s\S]*?removeM5InventoryItem/,
  "discard reducer does not validate and remove the scalar payload",
);
requireText(
  world,
  /appendM5DiscardCommand[\s\S]*?discardState\.applyCommands[\s\S]*?offlineQuestBoarsState != <uint>1[\s\S]*?restoredDiscardState/,
  "WOS49 discard lifecycle coverage is missing",
);

const discard = contracts.entries.find((entry) => entry.id === 24);
if (discard?.name !== "discard" || discard.kind !== "utf8_id_optional_u32") {
  throw new Error("discard command contract drifted");
}
requireText(commands, /id == <uint>24\) \{ return "discard";? \}/, "discard command id drifted");
requireText(payloads, /discardItemCommandId[\s\S]*?return <uint>24;/, "discard payload id is missing");
requireText(native, /struct DiscardItemCommandPayload[\s\S]*?count: Option<u32>/, "native discard payload is missing");
requireText(nativeTests, /DiscardItemCommandPayload/, "native discard payload coverage is missing");
requireText(main, /\\"world_state\\":\\"WOS67\\"/, "package WOS64 identity is missing");
requireText(nativeLib, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(nativeLib, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");

requireText(catalog, /field == "noDiscard"/, "discard policy flag is missing");
requireText(
  inventory,
  /discardItem[\s\S]*?flag\("item", item, "noDiscard"\)[\s\S]*?sellItem[\s\S]*?flag\("item", item, "noVendorSell"\)[\s\S]*?flag\("item", item, "soulbound"\)/,
  "standalone M5 inventory policy flags are missing",
);
for (const item of content.items) {
  if (item.definition.noDiscard !== undefined && typeof item.definition.noDiscard !== "boolean") {
    throw new Error(`invalid noDiscard content policy for ${item.id}`);
  }
}

console.log("WOS49 discard static guards passed");

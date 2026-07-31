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
const nativeLib = read("native", "crates", "woc_protocol", "src", "lib.rs");
const content = JSON.parse(read("contracts", "m5_content.json"));

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "current WOS writer must retain WOS48 quest semantics");
requireText(
  world,
  /schemaVersion != <uint>45 &&\s*schemaVersion != <uint>46 &&\s*schemaVersion != <uint>47 &&\s*schemaVersion != <uint>48 &&\s*schemaVersion != <uint>49 &&\s*schemaVersion != <uint>50 &&\s*schemaVersion != <uint>51 &&\s*schemaVersion != <uint>52 &&\s*schemaVersion != <uint>53 &&\s*schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS45-WOS55 decoder admission is missing",
);
requireText(
  world,
  /syncOfflineQuestBoarHidesFromInventory[\s\S]*?m5InventoryItemCount[\s\S]*?itemCodeForId\("boar_hide"\)[\s\S]*?count == <uint>5 \? <uint>2 : <uint>1/,
  "q_boars progress is not derived from the M5 boar_hide inventory count",
);
if (/offlineQuestBoarHideCount = state\.offlineQuestBoarHideCount \+ <uint>1/.test(world)) {
  throw new Error("q_boars still increments an independent boar-hide ledger");
}
requireText(
  world,
  /grantM5InventoryItem[\s\S]*?syncOfflineQuestBoarHidesFromInventory\(state\)/,
  "M5 grants do not refresh collect-quest progress",
);
requireText(
  world,
  /removeM5InventoryItem[\s\S]*?syncOfflineQuestBoarHidesFromInventory\(state\)[\s\S]*?return count - remaining/,
  "M5 removals do not refresh collect-quest progress",
);
requireText(
  world,
  /applyOfflineQuestCommand[\s\S]*?code == 2\) \{ syncOfflineQuestBoarHidesFromInventory\(state\); \}/,
  "q_boars acceptance does not read pre-existing inventory",
);
requireText(
  world,
  /if \(code == 2\) \{[\s\S]*?m5InventoryItemCount[\s\S]*?removeM5InventoryItem\(state, actorIndex,[\s\S]*?, 5\)/,
  "q_boars turn-in does not consume the authoritative inventory items",
);
requireText(
  world,
  /if \(schemaVersion < <uint>48\) \{[\s\S]*?migrateWos47QuestBoarHidesToInventory\(state\)/,
  "WOS47 collect-progress migration is missing",
);
requireText(
  world,
  /offlineQuestInventoryStateTest[\s\S]*?grantM5InventoryItem[\s\S]*?m5InventoryItemCount[\s\S]*?restored\.offlineQuestBoarsState/,
  "WOS48 inventory-derived collect lifecycle coverage is missing",
);

requireText(main, /\\"world_state\\":\\"WOS67\\"/, "package WOS64 identity is missing");
requireText(nativeLib, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(nativeLib, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");

const boars = content.quests.find((quest) => quest.id === "q_boars")?.definition;
const objective = boars?.objectives?.[0];
if (objective?.type !== "collect" || objective.itemId !== "boar_hide" || objective.count !== 5) {
  throw new Error("q_boars source collect objective drifted");
}

console.log("WOS48 quest inventory static guards passed");

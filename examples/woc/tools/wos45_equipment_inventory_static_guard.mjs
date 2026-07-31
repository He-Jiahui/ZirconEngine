import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const equipment = read("scripts", "woc_game", "src", "progression", "m5_equipment_state.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
const contentCodegen = read("tools", "m5_content_codegen.mjs");
const contentProjection = read("tools", "m5_content_zr_codegen.mjs");
const content = JSON.parse(read("contracts", "m5_content.json"));

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "current WOS writer must retain WOS45 semantics");
requireText(
  world,
  /schemaVersion != <uint>44 &&\s*schemaVersion != <uint>45 &&\s*schemaVersion != <uint>46 &&\s*schemaVersion != <uint>47 &&\s*schemaVersion != <uint>48 &&\s*schemaVersion != <uint>49 &&\s*schemaVersion != <uint>50 &&\s*schemaVersion != <uint>51 &&\s*schemaVersion != <uint>52 &&\s*schemaVersion != <uint>53 &&\s*schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS44-WOS55 decoder admission is missing",
);
requireText(
  world,
  /equipM5InventoryItem[\s\S]*?removeM5InventoryItem[\s\S]*?grantM5InventoryItem[\s\S]*?setM5EquipmentItemCode/,
  "M5 equipment replacement does not remove, return, then equip",
);
requireText(
  world,
  /unequipM5InventoryItem[\s\S]*?m5InventoryCanAddItem[\s\S]*?setM5EquipmentItemCode[\s\S]*?grantM5InventoryItem/,
  "M5 unequip does not preflight capacity before returning gear",
);
requireText(
  world,
  /m5Equipment\.startingMainhandStoredCode[\s\S]*?m5Equipment\.emptyMainhandStoredCode/,
  "M5 mainhand start-to-empty transaction boundary is missing",
);
requireText(
  world,
  /WOS45 distinguishes an equipped source starting weapon[\s\S]*?restoredEquipmentInventoryState/,
  "WOS45 lifecycle transaction coverage is missing",
);

requireText(
  equipment,
  /pub emptyMainhandStoredCode\(\): uint \{\s*return <uint>255;/,
  "explicit empty-mainhand stored code is missing",
);
requireText(
  equipment,
  /code == <uint>0\) \{ return -1; \}[\s\S]*?emptyMainhandStoredCode\(\).*?return -2;/,
  "starting and empty mainhand codes are not distinct",
);
requireText(
  equipment,
  /startingMainhandStoredCode[\s\S]*?baseline\.startingEquipmentText[\s\S]*?catalogItemIndexIsValidForSlot/,
  "source starting mainhand catalog bridge is missing",
);
requireText(
  equipment,
  /if \(state\.mainhandItemIndex != -1\)/,
  "empty mainhand does not remove the baseline weapon contribution",
);
requireText(
  equipment,
  /state\.mainhandItemIndex == -2[\s\S]*?return 1;/,
  "unarmed mainhand minimum fallback is missing",
);

requireText(main, /\\"world_state\\":\\"WOS67\\"/, "package WOS64 identity is missing");
requireText(protocol, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(protocol, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");
requireText(contentCodegen, /EXPECTED_CLASS_STARTING_EQUIPMENT_ITEM_IDS/, "class starting equipment pin is missing");
requireText(contentCodegen, /class_starting_equipment/, "class starting equipment provenance is missing");
requireText(contentProjection, /items: 35/, "M5 content projection item count is stale");

if (content.items.length !== 35) throw new Error("M5 content item count drifted");
const legacyItemPrefix = [
  "baked_bread",
  "bandit_bandana",
  "boar_hide",
  "cryptbone_helm",
  "elixir_of_the_bear",
  "gnarled_staff",
  "greyjaw_hide_boots",
  "milepost_boots",
  "minor_healing_potion",
  "roadwardens_helm",
  "spring_water",
  "wolf_fang",
  "wolfhide_satchel",
  "worn_sword",
];
if (JSON.stringify(content.items.slice(0, 14).map((item) => item.id)) !==
    JSON.stringify(legacyItemPrefix)) {
  throw new Error("WOS44 item-code prefix was reordered");
}
const items = new Map(content.items.map((item) => [item.id, item]));
for (const id of [
  "worn_sword",
  "gnarled_staff",
  "rusty_dagger",
  "training_mace",
  "rusty_hatchet",
  "eastbrook_buckler",
  "recruit_tunic",
  "apprentice_robe",
  "footpad_jerkin",
]) {
  if (!items.get(id)?.scenarios.includes("class_starting_equipment")) {
    throw new Error(`M5 class starting equipment source is missing ${id}`);
  }
}

console.log("WOS45 equipment/inventory static guards passed");

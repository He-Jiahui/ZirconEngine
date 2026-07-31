import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const rules = read("scripts", "woc_game", "src", "progression", "m5_inventory_rules.zr");
const main = read("scripts", "woc_game", "src", "main.zr");
const protocol = read("native", "crates", "woc_protocol", "src", "lib.rs");
const content = JSON.parse(read("contracts", "m5_content.json"));

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "WOS44 inventory is not retained by the current WOS envelope");
requireText(
  world,
  /schemaVersion != <uint>44 &&\s*schemaVersion != <uint>45 &&\s*schemaVersion != <uint>46 &&\s*schemaVersion != <uint>47 &&\s*schemaVersion != <uint>48 &&\s*schemaVersion != <uint>49 &&\s*schemaVersion != <uint>50 &&\s*schemaVersion != <uint>51 &&\s*schemaVersion != <uint>52 &&\s*schemaVersion != <uint>53 &&\s*schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS44-WOS55 decoder admission is missing",
);
requireText(world, /if \(schemaVersion >= <uint>44\)[\s\S]*?entityInventoryCopper\.add[\s\S]*?entityInventoryStackOffsets\.add/, "WOS44 decoder rows are missing");
requireText(world, /else \{\s*appendDefaultM5InventoryColumns\(state\);\s*\}/, "WOS2-WOS43 inventory default migration is missing");
requireText(world, /m5InventoryInsertStack[\s\S]*?entityInventoryStackOffsets\[offsetIndex\][\s\S]*?\+ <uint>1/, "inventory insert offset repair is missing");
requireText(world, /removeM5InventoryItem[\s\S]*?entityInventoryStackOffsets\[entityIndex \+ 1\] - 1/, "newest-first inventory removal is missing");
requireText(world, /m5InventoryCanAddItem[\s\S]*?grantM5InventoryItem/, "inventory grant/capacity boundary is missing");
requireText(rules, /BACKPACK_SLOTS: int = 16/, "source backpack capacity is missing");
requireText(rules, /BAG_SOCKETS: int = 4/, "source bag socket count is missing");
requireText(rules, /DEFAULT_STACK_SIZE: int = 20/, "source default stack limit is missing");
requireText(main, /m5InventoryRules\.contractTest\(\) != 1/, "lifecycle inventory rules contract is missing");
requireText(protocol, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(protocol, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");

const items = new Map(content.items.map((item) => [item.id, item.definition]));
for (const [id, kind] of [["baked_bread", "food"], ["worn_sword", "weapon"], ["wolfhide_satchel", "bag"]]) {
  if (items.get(id)?.kind !== kind) throw new Error(`M5 content item ${id} drifted`);
}
if (items.get("wolfhide_satchel")?.bagSlots !== 10) {
  throw new Error("M5 source satchel capacity drifted");
}

console.log("WOS44 inventory static guards passed");

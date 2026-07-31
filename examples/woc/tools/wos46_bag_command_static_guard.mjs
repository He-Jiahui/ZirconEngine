import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const payloads = read("scripts", "woc_game", "src", "protocol", "command_payloads.zr");
const commands = read("scripts", "woc_game", "src", "protocol", "commands.zr");
const nativePayloads = read("native", "crates", "woc_protocol", "src", "command_payload.rs");

requireText(commands, /id == <uint>126\) \{ return "equip_bag";? \}/, "equip_bag id drifted");
requireText(commands, /id == <uint>127\) \{ return "unequip_bag";? \}/, "unequip_bag id drifted");
requireText(payloads, /equipBagCommandId[\s\S]*?return <uint>126;/, "equip_bag payload id is missing");
requireText(payloads, /unequipBagCommandId[\s\S]*?return <uint>127;/, "unequip_bag payload id is missing");
requireText(payloads, /payloadLength[\s\S]*?id == <uint>126\) \{ return -1; \}[\s\S]*?id == <uint>127\) \{ return 4; \}/, "bag payload lengths drifted");
requireText(payloads, /payloadMinLength[\s\S]*?id == <uint>126\) \{ return 5; \}[\s\S]*?id == <uint>127\) \{ return 4; \}/, "bag payload minimums drifted");
requireText(nativePayloads, /struct EquipBagCommandPayload[\s\S]*?encode_utf8_id_optional_u32/, "native equip_bag shape drifted");
requireText(nativePayloads, /struct UnequipBagCommandPayload[\s\S]*?socket: u32/, "native unequip_bag shape drifted");

requireText(world, /var equipBagCommand = payloads\.equipBagCommandId\(true\);/, "equip_bag command is not loaded");
requireText(world, /var unequipBagCommand = payloads\.unequipBagCommandId\(true\);/, "unequip_bag command is not loaded");
requireText(world, /commandId == equipBagCommand[\s\S]*?applyM5EquipBagCommand/, "equip_bag is not dispatched");
requireText(world, /commandId == unequipBagCommand[\s\S]*?applyM5UnequipBagCommand/, "unequip_bag is not dispatched");
requireText(world, /applyM5EquipBagCommand[\s\S]*?itemLength != <uint>16[\s\S]*?equipM5InventoryBag/, "scalar equip_bag payload bridge is missing");
requireText(world, /applyM5UnequipBagCommand[\s\S]*?payloadLength != <uint>4[\s\S]*?unequipM5InventoryBag/, "scalar unequip_bag payload bridge is missing");
requireText(world, /appendM5EquipBagCommand[\s\S]*?appendM5UnequipBagCommand[\s\S]*?bagCommandState\.applyCommands/, "WOS46 command lifecycle coverage is missing");

console.log("WOS46 bag command static guards passed");

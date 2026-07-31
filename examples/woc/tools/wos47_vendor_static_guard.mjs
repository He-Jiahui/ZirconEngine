import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const world = read("scripts", "woc_game", "src", "world", "state.zr");
const inventoryVendorState = read(
  "scripts", "woc_game", "src", "progression", "inventory_vendor_state.zr",
);
const main = read("scripts", "woc_game", "src", "main.zr");
const catalog = read("scripts", "woc_game", "src", "generated", "m5_content_catalog.zr");
const payloads = read("scripts", "woc_game", "src", "protocol", "command_payloads.zr");
const commands = read("scripts", "woc_game", "src", "protocol", "commands.zr");
const native = read("native", "crates", "woc_protocol", "src", "command_payload.rs");
const nativeLib = read("native", "crates", "woc_protocol", "src", "lib.rs");
const nativeTests = read("native", "crates", "woc_protocol", "tests", "command_payloads.rs");
const sourceExtract = read("tools", "m5_content_source_extract.mjs");
const contentCodegen = read("tools", "m5_content_codegen.mjs");
const contentProjection = read("tools", "m5_content_zr_codegen.mjs");
const contracts = JSON.parse(read("contracts", "command_payloads.json"));
const content = JSON.parse(read("contracts", "m5_content.json"));

requireText(world, /writer\.u16\(<uint>67, 1, 1\)/, "current WOS writer must retain WOS47 vendor state");
requireText(
  world,
  /schemaVersion != <uint>45 &&\s*schemaVersion != <uint>46 &&\s*schemaVersion != <uint>47 &&\s*schemaVersion != <uint>48 &&\s*schemaVersion != <uint>49 &&\s*schemaVersion != <uint>50 &&\s*schemaVersion != <uint>51 &&\s*schemaVersion != <uint>52 &&\s*schemaVersion != <uint>53 &&\s*schemaVersion != <uint>54 &&\s*schemaVersion != <uint>55/,
  "WOS45-WOS55 decoder admission is missing",
);
requireText(
  world,
  /entityVendorBuybackOffsets[\s\S]*?entityVendorBuybackItemCodes[\s\S]*?entityVendorBuybackCounts/,
  "WOS47 buyback state columns are missing",
);
requireText(
  world,
  /if \(schemaVersion >= <uint>47\)[\s\S]*?buybackCount = reader\.byte[\s\S]*?buybackCount > <uint>12[\s\S]*?entityVendorBuybackOffsets\.add/,
  "WOS47 bounded buyback decoder is missing",
);
requireText(
  world,
  /m5VendorRecordBuyback[\s\S]*?m5VendorBuybackInsert[\s\S]*?end - start > 12/,
  "newest-first bounded buyback recorder is missing",
);
requireText(
  world,
  /var buyCommand = payloads\.buyCommandId\(true\);[\s\S]*?var sellCommand = payloads\.sellCommandId\(true\);[\s\S]*?var buybackCommand = payloads\.buybackCommandId\(true\);/,
  "vendor command ids are not loaded",
);
requireText(
  world,
  /commandId == buyCommand[\s\S]*?applyM5VendorBuyCommand[\s\S]*?commandId == sellCommand[\s\S]*?applyM5VendorSellCommand[\s\S]*?commandId == buybackCommand[\s\S]*?applyM5VendorBuybackCommand[\s\S]*?commandId == sellAllJunkCommand[\s\S]*?applyM5VendorSellAllJunkCommand/,
  "vendor commands are not dispatched",
);
requireText(
  world,
  /applyM5VendorBuyCommand[\s\S]*?m5VendorNpcIndex[\s\S]*?catalog\.npcSells[\s\S]*?m5InventoryCanAddItem/,
  "vendor buy reducer is incomplete",
);
requireText(
  world,
  /var quantity = m5VendorStack\.vendorPurchaseQuantity[\s\S]*?var copperCost = unitPrice \* quantity/,
  "authoritative vendor buy must charge the source per-unit price for each stack item",
);
requireText(
  inventoryVendorState,
  /var quantity = vendorStacks\.vendorPurchaseQuantity\(kind\);[\s\S]*?var cost = <int>catalog\.metric\("item", item, "buyValue"\) \* quantity/,
  "inventory vendor model must charge the source per-unit price for each stack item",
);
requireText(
  world,
  /presence == <uint>0 && trailing != <uint>1[\s\S]*?presence == <uint>1 && trailing != <uint>9/,
  "vendor buy rejects malformed optional target tails",
);
requireText(
  world,
  /applyM5VendorSellCommand[\s\S]*?flag\("item", itemIndex, "noVendorSell"\)[\s\S]*?flag\("item", itemIndex, "soulbound"\)[\s\S]*?removeM5InventoryItem[\s\S]*?m5VendorRecordBuyback/,
  "vendor sell reducer is incomplete",
);
requireText(
  world,
  /applyM5VendorBuybackCommand[\s\S]*?m5VendorBuybackAvailable[\s\S]*?m5VendorTakeBuyback/,
  "vendor buyback reducer is incomplete",
);
requireText(
  world,
  /applyM5VendorSellAllJunkCommand[\s\S]*?quality"\) == "poor"[\s\S]*?flag\("item", itemIndex, "noVendorSell"\)[\s\S]*?flag\("item", itemIndex, "soulbound"\)[\s\S]*?m5VendorRecordBuyback/,
  "sell-all-junk reducer is incomplete",
);
requireText(
  world,
  /WOS47 puts vendor purchases[\s\S]*?restoredVendorState/,
  "WOS47 lifecycle round-trip coverage is missing",
);

requireText(sourceExtract, /vendorItemIds[\s\S]*?vendor_item_ids/, "vendor source extraction is missing");
requireText(contentCodegen, /EXPECTED_VENDOR_ITEM_IDS[\s\S]*?addDerivedVendorItemUses/, "vendor item pin is missing");
requireText(contentProjection, /items: 35/, "WOS47 content projection count is stale");
requireText(catalog, /pub itemIdUtf8Length[\s\S]*?pub itemIdUtf8Byte/, "scalar UTF-8 catalog query is missing");
requireText(catalog, /field == "noVendorSell"[\s\S]*?field == "soulbound"/, "vendor policy flags are missing");
requireText(catalog, /pub npcHasVendorStock[\s\S]*?index == 5\) \{ return true; \}/, "Trader Wilkes vendor projection is missing");

for (const [id, commandId, kind] of [
  ["buy", 25, "utf8_id_optional_target_entity"],
  ["sell", 26, "utf8_id_optional_u32"],
  ["buyback", 27, "utf8_id"],
]) {
  const entry = contracts.entries.find((candidate) => candidate.id === commandId);
  if (entry?.name !== id || entry.kind !== kind) {
    throw new Error(`vendor command contract drifted for ${id}`);
  }
}
requireText(commands, /id == <uint>25\) \{ return "buy";? \}/, "buy command id drifted");
requireText(commands, /id == <uint>26\) \{ return "sell";? \}/, "sell command id drifted");
requireText(commands, /id == <uint>27\) \{ return "buyback";? \}/, "buyback command id drifted");
requireText(payloads, /buyCommandId[\s\S]*?return <uint>25;/, "buy payload id is missing");
requireText(payloads, /sellCommandId[\s\S]*?return <uint>26;/, "sell payload id is missing");
requireText(payloads, /buybackCommandId[\s\S]*?return <uint>27;/, "buyback payload id is missing");
requireText(native, /struct BuyItemCommandPayload[\s\S]*?npc_id: u64[\s\S]*?item_id: String/, "native buy payload is missing");
requireText(native, /struct SellItemCommandPayload[\s\S]*?count: Option<u32>/, "native sell payload is missing");
requireText(native, /struct BuybackItemCommandPayload[\s\S]*?encode_utf8_id/, "native buyback payload is missing");
requireText(nativeTests, /vendor_payloads_preserve_source_item_and_npc_fields/, "native vendor payload coverage is missing");
requireText(main, /\\"world_state\\":\\"WOS67\\"/, "package WOS64 identity is missing");
requireText(nativeLib, /WORLD_STATE_FORMAT: &str = "WOS67"/, "native WOS64 format is missing");
requireText(nativeLib, /WORLD_STATE_SCHEMA_VERSION: u16 = 67/, "native WOS64 version is missing");

if (content.items.length !== 35) throw new Error("WOS47 vendor catalog item count drifted");
const wilkes = content.npcs.find((npc) => npc.id === "trader_wilkes");
if (wilkes?.definition?.vendorItems?.length !== 17) {
  throw new Error("Trader Wilkes vendor stock drifted");
}

console.log("WOS47 vendor static guards passed");

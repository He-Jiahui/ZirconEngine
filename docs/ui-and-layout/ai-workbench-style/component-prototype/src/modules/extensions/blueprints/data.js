import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const dataBlueprints = {
  "data-table": blueprint({
    status: "Data table schema and selected row ready",
    actions: [["plus", "Add Row"], ["folder", "Import CSV"], ["check", "Validate Data"], ["save", "Save Table"]],
    tools: ["Schema", "CSV Import", "Diff Rows", "Validation", "References", "Bulk Edit"],
    assets: tree("Data", "grid", ["DT_Items", "Schema_Item", "Row_Sword_01", "Row_Potion_Health", "Localization"]),
    metrics: [["Rows", "128"], ["Columns", "14"], ["Invalid", "2", "warning"], ["Refs", "512"]],
    detailTabs: ["Rows", "Schema", "Validation"],
    settings: [["Row Name", inputValue("Potion_Health")], ["Row Type", selectValue("Gameplay Item")], ["Version", inputValue("12")], ["Localized", checkValue(true)], ["Deprecated", checkValue(false)]],
    primary: tablePrimary("Data Table Rows", ["Row", "Type", "Value", "State"], [["Potion_Health", "Consumable", "+50 HP", "Selected"], ["Sword_01", "Weapon", "12 DPS", "Ready"], ["Armor_Heavy", "Armor", "42 DEF", "Ready"], ["Debug_Item", "Item", "Missing Icon", "Warning"]], "1.2fr 1fr 1fr 0.8fr")
  }),
  "save-data": blueprint({
    status: "Save slot migration and diff selected",
    actions: [["save", "Save Slot"], ["folder", "Load Slot"], ["check", "Validate Save"], ["history", "Migrate Save"]],
    tools: ["Slot Schema", "Migration Map", "Object Diff", "Cloud Sync", "Corruption Scan", "Runtime Probe"],
    assets: tree("Saves", "save", ["AutoSave_01", "Manual_03", "Cloud_02", "Schema_v4", "Migration_v3_v4"]),
    metrics: [["Slots", "6"], ["Schemas", "4"], ["Migrations", "2"], ["Warnings", "1", "warning"]],
    detailTabs: ["Slots", "Migration", "Validation"],
    settings: [["Schema", selectValue("SaveData v4")], ["Slot", selectValue("AutoSave_01")], ["Compression", selectValue("LZ4")], ["Cloud Sync", checkValue(true)], ["Strict Load", checkValue(true)]],
    primary: tablePrimary("Save Data Diff", ["Object", "Field", "Value", "State"], [["PlayerState", "Level", "12", "Selected"], ["Inventory", "Items", "42", "Ready"], ["QuestLog", "Version", "v3", "Migrating"], ["DebugSlot", "Schema", "Old", "Warning"]], "1fr 1fr 0.8fr 0.8fr")
  })
};

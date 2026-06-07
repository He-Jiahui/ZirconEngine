import { checkbox, select } from "../../../../components/inputs/atoms.js";

export const runtimeRecipe = {
  detailTabs: ["Slots", "Migration", "Validation"],
  actions: (subject, shortLabel) => [["save", `Save ${shortLabel}`], ["folder", `Load ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["history", `Migrate ${shortLabel}`]],
  tools: (subject) => ["Slot Schema", "Migration Map", "Runtime Probe", "Cloud Sync", `${subject} Diff`, "Corruption Scan"],
  metrics: () => [["Slots", "6"], ["Schemas", "4"], ["Migrations", "2"], ["Warnings", "1", "warning"]],
  settings: (subject) => [["Schema", select(`${subject} v4`)], ["Slot", select("AutoSave_01")], ["Compression", select("LZ4")], ["Cloud Sync", checkbox("", true)], ["Strict Load", checkbox("", true)]],
  table: () => [["AutoSave_01", "Ready", "2.4 MB"], ["Manual_03", "Migrating", "1.8 MB"], ["Cloud_02", "Queued", "4.1 MB"], ["DebugSlot", "Warning", "Old"]]
};

import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const gameplayBlueprints = {
  "spawn-rules": blueprint({
    status: "Spawn rule simulation selected",
    actions: [["plus", "Add Spawn Rule"], ["play", "Simulate Spawn"], ["target", "Inspect Spawn"], ["check", "Validate Spawn"]],
    tools: ["Rule Stack", "Spawn Zone", "Condition", "Tag Filter", "Probe", "Conflict Check"],
    assets: tree("Spawn", "target", ["SpawnRules_Enemy", "Zone_A", "Condition_Night", "Tag_Combat", "Probe_01"]),
    metrics: [["Rules", "18"], ["Zones", "12"], ["Conflicts", "1", "warning"], ["Spawns", "96"]],
    detailTabs: ["Rules", "State", "Validation"],
    settings: [["Rule Set", selectValue("Enemy Spawn")], ["Authority", selectValue("Server")], ["Seed", inputValue("2026")], ["Live Preview", checkValue(true)], ["Strict Tags", checkValue(true)]],
    primary: graphPrimary("Spawn Rule Stack", [["Zone_A", "Volume", 12, 36, "cyan"], ["Condition_Night", "Condition", 34, 22, "blue"], ["Tag_Combat", "Filter", 54, 42, "green"], ["Spawn Enemy", "Action", 74, 30, "orange"], ["Conflict", "Validation", 48, 68, "purple"]])
  }),
  "world-state": blueprint({
    status: "World state keys and scenario timeline selected",
    actions: [["plus", "Add State Key"], ["play", "Simulate World State"], ["target", "Inspect State"], ["check", "Validate State"]],
    tools: ["State Layer", "Scenario", "Key Value", "Region System", "Timeline", "Conflict Check"],
    assets: tree("World State", "globe", ["Scenario_NightRaid", "Layer_Global", "Key_Alarm", "Region_A", "System_AI"]),
    metrics: [["Keys", "84"], ["Layers", "6"], ["Conflicts", "1", "warning"], ["Events", "42"]],
    detailTabs: ["Keys", "Scenario", "Timeline"],
    settings: [["Scenario", selectValue("Night Raid")], ["Layer", selectValue("Global")], ["Authority", selectValue("Server")], ["Live Preview", checkValue(true)], ["Strict Keys", checkValue(true)]],
    primary: tablePrimary("World State Keys", ["Key", "Layer", "Value", "State"], [["Alarm.Active", "Global", "true", "Selected"], ["Weather.Mode", "Region", "Storm", "Ready"], ["AI.Alert", "System", "High", "Ready"], ["Quest.Flag", "Scenario", "Conflict", "Warning"]], "1fr 0.8fr 0.8fr 0.8fr")
  })
};

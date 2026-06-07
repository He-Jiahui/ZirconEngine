import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const multiplayerBlueprints = {
  "lobby-editor": blueprint({
    status: "Lobby slots and online rules simulated",
    actions: [["play", "Simulate Lobby"], ["plus", "Add Slot"], ["check", "Validate Lobby"], ["save", "Publish Lobby"]],
    tools: ["Lobby Template", "Slot Rule", "Presence State", "Party Join", "Region Map", "Network Output"],
    assets: tree("Online", "component", ["Lobby_Default", "Slot_Leader", "Slot_Guest", "Rule_Crossplay", "Region_Auto"]),
    metrics: [["Slots", "8"], ["Players", "4"], ["Regions", "6"], ["Failures", "1", "warning"]],
    detailTabs: ["Slots", "Rules", "Telemetry"],
    settings: [["Template", selectValue("Lobby_Default")], ["Region", selectValue("Auto")], ["Max Players", inputValue("4")], ["Crossplay", checkValue(true)], ["Backfill", checkValue(false)]],
    primary: tablePrimary("Lobby Slot Simulation", ["Slot", "State", "Player", "Rule"], [["Leader", "Ready", "Player_01", "Host"], ["Guest_01", "Joined", "Player_02", "Open"], ["Guest_02", "Waiting", "-", "Open"], ["Spectator", "Disabled", "-", "Locked"]], "0.9fr 0.8fr 1fr 0.8fr")
  }),
  "matchmaking-editor": blueprint({
    status: "Matchmaking queue and playlist rule selected",
    actions: [["play", "Simulate Matchmaking"], ["target", "Match Queue"], ["check", "Validate Rules"], ["save", "Publish Playlist"]],
    tools: ["Playlist Rule", "Skill Bucket", "Latency Region", "Party Size", "Backfill", "Failure Report"],
    assets: tree("Matchmaking", "target", ["Playlist_Ranked", "Queue_Solo", "Rule_SkillRange", "Rule_Latency", "Backfill_Set"]),
    metrics: [["Queues", "6"], ["Players", "128"], ["Latency", "42 ms"], ["Failures", "2", "warning"]],
    detailTabs: ["Queues", "Rules", "Telemetry"],
    settings: [["Playlist", selectValue("Ranked")], ["Region", selectValue("NA-East")], ["Max Wait", inputValue("90")], ["Skill Relax", checkValue(true)], ["Backfill", checkValue(true)]],
    primary: tablePrimary("Matchmaking Queue", ["Bucket", "Players", "Latency", "State"], [["Bronze", "28", "42 ms", "Open"], ["Gold", "64", "48 ms", "Selected"], ["Diamond", "18", "58 ms", "Limited"], ["Backfill", "18", "62 ms", "Queued"]], "1fr 0.8fr 0.8fr 0.8fr")
  })
};

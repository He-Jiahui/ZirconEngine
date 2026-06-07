import { blueprint, checkValue, graphPrimary, inputValue, selectValue, tree } from "../helpers.js";

export const levelStreamingBlueprint = blueprint({
  status: "World cells and streaming rules selected",
  actions: [["plus", "Add Level"], ["grid", "Load Cell"], ["check", "Validate Streaming"], ["play", "Preview Streaming"]],
  tools: ["Cell Grid", "Load Rule", "HLOD", "Streaming Source", "Visibility Layer", "Event Trace"],
  assets: tree("World", "globe", ["World_Main", "Cell_A12", "Cell_A13", "HLOD_Cluster_04", "Rule_PlayerDistance"]),
  metrics: [["Cells", "96"], ["Loaded", "18"], ["HLOD", "24"], ["Warnings", "2", "warning"]],
  detailTabs: ["Cells", "Rules", "Events"],
  settings: [["Cell", selectValue("Cell_A12")], ["Rule", selectValue("Player Distance")], ["Distance", inputValue("5000")], ["Async Load", checkValue(true)], ["Show Bounds", checkValue(true)]],
  primary: graphPrimary("Level Streaming Map", [["Player", "Source", 16, 52, "cyan"], ["Cell_A12", "Loaded", 38, 34, "green"], ["Cell_A13", "Queued", 60, 42, "blue"], ["HLOD_04", "Visible", 44, 68, "orange"], ["Cell_B12", "Hidden", 76, 58, "purple"]])
});

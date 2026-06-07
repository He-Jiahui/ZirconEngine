import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const diagnosticsBlueprints = {
  "console-diagnostics": blueprint({
    status: "Console diagnostics log filtered",
    actions: [["search", "Filter Console"], ["trash", "Clear Console"], ["save", "Export Log"], ["check", "Open Issue"]],
    tools: ["Log Filter", "Counters", "Trace Events", "Warning Buckets", "Report", "Session Diff"],
    assets: tree("Diagnostics", "info", ["Session_12_10", "Renderer", "Asset", "Gameplay", "Network"]),
    metrics: [["FPS", "58"], ["Warnings", "24", "warning"], ["Errors", "1", "warning"], ["Marks", "82"]],
    detailTabs: ["Live Log", "Counters", "Report"],
    settings: [["Subsystem", selectValue("Renderer")], ["Severity", selectValue("Warnings+")], ["Regex", inputValue("texture|shader")], ["Collapse Repeats", checkValue(true)], ["Follow Tail", checkValue(true)]],
    primary: tablePrimary("Console Live Feed", ["Time", "Subsystem", "Level", "Message"], [["12:10:11", "Renderer", "Warning", "Missing transient view"], ["12:10:13", "Asset", "Info", "Import completed"], ["12:10:18", "Gameplay", "Warning", "Tag redirect used"], ["12:10:21", "Runtime", "Error", "Null object path"]], "0.8fr 1fr 0.8fr 1.6fr")
  }),
  "performance": blueprint({
    status: "Performance capture frame selected",
    actions: [["target", "Capture Frame"], ["search", "Filter Samples"], ["save", "Export Trace"], ["check", "Open Hotspot"]],
    tools: ["Frame Capture", "CPU Lane", "GPU Lane", "Memory Track", "Marker Filter", "Hotspot Report"],
    assets: tree("Performance", "info", ["Capture_1234", "CPU_GameThread", "GPU_Lighting", "Memory_Textures", "Marker_AI"]),
    metrics: [["Frame", "16.8 ms"], ["GPU", "9.2 ms"], ["CPU", "7.1 ms"], ["Spikes", "3", "warning"]],
    detailTabs: ["Samples", "Counters", "Report"],
    settings: [["Capture", selectValue("Frame 1234")], ["Lane", selectValue("GPU")], ["Threshold", inputValue("1.0")], ["Show Markers", checkValue(true)], ["Collapse Children", checkValue(true)]],
    primary: timelinePrimary("Performance Timeline", ["Lane", "Cost", "State"], [["Game Thread", "7.1 ms", "Ready"], ["Render Thread", "4.8 ms", "Selected"], ["GPU Lighting", "3.2 ms", "Hotspot"], ["Texture Upload", "1.4 ms", "Warning"]])
  }),
  "runtime-diagnostics": blueprint({
    status: "Runtime session watch tree selected",
    actions: [["search", "Filter Runtime"], ["trash", "Clear Runtime"], ["save", "Export Runtime"], ["check", "Open Runtime Issue"]],
    tools: ["Watch Tree", "Target Session", "Live Values", "Event Stream", "Console", "Snapshot Diff"],
    assets: tree("Runtime", "info", ["Session_Player_01", "World", "Actors", "Components", "Events"]),
    metrics: [["Actors", "420"], ["Events", "1.2K"], ["Errors", "1", "warning"], ["FPS", "58"]],
    detailTabs: ["Watch", "Events", "Console"],
    settings: [["Session", selectValue("Player_01")], ["Subsystem", selectValue("World")], ["Filter", inputValue("health")], ["Follow", checkValue(true)], ["Record", checkValue(false)]],
    primary: tablePrimary("Runtime Watch Values", ["Path", "Type", "Value", "State"], [["Player.Health", "Float", "82", "Selected"], ["AI.Guard.State", "Enum", "Alert", "Ready"], ["World.Time", "Float", "142.4", "Ready"], ["Weapon.Target", "Object", "Null", "Warning"]], "1.4fr 0.8fr 0.8fr 0.8fr")
  }),
  "telemetry-dashboard": blueprint({
    status: "Telemetry query dashboard filtered",
    actions: [["search", "Filter Telemetry"], ["target", "Run Query"], ["save", "Export Telemetry"], ["check", "Open Metric"]],
    tools: ["Query Builder", "Saved Query", "Event Segment", "Metric Detail", "Raw Events", "Dashboard"],
    assets: tree("Telemetry", "info", ["Query_Retention", "Event_Login", "Segment_NewUsers", "Metric_FPS", "Raw_Stream"]),
    metrics: [["Events", "2.4M"], ["Segments", "12"], ["Alerts", "3", "warning"], ["Latency", "120 ms"]],
    detailTabs: ["Metrics", "Segments", "Raw Events"],
    settings: [["Query", selectValue("Retention")], ["Range", selectValue("24h")], ["Segment", selectValue("New Users")], ["Live Refresh", checkValue(true)], ["Sample Raw", checkValue(false)]],
    primary: tablePrimary("Telemetry Metrics", ["Metric", "Value", "Delta", "State"], [["DAU", "42K", "+8%", "Ready"], ["FPS P95", "58", "-2", "Warning"], ["Crash Rate", "0.12%", "-0.04", "Ready"], ["Queue Wait", "42s", "+6s", "Selected"]], "1fr 0.8fr 0.8fr 0.8fr")
  })
};

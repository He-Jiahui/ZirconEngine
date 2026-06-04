export const extensionBlueprints = {
  "terrain-editor": blueprint({
    status: "Terrain sculpt layers and world cells staged",
    actions: [["plus", "Add Layer"], ["grid", "Sculpt Terrain"], ["check", "Build Terrain"], ["play", "Preview Erosion"]],
    tools: ["Sculpt Brush", "Paint Material", "Flatten", "Ramp", "Erosion Mask", "World Cell"],
    assets: tree("Terrain", "globe", ["Landscape_Main", "Heightfield_Ridge", "Layer_Rock", "Layer_Grass", "WorldPartition_A12"]),
    metrics: [["Cells", "64"], ["Layers", "7"], ["Brush", "512"], ["Warnings", "2", "warning"]],
    detailTabs: ["Brush", "Layers", "Streaming"],
    settings: [["Brush Preset", selectValue("Sculpt Soft")], ["Radius", inputValue("512")], ["Strength", inputValue("0.38")], ["Falloff", selectValue("Smooth")], ["Live Preview", checkValue(true)]],
    primary: tablePrimary("Terrain Cell Workspace", ["Cell", "Layer", "State", "LOD"], [["A12_08", "Rock", "Loaded", "3"], ["A12_09", "Grass", "Dirty", "3"], ["A13_08", "Mud", "Queued", "2"], ["Spline_Road_04", "Road", "Ready", "1"]], "1fr 1fr 0.8fr 0.6fr")
  }),
  "lighting-bake": blueprint({
    status: "Lighting bake probes and jobs queued",
    actions: [["sun", "Preview Bake"], ["check", "Build Lighting"], ["target", "Capture Probe"], ["save", "Save Bake"]],
    tools: ["Bake Preset", "Lightmap Density", "Probe Volume", "Reflection Capture", "Shadow Atlas", "Invalidation"],
    assets: tree("Lighting", "sun", ["Bake_High_Interior", "Directional_Key", "ProbeGrid_Lobby", "Reflection_Main", "LM_Floor_A"]),
    metrics: [["Lights", "18"], ["Probes", "420"], ["Bake", "68"], ["Leaks", "3", "warning"]],
    detailTabs: ["Presets", "Probes", "Progress"],
    settings: [["Preset", selectValue("Production")], ["Resolution", selectValue("1024")], ["Bounce Count", inputValue("5")], ["Denoise", checkValue(true)], ["GPU Bake", checkValue(true)]],
    primary: queuePrimary("Lighting Bake Queue", ["Task", "State", "Progress"], [["Direct Light", "Done", "100"], ["Probe Grid", "Running", "68"], ["Reflection Captures", "Queued", "0"], ["Leak Scan", "Warning", "34"]])
  }),
  "sequencer": blueprint({
    status: "Cinematic sequence timeline selected",
    actions: [["play", "Preview Sequence"], ["plus", "Add Track"], ["target", "Key Selection"], ["check", "Validate Sequence"]],
    tools: ["Camera Cut", "Actor Track", "Audio Track", "Event Track", "Curve Editor", "Shot Marker"],
    assets: tree("Cinematics", "history", ["SEQ_Intro", "Camera_A", "Hero_Actor", "Audio_Theme", "Shot_003"]),
    metrics: [["Shots", "12"], ["Tracks", "34"], ["Keys", "428"], ["Gaps", "1", "warning"]],
    detailTabs: ["Tracks", "Curves", "Validation"],
    settings: [["Sequence", selectValue("SEQ_Intro")], ["Frame Rate", selectValue("24 fps")], ["Work Range", inputValue("0100-1460")], ["Snap", checkValue(true)], ["Auto Key", checkValue(false)]],
    primary: timelinePrimary("Sequencer Timeline", ["Track", "Range", "State"], [["Camera Cut", "0000-0180", "Ready"], ["Hero Transform", "0180-0620", "Selected"], ["Audio Theme", "0000-1460", "Ready"], ["Event Cues", "0520-0860", "Warning"]])
  }),
  "montage-editor": blueprint({
    status: "Montage sections and notify tracks visible",
    actions: [["play", "Preview Montage"], ["plus", "Add Section"], ["target", "Add Notify"], ["check", "Validate Montage"]],
    tools: ["Section", "Notify", "Slot Track", "Branch Point", "Root Motion", "Sync Marker"],
    assets: tree("Animation", "play", ["AM_DashAttack", "Dash_Start", "Dash_Loop", "Dash_End", "Notify_HitWindow"]),
    metrics: [["Sections", "4"], ["Notifies", "18"], ["Frames", "240"], ["Root", "OK"]],
    detailTabs: ["Sections", "Notifies", "Blend"],
    settings: [["Montage", selectValue("AM_DashAttack")], ["Slot", selectValue("UpperBody")], ["Blend In", inputValue("0.12")], ["Root Motion", checkValue(true)], ["Loop Preview", checkValue(false)]],
    primary: timelinePrimary("Montage Timeline", ["Section", "Range", "State"], [["Start", "0000-0032", "Ready"], ["Loop", "0032-0140", "Selected"], ["Attack", "0140-0190", "Ready"], ["Recover", "0190-0240", "Ready"]])
  }),
  "physics-collision": blueprint({
    status: "Collider body and contact debug active",
    actions: [["plus", "Add Body"], ["check", "Validate Collision"], ["grid", "Bake Hulls"], ["play", "Run Contacts"]],
    tools: ["Body Setup", "Convex Hull", "Box Collider", "Material Pair", "Contact Debug", "Mass Preview"],
    assets: tree("Physics", "cube", ["PHYS_Crate", "Body_Root", "Hull_00", "Hull_01", "Mat_WoodMetal"]),
    metrics: [["Bodies", "12"], ["Hull Verts", "96"], ["Mass", "48 kg"], ["Errors", "1", "warning"]],
    detailTabs: ["Bodies", "Materials", "Contacts"],
    settings: [["Preset", selectValue("Dynamic Prop")], ["Mass", inputValue("48.0")], ["Friction", inputValue("0.62")], ["Hit Events", checkValue(true)], ["CCD", checkValue(false)]],
    primary: graphPrimary("Physics Collision Graph", [["Body_Root", "Rigid Body", 10, 20, "cyan"], ["Hull_00", "Convex", 34, 24, "blue"], ["Hull_01", "Convex", 58, 36, "green"], ["Contact Pair", "Debug", 42, 62, "orange"], ["Material Pair", "Friction", 72, 58, "purple"]])
  }),
  "navmesh-ai": blueprint({
    status: "Navigation tiles and agent query selected",
    actions: [["target", "Query Path"], ["grid", "Rebuild Tiles"], ["check", "Validate Navmesh"], ["play", "Simulate Agent"]],
    tools: ["Nav Area", "Tile Rebuild", "Agent Radius", "Offmesh Link", "Crowd Debug", "Path Query"],
    assets: tree("Navigation", "target", ["NavData_Main", "Agent_Guard", "Area_Default", "Area_Jump", "Query_Route_A"]),
    metrics: [["Tiles", "284"], ["Agents", "5"], ["Links", "18"], ["Blocked", "2", "warning"]],
    detailTabs: ["Tiles", "Agents", "Queries"],
    settings: [["Agent", selectValue("Guard")], ["Radius", inputValue("42")], ["Height", inputValue("180")], ["Draw Costs", checkValue(true)], ["Crowd Avoidance", checkValue(true)]],
    primary: graphPrimary("Navmesh Query Workspace", [["Start", "Agent", 12, 42, "cyan"], ["Tile A12", "Open", 34, 28, "green"], ["Door Link", "Offmesh", 56, 44, "orange"], ["Goal", "Target", 76, 30, "blue"], ["Blocked Area", "Cost", 48, 68, "purple"]])
  }),
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
  "accessibility-audit": blueprint({
    status: "Accessibility rule audit focused on HUD screen",
    actions: [["check", "Audit Screen"], ["target", "Focus Issue"], ["save", "Export Audit"], ["play", "Preview Fix"]],
    tools: ["Contrast Rules", "Focus Order", "Screen Reader", "Hit Target", "Motion Safe", "Remediation"],
    assets: tree("UI Screens", "check", ["Gameplay_HUD", "Pause_Menu", "Issue_Contrast_04", "Issue_Focus_02", "Rule_Set_A"]),
    metrics: [["Issues", "9", "warning"], ["Critical", "1", "warning"], ["Pass", "42"], ["Screens", "6"]],
    detailTabs: ["Issues", "Rules", "Fixes"],
    settings: [["Screen", selectValue("Gameplay_HUD")], ["Rule Set", selectValue("WCAG AA")], ["Breakpoint", selectValue("Desktop")], ["Show Bounds", checkValue(true)], ["Auto Fix", checkValue(false)]],
    primary: tablePrimary("Accessibility Issues", ["Issue", "Widget", "Severity", "Fix"], [["Contrast", "AmmoText", "High", "Token"], ["Focus Order", "InventoryGrid", "Medium", "Reorder"], ["Target Size", "MapButton", "Low", "Resize"], ["Motion", "HitFlash", "Medium", "Reduce"]], "1fr 1fr 0.8fr 0.8fr")
  }),
  "animation-compression": blueprint({
    status: "Animation compression batch compared",
    actions: [["check", "Compress Batch"], ["target", "Compare Error"], ["history", "Review Curves"], ["save", "Save Profile"]],
    tools: ["Compression Preset", "Error Metric", "Track Filter", "Memory Report", "Curve Trim", "Batch Queue"],
    assets: tree("Animation", "history", ["CMP_Humanoid", "Run_Fwd", "Jump_Land", "Curve_Facial", "Batch_Player"]),
    metrics: [["Clips", "38"], ["Saved", "42 MB"], ["Error", "0.18"], ["Warnings", "2", "warning"]],
    detailTabs: ["Tracks", "Error", "Memory"],
    settings: [["Profile", selectValue("Humanoid High")], ["Max Error", inputValue("0.18")], ["Key Reduction", selectValue("Adaptive")], ["Preserve Curves", checkValue(true)], ["Batch Mode", checkValue(true)]],
    primary: timelinePrimary("Compression Error Timeline", ["Clip", "Error", "Memory"], [["Run_Fwd", "0.12", "1.8 MB"], ["Jump_Land", "0.18", "1.2 MB"], ["Turn_90", "0.09", "0.9 MB"], ["Facial_A", "0.24", "Warning"]])
  }),
  "automation-report": blueprint({
    status: "Automation suite report filtered",
    actions: [["play", "Run Suite"], ["check", "Validate Tests"], ["save", "Publish Report"], ["history", "Review Failures"]],
    tools: ["Suite Filter", "Worker Pool", "Failure Bucket", "Flake History", "Artifact Set", "Report Output"],
    assets: tree("Automation", "check", ["Smoke", "Rendering", "Gameplay", "Failed_Test_12", "Worker_03"]),
    metrics: [["Tests", "642"], ["Failed", "7", "warning"], ["Workers", "16"], ["Flakes", "3", "warning"]],
    detailTabs: ["Failures", "Workers", "Artifacts"],
    settings: [["Suite", selectValue("Rendering")], ["Platform", selectValue("Windows")], ["Retry Count", inputValue("2")], ["Quarantine Flakes", checkValue(true)], ["Upload Artifacts", checkValue(true)]],
    primary: queuePrimary("Automation Worker Queue", ["Test", "State", "Progress"], [["Renderer.Smoke", "Running", "62"], ["Gameplay.Tags", "Queued", "0"], ["Asset.Import", "Passed", "100"], ["UI.Layout", "Failed", "100"]])
  }),
  "blend-space": blueprint({
    status: "Blend samples and preview point selected",
    actions: [["play", "Preview Blend"], ["plus", "Add Sample"], ["target", "Move Sample"], ["check", "Validate Blend"]],
    tools: ["Axis Setup", "Sample Point", "Preview Cursor", "Triangulation", "Sync Group", "Curve Overlay"],
    assets: tree("Blend Spaces", "play", ["BS_Locomotion", "Walk_Fwd", "Run_Fwd", "Strafe_Left", "Sample_Grid"]),
    metrics: [["Samples", "12"], ["Axes", "2"], ["Warnings", "1", "warning"], ["Sync", "OK"]],
    detailTabs: ["Samples", "Axes", "Preview"],
    settings: [["Blend Space", selectValue("BS_Locomotion")], ["X Axis", selectValue("Speed")], ["Y Axis", selectValue("Direction")], ["Snap Samples", checkValue(true)], ["Show Triangles", checkValue(true)]],
    primary: graphPrimary("Blend Sample Map", [["Walk", "Speed 150", 14, 70, "green"], ["Jog", "Speed 320", 38, 48, "cyan"], ["Run", "Speed 600", 68, 28, "blue"], ["Strafe", "Dir -90", 32, 78, "orange"], ["Preview", "Cursor", 52, 56, "purple"]])
  }),
  "build-export": blueprint({
    status: "Build export profile and package queue ready",
    actions: [["play", "Run Build"], ["check", "Validate Package"], ["save", "Publish Build"], ["history", "Review Build"]],
    tools: ["Build Profile", "Cook Step", "Package Output", "Signing", "Archive", "Release Notes"],
    assets: tree("Builds", "save", ["Windows_Client", "Cook_Content", "Pak_Chunk_0", "Installer", "Archive_2026_06"]),
    metrics: [["Steps", "9"], ["Queued", "3"], ["Size", "18 GB"], ["Warnings", "2", "warning"]],
    detailTabs: ["Profile", "Steps", "History"],
    settings: [["Profile", selectValue("Windows Client")], ["Target", selectValue("Shipping")], ["Version", inputValue("2026.06")], ["Strict Cook", checkValue(true)], ["Archive Output", checkValue(true)]],
    primary: queuePrimary("Build Export Queue", ["Step", "State", "Progress"], [["Cook Content", "Running", "62"], ["Package Paks", "Queued", "0"], ["Sign Installer", "Ready", "0"], ["Publish Build", "Waiting", "0"]])
  }),
  "collision-proxy": blueprint({
    status: "Collision proxy generation focused",
    actions: [["plus", "Add Proxy"], ["check", "Validate Proxy"], ["grid", "Bake Proxy"], ["play", "Test Contacts"]],
    tools: ["Proxy Hull", "Channel Mask", "LOD Proxy", "Convex Merge", "Contact Debug", "Bake"],
    assets: tree("Collision", "cube", ["Proxy_RockCliff", "Hull_Proxy_A", "Channel_Player", "Channel_Vehicle", "Bake_Output"]),
    metrics: [["Proxies", "18"], ["Channels", "9"], ["Invalid", "2", "warning"], ["Cost", "0.4 ms"]],
    detailTabs: ["Proxy", "Channels", "Contacts"],
    settings: [["Proxy", selectValue("Proxy_RockCliff")], ["Channel", selectValue("WorldStatic")], ["LOD", selectValue("LOD1")], ["Trace Complex", checkValue(false)], ["Generate Contacts", checkValue(true)]],
    primary: graphPrimary("Collision Proxy Stack", [["Source Mesh", "SM_Rock", 12, 22, "cyan"], ["Decimator", "Proxy", 36, 28, "blue"], ["Hull Merge", "Convex", 58, 44, "green"], ["Channel Mask", "Filter", 38, 66, "orange"], ["Bake Output", "Ready", 74, 62, "purple"]])
  }),
  "control-rig": blueprint({
    status: "Control rig solve graph selected",
    actions: [["play", "Preview Solve"], ["plus", "Add Control"], ["target", "Key Control"], ["check", "Validate Rig"]],
    tools: ["FK Control", "IK Chain", "Constraint", "Space Switch", "Pose Driver", "Solve Order"],
    assets: tree("Rig", "component", ["CR_Hero", "Spine_CTRL", "Hand_IK_L", "Foot_IK_R", "Space_World"]),
    metrics: [["Controls", "64"], ["Bones", "128"], ["Constraints", "18"], ["Warnings", "1", "warning"]],
    detailTabs: ["Controls", "Hierarchy", "Solve"],
    settings: [["Control", selectValue("Hand_IK_L")], ["Space", selectValue("World")], ["Weight", inputValue("1.0")], ["Mirror", checkValue(false)], ["Draw Axes", checkValue(true)]],
    primary: graphPrimary("Control Rig Solve Graph", [["Spine_CTRL", "FK", 12, 28, "cyan"], ["Arm_IK_L", "IK", 38, 18, "blue"], ["Hand_IK_L", "Selected", 60, 38, "green"], ["Foot_IK_R", "IK", 34, 66, "orange"], ["Output Pose", "Solve", 76, 56, "purple"]])
  }),
  "foliage-editor": blueprint({
    status: "Foliage brush and biome clusters visible",
    actions: [["plus", "Add Foliage"], ["grid", "Paint Foliage"], ["check", "Build Clusters"], ["play", "Preview Density"]],
    tools: ["Paint Brush", "Erase Brush", "Density Mask", "Biome Rule", "Cluster Bake", "Scatter Preview"],
    assets: tree("Foliage", "globe", ["FOL_Forest", "Oak_Tall", "Fern_A", "Grass_Clump", "Biome_Riverbank"]),
    metrics: [["Instances", "84K"], ["Types", "12"], ["Clusters", "128"], ["Warnings", "2", "warning"]],
    detailTabs: ["Brush", "Types", "Clusters"],
    settings: [["Foliage Type", selectValue("Oak_Tall")], ["Density", inputValue("0.72")], ["Radius", inputValue("480")], ["Align Normal", checkValue(true)], ["Cast Shadows", checkValue(true)]],
    primary: tablePrimary("Foliage Cluster Workspace", ["Cluster", "Type", "Density", "State"], [["Forest_A12", "Oak", "0.72", "Ready"], ["Forest_A13", "Fern", "0.58", "Selected"], ["River_02", "Grass", "0.81", "Queued"], ["Cliff_01", "Shrub", "0.24", "Warning"]], "1fr 0.8fr 0.8fr 0.8fr")
  }),
  "font-atlas": blueprint({
    status: "Font atlas glyph coverage focused",
    actions: [["plus", "Add Range"], ["check", "Bake Atlas"], ["save", "Export Font"], ["target", "Inspect Glyph"]],
    tools: ["Glyph Range", "Kerning Pair", "Atlas Page", "Fallback Font", "Coverage", "Bake"],
    assets: tree("Fonts", "file", ["Inter_UI", "Range_Latin", "Range_CJK", "Atlas_Page_0", "Fallback_Noto"]),
    metrics: [["Glyphs", "4096"], ["Pages", "4"], ["Missing", "12", "warning"], ["Size", "16 MB"]],
    detailTabs: ["Glyphs", "Kerning", "Coverage"],
    settings: [["Font", selectValue("Inter UI")], ["Range", selectValue("Latin Extended")], ["Size", inputValue("18")], ["SDF", checkValue(true)], ["Include Fallback", checkValue(true)]],
    primary: tablePrimary("Glyph Coverage", ["Range", "Glyphs", "Missing", "Atlas"], [["Latin", "512", "0", "Page 0"], ["Cyrillic", "384", "0", "Page 1"], ["CJK", "2840", "12", "Page 2"], ["Icons", "360", "0", "Page 3"]], "1fr 0.8fr 0.8fr 0.8fr")
  }),
  "icon-library": blueprint({
    status: "Icon usage and selected asset inspected",
    actions: [["plus", "Add Icon"], ["search", "Find Usage"], ["check", "Validate Icons"], ["save", "Export Sheet"]],
    tools: ["Category Filter", "Icon Set", "Usage Search", "Vector Source", "Color Token", "Export"],
    assets: tree("Icons", "image", ["Editor_Core", "icon-save", "icon-play", "icon-warning", "Usage_Topbar"]),
    metrics: [["Icons", "312"], ["Used", "268"], ["Missing", "4", "warning"], ["Sets", "8"]],
    detailTabs: ["Icon", "Usage", "Export"],
    settings: [["Icon", selectValue("icon-warning")], ["Set", selectValue("Editor Core")], ["Size", selectValue("16")], ["Mono", checkValue(true)], ["Snap Pixel", checkValue(true)]],
    primary: tablePrimary("Icon Usage Matrix", ["Icon", "Set", "Refs", "State"], [["icon-save", "Editor", "18", "Ready"], ["icon-play", "Editor", "22", "Ready"], ["icon-warning", "System", "14", "Selected"], ["icon-old", "Legacy", "0", "Warning"]], "1fr 0.8fr 0.6fr 0.8fr")
  }),
  "level-streaming": blueprint({
    status: "World cells and streaming rules selected",
    actions: [["plus", "Add Level"], ["grid", "Load Cell"], ["check", "Validate Streaming"], ["play", "Preview Streaming"]],
    tools: ["Cell Grid", "Load Rule", "HLOD", "Streaming Source", "Visibility Layer", "Event Trace"],
    assets: tree("World", "globe", ["World_Main", "Cell_A12", "Cell_A13", "HLOD_Cluster_04", "Rule_PlayerDistance"]),
    metrics: [["Cells", "96"], ["Loaded", "18"], ["HLOD", "24"], ["Warnings", "2", "warning"]],
    detailTabs: ["Cells", "Rules", "Events"],
    settings: [["Cell", selectValue("Cell_A12")], ["Rule", selectValue("Player Distance")], ["Distance", inputValue("5000")], ["Async Load", checkValue(true)], ["Show Bounds", checkValue(true)]],
    primary: graphPrimary("Level Streaming Map", [["Player", "Source", 16, 52, "cyan"], ["Cell_A12", "Loaded", 38, 34, "green"], ["Cell_A13", "Queued", 60, 42, "blue"], ["HLOD_04", "Visible", 44, 68, "orange"], ["Cell_B12", "Hidden", 76, 58, "purple"]])
  }),
  "level-variant": blueprint({
    status: "Level variant override stack selected",
    actions: [["plus", "Add Variant"], ["target", "Apply Variant"], ["check", "Validate Overrides"], ["history", "Review Diff"]],
    tools: ["Variant Set", "Actor Override", "Material Swap", "Visibility Override", "Property Capture", "Diff"],
    assets: tree("Variants", "columns", ["Vehicle_Showcase", "Variant_Red", "Variant_Blue", "Override_Material", "Actor_CarBody"]),
    metrics: [["Variants", "18"], ["Overrides", "124"], ["Conflicts", "2", "warning"], ["Actors", "42"]],
    detailTabs: ["Variant", "Overrides", "Diff"],
    settings: [["Variant", selectValue("Variant_Red")], ["Set", selectValue("Vehicle Showcase")], ["Capture Mode", selectValue("Selected Props")], ["Auto Apply", checkValue(false)], ["Record Diff", checkValue(true)]],
    primary: tablePrimary("Variant Overrides", ["Actor", "Property", "Value", "State"], [["CarBody", "Material", "M_RedPaint", "Selected"], ["Wheel_FL", "Visible", "true", "Ready"], ["Light_Rig", "Intensity", "4.2", "Ready"], ["Door_L", "Transform", "Conflict", "Warning"]], "1fr 1fr 1fr 0.8fr")
  }),
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
  }),
  "menu-flow": blueprint({
    status: "Menu route and focus graph selected",
    actions: [["plus", "Add Screen"], ["play", "Preview Flow"], ["check", "Validate Focus"], ["save", "Export Flow"]],
    tools: ["Screen Template", "Route Edge", "Focus Rule", "Transition", "Input Map", "Simulation"],
    assets: tree("Menu Flow", "columns", ["MainMenu", "Screen_Start", "Screen_Options", "Route_Play", "Focus_PlayButton"]),
    metrics: [["Screens", "12"], ["Routes", "28"], ["Focus", "64"], ["Issues", "2", "warning"]],
    detailTabs: ["Routes", "Focus", "Simulation"],
    settings: [["Screen", selectValue("Screen_Start")], ["Breakpoint", selectValue("Desktop")], ["Transition", selectValue("Fade")], ["Show Focus", checkValue(true)], ["Auto Layout", checkValue(true)]],
    primary: graphPrimary("Menu Flow Graph", [["Start", "Screen", 12, 34, "cyan"], ["Options", "Screen", 38, 18, "blue"], ["Loadout", "Screen", 38, 58, "green"], ["Match", "Screen", 66, 38, "orange"], ["Exit", "Route", 82, 62, "purple"]])
  }),
  "motion-matching": blueprint({
    status: "Motion matching query and pose cluster active",
    actions: [["play", "Preview Match"], ["target", "Query Pose"], ["check", "Validate Database"], ["history", "Review Match"]],
    tools: ["Pose Query", "Trajectory", "Feature Vector", "Pose Cluster", "Cost Debug", "Database"],
    assets: tree("Motion", "play", ["MM_Locomotion", "Pose_Run_42", "Pose_Stop_08", "Trajectory_A", "Cluster_Turn"]),
    metrics: [["Poses", "12K"], ["Clusters", "86"], ["Cost", "0.14"], ["Warnings", "1", "warning"]],
    detailTabs: ["Query", "Clusters", "Timeline"],
    settings: [["Database", selectValue("MM_Locomotion")], ["Trajectory", selectValue("2D Future")], ["Cost Bias", inputValue("0.42")], ["Mirror", checkValue(true)], ["Debug Draw", checkValue(true)]],
    primary: graphPrimary("Motion Matching Query", [["Current Pose", "Input", 12, 44, "cyan"], ["Trajectory", "Feature", 34, 22, "blue"], ["Pose Cluster", "Search", 58, 34, "green"], ["Best Match", "Pose", 76, 52, "orange"], ["Cost Curve", "Debug", 42, 68, "purple"]])
  }),
  "particle-library": blueprint({
    status: "Particle library emitter metadata selected",
    actions: [["play", "Simulate Particle"], ["plus", "Add Emitter"], ["check", "Compile Particle"], ["target", "Capture Particle"]],
    tools: ["Particle Filter", "Emitter Stack", "Spawn Module", "GPU Sort", "Bounds Debug", "Import"],
    assets: tree("Particles", "sun", ["P_Sparks", "Emitter_Core", "Module_Spawn", "Module_Color", "Texture_Spark"]),
    metrics: [["Emitters", "42"], ["GPU", "0.8 ms"], ["Warnings", "2", "warning"], ["Refs", "96"]],
    detailTabs: ["Emitters", "Metadata", "Compile"],
    settings: [["Emitter", selectValue("P_Sparks")], ["FPS", selectValue("60 fps")], ["Duration", inputValue("2.0")], ["Loop", checkValue(true)], ["Fixed Bounds", checkValue(false)]],
    primary: tablePrimary("Particle Library", ["Particle", "Type", "Refs", "State"], [["P_Sparks", "GPU", "18", "Selected"], ["P_Dust", "CPU", "12", "Ready"], ["P_Impact", "GPU", "24", "Ready"], ["P_Old", "CPU", "0", "Warning"]], "1fr 0.8fr 0.6fr 0.8fr")
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
  "plugin-manager": blueprint({
    status: "Plugin dependency graph selected",
    actions: [["plus", "Install Plugin"], ["check", "Validate Plugins"], ["save", "Publish Plugin"], ["history", "Review Dependency"]],
    tools: ["Category Filter", "Plugin Manifest", "Dependency Graph", "Version Lock", "Install Queue", "Compatibility"],
    assets: tree("Plugins", "component", ["ZirconAI", "ZirconPhysics", "Dependency_Core", "Marketplace", "Manifest.toml"]),
    metrics: [["Plugins", "42"], ["Enabled", "28"], ["Updates", "4"], ["Conflicts", "1", "warning"]],
    detailTabs: ["Manifest", "Dependencies", "Install"],
    settings: [["Plugin", selectValue("ZirconAI")], ["Version", inputValue("1.4.2")], ["Channel", selectValue("Stable")], ["Enabled", checkValue(true)], ["Auto Update", checkValue(false)]],
    primary: graphPrimary("Plugin Dependency Graph", [["ZirconAI", "Plugin", 12, 34, "cyan"], ["CoreRuntime", "Dependency", 36, 22, "blue"], ["EditorUI", "Dependency", 54, 46, "green"], ["Marketplace", "Source", 74, 28, "orange"], ["Conflict", "Version", 66, 68, "purple"]])
  }),
  "pose-library": blueprint({
    status: "Pose set and capture metadata selected",
    actions: [["plus", "Capture Pose"], ["play", "Preview Pose"], ["check", "Validate Pose"], ["save", "Export Pose"]],
    tools: ["Pose Capture", "Pose Set", "Mirror Pose", "Metadata", "Batch Apply", "Thumbnail"],
    assets: tree("Poses", "history", ["Pose_Combat", "Idle_A", "Aim_Offset", "Crouch_Start", "Pose_Metadata"]),
    metrics: [["Poses", "184"], ["Sets", "12"], ["Mirrored", "86"], ["Warnings", "1", "warning"]],
    detailTabs: ["Pose", "Metadata", "Batch"],
    settings: [["Pose", selectValue("Aim_Offset")], ["Set", selectValue("Combat")], ["Blend", inputValue("0.25")], ["Mirror", checkValue(false)], ["Apply Additive", checkValue(true)]],
    primary: tablePrimary("Pose Library", ["Pose", "Set", "Tags", "State"], [["Aim_Offset", "Combat", "UpperBody", "Selected"], ["Idle_A", "Base", "Loop", "Ready"], ["Crouch_Start", "Movement", "Start", "Ready"], ["Deprecated_Pose", "Legacy", "Old", "Warning"]], "1fr 0.8fr 1fr 0.8fr")
  }),
  "post-process": blueprint({
    status: "Post process volume effect stack selected",
    actions: [["play", "Preview Post Process"], ["check", "Compile Effect"], ["target", "Capture Compare"], ["save", "Save Volume"]],
    tools: ["Effect Stack", "LUT Profile", "Camera Volume", "Blend Weight", "Histogram", "Compare"],
    assets: tree("Post Process", "renderer", ["PPV_City", "Bloom", "ColorGrade_LUT", "DOF", "Exposure"]),
    metrics: [["Effects", "9"], ["GPU", "0.72 ms"], ["Volumes", "4"], ["Warnings", "1", "warning"]],
    detailTabs: ["Effects", "Volumes", "Compare"],
    settings: [["Volume", selectValue("PPV_City")], ["Blend", inputValue("0.85")], ["Quality", selectValue("High")], ["Enabled", checkValue(true)], ["Preview Split", checkValue(true)]],
    primary: tablePrimary("Post Process Stack", ["Effect", "Weight", "GPU", "State"], [["Exposure", "1.00", "0.08 ms", "Ready"], ["Bloom", "0.62", "0.21 ms", "Selected"], ["Color Grade", "0.80", "0.18 ms", "Ready"], ["DOF", "0.40", "0.25 ms", "Warning"]], "1fr 0.8fr 0.8fr 0.8fr")
  }),
  "prefab-editor": blueprint({
    status: "Prefab nested hierarchy and validation active",
    actions: [["plus", "Add Child"], ["target", "Apply Override"], ["check", "Validate Prefab"], ["save", "Save Prefab"]],
    tools: ["Component Add", "Nested Prefab", "Override Diff", "Variant", "Validation", "Placement"],
    assets: tree("Prefabs", "cube", ["PF_Chest", "Mesh_Chest", "LootSocket", "Light_Glow", "Override_Open"]),
    metrics: [["Children", "18"], ["Overrides", "6"], ["Refs", "32"], ["Warnings", "2", "warning"]],
    detailTabs: ["Hierarchy", "Overrides", "Validation"],
    settings: [["Prefab", selectValue("PF_Chest")], ["Variant", selectValue("Default")], ["Instance ID", inputValue("Chest_04")], ["Propagate", checkValue(true)], ["Lock Root", checkValue(false)]],
    primary: graphPrimary("Prefab Composition", [["PF_Chest", "Root", 12, 30, "cyan"], ["Mesh", "Component", 34, 20, "blue"], ["LootSocket", "Socket", 54, 44, "green"], ["Light", "Component", 34, 68, "orange"], ["Override", "Instance", 74, 58, "purple"]])
  }),
  "project-overview": blueprint({
    status: "Project overview health dashboard selected",
    actions: [["search", "Find Project"], ["check", "Validate Project"], ["save", "Publish Snapshot"], ["history", "Review Activity"]],
    tools: ["Project Tree", "Health Check", "Activity Feed", "Content Stats", "Source Engine", "Report"],
    assets: tree("Project", "grid", ["Nightingale", "Content", "Source", "Builds", "Reports"]),
    metrics: [["Assets", "12K"], ["Warnings", "24", "warning"], ["Builds", "8"], ["Users", "5"]],
    detailTabs: ["Health", "Activity", "Reports"],
    settings: [["Project", selectValue("Nightingale")], ["Profile", selectValue("Editor")], ["Time Range", selectValue("7 days")], ["Show Warnings", checkValue(true)], ["Auto Refresh", checkValue(true)]],
    primary: tablePrimary("Project Health", ["Area", "State", "Count", "Trend"], [["Assets", "Ready", "12K", "Stable"], ["Builds", "Warning", "2", "Up"], ["Source", "Clean", "0", "Stable"], ["Automation", "Warning", "7", "Down"]], "1fr 0.8fr 0.6fr 0.8fr")
  }),
  "retarget": blueprint({
    status: "Retarget chain mapping selected",
    actions: [["target", "Map Chain"], ["play", "Preview Retarget"], ["check", "Validate Skeletons"], ["save", "Export Retarget"]],
    tools: ["Chain Map", "Source Pose", "Target Pose", "Root Scale", "IK Goal", "Export Queue"],
    assets: tree("Retarget", "target", ["RTG_HeroToNPC", "SK_Hero", "SK_NPC", "Chain_Arm_L", "Pose_A"]),
    metrics: [["Chains", "18"], ["Mapped", "17"], ["Errors", "1", "warning"], ["Clips", "42"]],
    detailTabs: ["Chains", "Pose", "Export"],
    settings: [["Rig", selectValue("HeroToNPC")], ["Source", selectValue("SK_Hero")], ["Target", selectValue("SK_NPC")], ["Retarget Root", checkValue(true)], ["Preview Motion", checkValue(true)]],
    primary: tablePrimary("Retarget Chain Map", ["Source", "Target", "Mode", "State"], [["Arm_L", "Arm_L", "FK/IK", "Selected"], ["Arm_R", "Arm_R", "FK/IK", "Ready"], ["Spine", "Spine", "Root", "Ready"], ["Tail", "-", "Missing", "Warning"]], "1fr 1fr 0.8fr 0.8fr")
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
  "save-data": blueprint({
    status: "Save slot migration and diff selected",
    actions: [["save", "Save Slot"], ["folder", "Load Slot"], ["check", "Validate Save"], ["history", "Migrate Save"]],
    tools: ["Slot Schema", "Migration Map", "Object Diff", "Cloud Sync", "Corruption Scan", "Runtime Probe"],
    assets: tree("Saves", "save", ["AutoSave_01", "Manual_03", "Cloud_02", "Schema_v4", "Migration_v3_v4"]),
    metrics: [["Slots", "6"], ["Schemas", "4"], ["Migrations", "2"], ["Warnings", "1", "warning"]],
    detailTabs: ["Slots", "Migration", "Validation"],
    settings: [["Schema", selectValue("SaveData v4")], ["Slot", selectValue("AutoSave_01")], ["Compression", selectValue("LZ4")], ["Cloud Sync", checkValue(true)], ["Strict Load", checkValue(true)]],
    primary: tablePrimary("Save Data Diff", ["Object", "Field", "Value", "State"], [["PlayerState", "Level", "12", "Selected"], ["Inventory", "Items", "42", "Ready"], ["QuestLog", "Version", "v3", "Migrating"], ["DebugSlot", "Schema", "Old", "Warning"]], "1fr 1fr 0.8fr 0.8fr")
  }),
  "scatter-editor": blueprint({
    status: "Procedural scatter rule stack selected",
    actions: [["plus", "Add Rule"], ["grid", "Generate Scatter"], ["check", "Validate Scatter"], ["play", "Preview Scatter"]],
    tools: ["Spawn Rule", "Density Map", "Slope Filter", "Biome Mask", "Collision Test", "Seed Preview"],
    assets: tree("Scatter", "globe", ["SC_Forest", "Rule_Rocks", "Rule_Ferns", "Mask_Slope", "Seed_2026"]),
    metrics: [["Rules", "18"], ["Instances", "64K"], ["Conflicts", "1", "warning"], ["Seed", "2026"]],
    detailTabs: ["Rules", "Constraints", "Output"],
    settings: [["Rule Set", selectValue("SC_Forest")], ["Seed", inputValue("2026")], ["Density", inputValue("0.64")], ["Avoid Collisions", checkValue(true)], ["Strict Bounds", checkValue(true)]],
    primary: graphPrimary("Scatter Rule Graph", [["Biome Mask", "Input", 12, 30, "cyan"], ["Slope Filter", "Constraint", 34, 22, "blue"], ["Spawn Rule", "Rule", 56, 42, "green"], ["Collision Test", "Validation", 38, 66, "orange"], ["Output Set", "Instances", 76, 58, "purple"]])
  }),
  "shader-editor": blueprint({
    status: "Shader source, resources, and compiler output selected",
    actions: [["play", "Preview Shader"], ["check", "Compile Shader"], ["target", "Capture Shader"], ["save", "Save Shader"]],
    tools: ["Source File", "Include Tree", "Permutation", "Resource Binding", "Compiler Errors", "Preview Material"],
    assets: tree("Shaders", "code", ["lighting.wgsl", "common.wgsl", "BRDF", "BindGroup_0", "Permutation_SM5"]),
    metrics: [["Permutations", "24"], ["Bindings", "8"], ["GPU", "0.31 ms"], ["Warnings", "3", "warning"]],
    detailTabs: ["Source", "Resources", "Issues"],
    settings: [["Shader", selectValue("lighting.wgsl")], ["Target", selectValue("wgpu")], ["Entry", inputValue("fs_main")], ["Live Compile", checkValue(true)], ["Show Disasm", checkValue(false)]],
    primary: tablePrimary("Shader Compile Workspace", ["Stage", "Entry", "Resource", "State"], [["Vertex", "vs_main", "Camera", "Ready"], ["Fragment", "fs_main", "GBuffer", "Selected"], ["Compute", "cs_tile", "Lighting", "Warning"], ["Include", "common", "BRDF", "Ready"]], "0.8fr 1fr 1fr 0.8fr")
  }),
  "source-control": blueprint({
    status: "Source control changelist and diff review open",
    actions: [["play", "Run Source Control"], ["check", "Validate Change"], ["save", "Submit Change"], ["history", "Review Source Control"]],
    tools: ["Changelist", "Depot Tree", "Diff Viewer", "Ownership", "Merge Queue", "Submit Gate"],
    assets: tree("Source Control", "history", ["CL_2048", "zircon_runtime", "zircon_editor", "docs/ui", "Review_Alice"]),
    metrics: [["Files", "18"], ["Conflicts", "2", "warning"], ["Reviews", "1"], ["Checks", "6"]],
    detailTabs: ["Diff", "Owners", "Submit"],
    settings: [["Changelist", selectValue("CL_2048")], ["Owner", selectValue("Alice")], ["Gate", selectValue("Full")], ["Auto Rebase", checkValue(false)], ["Run Checks", checkValue(true)]],
    primary: tablePrimary("Source Control Changelist", ["File", "Type", "Owner", "State"], [["runtime/ui/render.rs", "Modified", "Alice", "Selected"], ["editor/painter.rs", "Modified", "Bob", "Review"], ["docs/ui.md", "Added", "Alice", "Ready"], ["asset/import.rs", "Conflict", "Chen", "Warning"]], "1.5fr 0.8fr 0.8fr 0.8fr")
  }),
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
  "telemetry-dashboard": blueprint({
    status: "Telemetry query dashboard filtered",
    actions: [["search", "Filter Telemetry"], ["target", "Run Query"], ["save", "Export Telemetry"], ["check", "Open Metric"]],
    tools: ["Query Builder", "Saved Query", "Event Segment", "Metric Detail", "Raw Events", "Dashboard"],
    assets: tree("Telemetry", "info", ["Query_Retention", "Event_Login", "Segment_NewUsers", "Metric_FPS", "Raw_Stream"]),
    metrics: [["Events", "2.4M"], ["Segments", "12"], ["Alerts", "3", "warning"], ["Latency", "120 ms"]],
    detailTabs: ["Metrics", "Segments", "Raw Events"],
    settings: [["Query", selectValue("Retention")], ["Range", selectValue("24h")], ["Segment", selectValue("New Users")], ["Live Refresh", checkValue(true)], ["Sample Raw", checkValue(false)]],
    primary: tablePrimary("Telemetry Metrics", ["Metric", "Value", "Delta", "State"], [["DAU", "42K", "+8%", "Ready"], ["FPS P95", "58", "-2", "Warning"], ["Crash Rate", "0.12%", "-0.04", "Ready"], ["Queue Wait", "42s", "+6s", "Selected"]], "1fr 0.8fr 0.8fr 0.8fr")
  }),
  "ui-asset-editor": blueprint({
    status: "UI asset widget tree and preview selected",
    actions: [["plus", "Add Widget"], ["play", "Preview UI Asset"], ["check", "Validate UI Asset"], ["save", "Export UI Asset"]],
    tools: ["Widget Palette", "Auto Layout", "Token Swatch", "State Preview", "Binding", "Responsive Rule"],
    assets: tree("UI Assets", "image", ["WBP_Inventory", "Panel_Root", "Button_Equip", "Token_Primary", "State_Hover"]),
    metrics: [["Widgets", "42"], ["States", "8"], ["Bindings", "18"], ["Issues", "3", "warning"]],
    detailTabs: ["Hierarchy", "States", "Bindings"],
    settings: [["Widget", selectValue("Button_Equip")], ["Breakpoint", selectValue("Desktop")], ["Theme", selectValue("Workbench Dark")], ["Show Bounds", checkValue(true)], ["Auto Layout", checkValue(true)]],
    primary: graphPrimary("UI Asset Layout Map", [["Root Panel", "Container", 12, 30, "cyan"], ["Inventory Grid", "List", 38, 22, "blue"], ["Equip Button", "Selected", 60, 44, "green"], ["Tooltip", "Overlay", 44, 68, "orange"], ["Binding", "ViewModel", 78, 58, "purple"]])
  }),
  "ui-binding": blueprint({
    status: "UI binding expression and view-model path selected",
    actions: [["plus", "Add Binding"], ["play", "Preview Binding"], ["check", "Validate Binding"], ["save", "Export Binding"]],
    tools: ["View Model", "Expression", "Widget Path", "Converter", "Error List", "Preview Value"],
    assets: tree("Bindings", "link", ["VM_PlayerHUD", "Health.Value", "Ammo.Count", "WBP_HealthBar", "Expr_Percent"]),
    metrics: [["Bindings", "18"], ["Invalid", "2", "warning"], ["Widgets", "42"], ["Latency", "0.2 ms"]],
    detailTabs: ["Bindings", "Expressions", "Validation"],
    settings: [["Binding", selectValue("Health.Value")], ["Widget", selectValue("WBP_HealthBar")], ["Converter", selectValue("Percent")], ["Two Way", checkValue(false)], ["Preview Values", checkValue(true)]],
    primary: graphPrimary("UI Binding Graph", [["View Model", "VM_PlayerHUD", 12, 34, "cyan"], ["Health.Value", "Field", 36, 20, "blue"], ["Converter", "Percent", 56, 42, "green"], ["HealthBar", "Widget", 76, 30, "orange"], ["Validation", "Issues", 52, 68, "purple"]])
  }),
  "volume-editor": blueprint({
    status: "Volume overlap and bounds details selected",
    actions: [["plus", "Add Volume"], ["target", "Inspect Overlap"], ["check", "Validate Volume"], ["play", "Preview Volume"]],
    tools: ["Box Volume", "Sphere Volume", "Bounds Edit", "Overlap Rule", "Profile", "Event Output"],
    assets: tree("Volumes", "cube", ["VOL_DamageZone", "Profile_Default", "Overlap_Player", "Event_OnEnter", "Bounds_A"]),
    metrics: [["Volumes", "24"], ["Overlaps", "12"], ["Events", "8"], ["Warnings", "1", "warning"]],
    detailTabs: ["Bounds", "Overlaps", "Events"],
    settings: [["Volume", selectValue("VOL_DamageZone")], ["Profile", selectValue("Damage")], ["Priority", inputValue("10")], ["Generate Events", checkValue(true)], ["Draw Bounds", checkValue(true)]],
    primary: graphPrimary("Volume Overlap Workspace", [["Volume", "Bounds", 14, 34, "cyan"], ["Player", "Overlap", 38, 24, "blue"], ["Damage Rule", "Effect", 60, 42, "green"], ["OnEnter", "Event", 42, 68, "orange"], ["OnExit", "Event", 78, 60, "purple"]])
  }),
  "weather-editor": blueprint({
    status: "Weather layers and timeline preview selected",
    actions: [["plus", "Add Weather Layer"], ["play", "Preview Weather"], ["check", "Build Weather"], ["target", "Inspect Region"]],
    tools: ["Weather Preset", "Region Profile", "Cloud Layer", "Wind Curve", "Event Track", "Timeline"],
    assets: tree("Weather", "sun", ["Weather_Storm", "Region_Mountains", "Layer_Clouds", "Layer_Rain", "Curve_Wind"]),
    metrics: [["Layers", "8"], ["Regions", "5"], ["Events", "18"], ["Warnings", "2", "warning"]],
    detailTabs: ["Layers", "Curves", "Timeline"],
    settings: [["Preset", selectValue("Storm")], ["Region", selectValue("Mountains")], ["Blend Time", inputValue("12.0")], ["Loop Preview", checkValue(true)], ["Affect Lighting", checkValue(true)]],
    primary: timelinePrimary("Weather Timeline", ["Layer", "Range", "State"], [["Cloud Build", "00:00-02:00", "Ready"], ["Rain Burst", "02:00-04:00", "Selected"], ["Wind Gust", "03:20-05:00", "Ready"], ["Lightning", "04:00-04:30", "Warning"]])
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

function blueprint(value) {
  return value;
}

function tablePrimary(title, headers, rows, columns) {
  return { kind: "table", title, headers, rows, columns };
}

function queuePrimary(title, headers, rows) {
  return { kind: "queue", title, headers, rows, columns: "1.2fr 0.8fr 1fr" };
}

function timelinePrimary(title, headers, rows) {
  return { kind: "timeline", title, headers, rows, columns: "1fr 0.8fr 0.8fr" };
}

function graphPrimary(title, nodes) {
  return { kind: "graph", title, nodes };
}

function tree(root, glyph, children) {
  return [
    [root, "folder", false, 0],
    ...children.map((label, index) => [label, index === 0 ? glyph : "file", index === 0, index === 0 ? 1 : 2])
  ];
}

function selectValue(value) {
  return { kind: "select", value };
}

function inputValue(value) {
  return { kind: "input", value };
}

function checkValue(value) {
  return { kind: "checkbox", value };
}

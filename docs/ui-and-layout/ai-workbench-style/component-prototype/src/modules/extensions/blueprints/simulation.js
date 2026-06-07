import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const simulationBlueprints = {
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
  "navmesh-ai": blueprint({
    status: "Navigation tiles and agent query selected",
    actions: [["target", "Query Path"], ["grid", "Rebuild Tiles"], ["check", "Validate Navmesh"], ["play", "Simulate Agent"]],
    tools: ["Nav Area", "Tile Rebuild", "Agent Radius", "Offmesh Link", "Crowd Debug", "Path Query"],
    assets: tree("Navigation", "target", ["NavData_Main", "Agent_Guard", "Area_Default", "Area_Jump", "Query_Route_A"]),
    metrics: [["Tiles", "284"], ["Agents", "5"], ["Links", "18"], ["Blocked", "2", "warning"]],
    detailTabs: ["Tiles", "Agents", "Queries"],
    settings: [["Agent", selectValue("Guard")], ["Radius", inputValue("42")], ["Height", inputValue("180")], ["Draw Costs", checkValue(true)], ["Crowd Avoidance", checkValue(true)]],
    primary: graphPrimary("Navmesh Query Workspace", [["Start", "Agent", 12, 42, "cyan"], ["Tile A12", "Open", 34, 28, "green"], ["Door Link", "Offmesh", 56, 44, "orange"], ["Goal", "Target", 76, 30, "blue"], ["Blocked Area", "Cost", 48, 68, "purple"]])
  })
};

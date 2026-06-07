import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const uiBlueprints = {
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
  })
};

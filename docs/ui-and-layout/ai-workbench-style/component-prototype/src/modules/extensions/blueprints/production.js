import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const productionBlueprints = {
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
  "source-control": blueprint({
    status: "Source control changelist and diff review open",
    actions: [["play", "Run Source Control"], ["check", "Validate Change"], ["save", "Submit Change"], ["history", "Review Source Control"]],
    tools: ["Changelist", "Depot Tree", "Diff Viewer", "Ownership", "Merge Queue", "Submit Gate"],
    assets: tree("Source Control", "history", ["CL_2048", "zircon_runtime", "zircon_editor", "docs/ui", "Review_Alice"]),
    metrics: [["Files", "18"], ["Conflicts", "2", "warning"], ["Reviews", "1"], ["Checks", "6"]],
    detailTabs: ["Diff", "Owners", "Submit"],
    settings: [["Changelist", selectValue("CL_2048")], ["Owner", selectValue("Alice")], ["Gate", selectValue("Full")], ["Auto Rebase", checkValue(false)], ["Run Checks", checkValue(true)]],
    primary: tablePrimary("Source Control Changelist", ["File", "Type", "Owner", "State"], [["runtime/ui/render.rs", "Modified", "Alice", "Selected"], ["editor/painter.rs", "Modified", "Bob", "Review"], ["docs/ui.md", "Added", "Alice", "Ready"], ["asset/import.rs", "Conflict", "Chen", "Warning"]], "1.5fr 0.8fr 0.8fr 0.8fr")
  })
};

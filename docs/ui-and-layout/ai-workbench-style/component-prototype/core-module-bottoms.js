import { icon } from "./icons.js";
import { checkbox, input, select, toggle } from "./atoms.js";
import { alerts } from "./collections.js";
import { actionButton, assetStrip, moduleTable, progress, settingsRows, tag, timeline } from "./module-components.js";

export function sceneBottom() {
  return `${moduleTable(["Name", "Type", "Size", "Modified"], [
    { cells: ["Item_01", "Mesh", "2.4 MB", "2m ago"] },
    { cells: ["Item_02", "Material", "512 KB", "10m ago"], selected: true },
    { cells: ["Item_03", "Texture", "1.20 MB", "1m ago"] }
  ], "minmax(120px,1.2fr) 110px 90px 120px")}${alerts([["info", "Scene selection ready"], ["success", "No runtime errors"], ["warning", "2 layout warnings"]])}`;
}

export function gameplayBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Instigator", select("Player_01")], ["Target", select("Player_01")], ["Level", input("", { value: "1" })], ["Duration", input("", { value: "10.0" })]])}
    ${moduleTable(["Time", "Event", "Attribute", "Base", "Delta", "Final", "Source"], [
      { cells: ["0.00", "Apply GE", "Health", "100.00", "+0.00", "100.00", "Player_01"] },
      { cells: ["1.00", "Periodic Exec", "Health", "100.00", "+10.00", "110.00", "Player_01"], selected: true },
      { cells: ["2.00", "Periodic Exec", "Health", "110.00", "+10.00", "120.00", "Player_01"] }
    ], "70px 1.1fr 90px 70px 70px 70px 1fr")}
    ${alerts([["success", "All Good"], ["warning", "0 Warnings"], ["info", "Compile successful"]])}
  </div>`;
}

export function abilityBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Speed", select("1.0x")], ["Duration", input("", { value: "4.00s" })], ["Playhead", input("", { value: "1.25s" })]])}
    ${timeline("ability")}
    ${moduleTable(["Time", "Event", "Result", "Asset"], [
      { cells: ["1.22s", "Ability Activated", tag("OK", "green"), "GA_DashAttack"], selected: true },
      { cells: ["1.30s", "Cost Applied", tag("OK", "green"), "GE_DashAttack_Cost"] },
      { cells: ["2.45s", "Gameplay Event", tag("Received", "cyan"), "Event.Data.Hit"] },
      { cells: ["2.48s", "Effect Applied", tag("OK", "green"), "GE_DashAttack_Damage"] }
    ], "70px 1.2fr 92px 1.4fr")}
  </div>`;
}

export function tagsBottom() {
  return `<div class="zr-module-output-grid">
    ${alerts([["error", "2 errors"], ["warning", "8 warnings"], ["info", "6 infos"]])}
    ${moduleTable(["Severity", "Tag", "Message", "Source"], [
      { cells: [tag("Error", "orange"), "Character.State.Stunned", "Redirect conflict: also redirected from Character.State.Stun", "DefaultGameplayTags.ini:42"], selected: true },
      { cells: [tag("Error", "orange"), "Ability.Unknown", "Invalid tag name", "DefaultGameplayTags.ini:113"] },
      { cells: [tag("Warning", "orange"), "Character.Type", "Deprecated tag used 62 times", "DefaultGameplayTags.ini:78"] },
      { cells: [tag("Info", "blue"), "Combat.Heal", "Tag is valid", "CombatTags.ini:55"] }
    ], "98px 1fr 2fr 1.2fr")}
    ${settingsRows([["Export", select("CSV")], ["Filter", select("All")], ["Auto Fix", toggle("", false)]])}
  </div>`;
}

export function perceptionBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Agents", select("All Agents")], ["Show Lost", checkbox("", true)], ["Speed", select("1.0x")]])}
    ${timeline("perception")}
    ${moduleTable(["Time", "Agent", "Event", "Sense"], [
      { cells: ["00:11.8", "AI_Guard_01", "Hearing stimulus", tag("Hearing", "purple")] },
      { cells: ["00:13.1", "AI_Guard_02", "Enemy_01 seen", tag("Sight", "cyan")], selected: true },
      { cells: ["00:14.0", "AI_Guard_02", "Lost sight", tag("Warning", "orange")] }
    ], "80px 1fr 1.4fr 92px")}
  </div>`;
}

export function materialBottom() {
  return `<div class="zr-module-output-grid">
    <div class="zr-module-log"><p>[SM5] M_Rock_Cliff: Compiling...</p><p>[SM5] 5 instructions / 2 texture samplers</p><p class="is-success">[SM5] Compile successful</p></div>
    ${assetStrip(["Default", "Wet", "Snowy", "Mossy", "Night"])}
    ${alerts([["warning", "Texture sample uses default sampler"], ["warning", "Consider a packed texture"]])}
  </div>`;
}

export function behaviorBottom() {
  return `<div class="zr-module-log is-debug"><p><span></span>[12:10.123] [BT_Enemy] Selector (1) - Running</p><p><span></span>[12:10.124] [BT_Enemy] Sequence (2) - Running</p><p class="is-success"><span></span>[12:50.125] Chase Target (3) - Success</p><p class="is-warning"><span></span>[12:45.230] Attack (4) - Running</p></div>`;
}

export function renderPipelineBottom() {
  return `<div class="zr-module-output-grid">
    ${settingsRows([["Frame", input("", { value: "1234" })], ["Platform", select("Windows DX12")], ["FPS", select("30 fps")]])}
    ${moduleTable(["Event", "Pass", "Description", "GPU ms"], [
      { cells: [tag("Info", "blue"), "Frame Start", "Frame 1234 captured", "0.000"] },
      { cells: [tag("OK", "green"), "Lighting Pass", "2 Lighting Pass", "1.872"] },
      { cells: [tag("OK", "green"), "Post Process Pass", "5 Post Process Pass", "0.450"], selected: true },
      { cells: [tag("OK", "green"), "UI Composite Pass", "7 UI Composite Pass", "0.184"] }
    ], "82px 1fr 1.8fr 90px")}
    ${alerts([["success", "Pipeline compile succeeded"], ["warning", "3 resource transition warnings"], ["info", "0 errors"]])}
  </div>`;
}

export function assetBottom() {
  return `<div class="zr-module-output-grid">
    ${moduleTable(["ID", "Task", "Path", "Status", "Progress"], [
      { cells: ["IMP-1021", "Import FBX", "/Game/Forest/SM_Cliff_Rock_02.fbx", "Importing", progress(62)] },
      { cells: ["IMP-1022", "Import Textures", "/Game/Textures/T_Forest_Rock_01.*", "Queued", progress(0)] },
      { cells: ["VAL-2041", "Validate Assets", "/Game/Environments/Forest/*", "Queued", progress(0)] }
    ], "76px 140px 1.6fr 100px 130px")}
    <div class="zr-module-log"><p>10:20:11 Import started: SM_Cliff_Rock_02.fbx</p><p class="is-warning">10:20:12 2 warnings</p><p class="is-error">10:20:15 Error: invalid collision</p></div>
  </div>`;
}

export function vfxBottom() {
  return `<div class="zr-module-output-grid is-vfx-bottom">${timeline("vfx")}${moduleTable(["Time", "System", "Emitter", "Event"], [
    { cells: ["00:00.00", "P_Bolt_01", "E_Bolt", "Activated"] },
    { cells: ["00:00.01", "P_Bolt_01", "E_Bolt", "Spawn Burst 20"] },
    { cells: ["00:00.45", "P_Bolt_01", "E_Bolt", "Collision 15"], selected: true }
  ], "90px 1fr 1fr 1.4fr")}</div>`;
}

export function hudBottom() {
  return `<div class="zr-module-output-grid">
    ${alerts([["warning", "Text MatchTimer is not localized"], ["error", "Binding AmmoCount could not be resolved"], ["info", "DPI scale set to 1.00"]])}
    ${moduleTable(["Type", "Severity", "Message", "Widget", "Line"], [
      { cells: [icon("warning"), "Warning", "Text 'MatchTimer' is not localized", "MatchTimer", "--"] },
      { cells: [icon("x"), "Error", "Binding 'AmmoCount' could not be resolved", "Ammo_Reserve", "57"], selected: true },
      { cells: [icon("info"), "Info", "Image 'Minimap' has no alt text", "Minimap", "--"] }
    ], "46px 82px 2fr 1fr 64px")}
    ${settingsRows([["Filter", select("All")], ["Clear", actionButton("Clear All", "trash")], ["Auto Preview", toggle("", true)]])}
  </div>`;
}

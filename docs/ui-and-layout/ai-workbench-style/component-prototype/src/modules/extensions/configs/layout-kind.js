export function layoutKindFor(key, category) {
  if (/particle|vfx/.test(key) || category === "VFX") return "vfx";
  if (/world-state|spawn-rules|gameplay/.test(key) || category === "Gameplay") return "gameplay";
  if (/save-data/.test(key) || category === "Runtime") return "runtime";
  if (/data-table/.test(key) || category === "Data") return "data";
  if (/console|diagnostics|performance|telemetry/.test(key) || category === "Diagnostics") return "diagnostics";
  if (/lobby|matchmaking/.test(key) || category === "Online") return "online";
  if (/physics|collision/.test(key) || category === "Simulation") return "simulation";
  if (/navmesh|perception/.test(key) || category === "AI") return "gameplay";
  if (/lighting|post-process|shader|render/.test(key) || category === "Rendering") return "rendering";
  if (/sequencer|montage|animation|blend|motion|pose|retarget|control-rig/.test(key) || category === "Animation" || category === "Cinematic") return "animation";
  if (/ui-|accessibility|font|icon|menu-flow/.test(key) || category === "UI/UX") return "ui";
  if (/automation|build-export|plugin|project-overview|source-control/.test(key) || category === "Production") return "production";
  if (/terrain|foliage|scatter|weather|level|volume|prefab/.test(key) || category === "World Building") return "world";
  return "default";
}

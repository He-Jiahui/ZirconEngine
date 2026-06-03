export const coreReferenceSamples = [
  { source: "ai-scene-editor-layout.png", moduleId: "scene", role: "core" },
  { source: "ai-gameplay-effect-layout.png", moduleId: "gameplay-effect", role: "approved-shell-core" },
  { source: "ai-gameplay-ability-layout.png", moduleId: "gameplay-ability", role: "core" },
  { source: "ai-gameplay-tags-layout.png", moduleId: "gameplay-tags", role: "core" },
  { source: "ai-ai-perception-layout.png", moduleId: "ai-perception", role: "core" },
  { source: "ai-material-editor-layout.png", moduleId: "material", role: "core" },
  { source: "ai-behavior-tree-layout.png", moduleId: "behavior-tree", role: "core" },
  { source: "ai-render-pipeline-layout.png", moduleId: "render-pipeline", role: "core" },
  { source: "ai-asset-browser-layout.png", moduleId: "asset-browser", role: "core" },
  { source: "ai-vfx-editor-layout.png", moduleId: "vfx", role: "core" },
  { source: "ai-hud-editor-layout.png", moduleId: "hud-editor", role: "core" }
];

export const extensionReferenceSamples = [
  { source: "ai-terrain-editor-layout.png", category: "World Building", glyph: "globe" },
  { source: "ai-lighting-bake-layout.png", category: "Rendering", glyph: "sun" },
  { source: "ai-sequencer-layout.png", category: "Cinematic", glyph: "history" },
  { source: "ai-montage-editor-layout.png", category: "Animation", glyph: "play" },
  { source: "ai-physics-collision-layout.png", category: "Simulation", glyph: "cube" },
  { source: "ai-navmesh-ai-layout.png", category: "AI", glyph: "target" },
  { source: "ai-data-table-layout.png", category: "Data", glyph: "grid" },
  { source: "ai-console-diagnostics-layout.png", category: "Diagnostics", glyph: "info" },
  { source: "ai-accessibility-audit-layout.png", category: "UI/UX", glyph: "check" },
  { source: "ai-animation-compression-layout.png", category: "Animation", glyph: "history" },
  { source: "ai-automation-report-layout.png", category: "Production", glyph: "check" },
  { source: "ai-blend-space-layout.png", category: "Animation", glyph: "play" },
  { source: "ai-build-export-layout.png", category: "Production", glyph: "save" },
  { source: "ai-collision-proxy-layout.png", category: "Simulation", glyph: "cube" },
  { source: "ai-control-rig-layout.png", category: "Animation", glyph: "component" },
  { source: "ai-foliage-editor-layout.png", category: "World Building", glyph: "globe" },
  { source: "ai-font-atlas-layout.png", category: "UI/UX", glyph: "file" },
  { source: "ai-icon-library-layout.png", category: "UI/UX", glyph: "image" },
  { source: "ai-level-streaming-layout.png", category: "World Building", glyph: "globe" },
  { source: "ai-level-variant-layout.png", category: "World Building", glyph: "columns" },
  { source: "ai-lobby-editor-layout.png", category: "Online", glyph: "component" },
  { source: "ai-matchmaking-editor-layout.png", category: "Online", glyph: "target" },
  { source: "ai-menu-flow-layout.png", category: "UI/UX", glyph: "columns" },
  { source: "ai-motion-matching-layout.png", category: "Animation", glyph: "play" },
  { source: "ai-particle-library-layout.png", category: "VFX", glyph: "sun" },
  { source: "ai-performance-layout.png", category: "Diagnostics", glyph: "info" },
  { source: "ai-plugin-manager-layout.png", category: "Production", glyph: "component" },
  { source: "ai-pose-library-layout.png", category: "Animation", glyph: "history" },
  { source: "ai-post-process-layout.png", category: "Rendering", glyph: "renderer" },
  { source: "ai-prefab-editor-layout.png", category: "World Building", glyph: "cube" },
  { source: "ai-project-overview-layout.png", category: "Production", glyph: "grid" },
  { source: "ai-retarget-layout.png", category: "Animation", glyph: "target" },
  { source: "ai-runtime-diagnostics-layout.png", category: "Diagnostics", glyph: "info" },
  { source: "ai-save-data-layout.png", category: "Runtime", glyph: "save" },
  { source: "ai-scatter-editor-layout.png", category: "World Building", glyph: "globe" },
  { source: "ai-shader-editor-layout.png", category: "Rendering", glyph: "code" },
  { source: "ai-source-control-layout.png", category: "Production", glyph: "history" },
  { source: "ai-spawn-rules-layout.png", category: "Gameplay", glyph: "target" },
  { source: "ai-telemetry-dashboard-layout.png", category: "Diagnostics", glyph: "info" },
  { source: "ai-ui-asset-editor-layout.png", category: "UI/UX", glyph: "image" },
  { source: "ai-ui-binding-layout.png", category: "UI/UX", glyph: "link" },
  { source: "ai-volume-editor-layout.png", category: "World Building", glyph: "cube" },
  { source: "ai-weather-editor-layout.png", category: "World Building", glyph: "sun" },
  { source: "ai-world-state-layout.png", category: "Gameplay", glyph: "globe" }
];

export const supplementalReferenceSamples = [
  { source: "ai-asset-browser-workbench.png", moduleId: "asset-browser", role: "variant" },
  { source: "ai-material-editor-workbench.png", moduleId: "material", role: "variant" },
  { source: "ai-montage-editor-workbench.png", moduleId: "montage-editor", role: "variant" },
  { source: "ai-workbench-web-framework.png", moduleId: "editor-library", role: "shell-style" }
];

export function allReferenceSampleSources() {
  return [
    ...coreReferenceSamples,
    ...extensionReferenceSamples,
    ...supplementalReferenceSamples
  ].map((sample) => sample.source);
}

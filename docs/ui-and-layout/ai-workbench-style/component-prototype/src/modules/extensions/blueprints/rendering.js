import { blueprint, checkValue, graphPrimary, inputValue, queuePrimary, selectValue, tablePrimary, timelinePrimary, tree } from "./helpers.js";

export const renderingBlueprints = {
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
  "shader-editor": blueprint({
    status: "Shader source, resources, and compiler output selected",
    actions: [["play", "Preview Shader"], ["check", "Compile Shader"], ["target", "Capture Shader"], ["save", "Save Shader"]],
    tools: ["Source File", "Include Tree", "Permutation", "Resource Binding", "Compiler Errors", "Preview Material"],
    assets: tree("Shaders", "code", ["lighting.wgsl", "common.wgsl", "BRDF", "BindGroup_0", "Permutation_SM5"]),
    metrics: [["Permutations", "24"], ["Bindings", "8"], ["GPU", "0.31 ms"], ["Warnings", "3", "warning"]],
    detailTabs: ["Source", "Resources", "Issues"],
    settings: [["Shader", selectValue("lighting.wgsl")], ["Target", selectValue("wgpu")], ["Entry", inputValue("fs_main")], ["Live Compile", checkValue(true)], ["Show Disasm", checkValue(false)]],
    primary: tablePrimary("Shader Compile Workspace", ["Stage", "Entry", "Resource", "State"], [["Vertex", "vs_main", "Camera", "Ready"], ["Fragment", "fs_main", "GBuffer", "Selected"], ["Compute", "cs_tile", "Lighting", "Warning"], ["Include", "common", "BRDF", "Ready"]], "0.8fr 1fr 1fr 0.8fr")
  })
};

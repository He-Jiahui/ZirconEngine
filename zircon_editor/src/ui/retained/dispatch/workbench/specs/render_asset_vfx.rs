use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const SHADER_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionShaderSourceRow",
    "WorkbenchExtensionShaderIncludeRow",
    "WorkbenchExtensionShaderPermutationRow",
    "WorkbenchExtensionShaderVertexRow",
    "WorkbenchExtensionShaderFragmentRow",
    "WorkbenchExtensionShaderComputeRow",
    "WorkbenchExtensionShaderCommonRow",
];
const SHADER_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.shader_editor.source_row.select",
        "WorkbenchExtensionShaderSourceRow",
    ),
    action(
        "workbench.extension.shader_editor.include_row.select",
        "WorkbenchExtensionShaderIncludeRow",
    ),
    action(
        "workbench.extension.shader_editor.permutation_row.select",
        "WorkbenchExtensionShaderPermutationRow",
    ),
    action(
        "workbench.extension.shader_editor.vertex_row.select",
        "WorkbenchExtensionShaderVertexRow",
    ),
    action(
        "workbench.extension.shader_editor.fragment_row.select",
        "WorkbenchExtensionShaderFragmentRow",
    ),
    action(
        "workbench.extension.shader_editor.compute_row.select",
        "WorkbenchExtensionShaderComputeRow",
    ),
    action(
        "workbench.extension.shader_editor.common_row.select",
        "WorkbenchExtensionShaderCommonRow",
    ),
];
const SHADER_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchRenderToolsMenu",
    "WorkbenchExtensionShaderPreviewButton",
    "WorkbenchExtensionShaderCompileButton",
];
const SHADER_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.shader_editor.open",
        "WorkbenchRenderToolsMenu",
    ),
    action(
        "workbench.extension.shader_editor.preview.invoke",
        "WorkbenchExtensionShaderPreviewButton",
    ),
    action(
        "workbench.extension.shader_editor.compile.invoke",
        "WorkbenchExtensionShaderCompileButton",
    ),
];
const SHADER_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.shader_editor.target.edit",
    "workbench.extension.shader_editor.target.commit",
    "workbench.extension.shader_editor.entry.edit",
    "workbench.extension.shader_editor.entry.commit",
    "workbench.extension.shader_editor.live_compile.edit",
    "workbench.extension.shader_editor.live_compile.commit",
];

pub(super) const SHADER_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.shader_editor.open",
    "WorkbenchExtensionShaderEditorWorkspace",
    SHADER_EDITOR_ROW_CONTROLS,
    SHADER_EDITOR_ROW_ACTIONS,
    SHADER_EDITOR_COMMAND_CONTROLS,
    SHADER_EDITOR_COMMAND_ACTIONS,
    SHADER_EDITOR_FIELD_ACTIONS,
);
const LIGHTING_BAKE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLightingBakeSceneBakeRow",
    "WorkbenchExtensionLightingBakeInteriorBakeRow",
    "WorkbenchExtensionLightingBakeReflectionProbeRow",
    "WorkbenchExtensionLightingBakeShadowMapTableRow",
    "WorkbenchExtensionLightingBakeLightmapUvTableRow",
    "WorkbenchExtensionLightingBakeProbeGridTableRow",
    "WorkbenchExtensionLightingBakeBleedWarningTableRow",
];
const LIGHTING_BAKE_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.lighting_bake.scene_bake_row.select",
        "WorkbenchExtensionLightingBakeSceneBakeRow",
    ),
    action(
        "workbench.extension.lighting_bake.interior_bake_row.select",
        "WorkbenchExtensionLightingBakeInteriorBakeRow",
    ),
    action(
        "workbench.extension.lighting_bake.reflection_probe_row.select",
        "WorkbenchExtensionLightingBakeReflectionProbeRow",
    ),
    action(
        "workbench.extension.lighting_bake.shadow_map_table_row.select",
        "WorkbenchExtensionLightingBakeShadowMapTableRow",
    ),
    action(
        "workbench.extension.lighting_bake.lightmap_uv_table_row.select",
        "WorkbenchExtensionLightingBakeLightmapUvTableRow",
    ),
    action(
        "workbench.extension.lighting_bake.probe_grid_table_row.select",
        "WorkbenchExtensionLightingBakeProbeGridTableRow",
    ),
    action(
        "workbench.extension.lighting_bake.bleed_warning_table_row.select",
        "WorkbenchExtensionLightingBakeBleedWarningTableRow",
    ),
];
const LIGHTING_BAKE_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchRenderToolsMenu",
    "WorkbenchExtensionLightingBakePreviewButton",
    "WorkbenchExtensionLightingBakeBakeButton",
];
const LIGHTING_BAKE_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.lighting_bake.open",
        "WorkbenchRenderToolsMenu",
    ),
    action(
        "workbench.extension.lighting_bake.preview.invoke",
        "WorkbenchExtensionLightingBakePreviewButton",
    ),
    action(
        "workbench.extension.lighting_bake.bake.invoke",
        "WorkbenchExtensionLightingBakeBakeButton",
    ),
];
const LIGHTING_BAKE_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.lighting_bake.quality.edit",
    "workbench.extension.lighting_bake.quality.commit",
    "workbench.extension.lighting_bake.target.edit",
    "workbench.extension.lighting_bake.target.commit",
    "workbench.extension.lighting_bake.denoise.edit",
    "workbench.extension.lighting_bake.denoise.commit",
];

pub(super) const LIGHTING_BAKE_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.lighting_bake.open",
    "WorkbenchExtensionLightingBakeWorkspace",
    LIGHTING_BAKE_ROW_CONTROLS,
    LIGHTING_BAKE_ROW_ACTIONS,
    LIGHTING_BAKE_COMMAND_CONTROLS,
    LIGHTING_BAKE_COMMAND_ACTIONS,
    LIGHTING_BAKE_FIELD_ACTIONS,
);
const POST_PROCESS_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPostProcessGlobalStackRow",
    "WorkbenchExtensionPostProcessCinematicStackRow",
    "WorkbenchExtensionPostProcessVolumeBlendRow",
    "WorkbenchExtensionPostProcessBloomTableRow",
    "WorkbenchExtensionPostProcessTonemapTableRow",
    "WorkbenchExtensionPostProcessLutTableRow",
    "WorkbenchExtensionPostProcessExposureWarningTableRow",
];
const POST_PROCESS_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.post_process.global_stack_row.select",
        "WorkbenchExtensionPostProcessGlobalStackRow",
    ),
    action(
        "workbench.extension.post_process.cinematic_stack_row.select",
        "WorkbenchExtensionPostProcessCinematicStackRow",
    ),
    action(
        "workbench.extension.post_process.volume_blend_row.select",
        "WorkbenchExtensionPostProcessVolumeBlendRow",
    ),
    action(
        "workbench.extension.post_process.bloom_table_row.select",
        "WorkbenchExtensionPostProcessBloomTableRow",
    ),
    action(
        "workbench.extension.post_process.tonemap_table_row.select",
        "WorkbenchExtensionPostProcessTonemapTableRow",
    ),
    action(
        "workbench.extension.post_process.lut_table_row.select",
        "WorkbenchExtensionPostProcessLutTableRow",
    ),
    action(
        "workbench.extension.post_process.exposure_warning_table_row.select",
        "WorkbenchExtensionPostProcessExposureWarningTableRow",
    ),
];
const POST_PROCESS_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchRenderToolsMenu",
    "WorkbenchExtensionPostProcessPreviewButton",
    "WorkbenchExtensionPostProcessApplyButton",
];
const POST_PROCESS_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.post_process.open",
        "WorkbenchRenderToolsMenu",
    ),
    action(
        "workbench.extension.post_process.preview.invoke",
        "WorkbenchExtensionPostProcessPreviewButton",
    ),
    action(
        "workbench.extension.post_process.apply.invoke",
        "WorkbenchExtensionPostProcessApplyButton",
    ),
];
const POST_PROCESS_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.post_process.profile.edit",
    "workbench.extension.post_process.profile.commit",
    "workbench.extension.post_process.volume.edit",
    "workbench.extension.post_process.volume.commit",
    "workbench.extension.post_process.blend.edit",
    "workbench.extension.post_process.blend.commit",
];

pub(super) const POST_PROCESS_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.post_process.open",
    "WorkbenchExtensionPostProcessWorkspace",
    POST_PROCESS_ROW_CONTROLS,
    POST_PROCESS_ROW_ACTIONS,
    POST_PROCESS_COMMAND_CONTROLS,
    POST_PROCESS_COMMAND_ACTIONS,
    POST_PROCESS_FIELD_ACTIONS,
);
const PARTICLE_LIBRARY_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionParticleLibrarySparksRow",
    "WorkbenchExtensionParticleLibraryDustRow",
    "WorkbenchExtensionParticleLibraryImpactRow",
    "WorkbenchExtensionParticleLibraryGpuSparkRow",
    "WorkbenchExtensionParticleLibraryCpuDustRow",
    "WorkbenchExtensionParticleLibraryImpactGpuRow",
    "WorkbenchExtensionParticleLibraryArchivedRow",
];
const PARTICLE_LIBRARY_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.particle_library.sparks_row.select",
        "WorkbenchExtensionParticleLibrarySparksRow",
    ),
    action(
        "workbench.extension.particle_library.dust_row.select",
        "WorkbenchExtensionParticleLibraryDustRow",
    ),
    action(
        "workbench.extension.particle_library.impact_row.select",
        "WorkbenchExtensionParticleLibraryImpactRow",
    ),
    action(
        "workbench.extension.particle_library.gpu_spark_row.select",
        "WorkbenchExtensionParticleLibraryGpuSparkRow",
    ),
    action(
        "workbench.extension.particle_library.cpu_dust_row.select",
        "WorkbenchExtensionParticleLibraryCpuDustRow",
    ),
    action(
        "workbench.extension.particle_library.impact_gpu_row.select",
        "WorkbenchExtensionParticleLibraryImpactGpuRow",
    ),
    action(
        "workbench.extension.particle_library.archived_row.select",
        "WorkbenchExtensionParticleLibraryArchivedRow",
    ),
];
const PARTICLE_LIBRARY_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchVfxParticleLibraryButton",
    "WorkbenchExtensionParticleLibrarySimulateButton",
    "WorkbenchExtensionParticleLibraryCompileButton",
];
const PARTICLE_LIBRARY_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.particle_library.open",
        "WorkbenchVfxParticleLibraryButton",
    ),
    action(
        "workbench.extension.particle_library.simulate.invoke",
        "WorkbenchExtensionParticleLibrarySimulateButton",
    ),
    action(
        "workbench.extension.particle_library.compile.invoke",
        "WorkbenchExtensionParticleLibraryCompileButton",
    ),
];
const PARTICLE_LIBRARY_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.particle_library.emitter.edit",
    "workbench.extension.particle_library.emitter.commit",
    "workbench.extension.particle_library.duration.edit",
    "workbench.extension.particle_library.duration.commit",
    "workbench.extension.particle_library.bounds.edit",
    "workbench.extension.particle_library.bounds.commit",
];

pub(super) const PARTICLE_LIBRARY_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.particle_library.open",
    "WorkbenchExtensionParticleLibraryWorkspace",
    PARTICLE_LIBRARY_ROW_CONTROLS,
    PARTICLE_LIBRARY_ROW_ACTIONS,
    PARTICLE_LIBRARY_COMMAND_CONTROLS,
    PARTICLE_LIBRARY_COMMAND_ACTIONS,
    PARTICLE_LIBRARY_FIELD_ACTIONS,
);

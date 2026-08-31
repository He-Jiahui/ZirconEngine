use crate::core::framework::render::{ShaderFeatureBits, ShadingModelDescriptor};
use crate::graphics::material::ShadingModelIncludeSourceSet;
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::shader::template::{
    ShaderModuleRegistry, ShaderModuleResolutionError, ShaderTemplateInclude,
    environment_standard_pbr_include, pbr_extras_include_for_features,
};

const GPU_SCENE_INCLUDE_TOKEN: &str = "zr_gpu_scene.wgsl";
const LIGHT_COOKIE_INCLUDE_TOKEN: &str = "zr_light_cookie.wgsl";
const LIGHTMAP_INCLUDE_TOKEN: &str = "zr_lightmap.wgsl";
const LIGHT_GRID_INCLUDE_TOKEN: &str = "zr_light_grid.wgsl";
const SHADOW_INCLUDE_TOKEN: &str = "zr_shadow.wgsl";
const ENVIRONMENT_INCLUDE_TOKEN: &str = "zr_environment.wgsl";
const PBR_EXTRAS_INCLUDE_TOKEN: &str = "zr_pbr_extras.wgsl";
const VOLUMETRIC_INCLUDE_TOKEN: &str = "zr_volumetric.wgsl";
const DEFERRED_STANDARD_PBR_INCLUDE_TOKEN: &str = "zr_shade_deferred_standard_pbr.wgsl";
const DEFERRED_BLINN_PHONG_INCLUDE_TOKEN: &str = "zr_shade_deferred_blinn_phong.wgsl";
const DEFERRED_UNLIT_INCLUDE_TOKEN: &str = "zr_shade_deferred_unlit.wgsl";
const DEFERRED_SUBSURFACE_INCLUDE_TOKEN: &str = "zr_shade_deferred_subsurface.wgsl";
const CUSTOM_DISPATCH_MARKER: &str = "    // zr-deferred-lighting-custom-shading-model-dispatch";

const DEFERRED_STANDARD_PBR_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_standard_pbr.wgsl");
const DEFERRED_BLINN_PHONG_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_blinn_phong.wgsl");
const DEFERRED_UNLIT_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_unlit.wgsl");
const DEFERRED_SUBSURFACE_INCLUDE: &str =
    include_str!("../../../../shader/wgsl/zr_shade_deferred_subsurface.wgsl");
const DEFERRED_LIGHTING_TEMPLATE: &str = include_str!("../shaders/deferred_lighting.wgsl");
const DEFERRED_ENVIRONMENT_ONLY_PBR_TEMPLATE: &str =
    include_str!("../shaders/deferred_environment_only_pbr.wgsl");
const VOLUMETRIC_DISABLED_INCLUDE: &str = r#"
fn zr_volumetric_transmittance(_fragment_position: vec2<f32>, _device_depth: f32) -> f32 {
    return 1.0;
}
fn zr_volumetric_scattering(_fragment_position: vec2<f32>, _device_depth: f32) -> vec3<f32> {
    return vec3<f32>(0.0);
}
fn zr_volumetric_apply(color: vec3<f32>, _fragment_position: vec2<f32>, _device_depth: f32) -> vec3<f32> {
    return color;
}
"#;
const FULL_LIGHT_VECTOR_DISPATCH: &str = r#"fn shade_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, roughness: f32, diffuse_color: vec3<f32>, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>, shading_model_id: u32) -> vec3<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return shade_blinn_phong_light_vector_normalized(light_vector, radiance, world_normal, roughness, diffuse_color, world_view);
    }
    return shade_standard_pbr_light_vector_normalized(light_vector, radiance, world_normal, roughness, direct_f0, direct_diffuse_brdf, world_view);
}
"#;
const STANDARD_PBR_LIGHT_VECTOR_DISPATCH: &str = r#"
fn shade_light_vector_normalized(light_vector: vec3<f32>, radiance: vec3<f32>, world_normal: vec3<f32>, roughness: f32, _diffuse_color: vec3<f32>, direct_f0: vec3<f32>, direct_diffuse_brdf: vec3<f32>, world_view: vec3<f32>, _shading_model_id: u32) -> vec3<f32> {
    return shade_standard_pbr_light_vector_normalized(light_vector, radiance, world_normal, roughness, direct_f0, direct_diffuse_brdf, world_view);
}
"#;
const FULL_PIXEL_DISPATCH: &str = r#"fn shade_deferred_pixel(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, emissive: vec3<f32>, depth: f32, shading_model_id: u32) -> vec4<f32> {
    if (shading_model_id == ZR_SHADING_MODEL_UNLIT_ID) {
        return apply_deferred_volumetric(
            add_deferred_emissive(shade_deferred_unlit(albedo), emissive),
            position,
            depth,
        );
    }
    if (shading_model_id == ZR_SHADING_MODEL_BLINN_PHONG_ID) {
        return apply_deferred_volumetric(
            add_deferred_emissive(
                shade_deferred_blinn_phong(position, coord, albedo, material, normal),
                emissive,
            ),
            position,
            depth,
        );
    }
    // zr-deferred-lighting-custom-shading-model-dispatch
    return apply_deferred_volumetric(
        add_deferred_emissive(
            shade_deferred_standard_pbr(position, coord, albedo, material, normal),
            emissive,
        ),
        position,
        depth,
    );
}
"#;
const STANDARD_PBR_PIXEL_DISPATCH: &str = r#"
fn shade_deferred_pixel(position: vec4<f32>, coord: vec2<i32>, albedo: vec4<f32>, material: vec4<f32>, normal: vec3<f32>, emissive: vec3<f32>, depth: f32, _shading_model_id: u32) -> vec4<f32> {
    return apply_deferred_volumetric(
        add_deferred_emissive(
            shade_deferred_standard_pbr(position, coord, albedo, material, normal),
            emissive,
        ),
        position,
        depth,
    );
}
"#;

pub(in crate::graphics::scene::scene_renderer::deferred) const DEFERRED_LIGHTING_SHADER: &str = concat!(
    "// include: zr_gpu_scene.wgsl\n",
    include_str!("../../mesh/shaders/zr_gpu_scene.wgsl"),
    "\n// include: zr_light_cookie.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_light_cookie.wgsl"),
    "\n// include: zr_irradiance_volume.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_irradiance_volume.wgsl"),
    "\n// include: zr_lightmap.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_lightmap.wgsl"),
    "\n// include: zr_light_grid.wgsl\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n// include: zr_shadow.wgsl\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n// include: zr_volumetric.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_volumetric.wgsl"),
    "\n// include: zr_pbr_common.wgsl\n",
    include_str!("../../../../shader/includes/zr_pbr_common.wgsl"),
    "\n// include: zr_pbr_extras.wgsl\n",
    include_str!("../../../../shader/includes/zr_pbr_extras_core.wgsl"),
    "\n// include: zr_shade_deferred_standard_pbr.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_shade_deferred_standard_pbr.wgsl"),
    "\n// include: zr_shade_deferred_blinn_phong.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_shade_deferred_blinn_phong.wgsl"),
    "\n// include: zr_shade_deferred_unlit.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_shade_deferred_unlit.wgsl"),
    "\n// include: deferred_lighting.wgsl\n",
    include_str!("../shaders/deferred_lighting.wgsl"),
    "\n// include: zr_environment.wgsl\n",
    include_str!("../../../../shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment_generic_api.wgsl"),
    "\n",
    include_str!("../../../../shader/wgsl/zr_environment.wgsl")
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) enum DeferredLightingShaderSourceError {
    CustomShadingModelsUnsupportedByProfile {
        profile: SceneRendererDeferredLightingProfile,
    },
    UnknownDeferredInclude {
        token: String,
    },
    UnknownShaderModule {
        token: String,
    },
    CircularShaderModuleDependency {
        cycle: Vec<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingShaderIncludeSource
{
    token: String,
    source: String,
}

impl DeferredLightingShaderIncludeSource {
    fn new(token: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingShaderSourceRequest
{
    volumetric_enabled: bool,
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    shading_model_descriptors: Vec<ShadingModelDescriptor>,
    shading_model_deferred_include_sources: Vec<DeferredLightingShaderIncludeSource>,
}

impl DeferredLightingShaderSourceRequest {
    pub(in crate::graphics::scene::scene_renderer::deferred) fn new() -> Self {
        Self::default()
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_volumetric_enabled(
        mut self,
        enabled: bool,
    ) -> Self {
        self.volumetric_enabled = enabled;
        self
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_deferred_lighting_profile(
        mut self,
        profile: SceneRendererDeferredLightingProfile,
    ) -> Self {
        self.deferred_lighting_profile = profile;
        self
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_shading_model_descriptor(
        mut self,
        descriptor: ShadingModelDescriptor,
    ) -> Self {
        self.shading_model_descriptors.push(descriptor);
        self
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_shading_model_deferred_include_source(
        mut self,
        token: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        self.shading_model_deferred_include_sources
            .push(DeferredLightingShaderIncludeSource::new(token, source));
        self
    }

    pub(in crate::graphics::scene::scene_renderer::deferred) fn with_shading_model_deferred_include_sources(
        mut self,
        sources: &ShadingModelIncludeSourceSet,
    ) -> Self {
        for source in sources.deferred() {
            self.shading_model_deferred_include_sources.push(
                DeferredLightingShaderIncludeSource::new(
                    source.token.clone(),
                    source.source.clone(),
                ),
            );
        }
        self
    }
}

pub(in crate::graphics::scene::scene_renderer::deferred) fn assemble_deferred_lighting_shader_source(
    request: DeferredLightingShaderSourceRequest,
) -> Result<String, DeferredLightingShaderSourceError> {
    if request.deferred_lighting_profile != SceneRendererDeferredLightingProfile::FullScene
        && !request.shading_model_descriptors.is_empty()
    {
        return Err(
            DeferredLightingShaderSourceError::CustomShadingModelsUnsupportedByProfile {
                profile: request.deferred_lighting_profile,
            },
        );
    }
    let custom_dispatch = custom_deferred_dispatch(&request)?;
    let (builtin_roots, light_vector_dispatch, pixel_dispatch): (&[&str], &str, &str) =
        match request.deferred_lighting_profile {
            SceneRendererDeferredLightingProfile::FullScene => (
                &[
                    DEFERRED_STANDARD_PBR_INCLUDE_TOKEN,
                    DEFERRED_BLINN_PHONG_INCLUDE_TOKEN,
                    DEFERRED_UNLIT_INCLUDE_TOKEN,
                    DEFERRED_SUBSURFACE_INCLUDE_TOKEN,
                ],
                FULL_LIGHT_VECTOR_DISPATCH,
                FULL_PIXEL_DISPATCH,
            ),
            SceneRendererDeferredLightingProfile::StandardPbrPreview => (
                &[DEFERRED_STANDARD_PBR_INCLUDE_TOKEN],
                STANDARD_PBR_LIGHT_VECTOR_DISPATCH,
                STANDARD_PBR_PIXEL_DISPATCH,
            ),
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview => (&[], "", ""),
        };
    let mut roots = if request.deferred_lighting_profile
        == SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
    {
        Vec::new()
    } else {
        vec![
            GPU_SCENE_INCLUDE_TOKEN.to_string(),
            LIGHT_COOKIE_INCLUDE_TOKEN.to_string(),
            LIGHTMAP_INCLUDE_TOKEN.to_string(),
            LIGHT_GRID_INCLUDE_TOKEN.to_string(),
            SHADOW_INCLUDE_TOKEN.to_string(),
            VOLUMETRIC_INCLUDE_TOKEN.to_string(),
            PBR_EXTRAS_INCLUDE_TOKEN.to_string(),
        ]
    };
    roots.extend(builtin_roots.iter().map(|token| (*token).to_string()));
    let mut source_includes = Vec::new();
    if roots.iter().any(|root| root == PBR_EXTRAS_INCLUDE_TOKEN) {
        source_includes.push(pbr_extras_include_for_features(ShaderFeatureBits::default()));
    }
    if !request.volumetric_enabled && roots.iter().any(|root| root == VOLUMETRIC_INCLUDE_TOKEN) {
        source_includes.push(ShaderTemplateInclude::new(
            VOLUMETRIC_INCLUDE_TOKEN,
            VOLUMETRIC_DISABLED_INCLUDE,
        ));
    }
    for token in builtin_roots {
        let source = builtin_deferred_include_source(token)
            .expect("deferred lighting profiles must select known builtin roots");
        source_includes.push(ShaderTemplateInclude::new(*token, source));
    }
    for descriptor in request.shading_model_descriptors.iter() {
        if builtin_deferred_include_token(descriptor.deferred_include.as_str()) {
            continue;
        }
        let include = request
            .shading_model_deferred_include_sources
            .iter()
            .find(|include| {
                deferred_include_tokens_match(&include.token, &descriptor.deferred_include)
            })
            .ok_or_else(
                || DeferredLightingShaderSourceError::UnknownDeferredInclude {
                    token: descriptor.deferred_include.clone(),
                },
            )?;
        source_includes.push(ShaderTemplateInclude::new(&include.token, &include.source));
        roots.push(include.token.clone());
    }

    let template = if request.deferred_lighting_profile
        == SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview
    {
        DEFERRED_ENVIRONMENT_ONLY_PBR_TEMPLATE.to_string()
    } else {
        DEFERRED_LIGHTING_TEMPLATE
            .replace(FULL_LIGHT_VECTOR_DISPATCH, light_vector_dispatch)
            .replace(FULL_PIXEL_DISPATCH, pixel_dispatch)
            .replace(
                CUSTOM_DISPATCH_MARKER,
                &format!("{CUSTOM_DISPATCH_MARKER}\n{custom_dispatch}"),
            )
    };
    source_includes.push(ShaderTemplateInclude::new(
        "deferred_lighting.wgsl",
        template,
    ));
    roots.push("deferred_lighting.wgsl".to_string());
    match request.deferred_lighting_profile {
        SceneRendererDeferredLightingProfile::FullScene
        | SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview => {}
        SceneRendererDeferredLightingProfile::StandardPbrPreview => {
            // The preview dispatch has no generic environment API callers, but it
            // still needs the complete local probe and planar-reflection provider.
            source_includes.push(environment_standard_pbr_include());
        }
    }
    roots.push(ENVIRONMENT_INCLUDE_TOKEN.to_string());

    let module_registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
        roots.iter().cloned(),
        source_includes,
    );
    let resolved = module_registry
        .resolve_roots(roots)
        .map_err(deferred_module_resolution_error)?;
    let mut source = String::new();
    for include in resolved.ordered_sources {
        push_include(&mut source, &include.token, &include.source);
    }
    Ok(source)
}

fn deferred_module_resolution_error(
    error: ShaderModuleResolutionError,
) -> DeferredLightingShaderSourceError {
    match error {
        ShaderModuleResolutionError::UnknownModule { token } => {
            DeferredLightingShaderSourceError::UnknownShaderModule { token }
        }
        ShaderModuleResolutionError::CircularDependency { cycle } => {
            DeferredLightingShaderSourceError::CircularShaderModuleDependency { cycle }
        }
    }
}

fn push_include(source: &mut String, token: &str, include: &str) {
    source.push_str("// include: ");
    source.push_str(token);
    source.push('\n');
    source.push_str(include);
    source.push('\n');
}

fn custom_deferred_dispatch(
    request: &DeferredLightingShaderSourceRequest,
) -> Result<String, DeferredLightingShaderSourceError> {
    let mut dispatch = String::new();
    for descriptor in request.shading_model_descriptors.iter() {
        if !builtin_deferred_include_token(descriptor.deferred_include.as_str())
            && !request
                .shading_model_deferred_include_sources
                .iter()
                .any(|include| {
                    deferred_include_tokens_match(&include.token, &descriptor.deferred_include)
                })
        {
            return Err(DeferredLightingShaderSourceError::UnknownDeferredInclude {
                token: descriptor.deferred_include.clone(),
            });
        }
        let function_name = deferred_shading_function_name(&descriptor.deferred_include);
        dispatch.push_str("    if (shading_model_id == ");
        dispatch.push_str(&descriptor.id.value().to_string());
        dispatch.push_str("u) {\n        return apply_deferred_volumetric(add_deferred_emissive(");
        dispatch.push_str(&function_name);
        dispatch.push_str(
            "(position, coord, albedo, material, normal), emissive), position, depth);\n    }\n",
        );
    }
    Ok(dispatch)
}

fn builtin_deferred_include_token(token: &str) -> bool {
    builtin_deferred_include_source(token).is_some()
}

fn builtin_deferred_include_source(token: &str) -> Option<&'static str> {
    let token = token.trim_end_matches(".wgsl");
    match token {
        "zr_shade_deferred_standard_pbr" => Some(DEFERRED_STANDARD_PBR_INCLUDE),
        "zr_shade_deferred_blinn_phong" => Some(DEFERRED_BLINN_PHONG_INCLUDE),
        "zr_shade_deferred_unlit" => Some(DEFERRED_UNLIT_INCLUDE),
        "zr_shade_deferred_subsurface" => Some(DEFERRED_SUBSURFACE_INCLUDE),
        _ => None,
    }
}

fn deferred_include_tokens_match(left: &str, right: &str) -> bool {
    left == right || left.trim_end_matches(".wgsl") == right.trim_end_matches(".wgsl")
}

fn deferred_shading_function_name(token: &str) -> String {
    let token = token
        .trim_end_matches(".wgsl")
        .trim_start_matches("zr_shade_deferred_");
    format!("shade_deferred_{token}")
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        GBufferChannelMask, SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR,
        SHADING_MODEL_ID_UNLIT, ShadingModelDescriptor, ShadingModelId,
    };
    use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;

    use super::*;

    #[test]
    fn builtin_deferred_include_tokens_accept_manifest_and_wgsl_forms() {
        for token in [
            "zr_shade_deferred_unlit",
            "zr_shade_deferred_unlit.wgsl",
            "zr_shade_deferred_blinn_phong",
            "zr_shade_deferred_standard_pbr.wgsl",
        ] {
            assert!(builtin_deferred_include_token(token));
        }
    }

    #[test]
    fn deferred_shading_function_names_strip_include_prefix_and_extension() {
        assert_eq!(
            deferred_shading_function_name("zr_shade_deferred_toon.wgsl"),
            "shade_deferred_toon"
        );
    }

    #[test]
    fn builtin_shading_model_ids_are_reserved_by_static_dispatch() {
        assert_eq!(SHADING_MODEL_ID_UNLIT.value(), 0);
        assert_eq!(SHADING_MODEL_ID_BLINN_PHONG.value(), 1);
        assert_eq!(SHADING_MODEL_ID_STANDARD_PBR.value(), 2);
    }

    #[test]
    fn fragment_template_keeps_the_fullscreen_vertex_entry_in_its_dedicated_module() {
        assert!(
            !DEFERRED_LIGHTING_TEMPLATE.contains("@vertex"),
            "the deferred fragment module must not retain the separately compiled fullscreen vertex entry"
        );
        assert!(!DEFERRED_LIGHTING_TEMPLATE.contains("fn vs_main("));
    }

    #[test]
    fn deferred_source_assembly_constructs_only_the_requested_module_closure() {
        let source = include_str!("shader_source.rs");
        let body = source
            .split_once("pub(in crate::graphics::scene::scene_renderer::deferred) fn assemble_deferred_lighting_shader_source")
            .expect("deferred source assembly function must exist")
            .1
            .split_once("\nfn deferred_module_resolution_error")
            .expect("deferred source assembly function must have a stable end")
            .0;

        assert!(body.contains("with_builtin_modules_for_roots("));
        assert!(!body.contains("with_builtin_modules();"));
        assert!(body.contains("roots.iter().any(|root| root == VOLUMETRIC_INCLUDE_TOKEN)"));
    }

    #[test]
    fn standard_pbr_preview_assembles_only_the_standard_builtin_material_variant() {
        let source = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::StandardPbrPreview,
            ),
        )
        .expect("standard PBR preview source should assemble");

        assert!(source.contains("// include: zr_shade_deferred_standard_pbr.wgsl"));
        assert!(source.contains("// include: zr_pbr_extras.wgsl"));
        assert!(source.contains("fn zr_pbr_isotropic_ggx("));
        assert!(!source.contains("// include: zr_shade_deferred_blinn_phong.wgsl"));
        assert!(!source.contains("// include: zr_shade_deferred_unlit.wgsl"));
        assert!(!source.contains("// include: zr_shade_deferred_subsurface.wgsl"));
        assert!(!source.contains("shade_deferred_blinn_phong("));
        assert!(!source.contains("shade_deferred_unlit("));
    }

    #[test]
    fn standard_pbr_preview_uses_source_independent_diffuse_and_shared_ggx_specular() {
        let source = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::StandardPbrPreview,
            ),
        )
        .expect("standard PBR preview source should assemble");

        for required in [
            "fn zr_pbr_isotropic_ggx(",
            "let specular = zr_pbr_isotropic_ggx(",
            "direct_diffuse_brdf * radiance * lambert",
            "radiance * specular * lambert",
            "zr_surface_metallic_diffuse_energy_scale(direct_metallic)",
        ] {
            assert!(
                source.contains(required),
                "deferred Standard PBR must retain source-independent diffuse/GGX contract `{required}`"
            );
        }
        for rejected in [
            "struct ZrPbrSpecularComponents",
            "fn zr_pbr_isotropic_ggx_components(",
            "specular_components.fresnel",
        ] {
            assert!(!source.contains(rejected));
        }
        assert!(
            !source.contains("zr_pbr_diffuse_energy_scale("),
            "standard PBR preview must use the shared metallic diffuse-energy owner"
        );
    }

    #[test]
    fn standard_pbr_preview_prunes_generic_environment_api_but_keeps_local_reflections() {
        let standard = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::StandardPbrPreview,
            ),
        )
        .expect("standard PBR preview source should assemble");
        let full_scene = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new()
                .with_deferred_lighting_profile(SceneRendererDeferredLightingProfile::FullScene),
        )
        .expect("full-scene deferred source should assemble");

        for required in [
            "@group(1) @binding(16)",
            "@group(1) @binding(17)",
            "@group(1) @binding(18)",
            "@group(1) @binding(29)",
            "@group(1) @binding(30)",
            "fn zr_environment_select_probes(",
            "fn zr_environment_planar_reflection(",
            "fn zr_environment_pbr_indirect(",
        ] {
            assert!(
                standard.contains(required),
                "standard PBR preview must retain local reflection source `{required}`"
            );
        }
        for excluded in [
            "fn zr_environment_fix_source_cube_lookup(",
            "fn zr_environment_source_cube_color_at_lod(",
            "fn zr_environment_specular_pmrem_color_at_lod(",
            "fn zr_environment_env_brdf_approx(",
            "fn zr_environment_sh9_eval(",
            "fn zr_environment_irradiance_cube_color(",
            "fn zr_environment_procedural_sky_color(",
            "fn zr_environment_sky_color(",
            "fn zr_environment_diffuse_color(",
        ] {
            assert!(
                !standard.contains(excluded),
                "standard PBR preview must prune unreachable API `{excluded}`"
            );
            assert!(
                full_scene.contains(excluded),
                "full-scene deferred must retain generic API `{excluded}`"
            );
        }
        assert!(
            standard.len() < full_scene.len(),
            "specialized preview should compile less source, standard={} full-scene={}",
            standard.len(),
            full_scene.len(),
        );
    }

    #[test]
    fn preview_profiles_reject_custom_shading_models_before_source_export() {
        let descriptor = ShadingModelDescriptor::new(
            ShadingModelId::new(128),
            "toon",
            "package://toon/forward.wgsl",
            "package://toon/gbuffer.wgsl",
            "package://toon/deferred.wgsl",
            GBufferChannelMask::standard_deferred_v1(),
        );

        for profile in [
            SceneRendererDeferredLightingProfile::StandardPbrPreview,
            SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        ] {
            let error = assemble_deferred_lighting_shader_source(
                DeferredLightingShaderSourceRequest::new()
                    .with_deferred_lighting_profile(profile)
                    .with_shading_model_descriptor(descriptor.clone()),
            )
            .expect_err("preview profiles must reject custom shading models");

            assert_eq!(
                error,
                DeferredLightingShaderSourceError::CustomShadingModelsUnsupportedByProfile {
                    profile,
                }
            );
        }
    }

    #[test]
    fn environment_only_pbr_preview_assembles_ibl_without_direct_lighting_dependencies() {
        let source = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            ),
        )
        .expect("environment-only PBR preview source should assemble");

        assert!(source.contains("// include: zr_environment.wgsl"));
        assert!(source.contains("zr_environment_pbr_indirect("));
        assert!(source.contains("fn zr_environment_pbr_indirect_with_dielectric_f0_normalized("));
        assert!(source.contains(
            "let environment_lights = zr_environment_pbr_indirect_with_dielectric_f0_normalized("
        ));
        assert!(
            !source.contains("zr_pbr_diffuse_energy_scale("),
            "environment-only deferred source must use the shared metallic diffuse-energy owner"
        );
        assert!(!source.contains("// include: zr_gpu_scene.wgsl"));
        assert!(!source.contains("// include: zr_light_grid.wgsl"));
        assert!(!source.contains("// include: zr_light_cookie.wgsl"));
        assert!(!source.contains("// include: zr_lightmap.wgsl"));
        assert!(!source.contains("// include: zr_shadow.wgsl"));
        assert!(!source.contains("// include: zr_volumetric.wgsl"));
        assert!(!source.contains("// include: zr_pbr_extras.wgsl"));
        assert!(!source.contains("fn zr_pbr_isotropic_ggx("));
        assert!(!source.contains("fn fs_main_sss("));
    }

    #[test]
    fn environment_only_pbr_preview_mirrors_rotation_abi() {
        let source = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            ),
        )
        .expect("environment-only PBR source should assemble");

        assert!(source.contains("fn zr_environment_pbr_indirect("));
        let scene_uniform = source
            .split("struct SceneUniform {")
            .nth(1)
            .and_then(|source| source.split("};").next())
            .expect("deferred preview must declare SceneUniform");
        assert!(
            scene_uniform.contains("environment_rotation_sin_cos: vec4<f32>,"),
            "deferred preview SceneUniform must mirror the rotation tail"
        );
        assert!(
            scene_uniform
                .find("environment_sample_params")
                .expect("sample params")
                < scene_uniform
                    .find("environment_rotation_sin_cos")
                    .expect("rotation tail"),
            "the rotation field must append after existing SceneUniform fields"
        );
    }

    #[test]
    fn environment_only_pbr_preview_retains_generic_environment_for_provider_upgrades() {
        let source = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            ),
        )
        .expect("environment-only PBR source should assemble");

        for required in [
            "@group(1) @binding(16)",
            "@group(1) @binding(17)",
            "@group(1) @binding(18)",
            "@group(1) @binding(29)",
            "@group(1) @binding(30)",
            "fn zr_environment_select_probes(",
            "fn zr_environment_planar_reflection(",
            "fn zr_environment_specular_pmrem_color_at_lod(",
            "fn zr_environment_irradiance_cube_color(",
        ] {
            assert!(
                source.contains(required),
                "environment-only deferred must retain provider-upgrade source `{required}`"
            );
        }
    }

    #[test]
    fn deferred_pbr_gbuffer_normal_decode_uses_zero_safe_normalization() {
        let environment_only = assemble_deferred_lighting_shader_source(
            DeferredLightingShaderSourceRequest::new().with_deferred_lighting_profile(
                SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
            ),
        )
        .expect("environment-only PBR source should assemble");

        for (label, source, expected_normal_decode) in [
            (
                "generic",
                DEFERRED_LIGHTING_SHADER,
                "let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0, 1.0, 1.0));",
            ),
            (
                "environment-only",
                environment_only.as_str(),
                "let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));",
            ),
        ] {
            assert!(
                source.contains("fn normalize_or_zero(value: vec3<f32>) -> vec3<f32>"),
                "{label} deferred source must retain the zero-safe normal helper"
            );
            assert!(
                source.contains(expected_normal_decode),
                "{label} deferred source must safely decode a degenerate GBuffer normal"
            );
            assert!(
                !source.contains("let normal = normalize(encoded_normal"),
                "{label} deferred source must not normalize a potentially zero GBuffer normal directly"
            );
        }

        assert!(
            DEFERRED_LIGHTING_SHADER.contains("fn fs_main_sss("),
            "generic deferred source must retain its SSS fragment entry point"
        );
        assert!(
            DEFERRED_LIGHTING_SHADER
                .contains("let normal = normalize_or_zero(encoded_normal * 2.0 - vec3<f32>(1.0));"),
            "generic SSS deferred source must safely decode a degenerate GBuffer normal"
        );
    }
}

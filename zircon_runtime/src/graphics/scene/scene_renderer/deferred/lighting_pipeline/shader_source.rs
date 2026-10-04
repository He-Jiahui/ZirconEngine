use crate::core::framework::render::{ShaderFeatureBits, ShadingModelDescriptor};
use crate::graphics::material::ShadingModelIncludeSourceSet;
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::shader::template::{
    environment_standard_pbr_include, pbr_extras_include_for_features, ShaderModuleRegistry,
    ShaderModuleResolutionError, ShaderTemplateInclude,
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

/// 按 profile 选择 WGSL 依赖闭包；任意非 FullScene profile 都拒绝非空描述列表。
/// 项目 include 由 FullScene 调用方先导出，此处只装配已提供的源码。
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
#[path = "tests/shader_source.rs"]
mod tests;

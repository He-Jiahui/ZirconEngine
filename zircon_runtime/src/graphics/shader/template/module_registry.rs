use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    is_generated_shader_module_token, strip_wgsl_include_directives, wgsl_include_paths,
    GeometrySourceDescriptor, ShaderFeatureBits, ShadingModelDescriptor,
    GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
};
use crate::plugin::ShaderModuleSourceBinding;

const SURFACE_TYPES_INCLUDE_TOKEN: &str = "zr_surface_types.wgsl";
const SCENE_RUNTIME_INCLUDE_TOKEN: &str = "zr_scene_runtime.wgsl";
const GPU_SCENE_INCLUDE_TOKEN: &str = "zr_gpu_scene.wgsl";
const LIGHTMAP_INCLUDE_TOKEN: &str = "zr_lightmap.wgsl";
const LIGHT_COOKIE_INCLUDE_TOKEN: &str = "zr_light_cookie.wgsl";
const IRRADIANCE_VOLUME_INCLUDE_TOKEN: &str = "zr_irradiance_volume.wgsl";
const ENVIRONMENT_INCLUDE_TOKEN: &str = "zr_environment.wgsl";
const VOLUMETRIC_INCLUDE_TOKEN: &str = "zr_volumetric.wgsl";
const OIT_INCLUDE_TOKEN: &str = "zr_oit.wgsl";
const PBR_EXTRAS_INCLUDE_TOKEN: &str = "zr_pbr_extras.wgsl";
const LIGHT_GRID_INCLUDE_TOKEN: &str = "zr_light_grid.wgsl";
const SHADOW_INCLUDE_TOKEN: &str = "zr_shadow.wgsl";
const STANDARD_PBR_SHADING_INCLUDE_TOKEN: &str = "zr_shading_standard_pbr.wgsl";
const STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN: &str = "zr_gbuffer_encode_standard_pbr.wgsl";
const SUBSURFACE_GBUFFER_ENCODE_INCLUDE_TOKEN: &str = "zr_gbuffer_encode_subsurface.wgsl";
const VIRTUAL_GEOMETRY_INCLUDE_TOKEN: &str = "zr_geometry_virtual_geometry.wgsl";
const BUILTIN_MODULE_INCLUDE_TOKENS: [&str; 19] = [
    SURFACE_TYPES_INCLUDE_TOKEN,
    SCENE_RUNTIME_INCLUDE_TOKEN,
    GPU_SCENE_INCLUDE_TOKEN,
    LIGHT_COOKIE_INCLUDE_TOKEN,
    IRRADIANCE_VOLUME_INCLUDE_TOKEN,
    LIGHTMAP_INCLUDE_TOKEN,
    ENVIRONMENT_INCLUDE_TOKEN,
    VOLUMETRIC_INCLUDE_TOKEN,
    OIT_INCLUDE_TOKEN,
    PBR_EXTRAS_INCLUDE_TOKEN,
    LIGHT_GRID_INCLUDE_TOKEN,
    SHADOW_INCLUDE_TOKEN,
    STANDARD_PBR_SHADING_INCLUDE_TOKEN,
    STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN,
    GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
    VIRTUAL_GEOMETRY_INCLUDE_TOKEN,
];

const STATIC_MESH_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_static.wgsl");
const SKINNED_MESH_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_skinned.wgsl");
const MORPHED_MESH_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_morphed.wgsl");
const SKINNED_MORPHED_MESH_GEOMETRY_INCLUDE: &str =
    include_str!("../wgsl/zr_geometry_skinned_morphed.wgsl");
const VIRTUAL_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_virtual_geometry.wgsl");
const SCENE_RUNTIME_INCLUDE: &str = include_str!("../wgsl/zr_scene_runtime.wgsl");
const GPU_SCENE_INCLUDE: &str =
    include_str!("../../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
const LIGHTMAP_INCLUDE: &str = include_str!("../wgsl/zr_lightmap.wgsl");
const LIGHT_COOKIE_INCLUDE: &str = include_str!("../wgsl/zr_light_cookie.wgsl");
const IRRADIANCE_VOLUME_INCLUDE: &str = include_str!("../wgsl/zr_irradiance_volume.wgsl");
const LIGHT_GRID_INCLUDE: &str =
    include_str!("../../scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl");
const SHADOW_INCLUDE: &str =
    include_str!("../../scene/scene_renderer/shadow/shaders/zr_shadow.wgsl");
const SHADOW_DISABLED_INCLUDE: &str = r#"
fn zr_gpu_light_shadow_visibility(
    _light: ZrGpuLightData,
    _light_type: u32,
    _world_position: vec3<f32>,
    _view_z: f32,
) -> f32 {
    return 1.0;
}
"#;
const SURFACE_TYPES_INCLUDE: &str = include_str!("../wgsl/zr_surface_types.wgsl");
const ENVIRONMENT_INCLUDE: &str = concat!(
    include_str!("../wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment_generic_api.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment.wgsl"),
);
const ENVIRONMENT_ONLY_PBR_INCLUDE: &str = concat!(
    include_str!("../wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment_only_pbr.wgsl"),
);
const ENVIRONMENT_STANDARD_PBR_INCLUDE: &str = concat!(
    include_str!("../wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment.wgsl"),
);
const VOLUMETRIC_INCLUDE: &str = include_str!("../wgsl/zr_volumetric.wgsl");
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
const OIT_INCLUDE: &str = include_str!("../includes/zr_oit.wgsl");
const PBR_EXTRAS_CORE_INCLUDE: &str = include_str!("../includes/zr_pbr_extras_core.wgsl");
const PBR_EXTRAS_INCLUDE: &str = concat!(
    include_str!("../includes/zr_pbr_extras_core.wgsl"),
    "\n",
    include_str!("../includes/zr_pbr_extras.wgsl"),
);
const STANDARD_PBR_SHADING_INCLUDE: &str = include_str!("../wgsl/zr_shading_standard_pbr.wgsl");
const STANDARD_PBR_SHADING_BASIC_INCLUDE: &str =
    include_str!("../wgsl/zr_shading_standard_pbr_basic.wgsl");
const ENVIRONMENT_ONLY_PBR_SHADING_INCLUDE: &str =
    include_str!("../wgsl/zr_shading_environment_only_pbr.wgsl");
const STANDARD_PBR_GBUFFER_ENCODE_INCLUDE: &str =
    include_str!("../wgsl/zr_gbuffer_encode_standard_pbr.wgsl");
const SUBSURFACE_GBUFFER_ENCODE_INCLUDE: &str =
    include_str!("../wgsl/zr_gbuffer_encode_subsurface.wgsl");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderTemplateInclude {
    pub(crate) token: String,
    pub(crate) owner_id: String,
    pub(crate) diagnostic_origin: String,
    pub(crate) source_hash: String,
    pub(crate) source: String,
    pub(crate) content_hash: String,
    pub(crate) dependencies: Vec<String>,
}

impl ShaderTemplateInclude {
    pub(crate) fn new(token: impl Into<String>, source: impl Into<String>) -> Self {
        let token = token.into();
        let source = source.into();
        let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
        let dependencies = wgsl_include_paths(&source);
        let source = strip_wgsl_include_directives(&source);
        Self {
            owner_id: format!("builtin:{token}"),
            diagnostic_origin: format!("builtin shader module `{token}`"),
            token,
            source_hash,
            content_hash: shader_module_content_hash(&source, &dependencies),
            source,
            dependencies,
        }
    }

    pub(crate) fn from_source_binding(binding: ShaderModuleSourceBinding) -> Self {
        let mut include = Self::new(binding.import_path, binding.source.as_ref());
        include.owner_id = binding.owner_id;
        include.diagnostic_origin = binding.diagnostic_origin;
        include.source_hash = binding.content_hash;
        include
    }

    pub(crate) fn with_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        for dependency in dependencies {
            let dependency = dependency.into();
            if !self.dependencies.contains(&dependency) {
                self.dependencies.push(dependency);
            }
        }
        self.content_hash = shader_module_content_hash(&self.source, &self.dependencies);
        self
    }
}

fn shader_module_content_hash(source: &str, dependencies: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(source.as_bytes());
    for dependency in dependencies {
        hasher.update(&[0]);
        hasher.update(dependency.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[derive(Default)]
pub(crate) struct ShaderTemplateIncludeRegistry {
    includes: Vec<ShaderTemplateInclude>,
    seen_tokens: HashSet<String>,
}

impl ShaderTemplateIncludeRegistry {
    pub(crate) fn push(&mut self, include: ShaderTemplateInclude) -> bool {
        if self.seen_tokens.insert(include.token.clone()) {
            self.includes.push(include);
            true
        } else {
            false
        }
    }

    pub(crate) fn into_manifest(self) -> (Vec<String>, Vec<String>) {
        let mut tokens = Vec::with_capacity(self.includes.len());
        let mut content_hashes = Vec::with_capacity(self.includes.len());
        for include in self.includes {
            tokens.push(include.token);
            content_hashes.push(include.content_hash);
        }
        (tokens, content_hashes)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedShaderModuleSet {
    pub(crate) ordered_sources: Vec<ShaderTemplateInclude>,
    pub(crate) content_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShaderModuleResolutionError {
    UnknownModule { token: String },
    CircularDependency { cycle: Vec<String> },
}

#[derive(Clone, Default)]
pub(crate) struct ShaderModuleRegistry {
    modules: HashMap<String, ShaderTemplateInclude>,
}

impl ShaderModuleRegistry {
    pub(crate) fn with_builtin_modules() -> Self {
        let mut registry = Self::default();
        for include in builtin_module_includes() {
            registry.register(include);
        }
        registry
    }

    pub(crate) fn register(&mut self, include: ShaderTemplateInclude) {
        self.modules.insert(include.token.clone(), include);
    }

    pub(crate) fn with_builtin_modules_for_roots(
        roots: impl IntoIterator<Item = String>,
        source_includes: impl IntoIterator<Item = ShaderTemplateInclude>,
    ) -> Self {
        let mut source_modules = source_includes
            .into_iter()
            .map(|include| (include.token.clone(), include))
            .collect::<HashMap<_, _>>();
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        let mut visited = HashSet::new();
        let mut registry = Self::default();

        while let Some(token) = pending.pop() {
            if !visited.insert(token.clone()) {
                continue;
            }
            let include = source_modules
                .remove(&token)
                .or_else(|| builtin_module_include_for_token(&token));
            let Some(include) = include else {
                continue;
            };
            pending.extend(include.dependencies.iter().cloned());
            registry.register(include);
        }
        registry
    }

    pub(crate) fn resolve_for_source(
        &self,
        source: &str,
    ) -> Result<ResolvedShaderModuleSet, ShaderModuleResolutionError> {
        self.resolve_roots(wgsl_include_paths(source))
    }

    pub(crate) fn resolve_roots(
        &self,
        roots: impl IntoIterator<Item = String>,
    ) -> Result<ResolvedShaderModuleSet, ShaderModuleResolutionError> {
        let mut ordered_sources = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = Vec::new();
        for root in roots {
            if is_generated_shader_module_token(&root) {
                continue;
            }
            self.visit(&root, &mut visiting, &mut visited, &mut ordered_sources)?;
        }
        let content_hash = module_set_hash(&ordered_sources);
        Ok(ResolvedShaderModuleSet {
            ordered_sources,
            content_hash,
        })
    }

    fn visit(
        &self,
        token: &str,
        visiting: &mut Vec<String>,
        visited: &mut HashSet<String>,
        ordered_sources: &mut Vec<ShaderTemplateInclude>,
    ) -> Result<(), ShaderModuleResolutionError> {
        if visited.contains(token) || is_generated_shader_module_token(token) {
            return Ok(());
        }
        if let Some(cycle_start) = visiting.iter().position(|entry| entry == token) {
            let mut cycle = visiting[cycle_start..].to_vec();
            cycle.push(token.to_string());
            return Err(ShaderModuleResolutionError::CircularDependency { cycle });
        }
        let module = self.modules.get(token).cloned().ok_or_else(|| {
            ShaderModuleResolutionError::UnknownModule {
                token: token.to_string(),
            }
        })?;
        visiting.push(token.to_string());
        for dependency in &module.dependencies {
            self.visit(dependency.as_str(), visiting, visited, ordered_sources)?;
        }
        visiting.pop();
        visited.insert(token.to_string());
        ordered_sources.push(module);
        Ok(())
    }
}

fn module_set_hash(modules: &[ShaderTemplateInclude]) -> String {
    let mut hasher = blake3::Hasher::new();
    for module in modules {
        hasher.update(module.token.as_bytes());
        hasher.update(&[0]);
        hasher.update(module.content_hash.as_bytes());
        hasher.update(&[0]);
    }
    hasher.finalize().to_hex().to_string()
}

fn builtin_module_includes() -> Vec<ShaderTemplateInclude> {
    BUILTIN_MODULE_INCLUDE_TOKENS
        .into_iter()
        .map(|token| {
            builtin_module_include_for_token(token)
                .expect("every builtin shader module token must have a source factory")
        })
        .collect()
}

fn builtin_module_include_for_token(token: &str) -> Option<ShaderTemplateInclude> {
    match token {
        SURFACE_TYPES_INCLUDE_TOKEN => Some(surface_types_include()),
        SCENE_RUNTIME_INCLUDE_TOKEN => Some(scene_runtime_include()),
        GPU_SCENE_INCLUDE_TOKEN => Some(gpu_scene_include()),
        LIGHT_COOKIE_INCLUDE_TOKEN => Some(light_cookie_include()),
        IRRADIANCE_VOLUME_INCLUDE_TOKEN => Some(irradiance_volume_include()),
        LIGHTMAP_INCLUDE_TOKEN => Some(lightmap_include()),
        ENVIRONMENT_INCLUDE_TOKEN => Some(environment_include()),
        VOLUMETRIC_INCLUDE_TOKEN => Some(volumetric_include()),
        OIT_INCLUDE_TOKEN => Some(oit_include()),
        PBR_EXTRAS_INCLUDE_TOKEN => Some(pbr_extras_include()),
        LIGHT_GRID_INCLUDE_TOKEN => Some(light_grid_include()),
        SHADOW_INCLUDE_TOKEN => Some(shadow_include()),
        STANDARD_PBR_SHADING_INCLUDE_TOKEN => Some(standard_pbr_shading_include()),
        STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN => Some(standard_pbr_gbuffer_encode_include()),
        GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
            STATIC_MESH_GEOMETRY_INCLUDE,
        )),
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
            SKINNED_MESH_GEOMETRY_INCLUDE,
        )),
        GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
            MORPHED_MESH_GEOMETRY_INCLUDE,
        )),
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
            SKINNED_MORPHED_MESH_GEOMETRY_INCLUDE,
        )),
        VIRTUAL_GEOMETRY_INCLUDE_TOKEN => Some(ShaderTemplateInclude::new(
            VIRTUAL_GEOMETRY_INCLUDE_TOKEN,
            VIRTUAL_GEOMETRY_INCLUDE,
        )),
        _ => None,
    }
}

pub(crate) fn builtin_shader_ide_module_includes() -> Vec<ShaderTemplateInclude> {
    builtin_module_includes()
        .into_iter()
        .map(|mut include| {
            if !include.dependencies.is_empty() {
                let mut source = include
                    .dependencies
                    .iter()
                    .map(|dependency| format!("#include <{dependency}>\n"))
                    .collect::<String>();
                source.push_str(&include.source);
                include.source = source;
            }
            include
        })
        .collect()
}

pub(crate) fn surface_types_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(SURFACE_TYPES_INCLUDE_TOKEN, SURFACE_TYPES_INCLUDE)
}

pub(crate) fn scene_runtime_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(SCENE_RUNTIME_INCLUDE_TOKEN, SCENE_RUNTIME_INCLUDE)
}

pub(crate) fn gpu_scene_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(GPU_SCENE_INCLUDE_TOKEN, GPU_SCENE_INCLUDE)
}

pub(crate) fn lightmap_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(LIGHTMAP_INCLUDE_TOKEN, LIGHTMAP_INCLUDE)
        .with_dependencies([IRRADIANCE_VOLUME_INCLUDE_TOKEN])
}

pub(crate) fn light_cookie_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(LIGHT_COOKIE_INCLUDE_TOKEN, LIGHT_COOKIE_INCLUDE)
}

pub(crate) fn irradiance_volume_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(IRRADIANCE_VOLUME_INCLUDE_TOKEN, IRRADIANCE_VOLUME_INCLUDE)
}

pub(crate) fn environment_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(ENVIRONMENT_INCLUDE_TOKEN, ENVIRONMENT_INCLUDE)
}

pub(crate) fn environment_only_pbr_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(ENVIRONMENT_INCLUDE_TOKEN, ENVIRONMENT_ONLY_PBR_INCLUDE)
}

pub(crate) fn environment_standard_pbr_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(ENVIRONMENT_INCLUDE_TOKEN, ENVIRONMENT_STANDARD_PBR_INCLUDE)
}

pub(crate) fn volumetric_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(VOLUMETRIC_INCLUDE_TOKEN, VOLUMETRIC_INCLUDE)
}

pub(crate) fn volumetric_disabled_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(VOLUMETRIC_INCLUDE_TOKEN, VOLUMETRIC_DISABLED_INCLUDE)
}

pub(crate) fn oit_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(OIT_INCLUDE_TOKEN, OIT_INCLUDE)
}

pub(crate) fn pbr_extras_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(PBR_EXTRAS_INCLUDE_TOKEN, PBR_EXTRAS_INCLUDE)
}

pub(crate) fn pbr_extras_include_for_features(
    features: ShaderFeatureBits,
) -> ShaderTemplateInclude {
    let source = if uses_advanced_standard_pbr(features) {
        PBR_EXTRAS_INCLUDE
    } else {
        PBR_EXTRAS_CORE_INCLUDE
    };
    ShaderTemplateInclude::new(PBR_EXTRAS_INCLUDE_TOKEN, source)
}

pub(crate) fn light_grid_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(LIGHT_GRID_INCLUDE_TOKEN, LIGHT_GRID_INCLUDE)
}

pub(crate) fn shadow_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(SHADOW_INCLUDE_TOKEN, SHADOW_INCLUDE)
}

pub(crate) fn shadow_disabled_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(SHADOW_INCLUDE_TOKEN, SHADOW_DISABLED_INCLUDE)
}

pub(crate) fn standard_pbr_shading_include() -> ShaderTemplateInclude {
    standard_pbr_shading_include_from_source(STANDARD_PBR_SHADING_INCLUDE)
}

pub(crate) fn standard_pbr_shading_include_for_features(
    features: ShaderFeatureBits,
) -> ShaderTemplateInclude {
    let source = if uses_advanced_standard_pbr(features) {
        STANDARD_PBR_SHADING_INCLUDE
    } else {
        STANDARD_PBR_SHADING_BASIC_INCLUDE
    };
    standard_pbr_shading_include_from_source(source)
}

pub(crate) fn standard_pbr_gbuffer_encode_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(
        STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN,
        STANDARD_PBR_GBUFFER_ENCODE_INCLUDE,
    )
}

pub(crate) fn shading_model_forward_include_token<'a>(
    descriptor: Option<&'a ShadingModelDescriptor>,
) -> &'a str {
    descriptor
        .map(|descriptor| descriptor.forward_include.as_str())
        .unwrap_or(STANDARD_PBR_SHADING_INCLUDE_TOKEN)
}

pub(crate) fn shading_model_forward_include_for(
    descriptor: Option<&ShadingModelDescriptor>,
    source_includes: &[ShaderTemplateInclude],
    features: ShaderFeatureBits,
    uses_builtin_standard_pbr: bool,
) -> Option<ShaderTemplateInclude> {
    if descriptor.is_none() {
        if uses_builtin_standard_pbr && features.contains(ShaderFeatureBits::ENVIRONMENT_ONLY_PBR) {
            return Some(ShaderTemplateInclude::new(
                STANDARD_PBR_SHADING_INCLUDE_TOKEN,
                ENVIRONMENT_ONLY_PBR_SHADING_INCLUDE,
            ));
        }
        return Some(if uses_builtin_standard_pbr {
            standard_pbr_shading_include_for_features(features)
        } else {
            standard_pbr_shading_include()
        });
    }
    let token = shading_model_forward_include_token(descriptor);
    forward_shading_include_for_token(token).or_else(|| {
        source_includes
            .iter()
            .find(|include| include.token.as_str() == token)
            .cloned()
    })
}

fn uses_advanced_standard_pbr(features: ShaderFeatureBits) -> bool {
    features.contains(ShaderFeatureBits::PBR_CLEARCOAT)
        || features.contains(ShaderFeatureBits::PBR_ANISOTROPY)
        || features.contains(ShaderFeatureBits::PBR_TRANSMISSION)
}

fn standard_pbr_shading_include_from_source(source: &str) -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(STANDARD_PBR_SHADING_INCLUDE_TOKEN, source)
        .with_dependencies([PBR_EXTRAS_INCLUDE_TOKEN, LIGHT_COOKIE_INCLUDE_TOKEN])
}

pub(crate) fn shading_model_gbuffer_include_token<'a>(
    descriptor: Option<&'a ShadingModelDescriptor>,
) -> &'a str {
    descriptor
        .map(|descriptor| descriptor.gbuffer_encode_include.as_str())
        .unwrap_or(STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN)
}

pub(crate) fn shading_model_gbuffer_include_for(
    descriptor: Option<&ShadingModelDescriptor>,
    source_includes: &[ShaderTemplateInclude],
) -> Option<ShaderTemplateInclude> {
    let token = shading_model_gbuffer_include_token(descriptor);
    gbuffer_encode_include_for_token(token).or_else(|| {
        source_includes
            .iter()
            .find(|include| include.token.as_str() == token)
            .cloned()
    })
}

pub(crate) fn geometry_source_include_for(
    descriptor: &GeometrySourceDescriptor,
) -> Option<ShaderTemplateInclude> {
    match descriptor.wgsl_include.as_str() {
        GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
            STATIC_MESH_GEOMETRY_INCLUDE,
        )),
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
            SKINNED_MESH_GEOMETRY_INCLUDE,
        )),
        GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
            MORPHED_MESH_GEOMETRY_INCLUDE,
        )),
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH => Some(ShaderTemplateInclude::new(
            GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
            SKINNED_MORPHED_MESH_GEOMETRY_INCLUDE,
        )),
        VIRTUAL_GEOMETRY_INCLUDE_TOKEN => Some(ShaderTemplateInclude::new(
            VIRTUAL_GEOMETRY_INCLUDE_TOKEN,
            VIRTUAL_GEOMETRY_INCLUDE,
        )),
        _ => None,
    }
}

fn forward_shading_include_for_token(token: &str) -> Option<ShaderTemplateInclude> {
    match token {
        STANDARD_PBR_SHADING_INCLUDE_TOKEN => Some(standard_pbr_shading_include()),
        _ => None,
    }
}

fn gbuffer_encode_include_for_token(token: &str) -> Option<ShaderTemplateInclude> {
    match token {
        STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN => Some(standard_pbr_gbuffer_encode_include()),
        SUBSURFACE_GBUFFER_ENCODE_INCLUDE_TOKEN => Some(ShaderTemplateInclude::new(
            SUBSURFACE_GBUFFER_ENCODE_INCLUDE_TOKEN,
            SUBSURFACE_GBUFFER_ENCODE_INCLUDE,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::strip_wgsl_include_directives;

    use super::*;

    #[test]
    fn shader_template_assemblies_move_the_include_manifest() {
        for source in [
            include_str!("assemble.rs"),
            include_str!("deferred_gbuffer.rs"),
            include_str!("taa_reactive_mask.rs"),
        ] {
            assert!(source.contains("registry.into_manifest()"));
            assert!(!source.contains(concat!("registry.include_", "tokens()")));
            assert!(!source.contains(concat!("registry.content_", "hashes()")));
        }
    }

    #[test]
    fn builtin_lightmap_resolves_irradiance_volume_dependency_first() {
        let registry = ShaderModuleRegistry::with_builtin_modules();
        let resolved = registry
            .resolve_roots([LIGHTMAP_INCLUDE_TOKEN.to_string()])
            .expect("builtin lightmap dependency graph should resolve");

        assert_eq!(
            resolved
                .ordered_sources
                .iter()
                .map(|module| module.token.as_str())
                .collect::<Vec<_>>(),
            vec![IRRADIANCE_VOLUME_INCLUDE_TOKEN, LIGHTMAP_INCLUDE_TOKEN]
        );
    }

    #[test]
    fn builtin_pbr_extras_is_independent_from_volumetric_uv_helpers() {
        let registry = ShaderModuleRegistry::with_builtin_modules();
        let resolved = registry
            .resolve_roots([PBR_EXTRAS_INCLUDE_TOKEN.to_string()])
            .expect("builtin PBR extras dependency graph should resolve");

        assert_eq!(
            resolved
                .ordered_sources
                .iter()
                .map(|module| module.token.as_str())
                .collect::<Vec<_>>(),
            vec![PBR_EXTRAS_INCLUDE_TOKEN]
        );
    }

    #[test]
    fn root_scoped_registry_constructs_only_the_requested_dependency_closure() {
        let project_include = ShaderTemplateInclude::new(
            "project::surface",
            "#include <zr_pbr_extras.wgsl>\nfn project_surface() {}",
        );
        let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
            ["project::surface".to_string()],
            [project_include],
        );

        assert!(registry.modules.contains_key("project::surface"));
        assert!(registry.modules.contains_key(PBR_EXTRAS_INCLUDE_TOKEN));
        assert!(!registry.modules.contains_key(SHADOW_INCLUDE_TOKEN));
        assert!(!registry.modules.contains_key(VOLUMETRIC_INCLUDE_TOKEN));
    }

    #[test]
    fn root_scoped_registry_prefers_supplied_source_over_builtin() {
        let disabled_volumetric = ShaderTemplateInclude::new(
            VOLUMETRIC_INCLUDE_TOKEN,
            "fn zr_apply_volumetric_fog(color: vec3<f32>) -> vec3<f32> { return color; }",
        );
        let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
            [VOLUMETRIC_INCLUDE_TOKEN.to_string()],
            [disabled_volumetric.clone()],
        );

        let resolved = registry
            .resolve_roots([VOLUMETRIC_INCLUDE_TOKEN.to_string()])
            .expect("supplied source should replace the builtin module for the same token");

        assert_eq!(resolved.ordered_sources, vec![disabled_volumetric]);
    }

    #[test]
    fn root_scoped_registry_preserves_unknown_dependency_errors() {
        let project_include = ShaderTemplateInclude::new(
            "project::surface",
            "#include <project::missing>\nfn project_surface() {}",
        );
        let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
            ["project::surface".to_string()],
            [project_include],
        );

        let error = registry
            .resolve_roots(["project::surface".to_string()])
            .expect_err("an unknown transitive dependency must remain an assembly error");

        assert_eq!(
            error,
            ShaderModuleResolutionError::UnknownModule {
                token: "project::missing".to_string(),
            }
        );
    }

    #[test]
    fn builtin_standard_pbr_resolves_advanced_lighting_dependencies_first() {
        let registry = ShaderModuleRegistry::with_builtin_modules();
        let resolved = registry
            .resolve_roots([STANDARD_PBR_SHADING_INCLUDE_TOKEN.to_string()])
            .expect("builtin Standard PBR dependency graph should resolve");

        assert_eq!(
            resolved
                .ordered_sources
                .iter()
                .map(|module| module.token.as_str())
                .collect::<Vec<_>>(),
            vec![
                PBR_EXTRAS_INCLUDE_TOKEN,
                LIGHT_COOKIE_INCLUDE_TOKEN,
                STANDARD_PBR_SHADING_INCLUDE_TOKEN,
            ]
        );
    }

    #[test]
    fn shader_module_registry_resolves_transitive_modules_once() {
        let mut registry = ShaderModuleRegistry::with_builtin_modules();
        registry.register(ShaderTemplateInclude::new(
            "project::a",
            "#include <project::b>\nfn a_value() -> f32 { return b_value(); }",
        ));
        registry.register(ShaderTemplateInclude::new(
            "project::b",
            "fn b_value() -> f32 { return 1.0; }",
        ));

        let resolved = registry
            .resolve_for_source("#include <project::a>\n#include <project::b>")
            .expect("modules should resolve");

        assert_eq!(
            resolved
                .ordered_sources
                .iter()
                .map(|module| module.token.as_str())
                .collect::<Vec<_>>(),
            vec!["project::b", "project::a"]
        );
        assert!(!resolved.content_hash.is_empty());
    }

    #[test]
    fn shader_module_registry_reports_cycles() {
        let mut registry = ShaderModuleRegistry::with_builtin_modules();
        registry.register(ShaderTemplateInclude::new(
            "project::a",
            "#include <project::b>",
        ));
        registry.register(ShaderTemplateInclude::new(
            "project::b",
            "#include <project::a>",
        ));

        let error = registry
            .resolve_for_source("#include <project::a>")
            .expect_err("cycle should fail");

        assert_eq!(
            error,
            ShaderModuleResolutionError::CircularDependency {
                cycle: vec![
                    "project::a".to_string(),
                    "project::b".to_string(),
                    "project::a".to_string(),
                ],
            }
        );
    }

    #[test]
    fn shader_module_registry_strips_include_directives() {
        let source = "// #include <ignored>\n#include <self::material>\nfn surface() {}";

        assert_eq!(
            wgsl_include_paths(source),
            vec!["self::material".to_string()]
        );
        assert_eq!(
            strip_wgsl_include_directives(source),
            "// #include <ignored>\nfn surface() {}"
        );
    }
}

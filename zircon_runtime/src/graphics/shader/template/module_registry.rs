//! 以逻辑导入 token 组织 WGSL 模块，按材质根裁剪依赖并输出依赖先行顺序。
//! 来源加载由上游资源流负责；此处将装配接受的 token 与内容散列交给模板装配器。

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
const NORMAL_INCLUDE_TOKEN: &str = "zr_normal.wgsl";
const BINDLESS_MATERIAL_INCLUDE_TOKEN: &str = "zr_bindless_material.wgsl";
const PBR_COMMON_INCLUDE_TOKEN: &str = "zr_pbr_common.wgsl";
const PBR_EXTRAS_INCLUDE_TOKEN: &str = "zr_pbr_extras.wgsl";
const LIGHT_GRID_INCLUDE_TOKEN: &str = "zr_light_grid.wgsl";
const SHADOW_INCLUDE_TOKEN: &str = "zr_shadow.wgsl";
const STANDARD_PBR_SHADING_INCLUDE_TOKEN: &str = "zr_shading_standard_pbr.wgsl";
const STANDARD_PBR_GBUFFER_ENCODE_INCLUDE_TOKEN: &str = "zr_gbuffer_encode_standard_pbr.wgsl";
const SUBSURFACE_GBUFFER_ENCODE_INCLUDE_TOKEN: &str = "zr_gbuffer_encode_subsurface.wgsl";
const VIRTUAL_GEOMETRY_INCLUDE_TOKEN: &str = "zr_geometry_virtual_geometry.wgsl";
const BUILTIN_MODULE_INCLUDE_TOKENS: [&str; 22] = [
    SURFACE_TYPES_INCLUDE_TOKEN,
    SCENE_RUNTIME_INCLUDE_TOKEN,
    GPU_SCENE_INCLUDE_TOKEN,
    LIGHT_COOKIE_INCLUDE_TOKEN,
    IRRADIANCE_VOLUME_INCLUDE_TOKEN,
    LIGHTMAP_INCLUDE_TOKEN,
    ENVIRONMENT_INCLUDE_TOKEN,
    VOLUMETRIC_INCLUDE_TOKEN,
    OIT_INCLUDE_TOKEN,
    NORMAL_INCLUDE_TOKEN,
    BINDLESS_MATERIAL_INCLUDE_TOKEN,
    PBR_COMMON_INCLUDE_TOKEN,
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
    include_str!("../wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment_generic_api.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment.wgsl"),
);
const ENVIRONMENT_ONLY_PBR_INCLUDE: &str = concat!(
    include_str!("../wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment_core.wgsl"),
    "\n",
    include_str!("../wgsl/zr_environment_only_pbr.wgsl"),
);
const ENVIRONMENT_STANDARD_PBR_INCLUDE: &str = concat!(
    include_str!("../wgsl/zr_procedural_sky.wgsl"),
    "\n",
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
const NORMAL_INCLUDE: &str = include_str!("../includes/zr_normal.wgsl");
const BINDLESS_MATERIAL_INCLUDE: &str = include_str!("../includes/zr_bindless_material.wgsl");
const PBR_COMMON_INCLUDE: &str = include_str!("../includes/zr_pbr_common.wgsl");
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

/// 持有剥离 include 指令的正文及有序依赖，另保留来源诊断信息。
/// `source_hash` 标识输入原文，`content_hash` 标识正文与依赖序列；token 仅用于查找和去重。
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

    /// 补充未写在 WGSL 指令中的依赖，保留既有顺序并避免新增重复项。
    /// 依赖变化后重算装配散列；原文散列继续描述构造时收到的输入文本。
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

/// 按首次出现的 token 保存已进入模板的模块及其装配身份。
/// 装配器先接受所选内建特性变体，再追加依赖解析结果，同 token 的后项不会替换它。
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

    /// 模板正文装配完成后，按相同顺序移出 token 与内容散列，形成等长且索引对应的 manifest。
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

/// 保存每个逻辑 token 的最终来源，登记同 token 时以后项覆盖前项。
/// 解析顺序由请求根与依赖向量决定，不依赖 HashMap 的枚举顺序。
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

    /// 仅构造请求根的依赖闭包，提供的来源优先于同 token 的内建 factory；提供来源内部以后项为准。
    /// 缺失 token 留待解析时报告；此处 visited 只防止重复构造，循环诊断由解析阶段完成。
    pub(crate) fn with_builtin_modules_for_roots(
        roots: impl IntoIterator<Item = String>,
        source_includes: impl IntoIterator<Item = ShaderTemplateInclude>,
    ) -> Self {
        let mut source_modules = source_includes
            .into_iter()
            .map(|include| (include.token.clone(), include))
            .collect::<HashMap<_, _>>();
        let mut pending = roots.into_iter().collect::<Vec<_>>();
        let traversal_capacity = source_modules.len().saturating_add(pending.len());
        let mut visited = HashSet::with_capacity(traversal_capacity);
        let mut registry = Self {
            modules: HashMap::with_capacity(traversal_capacity),
        };

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

    /// 按根的输入顺序解析，每个普通模块只输出一次，且依赖先于使用它的模块。
    /// `self::` 模块由模板装配器提供，不在此处查找；成功完成后才计算有序模块集合的内容身份。
    pub(crate) fn resolve_roots(
        &self,
        roots: impl IntoIterator<Item = String>,
    ) -> Result<ResolvedShaderModuleSet, ShaderModuleResolutionError> {
        let module_capacity = self.modules.len();
        let mut ordered_sources = Vec::with_capacity(module_capacity);
        let mut visited = HashSet::with_capacity(module_capacity);
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
        // visiting 表示当前递归路径，visited 只接纳依赖已全部完成的模块。
        // 先解析依赖，再退栈并标记完成，以保留循环诊断与依赖先行顺序。
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
        NORMAL_INCLUDE_TOKEN => Some(ShaderTemplateInclude::new(
            NORMAL_INCLUDE_TOKEN,
            NORMAL_INCLUDE,
        )),
        BINDLESS_MATERIAL_INCLUDE_TOKEN => Some(bindless_material_include()),
        PBR_COMMON_INCLUDE_TOKEN => Some(pbr_common_include()),
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

/// 为 IDE 将单独保存的依赖写回 include 指令，恢复可供导入工具读取的源码形式。
/// 直接消费者只取 token 与 source；此导出不重新定义运行时装配 manifest 中的内容身份。
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
        .with_dependencies([PBR_COMMON_INCLUDE_TOKEN])
}

pub(crate) fn scene_runtime_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(SCENE_RUNTIME_INCLUDE_TOKEN, SCENE_RUNTIME_INCLUDE)
}

pub(crate) fn gpu_scene_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(GPU_SCENE_INCLUDE_TOKEN, GPU_SCENE_INCLUDE)
}

pub(crate) fn bindless_material_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(BINDLESS_MATERIAL_INCLUDE_TOKEN, BINDLESS_MATERIAL_INCLUDE)
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
        .with_dependencies([PBR_COMMON_INCLUDE_TOKEN])
}

pub(crate) fn environment_only_pbr_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(ENVIRONMENT_INCLUDE_TOKEN, ENVIRONMENT_ONLY_PBR_INCLUDE)
        .with_dependencies([PBR_COMMON_INCLUDE_TOKEN])
}

pub(crate) fn environment_standard_pbr_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(ENVIRONMENT_INCLUDE_TOKEN, ENVIRONMENT_STANDARD_PBR_INCLUDE)
        .with_dependencies([PBR_COMMON_INCLUDE_TOKEN])
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

pub(crate) fn pbr_common_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(PBR_COMMON_INCLUDE_TOKEN, PBR_COMMON_INCLUDE)
}

pub(crate) fn pbr_extras_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(PBR_EXTRAS_INCLUDE_TOKEN, PBR_EXTRAS_INCLUDE)
        .with_dependencies([PBR_COMMON_INCLUDE_TOKEN])
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
        .with_dependencies([PBR_COMMON_INCLUDE_TOKEN])
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
#[path = "module_registry/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "module_registry/tests/optimization_batch_iq_runtime627_tests.rs"]
mod optimization_batch_iq_runtime627_tests;

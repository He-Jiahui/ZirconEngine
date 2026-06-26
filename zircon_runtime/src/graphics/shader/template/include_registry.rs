use std::collections::HashSet;

use crate::core::framework::render::{
    GeometrySourceDescriptor, GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH, GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
    GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
};

const SURFACE_TYPES_INCLUDE_TOKEN: &str = "zr_surface_types.wgsl";
const SCENE_RUNTIME_INCLUDE_TOKEN: &str = "zr_scene_runtime.wgsl";
const GPU_SCENE_INCLUDE_TOKEN: &str = "zr_gpu_scene.wgsl";
const LIGHT_GRID_INCLUDE_TOKEN: &str = "zr_light_grid.wgsl";
const SHADOW_INCLUDE_TOKEN: &str = "zr_shadow.wgsl";
const STANDARD_PBR_SHADING_INCLUDE_TOKEN: &str = "zr_shading_standard_pbr.wgsl";

const STATIC_MESH_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_static.wgsl");
const SKINNED_MESH_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_skinned.wgsl");
const MORPHED_MESH_GEOMETRY_INCLUDE: &str = include_str!("../wgsl/zr_geometry_morphed.wgsl");
const SKINNED_MORPHED_MESH_GEOMETRY_INCLUDE: &str =
    include_str!("../wgsl/zr_geometry_skinned_morphed.wgsl");
const SCENE_RUNTIME_INCLUDE: &str = include_str!("../wgsl/zr_scene_runtime.wgsl");
const GPU_SCENE_INCLUDE: &str =
    include_str!("../../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");
const LIGHT_GRID_INCLUDE: &str =
    include_str!("../../scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl");
const SHADOW_INCLUDE: &str =
    include_str!("../../scene/scene_renderer/shadow/shaders/zr_shadow.wgsl");
const SURFACE_TYPES_INCLUDE: &str = include_str!("../wgsl/zr_surface_types.wgsl");
const STANDARD_PBR_SHADING_INCLUDE: &str = include_str!("../wgsl/zr_shading_standard_pbr.wgsl");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShaderTemplateInclude {
    pub(crate) token: &'static str,
    pub(crate) source: &'static str,
    pub(crate) content_hash: String,
}

impl ShaderTemplateInclude {
    pub(crate) fn new(token: &'static str, source: &'static str) -> Self {
        Self {
            token,
            source,
            content_hash: blake3::hash(source.as_bytes()).to_hex().to_string(),
        }
    }
}

#[derive(Default)]
pub(crate) struct ShaderTemplateIncludeRegistry {
    includes: Vec<ShaderTemplateInclude>,
    seen_tokens: HashSet<&'static str>,
}

impl ShaderTemplateIncludeRegistry {
    pub(crate) fn push(&mut self, include: ShaderTemplateInclude) {
        if self.seen_tokens.insert(include.token) {
            self.includes.push(include);
        }
    }

    pub(crate) fn include_tokens(&self) -> Vec<String> {
        self.includes
            .iter()
            .map(|include| include.token.to_string())
            .collect()
    }

    pub(crate) fn content_hashes(&self) -> Vec<String> {
        self.includes
            .iter()
            .map(|include| include.content_hash.clone())
            .collect()
    }
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

pub(crate) fn light_grid_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(LIGHT_GRID_INCLUDE_TOKEN, LIGHT_GRID_INCLUDE)
}

pub(crate) fn shadow_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(SHADOW_INCLUDE_TOKEN, SHADOW_INCLUDE)
}

pub(crate) fn standard_pbr_shading_include() -> ShaderTemplateInclude {
    ShaderTemplateInclude::new(
        STANDARD_PBR_SHADING_INCLUDE_TOKEN,
        STANDARD_PBR_SHADING_INCLUDE,
    )
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
        _ => None,
    }
}

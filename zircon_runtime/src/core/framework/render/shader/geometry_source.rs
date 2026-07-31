use serde::{Deserialize, Serialize};

use super::definition_value::RenderShaderDefinitionValue;

pub const GEOMETRY_SOURCE_ID_STATIC_MESH: GeometrySourceId = GeometrySourceId::new(0);
pub const GEOMETRY_SOURCE_ID_SKINNED_MESH: GeometrySourceId = GeometrySourceId::new(1);
pub const GEOMETRY_SOURCE_ID_MORPHED_MESH: GeometrySourceId = GeometrySourceId::new(2);
pub const GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH: GeometrySourceId = GeometrySourceId::new(3);
pub const GEOMETRY_SOURCE_PLUGIN_ID_START: u8 = 4;

pub const GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH: &str = "zr_geometry_static.wgsl";
pub const GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH: &str = "zr_geometry_skinned.wgsl";
pub const GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH: &str = "zr_geometry_morphed.wgsl";
pub const GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH: &str =
    "zr_geometry_skinned_morphed.wgsl";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GeometrySourceId(u8);

impl GeometrySourceId {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn is_plugin_range(self) -> bool {
        self.0 >= GEOMETRY_SOURCE_PLUGIN_ID_START
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometrySourceVertexAttribute {
    Position,
    Normal,
    Tangent,
    Uv0,
    Color0,
    JointIndices,
    JointWeights,
    MorphPositionDelta,
    MorphNormalDelta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometrySourceBindingKind {
    GpuSceneInstance,
    SkinningPaletteStorage,
    MorphWeightsStorage,
    MorphTargetStorage,
    VirtualGeometryPages,
    VirtualGeometryClusters,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeometrySourceBindingRequirement {
    pub kind: GeometrySourceBindingKind,
    pub slot_token: String,
}

impl GeometrySourceBindingRequirement {
    pub fn new(kind: GeometrySourceBindingKind, slot_token: impl Into<String>) -> Self {
        Self {
            kind,
            slot_token: slot_token.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeometrySourceDescriptor {
    pub id: GeometrySourceId,
    pub token: String,
    pub wgsl_include: String,
    pub vertex_attributes: Vec<GeometrySourceVertexAttribute>,
    pub required_bindings: Vec<GeometrySourceBindingRequirement>,
    pub shader_defines: Vec<RenderShaderDefinitionValue>,
}

impl GeometrySourceDescriptor {
    pub fn requires_binding(&self, kind: GeometrySourceBindingKind) -> bool {
        self.required_bindings
            .iter()
            .any(|binding| binding.kind == kind)
    }

    pub fn has_vertex_attribute(&self, attribute: GeometrySourceVertexAttribute) -> bool {
        self.vertex_attributes.contains(&attribute)
    }
}

pub fn builtin_geometry_source_descriptors() -> Vec<GeometrySourceDescriptor> {
    vec![
        static_mesh_geometry_source_descriptor(),
        skinned_mesh_geometry_source_descriptor(),
        morphed_mesh_geometry_source_descriptor(),
        skinned_morphed_mesh_geometry_source_descriptor(),
    ]
}

pub fn builtin_geometry_source_descriptor(
    id: GeometrySourceId,
) -> Option<GeometrySourceDescriptor> {
    match id {
        GEOMETRY_SOURCE_ID_STATIC_MESH => Some(static_mesh_geometry_source_descriptor()),
        GEOMETRY_SOURCE_ID_SKINNED_MESH => Some(skinned_mesh_geometry_source_descriptor()),
        GEOMETRY_SOURCE_ID_MORPHED_MESH => Some(morphed_mesh_geometry_source_descriptor()),
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH => {
            Some(skinned_morphed_mesh_geometry_source_descriptor())
        }
        _ => None,
    }
}

fn static_mesh_geometry_source_descriptor() -> GeometrySourceDescriptor {
    geometry_source_descriptor(
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        "static_mesh",
        GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
        base_mesh_vertex_attributes(),
        vec![gpu_scene_instance_binding()],
        "ZR_GEOMETRY_SOURCE_STATIC_MESH",
    )
}

fn skinned_mesh_geometry_source_descriptor() -> GeometrySourceDescriptor {
    let mut attributes = base_mesh_vertex_attributes();
    attributes.extend([
        GeometrySourceVertexAttribute::JointIndices,
        GeometrySourceVertexAttribute::JointWeights,
    ]);
    geometry_source_descriptor(
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        "skinned_mesh",
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
        attributes,
        vec![gpu_scene_instance_binding(), skinning_palette_binding()],
        "ZR_GEOMETRY_SOURCE_SKINNED_MESH",
    )
}

fn morphed_mesh_geometry_source_descriptor() -> GeometrySourceDescriptor {
    let mut attributes = base_mesh_vertex_attributes();
    attributes.extend([
        GeometrySourceVertexAttribute::MorphPositionDelta,
        GeometrySourceVertexAttribute::MorphNormalDelta,
    ]);
    geometry_source_descriptor(
        GEOMETRY_SOURCE_ID_MORPHED_MESH,
        "morphed_mesh",
        GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
        attributes,
        vec![
            gpu_scene_instance_binding(),
            morph_weights_binding(),
            morph_target_binding(),
        ],
        "ZR_GEOMETRY_SOURCE_MORPHED_MESH",
    )
}

fn skinned_morphed_mesh_geometry_source_descriptor() -> GeometrySourceDescriptor {
    let mut attributes = base_mesh_vertex_attributes();
    attributes.extend([
        GeometrySourceVertexAttribute::JointIndices,
        GeometrySourceVertexAttribute::JointWeights,
        GeometrySourceVertexAttribute::MorphPositionDelta,
        GeometrySourceVertexAttribute::MorphNormalDelta,
    ]);
    geometry_source_descriptor(
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
        "skinned_morphed_mesh",
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
        attributes,
        vec![
            gpu_scene_instance_binding(),
            skinning_palette_binding(),
            morph_weights_binding(),
            morph_target_binding(),
        ],
        "ZR_GEOMETRY_SOURCE_SKINNED_MORPHED_MESH",
    )
}

fn geometry_source_descriptor(
    id: GeometrySourceId,
    token: &str,
    wgsl_include: &str,
    vertex_attributes: Vec<GeometrySourceVertexAttribute>,
    required_bindings: Vec<GeometrySourceBindingRequirement>,
    primary_define: &str,
) -> GeometrySourceDescriptor {
    GeometrySourceDescriptor {
        id,
        token: token.to_string(),
        wgsl_include: wgsl_include.to_string(),
        vertex_attributes,
        required_bindings,
        shader_defines: vec![
            RenderShaderDefinitionValue::uint("ZR_GEOMETRY_SOURCE_ID", u32::from(id.value())),
            RenderShaderDefinitionValue::bool(primary_define, true),
        ],
    }
}

fn base_mesh_vertex_attributes() -> Vec<GeometrySourceVertexAttribute> {
    vec![
        GeometrySourceVertexAttribute::Position,
        GeometrySourceVertexAttribute::Normal,
        GeometrySourceVertexAttribute::Tangent,
        GeometrySourceVertexAttribute::Uv0,
        GeometrySourceVertexAttribute::Color0,
    ]
}

fn gpu_scene_instance_binding() -> GeometrySourceBindingRequirement {
    GeometrySourceBindingRequirement::new(
        GeometrySourceBindingKind::GpuSceneInstance,
        "gpu_scene.instance_records",
    )
}

fn skinning_palette_binding() -> GeometrySourceBindingRequirement {
    GeometrySourceBindingRequirement::new(
        GeometrySourceBindingKind::SkinningPaletteStorage,
        "gpu_scene.skinning_palettes",
    )
}

fn morph_weights_binding() -> GeometrySourceBindingRequirement {
    GeometrySourceBindingRequirement::new(
        GeometrySourceBindingKind::MorphWeightsStorage,
        "gpu_scene.morph_weights",
    )
}

fn morph_target_binding() -> GeometrySourceBindingRequirement {
    GeometrySourceBindingRequirement::new(
        GeometrySourceBindingKind::MorphTargetStorage,
        "gpu_scene.morph_targets",
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
        GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        GEOMETRY_SOURCE_PLUGIN_ID_START, GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
        GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
        GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH, GeometrySourceBindingKind, GeometrySourceId,
        GeometrySourceVertexAttribute, builtin_geometry_source_descriptor,
        builtin_geometry_source_descriptors,
    };

    #[test]
    fn render_shader_geometry_source_ids_reserve_builtin_segment() {
        assert_eq!(GEOMETRY_SOURCE_ID_STATIC_MESH.value(), 0);
        assert_eq!(GEOMETRY_SOURCE_ID_SKINNED_MESH.value(), 1);
        assert_eq!(GEOMETRY_SOURCE_ID_MORPHED_MESH.value(), 2);
        assert_eq!(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH.value(), 3);
        assert!(!GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH.is_plugin_range());
        assert!(GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START).is_plugin_range());
    }

    #[test]
    fn render_shader_geometry_source_descriptors_cover_builtin_segment() {
        let descriptors = builtin_geometry_source_descriptors();
        let ids = descriptors
            .iter()
            .map(|descriptor| descriptor.id.value())
            .collect::<HashSet<_>>();
        let tokens = descriptors
            .iter()
            .map(|descriptor| descriptor.token.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(descriptors.len(), 4);
        assert_eq!(ids.len(), 4);
        assert_eq!(tokens.len(), 4);
        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| descriptor.wgsl_include.as_str())
                .collect::<Vec<_>>(),
            vec![
                GEOMETRY_SOURCE_WGSL_INCLUDE_STATIC_MESH,
                GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MESH,
                GEOMETRY_SOURCE_WGSL_INCLUDE_MORPHED_MESH,
                GEOMETRY_SOURCE_WGSL_INCLUDE_SKINNED_MORPHED_MESH,
            ]
        );
    }

    #[test]
    fn render_shader_geometry_source_descriptors_report_shape_requirements() {
        let static_mesh = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
            .expect("static mesh descriptor");
        let skinned = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_SKINNED_MESH)
            .expect("skinned mesh descriptor");
        let morphed = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_MORPHED_MESH)
            .expect("morphed mesh descriptor");
        let skinned_morphed =
            builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH)
                .expect("skinned morphed mesh descriptor");

        assert!(static_mesh.requires_binding(GeometrySourceBindingKind::GpuSceneInstance));
        assert!(!static_mesh.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));
        assert!(!static_mesh.requires_binding(GeometrySourceBindingKind::MorphWeightsStorage));

        assert!(skinned.has_vertex_attribute(GeometrySourceVertexAttribute::JointIndices));
        assert!(skinned.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));
        assert!(!skinned.requires_binding(GeometrySourceBindingKind::MorphTargetStorage));

        assert!(morphed.has_vertex_attribute(GeometrySourceVertexAttribute::MorphPositionDelta));
        assert!(morphed.requires_binding(GeometrySourceBindingKind::MorphWeightsStorage));
        assert!(morphed.requires_binding(GeometrySourceBindingKind::MorphTargetStorage));
        assert!(!morphed.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage));

        assert!(skinned_morphed.has_vertex_attribute(GeometrySourceVertexAttribute::JointWeights));
        assert!(
            skinned_morphed.has_vertex_attribute(GeometrySourceVertexAttribute::MorphNormalDelta)
        );
        assert!(
            skinned_morphed.requires_binding(GeometrySourceBindingKind::SkinningPaletteStorage)
        );
        assert!(skinned_morphed.requires_binding(GeometrySourceBindingKind::MorphTargetStorage));
    }
}

use std::sync::Arc;

use crate::core::math::{Real, Vec4};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle};

use super::super::super::{MaterialPropertyOverrideBlock, RenderMaterialAlphaMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderComponentMeshPrimitiveBinding {
    mesh: ResourceHandle<MeshMarker>,
    material: ResourceHandle<MaterialMarker>,
}

impl RenderComponentMeshPrimitiveBinding {
    pub(crate) const fn new(
        mesh: ResourceHandle<MeshMarker>,
        material: ResourceHandle<MaterialMarker>,
    ) -> Self {
        Self { mesh, material }
    }

    pub const fn mesh(self) -> ResourceHandle<MeshMarker> {
        self.mesh
    }

    pub const fn material(self) -> ResourceHandle<MaterialMarker> {
        self.material
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderComponentMeshLodLevel {
    min_distance: Real,
    model: ResourceHandle<ModelMarker>,
    mesh: Option<ResourceHandle<MeshMarker>>,
    material: ResourceHandle<MaterialMarker>,
    primitives: Arc<[RenderComponentMeshPrimitiveBinding]>,
}

impl RenderComponentMeshLodLevel {
    pub(crate) fn new(
        min_distance: Real,
        model: ResourceHandle<ModelMarker>,
        mesh: Option<ResourceHandle<MeshMarker>>,
        material: ResourceHandle<MaterialMarker>,
        primitives: Vec<RenderComponentMeshPrimitiveBinding>,
    ) -> Self {
        Self {
            min_distance,
            model,
            mesh,
            material,
            primitives: primitives.into(),
        }
    }

    pub const fn min_distance(&self) -> Real {
        self.min_distance
    }

    pub const fn model(&self) -> ResourceHandle<ModelMarker> {
        self.model
    }

    pub const fn mesh(&self) -> Option<ResourceHandle<MeshMarker>> {
        self.mesh
    }

    pub const fn material(&self) -> ResourceHandle<MaterialMarker> {
        self.material
    }

    pub fn primitives(&self) -> &[RenderComponentMeshPrimitiveBinding] {
        &self.primitives
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderComponentMeshPayload {
    model: ResourceHandle<ModelMarker>,
    mesh: Option<ResourceHandle<MeshMarker>>,
    material: ResourceHandle<MaterialMarker>,
    render_queue: i32,
    material_queue: i32,
    order_in_layer: i32,
    depth_bias: Real,
    morph_weights: Arc<[Real]>,
    primitives: Arc<[RenderComponentMeshPrimitiveBinding]>,
    lods: Arc<[RenderComponentMeshLodLevel]>,
    material_property_overrides: MaterialPropertyOverrideBlock,
    tint: Vec4,
    material_alpha_mode: RenderMaterialAlphaMode,
}

impl RenderComponentMeshPayload {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        model: ResourceHandle<ModelMarker>,
        mesh: Option<ResourceHandle<MeshMarker>>,
        material: ResourceHandle<MaterialMarker>,
        render_queue: i32,
        material_queue: i32,
        order_in_layer: i32,
        depth_bias: Real,
        morph_weights: Vec<Real>,
        primitives: Vec<RenderComponentMeshPrimitiveBinding>,
        lods: Vec<RenderComponentMeshLodLevel>,
        material_property_overrides: MaterialPropertyOverrideBlock,
        tint: Vec4,
        material_alpha_mode: RenderMaterialAlphaMode,
    ) -> Self {
        Self {
            model,
            mesh,
            material,
            render_queue,
            material_queue,
            order_in_layer,
            depth_bias,
            morph_weights: morph_weights.into(),
            primitives: primitives.into(),
            lods: lods.into(),
            material_property_overrides,
            tint,
            material_alpha_mode,
        }
    }

    pub const fn model(&self) -> ResourceHandle<ModelMarker> {
        self.model
    }

    pub const fn mesh(&self) -> Option<ResourceHandle<MeshMarker>> {
        self.mesh
    }

    pub const fn material(&self) -> ResourceHandle<MaterialMarker> {
        self.material
    }

    pub const fn render_queue(&self) -> i32 {
        self.render_queue
    }

    pub const fn material_queue(&self) -> i32 {
        self.material_queue
    }

    pub const fn order_in_layer(&self) -> i32 {
        self.order_in_layer
    }

    pub const fn depth_bias(&self) -> Real {
        self.depth_bias
    }

    pub fn morph_weights(&self) -> &[Real] {
        &self.morph_weights
    }

    pub fn primitives(&self) -> &[RenderComponentMeshPrimitiveBinding] {
        &self.primitives
    }

    pub fn lods(&self) -> &[RenderComponentMeshLodLevel] {
        &self.lods
    }

    pub const fn material_property_overrides(&self) -> &MaterialPropertyOverrideBlock {
        &self.material_property_overrides
    }

    pub const fn tint(&self) -> Vec4 {
        self.tint
    }

    pub const fn material_alpha_mode(&self) -> RenderMaterialAlphaMode {
        self.material_alpha_mode
    }
}

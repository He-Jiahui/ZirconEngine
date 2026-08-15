use crate::core::math::{Mat4, Real, Transform, UVec2, Vec4};
use crate::core::resource::{ResourceHandle, TextureMarker};
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

use super::camera_stack::CameraRenderDescriptor;
use super::view_family::MAX_RENDER_RESOLUTION_FRACTION;
use super::{
    CorePipelineKind, RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
    RenderVirtualGeometryDebugState, TemporalJitterSample,
};

pub type RenderLayer = u32;

pub const DEFAULT_RENDER_LAYER: RenderLayer = 0;
pub const DEFAULT_RENDER_LAYER_MASK: u32 = 0x0000_0001;
pub const DEFAULT_CAMERA_EXPOSURE_EV100: Real = 9.7;
pub const DEFAULT_CAMERA_MSAA_SAMPLES: u32 = 1;
pub const DEFAULT_DYNAMIC_RESOLUTION_SCALE: Real = 1.0;
pub const MIN_DYNAMIC_RESOLUTION_SCALE: Real = 0.1;
pub const MAX_DYNAMIC_RESOLUTION_SCALE: Real = 1.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportCameraSnapshot {
    pub transform: Transform,
    /// Selects the render schedule independently from the projection matrix.
    #[serde(default)]
    pub core_pipeline: CorePipelineKind,
    pub projection_mode: ProjectionMode,
    pub fov_y_radians: Real,
    pub ortho_size: Real,
    pub z_near: Real,
    pub z_far: Real,
    pub aspect_ratio: Real,
    #[serde(default)]
    pub projection_override: Option<Mat4>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    pub exposure_ev100: Real,
    #[serde(default = "default_camera_msaa_samples")]
    pub msaa_samples: u32,
    #[serde(default)]
    pub dynamic_resolution: RenderDynamicResolutionSettings,
    #[serde(default)]
    pub temporal_jitter: TemporalJitterSample,
}

impl ViewportCameraSnapshot {
    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.aspect_ratio =
            aspect_ratio_from_viewport_size(self.effective_viewport_size(viewport_size));
    }

    pub fn core_pipeline_kind(&self) -> CorePipelineKind {
        self.core_pipeline
    }

    pub fn effective_viewport_size(&self, target_size: UVec2) -> UVec2 {
        target_size
    }

    pub fn effective_render_size(&self, target_size: UVec2) -> UVec2 {
        self.render_view_family_pipeline(target_size, RenderUpscalerKind::Spatial)
            .resolution()
            .primary_extent()
    }

    /// Converts the legacy camera scale into the primary fraction of a view-family plan.
    ///
    /// The upscaler category remains a render-graph decision: a camera scale alone must not
    /// silently turn TAA, a vendor SDK, or temporal history on.
    pub fn render_view_family_pipeline(
        &self,
        target_size: UVec2,
        upscaler: RenderUpscalerKind,
    ) -> RenderViewFamilyPipeline {
        RenderViewFamilyPipeline::resolve(
            self.effective_viewport_size(target_size),
            RenderResolutionPolicy::with_scales(
                self.dynamic_resolution.clamped_scale(),
                MAX_RENDER_RESOLUTION_FRACTION,
            ),
            upscaler,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderDynamicResolutionSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_dynamic_resolution_scale")]
    pub scale: Real,
}

impl Default for RenderDynamicResolutionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            scale: DEFAULT_DYNAMIC_RESOLUTION_SCALE,
        }
    }
}

impl RenderDynamicResolutionSettings {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            scale: DEFAULT_DYNAMIC_RESOLUTION_SCALE,
        }
    }

    pub fn fixed_scale(scale: Real) -> Self {
        Self {
            enabled: true,
            scale,
        }
    }

    pub fn clamped_scale(self) -> Real {
        if !self.enabled || !self.scale.is_finite() {
            return DEFAULT_DYNAMIC_RESOLUTION_SCALE;
        }
        self.scale
            .clamp(MIN_DYNAMIC_RESOLUTION_SCALE, MAX_DYNAMIC_RESOLUTION_SCALE)
    }

    pub fn apply_to_size(self, viewport_size: UVec2) -> UVec2 {
        let scale = self.clamped_scale();
        let width = ((viewport_size.x.max(1) as Real) * scale).round().max(1.0) as u32;
        let height = ((viewport_size.y.max(1) as Real) * scale).round().max(1.0) as u32;
        UVec2::new(width, height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

impl Default for ProjectionMode {
    fn default() -> Self {
        Self::Perspective
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderCameraTarget {
    PrimarySurface,
    Texture(ResourceHandle<TextureMarker>),
    Headless { size: UVec2 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCameraTargetKind {
    #[default]
    PrimarySurface,
    Texture,
    Headless,
}

impl RenderCameraTargetKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::PrimarySurface => "primary_surface",
            Self::Texture => "texture",
            Self::Headless => "headless",
        }
    }
}

impl Default for RenderCameraTarget {
    fn default() -> Self {
        Self::PrimarySurface
    }
}

impl RenderCameraTarget {
    pub fn kind(&self) -> RenderCameraTargetKind {
        match self {
            Self::PrimarySurface => RenderCameraTargetKind::PrimarySurface,
            Self::Texture(_) => RenderCameraTargetKind::Texture,
            Self::Headless { .. } => RenderCameraTargetKind::Headless,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderCameraClearColor {
    Default,
    None,
    Color(Vec4),
}

impl Default for RenderCameraClearColor {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderViewportRect {
    pub physical_position: UVec2,
    pub physical_size: UVec2,
    pub depth_min: Real,
    pub depth_max: Real,
}

impl Default for RenderViewportRect {
    fn default() -> Self {
        Self {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(1, 1),
            depth_min: 0.0,
            depth_max: 1.0,
        }
    }
}

impl RenderViewportRect {
    pub fn new(physical_position: UVec2, physical_size: UVec2) -> Self {
        Self {
            physical_position,
            physical_size,
            ..Self::default()
        }
    }

    pub fn clamped_to_size(mut self, target_size: UVec2) -> Self {
        self.physical_position.x =
            clamp_viewport_axis_position(self.physical_position.x, target_size.x);
        self.physical_position.y =
            clamp_viewport_axis_position(self.physical_position.y, target_size.y);
        self.physical_size.x = self
            .physical_size
            .x
            .min(target_size.x.saturating_sub(self.physical_position.x));
        self.physical_size.y = self
            .physical_size
            .y
            .min(target_size.y.saturating_sub(self.physical_position.y));
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RenderLayerSet {
    #[serde(default)]
    blocks: Vec<u64>,
}

impl Default for RenderLayerSet {
    fn default() -> Self {
        Self::layer(DEFAULT_RENDER_LAYER)
    }
}

impl RenderLayerSet {
    pub fn layer(layer: RenderLayer) -> Self {
        Self::none().with(layer)
    }

    pub fn none() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn from_layers(layers: impl IntoIterator<Item = RenderLayer>) -> Self {
        layers
            .into_iter()
            .fold(Self::none(), |layers, layer| layers.with(layer))
    }

    pub fn from_scene_schema_v1_mask(mask: u32) -> Self {
        if mask == 0 {
            Self::none()
        } else {
            Self {
                blocks: vec![u64::from(mask)],
            }
        }
    }

    pub fn to_scene_schema_v1_mask_lossy(&self) -> u32 {
        self.blocks.first().copied().unwrap_or_default() as u32
    }

    pub fn with(mut self, layer: RenderLayer) -> Self {
        let block_index = layer_block_index(layer);
        if self.blocks.len() <= block_index {
            self.blocks.resize(block_index + 1, 0);
        }
        self.blocks[block_index] |= layer_bit(layer);
        self
    }

    pub fn without(mut self, layer: RenderLayer) -> Self {
        let block_index = layer_block_index(layer);
        if let Some(block) = self.blocks.get_mut(block_index) {
            *block &= !layer_bit(layer);
        }
        self.shrink()
    }

    pub fn contains(&self, layer: RenderLayer) -> bool {
        self.blocks
            .get(layer_block_index(layer))
            .is_some_and(|block| (*block & layer_bit(layer)) != 0)
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|block| *block == 0)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.blocks
            .iter()
            .zip(other.blocks.iter())
            .any(|(left, right)| (*left & *right) != 0)
    }

    pub fn union(&self, other: &Self) -> Self {
        let block_count = self.blocks.len().max(other.blocks.len());
        let blocks = (0..block_count)
            .map(|index| {
                self.blocks.get(index).copied().unwrap_or_default()
                    | other.blocks.get(index).copied().unwrap_or_default()
            })
            .collect::<Vec<_>>();
        Self { blocks }.shrink()
    }

    pub fn intersects_scene_schema_v1_mask(&self, mask: u32) -> bool {
        (self.blocks.first().copied().unwrap_or_default() & u64::from(mask)) != 0
    }

    pub fn iter(&self) -> impl Iterator<Item = RenderLayer> + '_ {
        self.blocks
            .iter()
            .enumerate()
            .flat_map(|(block_index, block)| {
                let mut block = *block;
                std::iter::from_fn(move || {
                    if block == 0 {
                        return None;
                    }
                    let bit = block.trailing_zeros();
                    block &= !(1u64 << bit);
                    Some((block_index as RenderLayer) * u64::BITS + bit)
                })
            })
    }

    fn shrink(mut self) -> Self {
        while self.blocks.last().is_some_and(|block| *block == 0) {
            self.blocks.pop();
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Shaded,
    WireOverlay,
    WireOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackSkyboxKind {
    None,
    ProceduralGradient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewportRenderSettings {
    pub projection_mode: ProjectionMode,
    pub display_mode: DisplayMode,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
}

impl Default for ViewportRenderSettings {
    fn default() -> Self {
        Self {
            projection_mode: ProjectionMode::Perspective,
            display_mode: DisplayMode::Shaded,
            preview_lighting: true,
            preview_skybox: true,
        }
    }
}

impl Default for ViewportCameraSnapshot {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            core_pipeline: CorePipelineKind::Core3d,
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: 60.0_f32.to_radians(),
            ortho_size: 5.0,
            z_near: 0.1,
            z_far: 200.0,
            aspect_ratio: default_viewport_aspect_ratio(),
            projection_override: None,
            is_active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
            dynamic_resolution: RenderDynamicResolutionSettings::default(),
            temporal_jitter: TemporalJitterSample::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportExtractRequest {
    pub settings: ViewportRenderSettings,
    pub active_camera_override: Option<EntityId>,
    pub camera: Option<CameraRenderDescriptor>,
    pub viewport_size: Option<UVec2>,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}

impl Default for SceneViewportExtractRequest {
    fn default() -> Self {
        Self {
            settings: ViewportRenderSettings::default(),
            active_camera_override: None,
            camera: None,
            viewport_size: None,
            virtual_geometry_debug: None,
        }
    }
}

pub const fn default_viewport_aspect_ratio() -> Real {
    16.0 / 9.0
}

pub fn aspect_ratio_from_viewport_size(viewport_size: UVec2) -> Real {
    viewport_size.x.max(1) as Real / viewport_size.y.max(1) as Real
}

const fn default_true() -> bool {
    true
}

const fn default_camera_exposure_ev100() -> Real {
    DEFAULT_CAMERA_EXPOSURE_EV100
}

const fn default_camera_msaa_samples() -> u32 {
    DEFAULT_CAMERA_MSAA_SAMPLES
}

const fn default_dynamic_resolution_scale() -> Real {
    DEFAULT_DYNAMIC_RESOLUTION_SCALE
}

fn layer_block_index(layer: RenderLayer) -> usize {
    (layer / u64::BITS) as usize
}

fn layer_bit(layer: RenderLayer) -> u64 {
    1u64 << (layer % u64::BITS)
}

fn clamp_viewport_axis_position(position: u32, target: u32) -> u32 {
    if target == 0 {
        0
    } else {
        position.min(target - 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::UVec2;

    use super::{RenderLayerSet, RenderUpscalerKind, ViewportCameraSnapshot};

    #[test]
    fn default_camera_preserves_the_view_family_full_resolution_contract() {
        let pipeline = ViewportCameraSnapshot::default()
            .render_view_family_pipeline(UVec2::new(1920, 1080), RenderUpscalerKind::Spatial);

        assert_eq!(
            pipeline.resolution().primary_extent(),
            UVec2::new(1920, 1080)
        );
        assert_eq!(
            pipeline.resolution().secondary_extent(),
            UVec2::new(1920, 1080)
        );
    }

    #[test]
    fn camera_dynamic_resolution_adapts_into_the_view_family_resolution_policy() {
        let mut camera = ViewportCameraSnapshot::default();
        camera.dynamic_resolution = super::RenderDynamicResolutionSettings::fixed_scale(2.0 / 3.0);

        let pipeline = camera
            .render_view_family_pipeline(UVec2::new(1920, 1080), RenderUpscalerKind::Temporal);

        assert_eq!(
            pipeline.resolution().display_extent(),
            UVec2::new(1920, 1080)
        );
        assert_eq!(
            pipeline.resolution().primary_extent(),
            UVec2::new(1280, 720)
        );
        assert_eq!(
            pipeline.resolution().temporal_history_extent(),
            Some(UVec2::new(1920, 1080))
        );
    }

    #[test]
    fn render_layer_schema_v1_uses_single_block_fast_paths() {
        let source = include_str!("camera.rs");
        assert!(!source.contains(concat!("for layer in 0..", "u32::BITS")));
        assert!(!source.contains(concat!(
            "self.intersects(&Self::from_scene_",
            "schema_v1_mask(mask))"
        )));

        let layers = RenderLayerSet::from_scene_schema_v1_mask(0b1010);
        assert_eq!(layers.iter().collect::<Vec<_>>(), vec![1, 3]);
        assert_eq!(layers.to_scene_schema_v1_mask_lossy(), 0b1010);
        assert!(layers.intersects_scene_schema_v1_mask(0b1000));
        assert!(!layers.intersects_scene_schema_v1_mask(0b0100));
        assert!(RenderLayerSet::from_scene_schema_v1_mask(0).is_empty());

        let wide = RenderLayerSet::layer(70).with(3);
        assert!(wide.intersects_scene_schema_v1_mask(0b1000));
        assert!(!wide.intersects_scene_schema_v1_mask(0b0100));
    }
}

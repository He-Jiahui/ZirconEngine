use crate::core::math::{Real, Transform, UVec2, Vec4};
use crate::core::resource::{ResourceHandle, TextureMarker};
use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

use super::{CorePipelineKind, RenderVirtualGeometryDebugState};

pub type RenderLayer = u32;

pub const DEFAULT_RENDER_LAYER: RenderLayer = 0;
pub const DEFAULT_RENDER_LAYER_MASK: u32 = 0x0000_0001;
pub const DEFAULT_CAMERA_EXPOSURE_EV100: Real = 9.7;
pub const DEFAULT_CAMERA_MSAA_SAMPLES: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewportCameraSnapshot {
    pub transform: Transform,
    pub projection_mode: ProjectionMode,
    pub fov_y_radians: Real,
    pub ortho_size: Real,
    pub z_near: Real,
    pub z_far: Real,
    pub aspect_ratio: Real,
    #[serde(default)]
    pub target: RenderCameraTarget,
    #[serde(default)]
    pub viewport: Option<RenderViewportRect>,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub hdr: bool,
    #[serde(default = "default_camera_exposure_ev100")]
    pub exposure_ev100: Real,
    #[serde(default)]
    pub clear_color: RenderCameraClearColor,
    #[serde(default = "default_camera_msaa_samples")]
    pub msaa_samples: u32,
    #[serde(default)]
    pub render_layers: RenderLayerSet,
}

impl ViewportCameraSnapshot {
    pub fn apply_viewport_size(&mut self, viewport_size: UVec2) {
        self.aspect_ratio =
            aspect_ratio_from_viewport_size(self.effective_viewport_size(viewport_size));
    }

    pub fn core_pipeline_kind(&self) -> CorePipelineKind {
        self.projection_mode.core_pipeline_kind()
    }

    pub fn effective_viewport_size(&self, target_size: UVec2) -> UVec2 {
        self.viewport
            .as_ref()
            .map(|viewport| viewport.clamped_to_size(target_size).physical_size)
            .unwrap_or(target_size)
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

impl ProjectionMode {
    pub const fn core_pipeline_kind(self) -> CorePipelineKind {
        match self {
            Self::Orthographic => CorePipelineKind::Core2d,
            Self::Perspective => CorePipelineKind::Core3d,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderCameraTarget {
    PrimarySurface,
    Texture(ResourceHandle<TextureMarker>),
    Headless { size: UVec2 },
}

impl Default for RenderCameraTarget {
    fn default() -> Self {
        Self::PrimarySurface
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn from_legacy_mask(mask: u32) -> Self {
        let mut layers = Self::none();
        for layer in 0..u32::BITS {
            if (mask & (1u32 << layer)) != 0 {
                layers = layers.with(layer);
            }
        }
        layers
    }

    pub fn to_legacy_mask_lossy(&self) -> u32 {
        self.iter()
            .filter(|layer| *layer < u32::BITS)
            .fold(0u32, |mask, layer| mask | (1u32 << layer))
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

    pub fn intersects_legacy_mask(&self, mask: u32) -> bool {
        self.intersects(&Self::from_legacy_mask(mask))
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
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: 60.0_f32.to_radians(),
            ortho_size: 5.0,
            z_near: 0.1,
            z_far: 200.0,
            aspect_ratio: default_viewport_aspect_ratio(),
            target: RenderCameraTarget::default(),
            viewport: None,
            order: 0,
            is_active: true,
            hdr: false,
            exposure_ev100: DEFAULT_CAMERA_EXPOSURE_EV100,
            clear_color: RenderCameraClearColor::default(),
            msaa_samples: DEFAULT_CAMERA_MSAA_SAMPLES,
            render_layers: RenderLayerSet::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportExtractRequest {
    pub settings: ViewportRenderSettings,
    pub active_camera_override: Option<EntityId>,
    pub camera: Option<ViewportCameraSnapshot>,
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

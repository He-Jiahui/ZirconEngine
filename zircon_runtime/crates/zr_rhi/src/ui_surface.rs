use std::collections::BTreeSet;

mod compact_styles;
mod image_resources;

use compact_styles::{compact_commands, resolved_kind};
pub use compact_styles::{
    UiSurfaceResolvedCommandKind, UiSurfaceStyle, UiSurfaceStyleHandle, UiSurfaceStyledPayload,
};
use image_resources::compact_image_resources;
pub use image_resources::{UiSurfaceImageResource, UiSurfaceImageResourceTable};

use crate::RenderNativeSurfaceTarget;

use super::RhiError;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UiSurfaceRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiSurfaceRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[doc(hidden)]
    pub fn has_finite_positive_area(self) -> bool {
        let right = self.x + self.width;
        let bottom = self.y + self.height;
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && right.is_finite()
            && bottom.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiSurfaceTextStyle {
    Regular,
    Strong,
    Emphasis,
    StrongEmphasis,
}

impl Default for UiSurfaceTextStyle {
    fn default() -> Self {
        Self::Regular
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceImagePayload {
    pub resource_key: String,
    /// Revision of the resource addressed by `resource_key`.
    pub resource_generation: u64,
    pub width: u32,
    pub height: u32,
    pub upload_bytes: u64,
    /// Straight-alpha RGBA8 bytes. Backends must convert once before filtered premultiplied-alpha
    /// composition; producers must not pre-premultiply this payload.
    pub rgba: Option<Vec<u8>>,
    pub atlas_uv: Option<UiSurfaceImageUvRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSurfaceImageUvRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl UiSurfaceImageUvRect {
    pub fn is_valid(&self) -> bool {
        self.min[0].is_finite()
            && self.min[1].is_finite()
            && self.max[0].is_finite()
            && self.max[1].is_finite()
            && self.min[0] >= 0.0
            && self.min[1] >= 0.0
            && self.max[0] <= 1.0
            && self.max[1] <= 1.0
            && self.min[0] < self.max[0]
            && self.min[1] < self.max[1]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiSurfaceCommandKind {
    Quad {
        color: [u8; 4],
        corner_radius: f32,
    },
    Border {
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    },
    Text {
        text: String,
        color: [u8; 4],
        font_family: Option<String>,
        font_weight: u16,
        font_size: f32,
        line_height: f32,
        style: UiSurfaceTextStyle,
    },
    Image {
        payload: UiSurfaceImagePayload,
    },
    Styled {
        style: UiSurfaceStyleHandle,
        payload: UiSurfaceStyledPayload,
    },
    Clip,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceCommand {
    pub z_index: i32,
    pub frame: UiSurfaceRect,
    pub clip: Option<UiSurfaceRect>,
    pub kind: UiSurfaceCommandKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiSurfaceDescriptor {
    pub label: Option<&'static str>,
    pub width: u32,
    pub height: u32,
    pub target: Option<RenderNativeSurfaceTarget>,
    /// Creates timestamp-query resources only for an explicit profiling surface.
    pub allow_gpu_timing: bool,
}

impl UiSurfaceDescriptor {
    pub const fn headless(label: &'static str, width: u32, height: u32) -> Self {
        Self {
            label: Some(label),
            width,
            height,
            target: None,
            allow_gpu_timing: false,
        }
    }

    pub const fn native(
        label: &'static str,
        width: u32,
        height: u32,
        target: RenderNativeSurfaceTarget,
    ) -> Self {
        Self {
            label: Some(label),
            width,
            height,
            target: Some(target),
            allow_gpu_timing: false,
        }
    }

    pub const fn with_gpu_timing(mut self) -> Self {
        self.allow_gpu_timing = true;
        self
    }

    pub fn validate(&self) -> Result<(), RhiError> {
        if self.width == 0 || self.height == 0 {
            return Err(RhiError::InvalidSurfaceDescriptor {
                label: self.label.map(str::to_string),
                reason: "width and height must be greater than zero".to_string(),
            });
        }
        Ok(())
    }

    pub fn clamped_size(&self) -> (u32, u32) {
        (self.width.max(1), self.height.max(1))
    }

    #[cfg(feature = "platform-winit")]
    pub fn from_winit_window(
        label: &'static str,
        window: &dyn winit::window::Window,
    ) -> Result<Self, RhiError> {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

        let size = window.surface_size();
        let raw = window
            .window_handle()
            .map_err(|error| RhiError::SurfaceUnavailable(error.to_string()))?
            .as_raw();
        match raw {
            RawWindowHandle::Win32(handle) => Ok(Self::native(
                label,
                size.width.max(1),
                size.height.max(1),
                RenderNativeSurfaceTarget::Win32 {
                    hwnd: handle.hwnd.get() as u64,
                    hinstance: handle.hinstance.map(|hinstance| hinstance.get() as u64),
                },
            )),
            other => Err(RhiError::SurfaceUnavailable(format!(
                "unsupported native window handle for retained UI surface: {other:?}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceDrawList {
    pub surface_size: (u32, u32),
    projection_size: (u32, u32),
    target_only_resize: bool,
    pub damage: Option<UiSurfaceRect>,
    pub commands: Vec<UiSurfaceCommand>,
    generation: Option<u64>,
    styles: Vec<UiSurfaceStyle>,
    image_resources: UiSurfaceImageResourceTable,
}

impl UiSurfaceDrawList {
    pub fn new(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
    ) -> Self {
        let surface_size = (surface_size.0.max(1), surface_size.1.max(1));
        Self {
            surface_size,
            projection_size: surface_size,
            target_only_resize: false,
            damage,
            commands,
            generation: None,
            styles: Vec::new(),
            image_resources: UiSurfaceImageResourceTable::default(),
        }
    }

    /// Constructs a list whose command payload is identified by a producer-owned revision.
    ///
    /// The revision must advance whenever the commands or their payloads change. A list made
    /// with [`Self::new`] deliberately has no revision and is never eligible for a compiled
    /// WGPU batch-plan cache, which preserves correctness for callers without a producer revision.
    pub fn with_generation(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        generation: u64,
    ) -> Self {
        let surface_size = (surface_size.0.max(1), surface_size.1.max(1));
        Self {
            surface_size,
            projection_size: surface_size,
            target_only_resize: false,
            damage,
            commands,
            generation: Some(generation),
            styles: Vec::new(),
            image_resources: UiSurfaceImageResourceTable::default(),
        }
    }

    pub fn with_compact_styles(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
    ) -> Self {
        Self::compact(surface_size, damage, commands, None)
    }

    pub fn with_generation_and_compact_styles(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        generation: u64,
    ) -> Self {
        Self::compact(surface_size, damage, commands, Some(generation))
    }

    pub fn with_compact_styles_and_image_resources(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        image_resources: UiSurfaceImageResourceTable,
    ) -> Self {
        Self::compact_with_image_resources(surface_size, damage, commands, None, image_resources)
    }

    pub fn with_generation_and_compact_styles_and_image_resources(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        generation: u64,
        image_resources: UiSurfaceImageResourceTable,
    ) -> Self {
        Self::compact_with_image_resources(
            surface_size,
            damage,
            commands,
            Some(generation),
            image_resources,
        )
    }

    fn compact(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        generation: Option<u64>,
    ) -> Self {
        Self::compact_with_image_resources(
            surface_size,
            damage,
            commands,
            generation,
            UiSurfaceImageResourceTable::default(),
        )
    }

    fn compact_with_image_resources(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        generation: Option<u64>,
        mut image_resources: UiSurfaceImageResourceTable,
    ) -> Self {
        let (commands, compacted_image_resources) = compact_image_resources(commands);
        image_resources.extend(compacted_image_resources);
        let (commands, styles) = compact_commands(commands);
        let surface_size = (surface_size.0.max(1), surface_size.1.max(1));
        Self {
            surface_size,
            projection_size: surface_size,
            target_only_resize: false,
            damage,
            commands,
            generation,
            styles,
            image_resources,
        }
    }

    pub fn style_count(&self) -> usize {
        self.styles.len()
    }

    pub fn image_resource(
        &self,
        resource_key: &str,
        generation: u64,
    ) -> Option<&UiSurfaceImageResource> {
        self.image_resources.get(resource_key, generation)
    }

    #[doc(hidden)]
    pub fn take_image_resources(&mut self) -> UiSurfaceImageResourceTable {
        std::mem::take(&mut self.image_resources)
    }

    pub fn resolved_kind<'a>(
        &'a self,
        command: &'a UiSurfaceCommand,
    ) -> Option<UiSurfaceResolvedCommandKind<'a>> {
        resolved_kind(self, command)
    }

    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }

    /// Coordinate extent used to compile immutable geometry for this producer generation.
    pub const fn projection_size(&self) -> (u32, u32) {
        self.projection_size
    }

    /// Changes the current render target without changing the generation's coordinate space.
    /// Native resize transactions use this to retain batch, vertex, and text projections.
    #[doc(hidden)]
    pub fn retarget_surface_size_preserving_projection(&mut self, surface_size: (u32, u32)) {
        self.surface_size = (surface_size.0.max(1), surface_size.1.max(1));
        self.target_only_resize = true;
    }

    #[doc(hidden)]
    pub const fn is_target_only_resize(&self) -> bool {
        self.target_only_resize
    }

    pub fn stats(&self) -> UiSurfacePresentStats {
        let mut stats = UiSurfacePresentStatsAccumulator::new(self);
        for command in &self.commands {
            if !self.command_visible_with_damage(command, self.damage) {
                continue;
            }
            stats.record_visible(command, self);
        }
        stats.finish()
    }

    #[doc(hidden)]
    pub fn command_visible_with_damage(
        &self,
        command: &UiSurfaceCommand,
        damage: Option<UiSurfaceRect>,
    ) -> bool {
        command_effective_rect(command, self, damage).is_some()
    }
}

fn command_effective_rect(
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
) -> Option<UiSurfaceRect> {
    let surface = UiSurfaceRect::new(
        0.0,
        0.0,
        draw_list.projection_size.0 as f32,
        draw_list.projection_size.1 as f32,
    );
    let mut rect = rect_intersection(command.frame, surface)?;
    if let Some(clip) = command.clip {
        rect = rect_intersection(rect, clip)?;
    }
    if let Some(damage) = damage {
        rect = rect_intersection(rect, damage)?;
    }
    Some(rect)
}

/// Aggregates one ordered command projection without requiring a second stats-only walk.
#[doc(hidden)]
pub struct UiSurfacePresentStatsAccumulator<'a> {
    uploaded_image_versions: BTreeSet<(&'a str, u64)>,
    recorded_style_handles: BTreeSet<UiSurfaceStyleHandle>,
    stats: UiSurfacePresentStats,
}

impl<'a> UiSurfacePresentStatsAccumulator<'a> {
    pub fn new(draw_list: &UiSurfaceDrawList) -> Self {
        Self {
            uploaded_image_versions: BTreeSet::new(),
            recorded_style_handles: BTreeSet::new(),
            stats: UiSurfacePresentStats {
                surface_size: draw_list.surface_size,
                command_visibility_scan_count: 1,
                ..UiSurfacePresentStats::default()
            },
        }
    }

    pub fn record_visible(
        &mut self,
        command: &'a UiSurfaceCommand,
        draw_list: &'a UiSurfaceDrawList,
    ) {
        let payload_bytes = self.command_dynamic_payload_bytes(command, draw_list);
        self.stats.visible_command_payload_bytes = self
            .stats
            .visible_command_payload_bytes
            .saturating_add(payload_bytes);
        let Some(kind) = draw_list.resolved_kind(command) else {
            return;
        };
        match kind {
            UiSurfaceResolvedCommandKind::Quad { .. }
            | UiSurfaceResolvedCommandKind::Border { .. }
            | UiSurfaceResolvedCommandKind::Text { .. } => {
                self.stats.visible_command_count =
                    self.stats.visible_command_count.saturating_add(1);
                self.stats.visible_draw_item_count =
                    self.stats.visible_draw_item_count.saturating_add(1);
                self.stats.draw_calls = self.stats.draw_calls.saturating_add(1);
            }
            UiSurfaceResolvedCommandKind::Image { payload } => {
                self.stats.visible_command_count =
                    self.stats.visible_command_count.saturating_add(1);
                self.stats.visible_draw_item_count =
                    self.stats.visible_draw_item_count.saturating_add(1);
                self.stats.draw_calls = self.stats.draw_calls.saturating_add(1);
                self.stats.image_count = self.stats.image_count.saturating_add(1);
                let upload_bytes = draw_list
                    .image_resource(&payload.resource_key, payload.resource_generation)
                    .map(|resource| resource.upload_bytes)
                    .or_else(|| payload.rgba.as_ref().map(|_| payload.upload_bytes));
                if let Some(upload_bytes) = upload_bytes.filter(|_| {
                    self.uploaded_image_versions
                        .insert((payload.resource_key.as_str(), payload.resource_generation))
                }) {
                    self.stats.image_upload_bytes =
                        self.stats.image_upload_bytes.saturating_add(upload_bytes);
                }
            }
            UiSurfaceResolvedCommandKind::Clip => {
                self.stats.clip_count = self.stats.clip_count.saturating_add(1);
            }
        }
    }

    fn command_dynamic_payload_bytes(
        &mut self,
        command: &UiSurfaceCommand,
        draw_list: &UiSurfaceDrawList,
    ) -> u64 {
        match &command.kind {
            UiSurfaceCommandKind::Styled { style, payload } => {
                let payload_bytes = match payload {
                    UiSurfaceStyledPayload::None => 0,
                    UiSurfaceStyledPayload::Text(text) => text.len() as u64,
                };
                let style_bytes = if self.recorded_style_handles.insert(*style) {
                    draw_list
                        .styles
                        .get(style.index())
                        .and_then(|style| match style {
                            UiSurfaceStyle::Text { font_family, .. } => {
                                Some(font_family.as_ref().map_or(0, String::len) as u64)
                            }
                            UiSurfaceStyle::Quad { .. } | UiSurfaceStyle::Border { .. } => None,
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                payload_bytes.saturating_add(style_bytes)
            }
            kind => command_dynamic_payload_bytes(kind),
        }
    }

    pub fn finish(mut self) -> UiSurfacePresentStats {
        self.stats.visible_command_style_count = self.recorded_style_handles.len() as u64;
        self.stats
    }
}

fn rect_intersection(left: UiSurfaceRect, right: UiSurfaceRect) -> Option<UiSurfaceRect> {
    if !left.has_finite_positive_area() || !right.has_finite_positive_area() {
        return None;
    }
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    (x1 > x0 && y1 > y0).then(|| UiSurfaceRect::new(x0, y0, x1 - x0, y1 - y0))
}

fn command_dynamic_payload_bytes(kind: &UiSurfaceCommandKind) -> u64 {
    match kind {
        UiSurfaceCommandKind::Text {
            text, font_family, ..
        } => text
            .len()
            .saturating_add(font_family.as_ref().map_or(0, String::len)) as u64,
        UiSurfaceCommandKind::Image { payload } => payload
            .resource_key
            .len()
            .saturating_add(payload.rgba.as_ref().map_or(0, Vec::len))
            as u64,
        UiSurfaceCommandKind::Quad { .. }
        | UiSurfaceCommandKind::Border { .. }
        | UiSurfaceCommandKind::Styled { .. }
        | UiSurfaceCommandKind::Clip => 0,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum UiSurfacePresentOutcome {
    #[default]
    Submitted,
    RetryableNoSubmit,
}

impl UiSurfacePresentOutcome {
    pub const fn is_submitted(self) -> bool {
        matches!(self, Self::Submitted)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct UiSurfacePresentStats {
    pub outcome: UiSurfacePresentOutcome,
    pub surface_size: (u32, u32),
    pub draw_calls: u64,
    /// Draw ops in the full compiled plan before native damage-scissor culling.
    pub compiled_draw_calls: u64,
    /// Native WGPU render passes actually begun for the current present.
    pub render_pass_count: u64,
    pub visible_command_count: u64,
    /// Dynamic string and RGBA bytes referenced by visible commands.
    pub visible_command_payload_bytes: u64,
    /// Distinct interned command styles referenced by visible commands.
    pub visible_command_style_count: u64,
    pub visible_draw_item_count: u64,
    /// Draw items in the full compiled plan before native damage-scissor culling.
    pub compiled_visible_draw_item_count: u64,
    /// Command rows visited while deriving the current visible statistics.
    pub command_visibility_scan_count: u64,
    /// Reuses a versioned full-projection statistics snapshot instead of rescanning commands.
    pub command_stats_cache_hit_count: u64,
    /// Solid vertices actually submitted for the current native present.
    pub solid_vertex_count: u64,
    /// Ordinary solid rectangle instances actually submitted for this present.
    pub solid_instance_count: u64,
    /// Solid vertices in the full compiled plan.
    pub compiled_solid_vertex_count: u64,
    /// Ordinary solid rectangle instances in the full compiled plan.
    pub compiled_solid_instance_count: u64,
    /// Image vertices actually submitted for the current native present.
    pub image_vertex_count: u64,
    /// Image vertices in the full compiled plan.
    pub compiled_image_vertex_count: u64,
    /// Compiled layers that emitted at least one draw for the current native present.
    pub batch_layer_count: u64,
    /// Layers in the full compiled plan.
    pub compiled_batch_layer_count: u64,
    /// Reserved for an explicitly materialized submitted dependency graph.
    pub batch_dependency_count: u64,
    /// Dependency edges in the full compiled plan.
    pub compiled_batch_dependency_count: u64,
    /// Submitted draw items eliminated by material-compatible batching.
    pub batch_merge_count: u64,
    /// Full-plan draw items eliminated by material-compatible batching.
    pub compiled_batch_merge_count: u64,
    /// Candidate pairs examined by the runtime UI overlap planner.
    pub overlap_candidate_count: u64,
    pub batch_plan_build_count: u64,
    pub batch_plan_cache_hit_count: u64,
    pub vertex_buffer_create_count: u64,
    pub vertex_upload_bytes: u64,
    /// Bytes copied from the retained UI cache into a COPY_DST swapchain texture.
    pub retained_cache_copy_bytes: u64,
    /// Whether the active native UI device supports non-blocking timestamp queries.
    pub gpu_timestamp_supported: bool,
    /// Most recently completed whole-UI GPU sample, reported after asynchronous readback.
    pub gpu_time_us: Option<u64>,
    /// Frames between the sampled present and the present that consumed its result.
    pub gpu_profile_latency_frames: u32,
    pub text_shape_count: u64,
    pub text_renderer_build_count: u64,
    pub text_renderer_cache_hit_count: u64,
    /// Glyph-atlas preparation failures; failed generations remain retryable.
    pub text_prepare_failure_count: u64,
    /// Image command rows visited while resolving compiled upload sources.
    pub image_prepare_command_visit_count: u64,
    /// Compiled image resources reused without probing their source commands.
    pub image_prepare_cache_hit_count: u64,
    pub image_upload_bytes: u64,
    /// Native image texture writes performed for the current present.
    pub image_upload_write_count: u64,
    /// Device-shared image products resolved instead of uploading a presenter-local texture.
    pub image_shared_resolve_count: u64,
    /// Device-shared image texture writes performed for the current present.
    pub image_shared_upload_write_count: u64,
    /// Bytes written while publishing new device-shared image products.
    pub image_shared_upload_bytes: u64,
    /// Texture bytes retained by the device-shared image registry after this present.
    pub image_shared_resident_bytes: u64,
    /// Owned image-cache keys allocated for insertion or admission-time eviction planning.
    pub image_cache_key_allocation_count: u64,
    /// Image-cache entries visited by bounded admission-time LRU planning.
    pub image_cache_prune_visit_count: u64,
    /// New image resources rejected after the hard entry/byte budget became fully active.
    pub image_cache_admission_reject_count: u64,
    /// Supplied image payloads rejected before texture creation or upload.
    pub image_invalid_payload_count: u64,
    /// Resident RGBA texture bytes after the current present.
    pub image_cache_resident_bytes: u64,
    /// CPU decoded RGBA bytes retained by the bounded native image cache.
    pub image_cache_cpu_resident_bytes: u64,
    pub image_count: u64,
    pub clip_count: u64,
    pub presented_frame_count: u64,
}

pub trait UiSurfacePresenter: Send {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError>;
    /// Returns whether the runtime-owned UI registry can render this exact resource revision
    /// without another producer-side pixel payload.
    fn is_image_resource_resident(&self, _resource_key: &str, _generation: u64) -> bool {
        false
    }
    fn present(&mut self, draw_list: &UiSurfaceDrawList)
        -> Result<UiSurfacePresentStats, RhiError>;
    fn present_owned(
        &mut self,
        draw_list: UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        self.present(&draw_list)
    }
    fn last_present_stats(&self) -> UiSurfacePresentStats;
}

impl<T: UiSurfacePresenter + ?Sized> UiSurfacePresenter for Box<T> {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError> {
        self.as_mut().resize(width, height)
    }

    fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        self.as_mut().present(draw_list)
    }

    fn is_image_resource_resident(&self, resource_key: &str, generation: u64) -> bool {
        self.as_ref()
            .is_image_resource_resident(resource_key, generation)
    }

    fn present_owned(
        &mut self,
        draw_list: UiSurfaceDrawList,
    ) -> Result<UiSurfacePresentStats, RhiError> {
        self.as_mut().present_owned(draw_list)
    }

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.as_ref().last_present_stats()
    }
}

#[cfg(test)]
#[path = "ui_surface/tests.rs"]
mod tests;

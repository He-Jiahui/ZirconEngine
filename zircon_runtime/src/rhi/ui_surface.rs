use std::collections::BTreeSet;

use crate::core::framework::render::RenderNativeSurfaceTarget;

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

    pub(crate) fn has_finite_positive_area(self) -> bool {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub width: u32,
    pub height: u32,
    pub upload_bytes: u64,
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
}

impl UiSurfaceDescriptor {
    pub const fn headless(label: &'static str, width: u32, height: u32) -> Self {
        Self {
            label: Some(label),
            width,
            height,
            target: None,
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
        }
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
    pub damage: Option<UiSurfaceRect>,
    pub commands: Vec<UiSurfaceCommand>,
    generation: Option<u64>,
}

impl UiSurfaceDrawList {
    pub fn new(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
    ) -> Self {
        Self {
            surface_size: (surface_size.0.max(1), surface_size.1.max(1)),
            damage,
            commands,
            generation: None,
        }
    }

    /// Constructs a list whose command payload is identified by a producer-owned revision.
    ///
    /// The revision must advance whenever the commands or their payloads change. A list made
    /// with [`Self::new`] deliberately has no revision and is never eligible for a compiled
    /// WGPU batch-plan cache, which preserves correctness for legacy callers.
    pub fn with_generation(
        surface_size: (u32, u32),
        damage: Option<UiSurfaceRect>,
        commands: Vec<UiSurfaceCommand>,
        generation: u64,
    ) -> Self {
        Self {
            surface_size: (surface_size.0.max(1), surface_size.1.max(1)),
            damage,
            commands,
            generation: Some(generation),
        }
    }

    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }

    pub fn stats(&self) -> UiSurfacePresentStats {
        let mut stats = UiSurfacePresentStatsAccumulator::new(self.surface_size);
        for command in &self.commands {
            if !self.command_visible_with_damage(command, self.damage) {
                continue;
            }
            stats.record_visible(command);
        }
        stats.finish()
    }

    pub(crate) fn command_visible_with_damage(
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
        draw_list.surface_size.0 as f32,
        draw_list.surface_size.1 as f32,
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
pub(crate) struct UiSurfacePresentStatsAccumulator<'a> {
    uploaded_image_keys: BTreeSet<&'a str>,
    stats: UiSurfacePresentStats,
}

impl<'a> UiSurfacePresentStatsAccumulator<'a> {
    pub(crate) fn new(surface_size: (u32, u32)) -> Self {
        Self {
            uploaded_image_keys: BTreeSet::new(),
            stats: UiSurfacePresentStats {
                surface_size,
                command_visibility_scan_count: 1,
                ..UiSurfacePresentStats::default()
            },
        }
    }

    pub(crate) fn record_visible(&mut self, command: &'a UiSurfaceCommand) {
        self.stats.visible_command_payload_bytes = self
            .stats
            .visible_command_payload_bytes
            .saturating_add(command_dynamic_payload_bytes(&command.kind));
        match &command.kind {
            UiSurfaceCommandKind::Quad { .. }
            | UiSurfaceCommandKind::Border { .. }
            | UiSurfaceCommandKind::Text { .. } => {
                self.stats.visible_command_count =
                    self.stats.visible_command_count.saturating_add(1);
                self.stats.visible_draw_item_count =
                    self.stats.visible_draw_item_count.saturating_add(1);
                self.stats.draw_calls = self.stats.draw_calls.saturating_add(1);
            }
            UiSurfaceCommandKind::Image { payload } => {
                self.stats.visible_command_count =
                    self.stats.visible_command_count.saturating_add(1);
                self.stats.visible_draw_item_count =
                    self.stats.visible_draw_item_count.saturating_add(1);
                self.stats.draw_calls = self.stats.draw_calls.saturating_add(1);
                self.stats.image_count = self.stats.image_count.saturating_add(1);
                if payload.rgba.is_some()
                    && self
                        .uploaded_image_keys
                        .insert(payload.resource_key.as_str())
                {
                    self.stats.image_upload_bytes = self
                        .stats
                        .image_upload_bytes
                        .saturating_add(payload.upload_bytes);
                }
            }
            UiSurfaceCommandKind::Clip => {
                self.stats.clip_count = self.stats.clip_count.saturating_add(1);
            }
        }
    }

    pub(crate) fn finish(self) -> UiSurfacePresentStats {
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
        | UiSurfaceCommandKind::Clip => 0,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiSurfacePresentStats {
    pub surface_size: (u32, u32),
    pub draw_calls: u64,
    /// Draw ops in the full compiled plan before native damage-scissor culling.
    pub compiled_draw_calls: u64,
    /// Native WGPU render passes actually begun for the current present.
    pub render_pass_count: u64,
    pub visible_command_count: u64,
    /// Dynamic string and RGBA bytes referenced by visible commands.
    pub visible_command_payload_bytes: u64,
    pub visible_draw_item_count: u64,
    /// Draw items in the full compiled plan before native damage-scissor culling.
    pub compiled_visible_draw_item_count: u64,
    /// Command rows visited while deriving the current visible statistics.
    pub command_visibility_scan_count: u64,
    /// Reuses a versioned full-projection statistics snapshot instead of rescanning commands.
    pub command_stats_cache_hit_count: u64,
    /// Solid vertices actually submitted for the current native present.
    pub solid_vertex_count: u64,
    /// Solid vertices in the full compiled plan.
    pub compiled_solid_vertex_count: u64,
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
    /// Owned image-cache keys allocated while creating or replacing resources.
    pub image_cache_key_allocation_count: u64,
    /// Image-cache entries visited by bounded admission-time LRU planning.
    pub image_cache_prune_visit_count: u64,
    /// New image resources rejected after the hard entry/byte budget became fully active.
    pub image_cache_admission_reject_count: u64,
    /// Supplied image payloads rejected before texture creation or upload.
    pub image_invalid_payload_count: u64,
    /// Resident RGBA texture bytes after the current present.
    pub image_cache_resident_bytes: u64,
    pub image_count: u64,
    pub clip_count: u64,
    pub presented_frame_count: u64,
}

pub trait UiSurfacePresenter: Send {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RhiError>;
    fn present(&mut self, draw_list: &UiSurfaceDrawList)
        -> Result<UiSurfacePresentStats, RhiError>;
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

    fn last_present_stats(&self) -> UiSurfacePresentStats {
        self.as_ref().last_present_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_list_stats_count_draw_upload_and_clip_commands() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            Some(UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0)),
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Clip,
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [1, 2, 3, 255],
                        corner_radius: 6.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 2,
                    frame: UiSurfaceRect::new(1.0, 1.0, 8.0, 8.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Border {
                        color: [4, 5, 6, 255],
                        width: 1.0,
                        corner_radius: 6.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 3,
                    frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "viewport".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                            atlas_uv: None,
                        },
                    },
                },
            ],
        );

        let stats = draw_list.stats();
        assert_eq!(stats.surface_size, (64, 32));
        assert_eq!(stats.draw_calls, 3);
        assert_eq!(stats.render_pass_count, 0);
        assert_eq!(stats.retained_cache_copy_bytes, 0);
        assert_eq!(stats.visible_command_count, 3);
        assert_eq!(stats.visible_draw_item_count, 3);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 16);
        assert_eq!(stats.clip_count, 1);
    }

    #[test]
    fn draw_list_stats_skip_commands_outside_damage() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            Some(UiSurfaceRect::new(40.0, 20.0, 8.0, 8.0)),
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [1, 2, 3, 255],
                        corner_radius: 0.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(42.0, 22.0, 2.0, 2.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "viewport".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                            atlas_uv: None,
                        },
                    },
                },
            ],
        );

        let stats = draw_list.stats();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 16);
    }

    #[test]
    fn draw_list_stats_skip_commands_with_non_finite_or_non_positive_rects() {
        let commands = [
            UiSurfaceRect::new(f32::NAN, 0.0, 10.0, 10.0),
            UiSurfaceRect::new(0.0, 0.0, f32::INFINITY, 10.0),
            UiSurfaceRect::new(0.0, 0.0, 10.0, -1.0),
            UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
        ]
        .into_iter()
        .enumerate()
        .map(|(z_index, frame)| UiSurfaceCommand {
            z_index: z_index as i32,
            frame,
            clip: None,
            kind: UiSurfaceCommandKind::Quad {
                color: [255, 255, 255, 255],
                corner_radius: 0.0,
            },
        })
        .collect();
        let draw_list = UiSurfaceDrawList::new((64, 32), None, commands);

        let stats = draw_list.stats();

        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
    }

    #[test]
    fn draw_list_generation_is_opt_in_for_compiled_presenters() {
        let legacy = UiSurfaceDrawList::new((64, 32), None, Vec::new());
        let versioned = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);

        assert_eq!(legacy.generation(), None);
        assert_eq!(versioned.generation(), Some(9));
    }

    #[test]
    fn draw_list_stats_do_not_count_cached_images_as_uploads() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            None,
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "cached".to_string(),
                        width: 2,
                        height: 2,
                        upload_bytes: 16,
                        rgba: None,
                        atlas_uv: None,
                    },
                },
            }],
        );

        let stats = draw_list.stats();

        assert_eq!(stats.draw_calls, 1);
        assert_eq!(stats.visible_command_count, 1);
        assert_eq!(stats.visible_draw_item_count, 1);
        assert_eq!(stats.image_count, 1);
        assert_eq!(stats.image_upload_bytes, 0);
    }

    #[test]
    fn draw_list_stats_count_same_resource_image_upload_once() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            None,
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "atlas://editor/icons".to_string(),
                            width: 4,
                            height: 4,
                            upload_bytes: 64,
                            rgba: Some(vec![255; 64]),
                            atlas_uv: Some(UiSurfaceImageUvRect {
                                min: [0.0, 0.0],
                                max: [0.5, 0.5],
                            }),
                        },
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(4.0, 0.0, 2.0, 2.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "atlas://editor/icons".to_string(),
                            width: 4,
                            height: 4,
                            upload_bytes: 64,
                            rgba: Some(vec![255; 64]),
                            atlas_uv: Some(UiSurfaceImageUvRect {
                                min: [0.5, 0.0],
                                max: [1.0, 0.5],
                            }),
                        },
                    },
                },
            ],
        );

        let stats = draw_list.stats();

        assert_eq!(stats.visible_command_count, 2);
        assert_eq!(stats.image_count, 2);
        assert_eq!(stats.image_upload_bytes, 64);
    }

    #[test]
    fn draw_list_stats_measure_visible_dynamic_command_payloads() {
        let draw_list = UiSurfaceDrawList::new(
            (64, 32),
            None,
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(0.0, 0.0, 16.0, 8.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Text {
                        text: "text".to_string(),
                        color: [255, 255, 255, 255],
                        font_family: Some("ui".to_string()),
                        font_weight: 400,
                        font_size: 12.0,
                        line_height: 14.0,
                        style: UiSurfaceTextStyle::Regular,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(20.0, 0.0, 8.0, 8.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "icon".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                            atlas_uv: None,
                        },
                    },
                },
            ],
        );

        let stats = draw_list.stats();

        assert_eq!(stats.visible_command_payload_bytes, 26);
    }

    #[test]
    fn atlas_uv_rect_validates_normalized_finite_bounds() {
        assert!(UiSurfaceImageUvRect {
            min: [0.25, 0.25],
            max: [0.75, 0.75],
        }
        .is_valid());
        assert!(!UiSurfaceImageUvRect {
            min: [0.75, 0.25],
            max: [0.75, 0.75],
        }
        .is_valid());
        assert!(!UiSurfaceImageUvRect {
            min: [0.0, f32::NAN],
            max: [1.0, 1.0],
        }
        .is_valid());
    }

    #[test]
    fn surface_descriptor_rejects_zero_size() {
        assert_eq!(
            UiSurfaceDescriptor::headless("bad", 0, 1)
                .validate()
                .unwrap_err(),
            RhiError::InvalidSurfaceDescriptor {
                label: Some("bad".to_string()),
                reason: "width and height must be greater than zero".to_string(),
            }
        );
    }
}

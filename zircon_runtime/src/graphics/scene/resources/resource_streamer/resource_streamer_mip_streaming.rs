use core::ops::Range;
use std::collections::HashMap;

use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{view_matrix, Vec3};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::MaterialRuntime;

use crate::graphics::types::ViewportRenderFrame;

use super::ResourceStreamer;

mod frame_apply;

/// Visibility data collected during extract without copying texture metadata or GPU state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MipStreamingVisibility {
    pub(super) texture: ResourceId,
    /// Fraction of the source texture represented on screen, quantized to `u16::MAX`.
    pub(super) screen_coverage: u16,
    /// Stable extract order used to make equal-priority scheduling deterministic.
    pub(super) stable_order: u64,
}

/// A texture observation prepared from the visible scene before GPU work begins.
#[derive(Clone, Debug)]
struct MipStreamingDemand {
    texture: ResourceId,
    mip_count: u8,
    resident_mips: Range<u8>,
    /// Fraction of the source texture represented on screen, quantized to `u16::MAX`.
    screen_coverage: u16,
    streaming_enabled: bool,
    /// Stable extract order used to make equal-priority scheduling deterministic.
    stable_order: u64,
    /// CPU/source bytes that must be uploaded if this target range is promoted this frame.
    upload_bytes: u64,
    /// Current physical allocation for this texture's resident mip tail.
    resident_bytes: u64,
    /// Physical allocation after the planned residency transition completes.
    wanted_bytes: u64,
    /// Allocation after a one-mip budget emergency eviction from the current resident range.
    forced_eviction_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MipStreamingPlan {
    pub(super) texture: ResourceId,
    /// The normalized resident range before a rebuild or eviction is applied.
    pub(super) resident_mips: Range<u8>,
    /// The target range, always retaining the lowest-resolution tail mip.
    pub(super) wanted_mips: Range<u8>,
    /// Quantized screen coverage; larger values are scheduled first.
    pub(super) priority: u32,
    /// Missing source bytes required by the target range after GPU-to-GPU common-mip copies.
    pub(super) upload_bytes: u64,
    pub(super) resident_bytes: u64,
    pub(super) wanted_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MipStreamingTransitionKind {
    Promotion,
    Eviction,
}

/// A single rebuild-and-rebind request whose completion may update residency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MipStreamingTask {
    pub(super) texture: ResourceId,
    transition_id: u64,
    pub(super) kind: MipStreamingTransitionKind,
    pub(super) resident_mips: Range<u8>,
    pub(super) wanted_mips: Range<u8>,
    pub(super) priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingMipStreamingTransition {
    transition_id: u64,
    wanted_mips: Range<u8>,
}

/// Per-texture transition state. GPU residency changes only after a matching successful task.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct MipStreamingState {
    next_transition_id: u64,
    pending: Option<PendingMipStreamingTransition>,
}

impl MipStreamingState {
    fn begin(&mut self, plan: MipStreamingPlan) -> Option<MipStreamingTask> {
        if plan.resident_mips == plan.wanted_mips || self.pending.is_some() {
            return None;
        }

        let transition_id = self.next_transition_id;
        self.next_transition_id = self.next_transition_id.wrapping_add(1);
        let kind = if plan.wanted_mips.start < plan.resident_mips.start {
            MipStreamingTransitionKind::Promotion
        } else {
            MipStreamingTransitionKind::Eviction
        };
        self.pending = Some(PendingMipStreamingTransition {
            transition_id,
            wanted_mips: plan.wanted_mips.clone(),
        });
        Some(MipStreamingTask {
            texture: plan.texture,
            transition_id,
            kind,
            resident_mips: plan.resident_mips,
            wanted_mips: plan.wanted_mips,
            priority: plan.priority,
        })
    }

    fn finish(&mut self, task: &MipStreamingTask, succeeded: bool) -> Option<Range<u8>> {
        let pending = self.pending.as_ref()?;
        if pending.transition_id != task.transition_id || pending.wanted_mips != task.wanted_mips {
            return None;
        }

        self.pending = None;
        succeeded.then(|| task.wanted_mips.clone())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MipStreamingSettings {
    pub(super) max_transitions: usize,
    pub(super) max_upload_bytes: u64,
    pub(super) max_resident_bytes: u64,
    pub(super) current_resident_bytes: u64,
    pub(super) hysteresis_mips: u8,
    pub(super) mip_bias: u8,
}

impl Default for MipStreamingSettings {
    fn default() -> Self {
        Self {
            max_transitions: DEFAULT_MIP_STREAMING_TRANSITIONS_PER_FRAME,
            max_upload_bytes: DEFAULT_MIP_STREAMING_UPLOAD_BUDGET_BYTES,
            max_resident_bytes: u64::MAX,
            current_resident_bytes: 0,
            hysteresis_mips: DEFAULT_MIP_STREAMING_HYSTERESIS_MIPS,
            mip_bias: 0,
        }
    }
}

const DEFAULT_MIP_STREAMING_TRANSITIONS_PER_FRAME: usize = 16;
const DEFAULT_MIP_STREAMING_UPLOAD_BUDGET_BYTES: u64 = 32 * 1024 * 1024;
const DEFAULT_MIP_STREAMING_HYSTERESIS_MIPS: u8 = 1;
const FULL_SCREEN_COVERAGE: u32 = u16::MAX as u32;

// 先处理高优先级提升，再处理低优先级驱逐，并以稳定顺序打破平局；逐项更新预计驻留量，超预算提升留待后续帧。
fn plan_mip_streaming(
    demands: impl IntoIterator<Item = MipStreamingDemand>,
    settings: MipStreamingSettings,
) -> Vec<MipStreamingPlan> {
    let mut candidates = demands
        .into_iter()
        .filter_map(|demand| mip_streaming_candidate(demand, settings))
        .collect::<Vec<_>>();
    candidates.sort_by(mip_streaming_candidate_order);
    let mut scheduled = Vec::with_capacity(candidates.len().min(settings.max_transitions));
    let mut scheduled_upload_bytes = 0_u64;
    let mut projected_resident_bytes = settings.current_resident_bytes;
    for candidate in candidates {
        if scheduled.len() == settings.max_transitions {
            break;
        }
        if candidate.plan.upload_bytes
            > settings
                .max_upload_bytes
                .saturating_sub(scheduled_upload_bytes)
        {
            continue;
        }
        let projected_after_transition = projected_resident_bytes
            .saturating_sub(candidate.plan.resident_bytes)
            .saturating_add(candidate.plan.wanted_bytes);
        if candidate.plan.wanted_bytes > candidate.plan.resident_bytes
            && projected_after_transition > settings.max_resident_bytes
        {
            continue;
        }
        scheduled_upload_bytes = scheduled_upload_bytes.saturating_add(candidate.plan.upload_bytes);
        projected_resident_bytes = projected_after_transition;
        scheduled.push(candidate.plan);
    }
    scheduled
}

fn wanted_mip_start(mip_count: u8, screen_coverage: u16, mip_bias: u8) -> u8 {
    let mip_count = mip_count.max(1);
    let last_mip = mip_count - 1;
    let coverage = u32::from(screen_coverage);
    let mut mip = 0_u8;

    while mip < last_mip {
        let next_mip = mip + 1;
        let shift = u32::from(next_mip) * 2;
        let next_threshold = FULL_SCREEN_COVERAGE.checked_shr(shift).unwrap_or(0);
        if coverage > next_threshold {
            break;
        }
        mip = next_mip;
    }

    mip.saturating_add(mip_bias).min(last_mip)
}

struct MipStreamingCandidate {
    plan: MipStreamingPlan,
    stable_order: u64,
}

fn mip_streaming_candidate_order(
    left: &MipStreamingCandidate,
    right: &MipStreamingCandidate,
) -> core::cmp::Ordering {
    let left_is_eviction = left.plan.wanted_mips.start > left.plan.resident_mips.start;
    let right_is_eviction = right.plan.wanted_mips.start > right.plan.resident_mips.start;
    left_is_eviction
        .cmp(&right_is_eviction)
        .then_with(|| {
            if left_is_eviction {
                left.plan.priority.cmp(&right.plan.priority)
            } else {
                right.plan.priority.cmp(&left.plan.priority)
            }
        })
        .then_with(|| left.stable_order.cmp(&right.stable_order))
}

fn mip_streaming_candidate(
    demand: MipStreamingDemand,
    settings: MipStreamingSettings,
) -> Option<MipStreamingCandidate> {
    let mip_count = demand.mip_count.max(1);
    let last_mip = mip_count - 1;
    let resident_start = demand.resident_mips.start.min(last_mip);
    let wanted_start = if demand.streaming_enabled {
        wanted_mip_start(mip_count, demand.screen_coverage, settings.mip_bias)
    } else {
        0
    };
    let force_budget_eviction = demand.streaming_enabled
        && settings.current_resident_bytes > settings.max_resident_bytes
        && resident_start == wanted_start
        && resident_start < last_mip;

    let (wanted_start, wanted_bytes) = if force_budget_eviction {
        (
            resident_start.saturating_add(1).min(last_mip),
            demand.forced_eviction_bytes,
        )
    } else {
        (wanted_start, demand.wanted_bytes)
    };

    if resident_start == wanted_start
        || (demand.streaming_enabled
            && !force_budget_eviction
            && resident_start.abs_diff(wanted_start) <= settings.hysteresis_mips)
    {
        return None;
    }

    Some(MipStreamingCandidate {
        plan: MipStreamingPlan {
            texture: demand.texture,
            resident_mips: resident_start..mip_count,
            wanted_mips: wanted_start..mip_count,
            priority: u32::from(demand.screen_coverage),
            upload_bytes: demand.upload_bytes,
            resident_bytes: demand.resident_bytes,
            wanted_bytes,
        },
        stable_order: demand.stable_order,
    })
}

impl ResourceStreamer {
    pub(super) fn collect_texture_mip_streaming_visibility(&mut self, frame: &ViewportRenderFrame) {
        self.mip_streaming_visibility.clear();
        self.mip_streaming_visible_instance_keys.clear();

        let Some(frame_visibility) = frame.frame_visibility() else {
            return;
        };
        let Some(main_view) = frame_visibility.main_view() else {
            return;
        };
        self.mip_streaming_visible_instance_keys.extend(
            main_view
                .visible
                .iter()
                .filter_map(|index| frame_visibility.stable_instance_keys.get(*index as usize))
                .copied(),
        );

        let camera = frame.effective_camera();
        let mut stable_order = 0_u64;
        for mesh in frame.meshes() {
            if !self
                .mip_streaming_visible_instance_keys
                .contains(&mesh.stable_instance_key)
            {
                continue;
            }
            let screen_coverage = quantized_screen_coverage(
                &camera,
                mesh.transform.translation,
                mesh.transform.scale.abs().length() * 0.5,
            );
            if screen_coverage == 0 {
                continue;
            }
            let Some(material) = self.materials.get(&mesh.material.id()) else {
                continue;
            };

            let Some(published) = material.published.as_ref() else {
                continue;
            };
            for texture in material_texture_ids(&published.runtime) {
                if self.textures.contains_key(&texture) {
                    self.mip_streaming_visibility.push(MipStreamingVisibility {
                        texture,
                        screen_coverage,
                        stable_order,
                    });
                    stable_order = stable_order.wrapping_add(1);
                }
            }
        }
    }

    pub(super) fn plan_texture_mip_streaming(
        &self,
        visibility: impl IntoIterator<Item = MipStreamingVisibility>,
        settings: MipStreamingSettings,
    ) -> Vec<MipStreamingPlan> {
        let mut settings = settings;
        settings.current_resident_bytes = self.persistent_texture_resident_bytes();
        let visibility = include_non_visible_resident_texture_visibility(
            coalesce_mip_streaming_visibility(visibility),
            self.textures.keys().copied(),
        );
        let demands = visibility.into_values().filter_map(|visibility| {
            let prepared = self.textures.get(&visibility.texture)?;
            let descriptor = &prepared.resource.descriptor;
            let streaming_enabled = prepared.resource.supports_mip_streaming()
                && descriptor.metadata.allows_mip_streaming(
                    descriptor.width,
                    descriptor.height,
                    descriptor.mip_count,
                );
            let mip_count = prepared.resident_mip_range.end;
            let wanted_start = if streaming_enabled {
                wanted_mip_start(mip_count, visibility.screen_coverage, settings.mip_bias)
            } else {
                0
            };
            Some(MipStreamingDemand {
                texture: visibility.texture,
                mip_count,
                resident_mips: prepared.resident_mip_range.clone(),
                screen_coverage: visibility.screen_coverage,
                streaming_enabled,
                stable_order: visibility.stable_order,
                upload_bytes: prepared.resource.mip_streaming_upload_bytes(
                    prepared.resident_mip_range.clone(),
                    wanted_start..mip_count,
                ),
                resident_bytes: prepared.resource.resident_texture_bytes(),
                wanted_bytes: prepared
                    .resource
                    .mip_streaming_resident_bytes(wanted_start..mip_count),
                forced_eviction_bytes: prepared.resource.mip_streaming_resident_bytes(
                    prepared
                        .resident_mip_range
                        .start
                        .saturating_add(1)
                        .min(mip_count.saturating_sub(1))..mip_count,
                ),
            })
        });
        plan_mip_streaming(demands, settings)
    }

    pub(super) fn schedule_texture_mip_streaming(
        &mut self,
        visibility: impl IntoIterator<Item = MipStreamingVisibility>,
        settings: MipStreamingSettings,
    ) -> Vec<MipStreamingTask> {
        self.plan_texture_mip_streaming(visibility, settings)
            .into_iter()
            .filter_map(|plan| {
                self.mip_streaming_states
                    .entry(plan.texture)
                    .or_default()
                    .begin(plan)
            })
            .collect()
    }

    pub(super) fn finish_texture_mip_streaming_task(
        &mut self,
        task: &MipStreamingTask,
        succeeded: bool,
    ) -> Option<Range<u8>> {
        self.mip_streaming_states
            .get_mut(&task.texture)?
            .finish(task, succeeded)
    }

    pub(crate) fn persistent_texture_resident_bytes(&self) -> u64 {
        self.textures
            .values()
            .map(|prepared| prepared.resource.resident_texture_bytes())
            .fold(0_u64, u64::saturating_add)
    }

    pub(crate) fn set_mip_streaming_residency_budget(&mut self, bytes: u64) {
        self.mip_streaming_residency_budget_bytes = bytes;
    }
}

fn material_texture_ids(runtime: &MaterialRuntime) -> impl Iterator<Item = ResourceId> + '_ {
    [
        runtime.base_color_texture,
        runtime.normal_texture,
        runtime.metallic_roughness_texture,
        runtime.occlusion_texture,
        runtime.emissive_texture,
        runtime.clearcoat_normal_texture,
    ]
    .into_iter()
    .flatten()
    .chain(
        runtime
            .non_standard_texture_slots
            .values()
            .copied()
            .flatten(),
    )
}

fn quantized_screen_coverage(camera: &ViewportCameraSnapshot, center: Vec3, radius: f32) -> u16 {
    if camera.projection_override.is_some() {
        // A custom projection can distort the analytic bound; preserve detail until it exposes
        // a matching projected-bounds implementation.
        return u16::MAX;
    }

    let radius = radius.max(0.0);
    if !radius.is_finite() {
        return 0;
    }
    let view_position = view_matrix(camera.transform).transform_point3(center);
    let depth = -view_position.z;
    if !depth.is_finite() || depth + radius < camera.z_near.max(0.001) {
        return 0;
    }
    let near_depth = (depth - radius).max(camera.z_near.max(0.001));
    let aspect_ratio = camera.aspect_ratio.max(0.001);
    let (radius_ndc_x, radius_ndc_y) = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let half_fov_tangent = (camera.fov_y_radians * 0.5).tan().max(0.001);
            let radius_ndc_y = radius / (near_depth * half_fov_tangent);
            (radius_ndc_y / aspect_ratio, radius_ndc_y)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let radius_ndc_y = radius / half_height;
            (radius_ndc_y / aspect_ratio, radius_ndc_y)
        }
    };
    let coverage = (core::f32::consts::PI * radius_ndc_x * radius_ndc_y * 0.25).clamp(0.0, 1.0);
    (coverage * f32::from(u16::MAX)).round() as u16
}

fn coalesce_mip_streaming_visibility(
    visibility: impl IntoIterator<Item = MipStreamingVisibility>,
) -> HashMap<ResourceId, MipStreamingVisibility> {
    let mut coalesced = HashMap::new();
    for candidate in visibility {
        coalesced
            .entry(candidate.texture)
            .and_modify(|existing: &mut MipStreamingVisibility| {
                if candidate.screen_coverage > existing.screen_coverage
                    || (candidate.screen_coverage == existing.screen_coverage
                        && candidate.stable_order < existing.stable_order)
                {
                    *existing = candidate;
                }
            })
            .or_insert(candidate);
    }
    coalesced
}

/// Preserve an eviction candidate for every resident texture, including assets that left the
/// current view after being promoted. Visible observations always keep their measured priority.
fn include_non_visible_resident_texture_visibility(
    mut visibility: HashMap<ResourceId, MipStreamingVisibility>,
    texture_ids: impl IntoIterator<Item = ResourceId>,
) -> HashMap<ResourceId, MipStreamingVisibility> {
    let first_unobserved_order = visibility
        .values()
        .map(|observation| observation.stable_order)
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    let mut unobserved_texture_ids = texture_ids
        .into_iter()
        .filter(|texture| !visibility.contains_key(texture))
        .collect::<Vec<_>>();
    unobserved_texture_ids.sort_unstable();
    for (index, texture) in unobserved_texture_ids.into_iter().enumerate() {
        visibility.insert(
            texture,
            MipStreamingVisibility {
                texture,
                screen_coverage: 0,
                stable_order: first_unobserved_order.saturating_add(index as u64),
            },
        );
    }
    visibility
}

#[cfg(test)]
#[path = "tests/resource_streamer_mip_streaming.rs"]
mod tests;

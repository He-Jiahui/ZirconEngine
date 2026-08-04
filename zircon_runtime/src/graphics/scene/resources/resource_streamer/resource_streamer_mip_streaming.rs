use core::ops::Range;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{view_matrix, Vec3};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::GpuTextureResource;
use crate::graphics::scene::resources::MaterialRuntime;

use super::super::prepared::PreparedTexture;
use crate::graphics::types::ViewportRenderFrame;

use super::ResourceStreamer;

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

            for texture in material_texture_ids(&material.runtime) {
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

    pub(super) fn apply_texture_mip_streaming(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        mip_bias: u8,
    ) {
        let tasks = self.schedule_texture_mip_streaming(
            self.mip_streaming_visibility.clone(),
            MipStreamingSettings {
                mip_bias,
                max_resident_bytes: self.mip_streaming_residency_budget_bytes,
                ..Default::default()
            },
        );
        for task in tasks {
            let rebuilt =
                self.rebuild_texture_mip_streaming_task(device, queue, texture_layout, &task);
            match rebuilt {
                Some((revision, resource)) => {
                    if let Some(resident_mip_range) =
                        self.finish_texture_mip_streaming_task(&task, true)
                    {
                        self.textures.insert(
                            task.texture,
                            PreparedTexture {
                                revision,
                                resource,
                                resident_mip_range,
                            },
                        );
                    }
                }
                None => {
                    self.finish_texture_mip_streaming_task(&task, false);
                }
            }
        }
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

    fn rebuild_texture_mip_streaming_task(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        task: &MipStreamingTask,
    ) -> Option<(u64, Arc<GpuTextureResource>)> {
        let prepared = self.textures.get(&task.texture)?;
        if prepared.resident_mip_range != task.resident_mips
            || self.resource_revision(task.texture).ok()? != prepared.revision
        {
            return None;
        }
        let revision = prepared.revision;
        let previous = Arc::clone(&prepared.resource);
        let previous_range = prepared.resident_mip_range.clone();
        let payload = self
            .asset_manager()
            .ok()?
            .load_texture_asset(task.texture)
            .ok()?;
        let resource = GpuTextureResource::rebuild_resident_mips(
            device,
            queue,
            texture_layout,
            task.texture,
            payload,
            previous.as_ref(),
            previous_range,
            task.wanted_mips.clone(),
        )
        .ok()?;
        Some((revision, Arc::new(resource)))
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
mod tests {
    use std::collections::HashMap;

    use crate::core::resource::ResourceId;

    use super::*;

    fn demand(
        label: &str,
        mip_count: u8,
        resident_mips: core::ops::Range<u8>,
        screen_coverage: u16,
        streaming_enabled: bool,
        stable_order: u64,
    ) -> MipStreamingDemand {
        MipStreamingDemand {
            texture: ResourceId::from_stable_label(label),
            mip_count,
            resident_mips,
            screen_coverage,
            streaming_enabled,
            stable_order,
            upload_bytes: 0,
            resident_bytes: 0,
            wanted_bytes: 0,
            forced_eviction_bytes: 0,
        }
    }

    impl MipStreamingDemand {
        fn with_upload_bytes(mut self, upload_bytes: u64) -> Self {
            self.upload_bytes = upload_bytes;
            self
        }

        fn with_resident_bytes(mut self, resident_bytes: u64, wanted_bytes: u64) -> Self {
            self.resident_bytes = resident_bytes;
            self.wanted_bytes = wanted_bytes;
            self.forced_eviction_bytes = wanted_bytes;
            self
        }
    }

    #[test]
    fn render_mip_streaming_wanted_mip_tracks_screen_coverage_and_bias() {
        assert_eq!(wanted_mip_start(6, u16::MAX, 0), 0);
        assert_eq!(wanted_mip_start(6, u16::MAX / 4, 0), 1);
        assert_eq!(wanted_mip_start(6, u16::MAX / 16, 0), 2);
        assert_eq!(wanted_mip_start(6, u16::MAX / 16, 2), 4);
        assert_eq!(wanted_mip_start(6, 0, 0), 5);
    }

    #[test]
    fn render_mip_streaming_prioritizes_visible_promotions_with_a_bounded_queue() {
        let near = demand("mip-streaming-near", 6, 2..6, u16::MAX, true, 7);
        let far = demand("mip-streaming-far", 6, 0..6, u16::MAX / 16, true, 3);

        let plans = plan_mip_streaming(
            [near, far],
            MipStreamingSettings {
                max_transitions: 1,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: u64::MAX,
                current_resident_bytes: 0,
                hysteresis_mips: 0,
                mip_bias: 0,
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].texture,
            ResourceId::from_stable_label("mip-streaming-near")
        );
        assert_eq!(plans[0].resident_mips, 2..6);
        assert_eq!(plans[0].wanted_mips, 0..6);
    }

    #[test]
    fn render_mip_streaming_defers_promotions_that_exceed_the_frame_upload_budget() {
        let large = demand("mip-streaming-large", 6, 2..6, u16::MAX, true, 0).with_upload_bytes(64);
        let small =
            demand("mip-streaming-small", 6, 2..6, u16::MAX / 2, true, 1).with_upload_bytes(16);

        let plans = plan_mip_streaming(
            [large, small],
            MipStreamingSettings {
                max_transitions: 2,
                max_upload_bytes: 32,
                max_resident_bytes: u64::MAX,
                current_resident_bytes: 0,
                hysteresis_mips: 0,
                mip_bias: 0,
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].texture,
            ResourceId::from_stable_label("mip-streaming-small")
        );
        assert_eq!(plans[0].upload_bytes, 16);
    }

    #[test]
    fn render_mip_streaming_respects_persistent_texture_budget_before_promotion() {
        let promotion = demand("mip-streaming-budget-promotion", 6, 2..6, u16::MAX, true, 0)
            .with_upload_bytes(32)
            .with_resident_bytes(64, 128);
        let eviction = demand("mip-streaming-budget-eviction", 6, 0..6, 0, true, 1)
            .with_resident_bytes(128, 64);
        let plans = plan_mip_streaming(
            [promotion, eviction],
            MipStreamingSettings {
                max_transitions: 2,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: 96,
                current_resident_bytes: 128,
                hysteresis_mips: 0,
                mip_bias: 0,
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].texture,
            ResourceId::from_stable_label("mip-streaming-budget-eviction")
        );
        assert!(plans[0].wanted_bytes < plans[0].resident_bytes);
    }

    #[test]
    fn render_mip_streaming_evicts_lowest_priority_texture_before_global_mip_bias() {
        let high_priority = demand("mip-streaming-budget-high", 6, 0..6, u16::MAX, true, 0)
            .with_resident_bytes(128, 128);
        let low_priority =
            demand("mip-streaming-budget-low", 6, 0..6, 0, true, 1).with_resident_bytes(128, 64);
        let plans = plan_mip_streaming(
            [high_priority, low_priority],
            MipStreamingSettings {
                max_transitions: 1,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: 96,
                current_resident_bytes: 256,
                hysteresis_mips: 0,
                mip_bias: 0,
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].texture,
            ResourceId::from_stable_label("mip-streaming-budget-low")
        );
        assert!(plans[0].wanted_mips.start > plans[0].resident_mips.start);
    }

    #[test]
    fn render_mip_streaming_hysteresis_avoids_single_mip_thrash() {
        let demand = demand("mip-streaming-hysteresis", 5, 1..5, u16::MAX, true, 0);

        assert!(plan_mip_streaming(
            [demand.clone()],
            MipStreamingSettings {
                max_transitions: 1,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: u64::MAX,
                current_resident_bytes: 0,
                hysteresis_mips: 1,
                mip_bias: 0,
            },
        )
        .is_empty());

        let plans = plan_mip_streaming(
            [demand],
            MipStreamingSettings {
                max_transitions: 1,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: u64::MAX,
                current_resident_bytes: 0,
                hysteresis_mips: 0,
                mip_bias: 0,
            },
        );
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].wanted_mips, 0..5);
    }

    #[test]
    fn render_mip_streaming_keeps_the_tail_mip_resident() {
        let plans = plan_mip_streaming(
            [demand("mip-streaming-tail", 5, 9..10, u16::MAX, true, 0)],
            MipStreamingSettings {
                max_transitions: 1,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: u64::MAX,
                current_resident_bytes: 0,
                hysteresis_mips: 0,
                mip_bias: 0,
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].resident_mips, 4..5);
        assert_eq!(plans[0].wanted_mips, 0..5);
    }

    #[test]
    fn render_mip_streaming_disabled_texture_restores_full_residency_without_hysteresis() {
        let plans = plan_mip_streaming(
            [demand("mip-streaming-disabled", 5, 3..5, 0, false, 0)],
            MipStreamingSettings {
                max_transitions: 1,
                max_upload_bytes: u64::MAX,
                max_resident_bytes: u64::MAX,
                current_resident_bytes: 0,
                hysteresis_mips: u8::MAX,
                mip_bias: 4,
            },
        );

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].resident_mips, 3..5);
        assert_eq!(plans[0].wanted_mips, 0..5);
    }

    #[test]
    fn render_mip_streaming_coalesces_visible_instances_by_texture_and_coverage() {
        let texture = ResourceId::from_stable_label("mip-streaming-shared");
        let other_texture = ResourceId::from_stable_label("mip-streaming-other");
        let coalesced = coalesce_mip_streaming_visibility([
            MipStreamingVisibility {
                texture,
                screen_coverage: 2_000,
                stable_order: 9,
            },
            MipStreamingVisibility {
                texture,
                screen_coverage: 6_000,
                stable_order: 8,
            },
            MipStreamingVisibility {
                texture,
                screen_coverage: 6_000,
                stable_order: 3,
            },
            MipStreamingVisibility {
                texture: other_texture,
                screen_coverage: 4_000,
                stable_order: 4,
            },
        ]);

        assert_eq!(coalesced.len(), 2);
        let selected = coalesced.get(&texture).expect("shared texture is retained");
        assert_eq!(selected.screen_coverage, 6_000);
        assert_eq!(selected.stable_order, 3);
    }

    #[test]
    fn render_mip_streaming_keeps_offscreen_resident_textures_eligible_for_budget_eviction() {
        let visible = ResourceId::from_stable_label("mip-streaming-visible");
        let offscreen = ResourceId::from_stable_label("mip-streaming-offscreen");
        let visibility = include_non_visible_resident_texture_visibility(
            HashMap::from([(
                visible,
                MipStreamingVisibility {
                    texture: visible,
                    screen_coverage: u16::MAX,
                    stable_order: 7,
                },
            )]),
            [offscreen, visible],
        );

        assert_eq!(visibility.len(), 2);
        assert_eq!(visibility[&visible].screen_coverage, u16::MAX);
        assert_eq!(visibility[&offscreen].screen_coverage, 0);
        assert!(visibility[&offscreen].stable_order > visibility[&visible].stable_order);
    }

    #[test]
    fn render_mip_streaming_state_commits_only_a_successful_promotion() {
        let plan = MipStreamingPlan {
            texture: ResourceId::from_stable_label("mip-streaming-state"),
            resident_mips: 2..6,
            wanted_mips: 0..6,
            priority: u32::from(u16::MAX),
            upload_bytes: 0,
            resident_bytes: 0,
            wanted_bytes: 0,
        };
        let mut state = MipStreamingState::default();

        let task = state
            .begin(plan.clone())
            .expect("the initial promotion is scheduled");
        assert_eq!(task.kind, MipStreamingTransitionKind::Promotion);
        assert!(state.begin(plan.clone()).is_none());
        assert_eq!(state.finish(&task, false), None);

        let retry = state.begin(plan).expect("a failed task can be retried");
        assert_eq!(state.finish(&retry, true), Some(0..6));
    }

    #[test]
    fn render_mip_streaming_state_classifies_eviction_and_rejects_stale_completion() {
        let plan = MipStreamingPlan {
            texture: ResourceId::from_stable_label("mip-streaming-eviction"),
            resident_mips: 0..6,
            wanted_mips: 3..6,
            priority: 0,
            upload_bytes: 0,
            resident_bytes: 0,
            wanted_bytes: 0,
        };
        let mut state = MipStreamingState::default();
        let first = state.begin(plan.clone()).expect("first task is scheduled");
        assert_eq!(first.kind, MipStreamingTransitionKind::Eviction);
        assert_eq!(state.finish(&first, false), None);

        let retry = state.begin(plan).expect("failed eviction is retried");
        assert_eq!(state.finish(&first, true), None);
        assert_eq!(state.finish(&retry, true), Some(3..6));
    }

    #[test]
    fn render_mip_streaming_screen_coverage_tracks_projection_without_understreaming_overrides() {
        let camera = crate::core::framework::render::ViewportCameraSnapshot::default();
        let near =
            quantized_screen_coverage(&camera, crate::core::math::Vec3::new(0.0, 0.0, -4.0), 1.0);
        let far =
            quantized_screen_coverage(&camera, crate::core::math::Vec3::new(0.0, 0.0, -16.0), 1.0);
        assert!(near > far);
        assert!(far > 0);

        let mut orthographic = camera.clone();
        orthographic.projection_mode = crate::core::framework::render::ProjectionMode::Orthographic;
        orthographic.ortho_size = 4.0;
        assert_eq!(
            quantized_screen_coverage(
                &orthographic,
                crate::core::math::Vec3::new(0.0, 0.0, -4.0),
                1.0,
            ),
            quantized_screen_coverage(
                &orthographic,
                crate::core::math::Vec3::new(0.0, 0.0, -32.0),
                1.0,
            )
        );

        orthographic.projection_override = Some(crate::core::math::Mat4::IDENTITY);
        assert_eq!(
            quantized_screen_coverage(
                &orthographic,
                crate::core::math::Vec3::new(0.0, 0.0, -32.0),
                1.0,
            ),
            u16::MAX
        );
    }

    #[test]
    fn render_mip_streaming_rebuild_commits_only_after_matching_success() {
        let source = include_str!("resource_streamer_mip_streaming.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("mip streaming implementation before tests");
        let rebuild = implementation
            .find("self.rebuild_texture_mip_streaming_task(")
            .expect("streaming task rebuilds a replacement resource");
        let finish = implementation
            .find("self.finish_texture_mip_streaming_task(&task, true)")
            .expect("replacement waits for matching successful completion");
        let publish = implementation
            .find("self.textures.insert(")
            .expect("successful task atomically publishes prepared texture");

        assert!(rebuild < finish);
        assert!(finish < publish);
    }
}

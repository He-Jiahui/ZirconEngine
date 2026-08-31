use crate::core::math::{Real, UVec2};

use super::super::{camera::RenderViewportRect, post_process::RenderOutputTransfer};
use super::{
    RenderDynamicResolutionDecision, RenderResolutionPlan, RenderResolutionPolicy,
    RenderTemporalHistoryKey, RenderUpscalerKind, RenderViewFamilyPhaseTargets,
    RenderViewFamilyTarget,
};

/// Stable phases for the view-family render graph. Feature descriptors attach to a phase instead
/// of encoding their order through a single monolithic post-process list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderPipelinePhase {
    SceneLinear,
    PreReconstructionScenePostProcess,
    TemporalReconstruction,
    PostReconstructionScenePostProcess,
    DisplayMapping,
    DisplayPostProcess,
    PrimarySpatialUpscale,
    SecondarySpatialUpscale,
    OutputTransform,
    Present,
}

impl RenderPipelinePhase {
    /// Returns the canonical graph order without relying on enum declaration order.
    pub const fn order(self) -> u8 {
        match self {
            Self::SceneLinear => 0,
            Self::PreReconstructionScenePostProcess => 1,
            Self::TemporalReconstruction => 2,
            Self::PostReconstructionScenePostProcess => 3,
            Self::DisplayMapping => 4,
            Self::DisplayPostProcess => 5,
            Self::PrimarySpatialUpscale => 6,
            Self::SecondarySpatialUpscale => 7,
            Self::OutputTransform => 8,
            Self::Present => 9,
        }
    }
}

/// Resolved frame pipeline shared by resource allocation, graph compilation, and presentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewFamilyPipeline {
    resolution: RenderResolutionPlan,
    upscaler: RenderUpscalerKind,
    output_transfer: RenderOutputTransfer,
    phases: [RenderPipelinePhase; 10],
    phase_count: usize,
}

impl RenderViewFamilyPipeline {
    pub fn resolve(
        display_extent: UVec2,
        policy: RenderResolutionPolicy,
        upscaler: RenderUpscalerKind,
    ) -> Self {
        let display_extent = sanitize_extent(display_extent);
        Self::resolve_for_viewport(
            display_extent,
            RenderViewportRect::new(UVec2::ZERO, display_extent),
            policy,
            upscaler,
        )
    }

    pub fn resolve_for_viewport(
        display_extent: UVec2,
        display_viewport: RenderViewportRect,
        policy: RenderResolutionPolicy,
        upscaler: RenderUpscalerKind,
    ) -> Self {
        Self::resolve_for_viewport_with_output(
            display_extent,
            display_viewport,
            policy,
            upscaler,
            RenderOutputTransfer::default(),
        )
    }

    /// Resolves a view family from the runtime owner's immutable dynamic-resolution decision.
    ///
    /// The graphics runtime must validate that `decision.scope()` belongs to the current
    /// viewport and upscaler before calling this neutral contract. This layer only preserves the
    /// decision's primary fraction while retaining the caller-selected secondary transition and
    /// allocation alignment.
    pub fn resolve_for_viewport_with_dynamic_resolution_decision(
        display_extent: UVec2,
        display_viewport: RenderViewportRect,
        policy: RenderResolutionPolicy,
        upscaler: RenderUpscalerKind,
        decision: RenderDynamicResolutionDecision,
    ) -> Self {
        Self::resolve_for_viewport(
            display_extent,
            display_viewport,
            policy.with_primary_fraction(decision.primary_fraction()),
            upscaler,
        )
    }

    pub fn resolve_with_output(
        display_extent: UVec2,
        policy: RenderResolutionPolicy,
        upscaler: RenderUpscalerKind,
        output_transfer: RenderOutputTransfer,
    ) -> Self {
        let display_extent = sanitize_extent(display_extent);
        Self::resolve_for_viewport_with_output(
            display_extent,
            RenderViewportRect::new(UVec2::ZERO, display_extent),
            policy,
            upscaler,
            output_transfer,
        )
    }

    pub fn resolve_for_viewport_with_output(
        display_extent: UVec2,
        display_viewport: RenderViewportRect,
        policy: RenderResolutionPolicy,
        upscaler: RenderUpscalerKind,
        output_transfer: RenderOutputTransfer,
    ) -> Self {
        let display_extent = sanitize_extent(display_extent);
        let display_viewport = clamp_viewport_to_display(display_viewport, display_extent);
        let secondary_viewport = scale_viewport(display_viewport, policy.secondary_fraction());
        let primary_viewport = scale_viewport(
            display_viewport,
            policy.primary_fraction() * policy.secondary_fraction(),
        );
        let secondary_allocation_extent =
            allocation_extent_for(secondary_viewport, policy.alignment());
        let primary_allocation_extent = allocation_extent_for(primary_viewport, policy.alignment());
        let temporal_history_extent =
            (upscaler == RenderUpscalerKind::Temporal).then_some(secondary_allocation_extent);
        let resolution = RenderResolutionPlan {
            display_extent,
            display_viewport,
            secondary_viewport,
            primary_viewport,
            secondary_allocation_extent,
            primary_allocation_extent,
            temporal_history_extent,
        };
        let mut phases = [RenderPipelinePhase::Present; 10];
        let mut phase_count = 0;
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::SceneLinear,
        );
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::PreReconstructionScenePostProcess,
        );
        if upscaler == RenderUpscalerKind::Temporal {
            push_phase(
                &mut phases,
                &mut phase_count,
                RenderPipelinePhase::TemporalReconstruction,
            );
        }
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::PostReconstructionScenePostProcess,
        );
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::DisplayMapping,
        );
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::DisplayPostProcess,
        );
        if requires_primary_spatial_upscale(resolution, upscaler) {
            push_phase(
                &mut phases,
                &mut phase_count,
                RenderPipelinePhase::PrimarySpatialUpscale,
            );
        }
        if requires_secondary_spatial_upscale(resolution) {
            push_phase(
                &mut phases,
                &mut phase_count,
                RenderPipelinePhase::SecondarySpatialUpscale,
            );
        }
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::OutputTransform,
        );
        push_phase(&mut phases, &mut phase_count, RenderPipelinePhase::Present);

        Self {
            resolution,
            upscaler,
            output_transfer,
            phases,
            phase_count,
        }
    }

    pub const fn resolution(self) -> RenderResolutionPlan {
        self.resolution
    }

    pub const fn upscaler(self) -> RenderUpscalerKind {
        self.upscaler
    }

    pub const fn output_transfer(self) -> RenderOutputTransfer {
        self.output_transfer
    }

    pub fn phases(&self) -> &[RenderPipelinePhase] {
        &self.phases[..self.phase_count]
    }

    /// Returns the phase output geometry from the single resolved view-family plan.
    ///
    /// Submission, allocation, viewport/scissor setup, and presentation use this contract rather
    /// than independently reconstructing a size from a global render-resolution scalar.
    pub fn output_target_for_phase(
        &self,
        phase: RenderPipelinePhase,
    ) -> Option<RenderViewFamilyTarget> {
        self.phase_targets(phase)
            .map(RenderViewFamilyPhaseTargets::output)
    }

    /// Returns the source and destination geometry for one enabled phase.
    ///
    /// The temporal path reconstructs the primary scene into secondary space. The optional
    /// spatial path then scales that post-process result into display space. This is deliberately
    /// a phase boundary contract, not a texture registry: graph compilation remains free to alias
    /// compatible resources while executors retain one authoritative viewport/scissor pair.
    pub fn phase_targets(
        &self,
        phase: RenderPipelinePhase,
    ) -> Option<RenderViewFamilyPhaseTargets> {
        self.phases().contains(&phase).then(|| {
            let primary = self.primary_target();
            let post_process = self.post_process_target();
            let display = self.display_target();
            match phase {
                RenderPipelinePhase::SceneLinear => RenderViewFamilyPhaseTargets {
                    input: None,
                    output: primary,
                },
                RenderPipelinePhase::PreReconstructionScenePostProcess => {
                    RenderViewFamilyPhaseTargets {
                        input: Some(primary),
                        output: primary,
                    }
                }
                RenderPipelinePhase::TemporalReconstruction => RenderViewFamilyPhaseTargets {
                    input: Some(primary),
                    output: self.secondary_target(),
                },
                RenderPipelinePhase::PostReconstructionScenePostProcess
                | RenderPipelinePhase::DisplayMapping
                | RenderPipelinePhase::DisplayPostProcess => RenderViewFamilyPhaseTargets {
                    input: Some(post_process),
                    output: post_process,
                },
                RenderPipelinePhase::PrimarySpatialUpscale => RenderViewFamilyPhaseTargets {
                    input: Some(primary),
                    output: self.secondary_target(),
                },
                RenderPipelinePhase::SecondarySpatialUpscale => RenderViewFamilyPhaseTargets {
                    input: Some(self.secondary_target()),
                    output: display,
                },
                RenderPipelinePhase::OutputTransform | RenderPipelinePhase::Present => {
                    RenderViewFamilyPhaseTargets {
                        input: Some(display),
                        output: display,
                    }
                }
            }
        })
    }

    pub fn temporal_history_key(&self) -> Option<RenderTemporalHistoryKey> {
        self.resolution
            .temporal_history_extent()
            .map(|history_extent| RenderTemporalHistoryKey {
                display_extent: self.resolution.display_extent(),
                history_viewport_position: self.resolution.secondary_viewport().physical_position,
                history_viewport_size: self.resolution.secondary_viewport().physical_size,
                history_allocation_extent: history_extent,
                upscaler: self.upscaler,
            })
    }

    const fn primary_target(self) -> RenderViewFamilyTarget {
        RenderViewFamilyTarget {
            viewport: self.resolution.primary_viewport(),
            allocation_extent: self.resolution.primary_allocation_extent(),
        }
    }

    const fn secondary_target(self) -> RenderViewFamilyTarget {
        RenderViewFamilyTarget {
            viewport: self.resolution.secondary_viewport(),
            allocation_extent: self.resolution.secondary_allocation_extent(),
        }
    }

    const fn post_process_target(self) -> RenderViewFamilyTarget {
        match self.upscaler {
            RenderUpscalerKind::Spatial => self.primary_target(),
            RenderUpscalerKind::Temporal => self.secondary_target(),
        }
    }

    const fn display_target(self) -> RenderViewFamilyTarget {
        RenderViewFamilyTarget {
            viewport: self.resolution.display_viewport(),
            allocation_extent: self.resolution.display_extent(),
        }
    }
}

fn sanitize_extent(extent: UVec2) -> UVec2 {
    UVec2::new(extent.x.max(1), extent.y.max(1))
}

fn clamp_viewport_to_display(
    viewport: RenderViewportRect,
    display_extent: UVec2,
) -> RenderViewportRect {
    let display_extent = sanitize_extent(display_extent);
    let origin = UVec2::new(
        viewport
            .physical_position
            .x
            .min(display_extent.x.saturating_sub(1)),
        viewport
            .physical_position
            .y
            .min(display_extent.y.saturating_sub(1)),
    );
    let maximum = viewport_max(viewport);
    let clamped_maximum = UVec2::new(
        maximum.x.min(display_extent.x),
        maximum.y.min(display_extent.y),
    );
    viewport_with_extent(
        viewport,
        origin,
        UVec2::new(
            clamped_maximum.x.saturating_sub(origin.x).max(1),
            clamped_maximum.y.saturating_sub(origin.y).max(1),
        ),
    )
}

fn scale_viewport(viewport: RenderViewportRect, fraction: Real) -> RenderViewportRect {
    let maximum = viewport_max(viewport);
    let scaled_origin = UVec2::new(
        scale_axis_floor(viewport.physical_position.x, fraction),
        scale_axis_floor(viewport.physical_position.y, fraction),
    );
    let scaled_maximum = UVec2::new(
        scale_axis(maximum.x, fraction),
        scale_axis(maximum.y, fraction),
    );
    viewport_with_extent(
        viewport,
        scaled_origin,
        UVec2::new(
            scaled_maximum.x.saturating_sub(scaled_origin.x).max(1),
            scaled_maximum.y.saturating_sub(scaled_origin.y).max(1),
        ),
    )
}

fn viewport_with_extent(
    template: RenderViewportRect,
    physical_position: UVec2,
    physical_size: UVec2,
) -> RenderViewportRect {
    RenderViewportRect {
        physical_position,
        physical_size,
        depth_min: template.depth_min,
        depth_max: template.depth_max,
    }
}

fn scale_axis(extent: u32, fraction: Real) -> u32 {
    let extent = extent.max(1);
    ((extent as Real * fraction).ceil() as u32).clamp(1, extent)
}

fn scale_axis_floor(position: u32, fraction: Real) -> u32 {
    (position as Real * fraction).floor() as u32
}

fn allocation_extent_for(viewport: RenderViewportRect, alignment: UVec2) -> UVec2 {
    let maximum = viewport_max(viewport);
    UVec2::new(
        align_up(maximum.x, alignment.x),
        align_up(maximum.y, alignment.y),
    )
}

fn viewport_max(viewport: RenderViewportRect) -> UVec2 {
    UVec2::new(
        viewport
            .physical_position
            .x
            .saturating_add(viewport.physical_size.x),
        viewport
            .physical_position
            .y
            .saturating_add(viewport.physical_size.y),
    )
}

fn align_up(value: u32, alignment: u32) -> u32 {
    let alignment = alignment.max(1);
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value.saturating_add(alignment - remainder)
    }
}

fn requires_primary_spatial_upscale(
    resolution: RenderResolutionPlan,
    upscaler: RenderUpscalerKind,
) -> bool {
    upscaler == RenderUpscalerKind::Spatial
        && resolution.primary_viewport() != resolution.secondary_viewport()
}

fn requires_secondary_spatial_upscale(resolution: RenderResolutionPlan) -> bool {
    resolution.secondary_viewport() != resolution.display_viewport()
}

fn push_phase(
    phases: &mut [RenderPipelinePhase; 10],
    phase_count: &mut usize,
    phase: RenderPipelinePhase,
) {
    phases[*phase_count] = phase;
    *phase_count += 1;
}

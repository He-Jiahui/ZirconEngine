use crate::core::math::{Real, UVec2};

use super::{camera::RenderViewportRect, post_process::RenderOutputTransfer};

pub const MIN_RENDER_RESOLUTION_FRACTION: Real = 0.1;
pub const MAX_RENDER_RESOLUTION_FRACTION: Real = 1.0;

/// Describes the output-space and internal-resolution contract for one view family.
///
/// The display extent always remains the presenter-facing physical extent. Primary and secondary
/// fractions only affect intermediate resources, so dynamic resolution cannot resize the output
/// surface or invalidate unrelated presentation products.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderResolutionPolicy {
    primary_fraction: Real,
    secondary_fraction: Real,
    alignment: UVec2,
}

impl Default for RenderResolutionPolicy {
    fn default() -> Self {
        Self::with_scales(
            MAX_RENDER_RESOLUTION_FRACTION,
            MAX_RENDER_RESOLUTION_FRACTION,
        )
    }
}

impl RenderResolutionPolicy {
    pub fn with_scales(primary_fraction: Real, secondary_fraction: Real) -> Self {
        Self {
            primary_fraction: normalize_fraction(primary_fraction),
            secondary_fraction: normalize_fraction(secondary_fraction),
            alignment: UVec2::new(8, 8),
        }
    }

    /// Resolves the direct primary-to-display spatial path used by the current MVP.
    ///
    /// A separate secondary spatial fraction is intentionally not accepted here. Supporting two
    /// spatial transitions requires the M4-S3 primary/secondary graph-node hard cut rather than
    /// overloading the current single upscale node.
    pub fn with_spatial_primary_fraction(primary_fraction: Real) -> Self {
        Self::with_scales(primary_fraction, MAX_RENDER_RESOLUTION_FRACTION)
    }

    /// Resolves a temporal primary-to-secondary reconstruction path.
    pub fn with_temporal_fractions(primary_fraction: Real, secondary_fraction: Real) -> Self {
        Self::with_scales(primary_fraction, secondary_fraction)
    }

    pub fn with_alignment(mut self, alignment: UVec2) -> Self {
        self.alignment = UVec2::new(alignment.x.max(1), alignment.y.max(1));
        self
    }

    pub const fn primary_fraction(self) -> Real {
        self.primary_fraction
    }

    pub const fn secondary_fraction(self) -> Real {
        self.secondary_fraction
    }

    pub const fn alignment(self) -> UVec2 {
        self.alignment
    }

    /// Replaces only the primary render fraction while preserving the secondary transition and
    /// allocation alignment selected for this ViewFamily.
    pub fn with_primary_fraction(mut self, primary_fraction: Real) -> Self {
        self.primary_fraction = normalize_fraction(primary_fraction);
        self
    }
}

/// A bounded dynamic-resolution controller driven by GPU frame time.
///
/// Raster cost is approximately proportional to pixel count, so the controller uses square-root
/// feedback to convert a time ratio into a linear resolution fraction. Hysteresis and a bounded
/// per-frame step prevent frame-time noise from causing visible scale oscillation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDynamicResolutionController {
    min_primary_fraction: Real,
    max_primary_fraction: Real,
    target_frame_time_ms: Real,
    max_fraction_step: Real,
    hysteresis_ms: Real,
}

impl RenderDynamicResolutionController {
    pub fn new(
        min_primary_fraction: Real,
        max_primary_fraction: Real,
        target_frame_time_ms: Real,
        max_fraction_step: Real,
        hysteresis_ms: Real,
    ) -> Self {
        let first_fraction = normalize_fraction(min_primary_fraction);
        let second_fraction = normalize_fraction(max_primary_fraction);
        Self {
            min_primary_fraction: first_fraction.min(second_fraction),
            max_primary_fraction: first_fraction.max(second_fraction),
            target_frame_time_ms: normalize_positive(target_frame_time_ms, 16.6),
            max_fraction_step: normalize_nonnegative(max_fraction_step).min(1.0),
            hysteresis_ms: normalize_nonnegative(hysteresis_ms),
        }
    }

    pub fn next_primary_fraction(
        self,
        current_primary_fraction: Real,
        gpu_frame_time_ms: Real,
    ) -> Real {
        let current =
            current_primary_fraction.clamp(self.min_primary_fraction, self.max_primary_fraction);
        if !gpu_frame_time_ms.is_finite() || gpu_frame_time_ms <= 0.0 {
            return current;
        }
        if (gpu_frame_time_ms - self.target_frame_time_ms).abs() <= self.hysteresis_ms {
            return current;
        }
        let desired = (current * (self.target_frame_time_ms / gpu_frame_time_ms).sqrt())
            .clamp(self.min_primary_fraction, self.max_primary_fraction);
        let bounded_delta =
            (desired - current).clamp(-self.max_fraction_step, self.max_fraction_step);
        (current + bounded_delta).clamp(self.min_primary_fraction, self.max_primary_fraction)
    }

    pub const fn primary_upper_bound(self) -> Real {
        self.max_primary_fraction
    }
}

/// Identifies the view-family state allowed to consume a GPU timing result.
///
/// A viewport generation changes on resize or device recreation. The reconstruction category is
/// part of the scope because changing it also changes temporal-history compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderDynamicResolutionScope {
    view_family_id: u64,
    viewport_generation: u64,
    upscaler: RenderUpscalerKind,
}

impl RenderDynamicResolutionScope {
    pub const fn new(
        view_family_id: u64,
        viewport_generation: u64,
        upscaler: RenderUpscalerKind,
    ) -> Self {
        Self {
            view_family_id,
            viewport_generation,
            upscaler,
        }
    }

    pub const fn view_family_id(self) -> u64 {
        self.view_family_id
    }

    pub const fn viewport_generation(self) -> u64 {
        self.viewport_generation
    }

    pub const fn upscaler(self) -> RenderUpscalerKind {
        self.upscaler
    }
}

/// A delayed timing result published by the render-device owner after submission completes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderDynamicResolutionGpuSample {
    Completed {
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
        gpu_frame_time_ms: Real,
    },
    Unavailable {
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    },
    TimedOut {
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    },
}

impl RenderDynamicResolutionGpuSample {
    pub const fn completed(
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
        gpu_frame_time_ms: Real,
    ) -> Self {
        Self::Completed {
            scope,
            source_frame_generation,
            gpu_frame_time_ms,
        }
    }

    pub const fn unavailable(
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    ) -> Self {
        Self::Unavailable {
            scope,
            source_frame_generation,
        }
    }

    pub const fn timed_out(
        scope: RenderDynamicResolutionScope,
        source_frame_generation: u64,
    ) -> Self {
        Self::TimedOut {
            scope,
            source_frame_generation,
        }
    }

    pub const fn scope(self) -> RenderDynamicResolutionScope {
        match self {
            Self::Completed { scope, .. }
            | Self::Unavailable { scope, .. }
            | Self::TimedOut { scope, .. } => scope,
        }
    }

    pub const fn source_frame_generation(self) -> u64 {
        match self {
            Self::Completed {
                source_frame_generation,
                ..
            }
            | Self::Unavailable {
                source_frame_generation,
                ..
            }
            | Self::TimedOut {
                source_frame_generation,
                ..
            } => source_frame_generation,
        }
    }
}

/// Explains why a frame received its immutable dynamic-resolution decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderDynamicResolutionDecisionReason {
    Disabled,
    Unsupported,
    AwaitingGpuSample,
    CompletedGpuSample,
    UnavailableGpuSample,
    TimedOutGpuSample,
}

/// Immutable dynamic-resolution input to one ViewFamily plan resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDynamicResolutionDecision {
    scope: RenderDynamicResolutionScope,
    decision_generation: u64,
    source_frame_generation: Option<u64>,
    primary_fraction: Real,
    primary_upper_bound: Real,
    reason: RenderDynamicResolutionDecisionReason,
    requires_temporal_history_reset: bool,
}

impl RenderDynamicResolutionDecision {
    pub fn new(
        scope: RenderDynamicResolutionScope,
        decision_generation: u64,
        source_frame_generation: Option<u64>,
        primary_fraction: Real,
        primary_upper_bound: Real,
        reason: RenderDynamicResolutionDecisionReason,
        requires_temporal_history_reset: bool,
    ) -> Self {
        let primary_upper_bound = normalize_fraction(primary_upper_bound);
        Self {
            scope,
            decision_generation,
            source_frame_generation,
            primary_fraction: normalize_fraction(primary_fraction).min(primary_upper_bound),
            primary_upper_bound,
            reason,
            requires_temporal_history_reset,
        }
    }

    pub const fn scope(self) -> RenderDynamicResolutionScope {
        self.scope
    }

    pub const fn decision_generation(self) -> u64 {
        self.decision_generation
    }

    pub const fn source_frame_generation(self) -> Option<u64> {
        self.source_frame_generation
    }

    pub const fn primary_fraction(self) -> Real {
        self.primary_fraction
    }

    pub const fn primary_upper_bound(self) -> Real {
        self.primary_upper_bound
    }

    pub const fn reason(self) -> RenderDynamicResolutionDecisionReason {
        self.reason
    }

    pub const fn requires_temporal_history_reset(self) -> bool {
        self.requires_temporal_history_reset
    }
}

/// A backend-neutral decision made once per view family before graph resource allocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderResolutionPlan {
    display_extent: UVec2,
    display_viewport: RenderViewportRect,
    secondary_viewport: RenderViewportRect,
    primary_viewport: RenderViewportRect,
    secondary_allocation_extent: UVec2,
    primary_allocation_extent: UVec2,
    temporal_history_extent: Option<UVec2>,
}

impl RenderResolutionPlan {
    pub const fn display_extent(self) -> UVec2 {
        self.display_extent
    }

    pub const fn display_viewport(self) -> RenderViewportRect {
        self.display_viewport
    }

    pub const fn secondary_viewport(self) -> RenderViewportRect {
        self.secondary_viewport
    }

    pub const fn primary_viewport(self) -> RenderViewportRect {
        self.primary_viewport
    }

    pub const fn secondary_extent(self) -> UVec2 {
        self.secondary_viewport.physical_size
    }

    pub const fn primary_extent(self) -> UVec2 {
        self.primary_viewport.physical_size
    }

    pub const fn secondary_allocation_extent(self) -> UVec2 {
        self.secondary_allocation_extent
    }

    pub const fn primary_allocation_extent(self) -> UVec2 {
        self.primary_allocation_extent
    }

    pub const fn temporal_history_extent(self) -> Option<UVec2> {
        self.temporal_history_extent
    }
}

/// The output geometry a render phase must use when writing a view-family resource.
///
/// `viewport` remains the logical scissor and projection space. `allocation_extent` describes
/// only the backing image, which may be padded for backend alignment. Consumers must never use
/// the padded size as the logical viewport or presentation extent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewFamilyTarget {
    viewport: RenderViewportRect,
    allocation_extent: UVec2,
}

impl RenderViewFamilyTarget {
    pub const fn viewport(self) -> RenderViewportRect {
        self.viewport
    }

    pub const fn allocation_extent(self) -> UVec2 {
        self.allocation_extent
    }
}

/// The geometry contract for one graph phase.
///
/// Render graph resources have their own identities, but every producer and consumer needs the
/// same logical viewport and backing allocation for a phase boundary. Keeping that geometry here
/// prevents TAA, post process, spatial upscale, and presentation from each reconstructing a
/// private interpretation of the current resolution scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewFamilyPhaseTargets {
    input: Option<RenderViewFamilyTarget>,
    output: RenderViewFamilyTarget,
}

impl RenderViewFamilyPhaseTargets {
    /// Scene rendering has no prior color target in the view-family pipeline.
    pub const fn input(self) -> Option<RenderViewFamilyTarget> {
        self.input
    }

    pub const fn output(self) -> RenderViewFamilyTarget {
        self.output
    }
}

/// Persistent temporal-history identity.
///
/// Primary resolution is intentionally excluded: temporal reconstruction accepts changing input
/// resolution, while a display, secondary viewport, or padded history allocation change requires
/// fresh history.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderTemporalHistoryKey {
    display_extent: UVec2,
    history_viewport_position: UVec2,
    history_viewport_size: UVec2,
    history_allocation_extent: UVec2,
    upscaler: RenderUpscalerKind,
}

impl RenderTemporalHistoryKey {
    pub const fn display_extent(self) -> UVec2 {
        self.display_extent
    }

    pub const fn history_viewport_position(self) -> UVec2 {
        self.history_viewport_position
    }

    pub const fn history_viewport_size(self) -> UVec2 {
        self.history_viewport_size
    }

    pub const fn history_allocation_extent(self) -> UVec2 {
        self.history_allocation_extent
    }

    pub const fn upscaler(self) -> RenderUpscalerKind {
        self.upscaler
    }
}

/// The reconstruction algorithm category, deliberately independent of vendor SDK names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderUpscalerKind {
    Spatial,
    Temporal,
}

/// Stable phases for the view-family render graph. Feature descriptors attach to a phase instead
/// of encoding their order through a single monolithic post-process list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderPipelinePhase {
    SceneLinear,
    TemporalReconstruction,
    SceneLinearPostProcess,
    DisplayMapping,
    DisplayPostProcess,
    SpatialUpscale,
    OutputTransform,
    Present,
}

impl RenderPipelinePhase {
    /// Returns the canonical graph order without relying on enum declaration order.
    pub const fn order(self) -> u8 {
        match self {
            Self::SceneLinear => 0,
            Self::TemporalReconstruction => 1,
            Self::SceneLinearPostProcess => 2,
            Self::DisplayMapping => 3,
            Self::DisplayPostProcess => 4,
            Self::SpatialUpscale => 5,
            Self::OutputTransform => 6,
            Self::Present => 7,
        }
    }
}

/// Resolved frame pipeline shared by resource allocation, graph compilation, and presentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderViewFamilyPipeline {
    resolution: RenderResolutionPlan,
    upscaler: RenderUpscalerKind,
    output_transfer: RenderOutputTransfer,
    phases: [RenderPipelinePhase; 8],
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
        let mut phases = [RenderPipelinePhase::Present; 8];
        let mut phase_count = 0;
        push_phase(
            &mut phases,
            &mut phase_count,
            RenderPipelinePhase::SceneLinear,
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
            RenderPipelinePhase::SceneLinearPostProcess,
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
        if requires_spatial_upscale(resolution, upscaler) {
            push_phase(
                &mut phases,
                &mut phase_count,
                RenderPipelinePhase::SpatialUpscale,
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
                RenderPipelinePhase::TemporalReconstruction => RenderViewFamilyPhaseTargets {
                    input: Some(primary),
                    output: self.secondary_target(),
                },
                RenderPipelinePhase::SceneLinearPostProcess
                | RenderPipelinePhase::DisplayMapping
                | RenderPipelinePhase::DisplayPostProcess => RenderViewFamilyPhaseTargets {
                    input: Some(post_process),
                    output: post_process,
                },
                RenderPipelinePhase::SpatialUpscale => RenderViewFamilyPhaseTargets {
                    input: Some(post_process),
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

fn normalize_fraction(fraction: Real) -> Real {
    if fraction.is_finite() {
        fraction.clamp(
            MIN_RENDER_RESOLUTION_FRACTION,
            MAX_RENDER_RESOLUTION_FRACTION,
        )
    } else {
        MAX_RENDER_RESOLUTION_FRACTION
    }
}

fn normalize_positive(value: Real, fallback: Real) -> Real {
    (value.is_finite() && value > 0.0)
        .then_some(value)
        .unwrap_or(fallback)
}

fn normalize_nonnegative(value: Real) -> Real {
    (value.is_finite() && value >= 0.0)
        .then_some(value)
        .unwrap_or_default()
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

fn scale_extent(extent: UVec2, fraction: Real) -> UVec2 {
    let extent = sanitize_extent(extent);
    UVec2::new(
        scale_axis(extent.x, fraction),
        scale_axis(extent.y, fraction),
    )
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

fn requires_spatial_upscale(
    resolution: RenderResolutionPlan,
    upscaler: RenderUpscalerKind,
) -> bool {
    match upscaler {
        RenderUpscalerKind::Spatial => {
            resolution.primary_extent() != resolution.display_viewport().physical_size
        }
        RenderUpscalerKind::Temporal => {
            resolution.secondary_extent() != resolution.display_viewport().physical_size
        }
    }
}

fn push_phase(
    phases: &mut [RenderPipelinePhase; 8],
    phase_count: &mut usize,
    phase: RenderPipelinePhase,
) {
    phases[*phase_count] = phase;
    *phase_count += 1;
}

#[cfg(test)]
mod tests {
    use crate::core::math::UVec2;

    use super::{
        RenderOutputTransfer, RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind,
        RenderViewFamilyPipeline, RenderViewportRect,
    };

    #[test]
    fn temporal_reconstruction_keeps_display_and_history_at_secondary_extent() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(3840, 2160),
            RenderResolutionPolicy::with_temporal_fractions(0.75, 0.5),
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            pipeline.resolution().display_extent(),
            UVec2::new(3840, 2160)
        );
        assert_eq!(
            pipeline.resolution().secondary_extent(),
            UVec2::new(1920, 1080)
        );
        assert_eq!(
            pipeline.resolution().primary_extent(),
            UVec2::new(1440, 810)
        );
        assert_eq!(
            pipeline.resolution().temporal_history_extent(),
            Some(UVec2::new(1920, 1080))
        );
    }

    #[test]
    fn temporal_reconstruction_precedes_hdr_post_process_and_display_mapping() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(2.0 / 3.0, 1.0),
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            pipeline.phases(),
            &[
                RenderPipelinePhase::SceneLinear,
                RenderPipelinePhase::TemporalReconstruction,
                RenderPipelinePhase::SceneLinearPostProcess,
                RenderPipelinePhase::DisplayMapping,
                RenderPipelinePhase::DisplayPostProcess,
                RenderPipelinePhase::OutputTransform,
                RenderPipelinePhase::Present,
            ]
        );
        assert_eq!(
            pipeline.output_transfer(),
            RenderOutputTransfer::SrgbNonlinear
        );
    }

    #[test]
    fn secondary_spatial_upscale_runs_after_display_mapping() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 0.5),
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            pipeline.phases(),
            &[
                RenderPipelinePhase::SceneLinear,
                RenderPipelinePhase::TemporalReconstruction,
                RenderPipelinePhase::SceneLinearPostProcess,
                RenderPipelinePhase::DisplayMapping,
                RenderPipelinePhase::DisplayPostProcess,
                RenderPipelinePhase::SpatialUpscale,
                RenderPipelinePhase::OutputTransform,
                RenderPipelinePhase::Present,
            ]
        );
    }

    #[test]
    fn temporal_history_survives_primary_scale_changes_but_not_secondary_scale_changes() {
        let initial = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 1.0),
            RenderUpscalerKind::Temporal,
        )
        .temporal_history_key()
        .expect("temporal reconstruction owns history");
        let primary_scale_changed = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.75, 1.0),
            RenderUpscalerKind::Temporal,
        )
        .temporal_history_key()
        .expect("temporal reconstruction owns history");
        let secondary_scale_changed = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.75, 0.5),
            RenderUpscalerKind::Temporal,
        )
        .temporal_history_key()
        .expect("temporal reconstruction owns history");

        assert_eq!(initial, primary_scale_changed);
        assert_ne!(initial, secondary_scale_changed);
    }

    #[test]
    fn odd_device_extent_preserves_logical_size_and_pads_only_allocations() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1919, 1079),
            RenderResolutionPolicy::with_scales(0.5, 0.5),
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            pipeline.resolution().secondary_extent(),
            UVec2::new(960, 540)
        );
        assert_eq!(
            pipeline.resolution().secondary_allocation_extent(),
            UVec2::new(960, 544)
        );
        assert_eq!(pipeline.resolution().primary_extent(), UVec2::new(480, 270));
        assert_eq!(
            pipeline.resolution().primary_allocation_extent(),
            UVec2::new(480, 272)
        );
        assert_eq!(
            pipeline.resolution().temporal_history_extent(),
            Some(UVec2::new(960, 544))
        );
    }

    #[test]
    fn temporal_history_identity_includes_the_viewport_rect_and_allocation() {
        let policy = RenderResolutionPolicy::with_scales(0.5, 1.0);
        let left = RenderViewFamilyPipeline::resolve_for_viewport(
            UVec2::new(1920, 1080),
            RenderViewportRect::new(UVec2::ZERO, UVec2::new(960, 1080)),
            policy,
            RenderUpscalerKind::Temporal,
        );
        let right = RenderViewFamilyPipeline::resolve_for_viewport(
            UVec2::new(1920, 1080),
            RenderViewportRect::new(UVec2::new(960, 0), UVec2::new(960, 1080)),
            policy,
            RenderUpscalerKind::Temporal,
        );

        assert_eq!(
            right.resolution().primary_viewport(),
            RenderViewportRect::new(UVec2::new(480, 0), UVec2::new(480, 540))
        );
        assert_eq!(
            right.resolution().primary_allocation_extent(),
            UVec2::new(960, 544)
        );
        assert_ne!(left.temporal_history_key(), right.temporal_history_key());
        assert!(!right
            .phases()
            .contains(&RenderPipelinePhase::SpatialUpscale));
    }

    #[test]
    fn non_aligned_viewport_origin_is_scaled_without_allocation_alignment_shift() {
        let pipeline = RenderViewFamilyPipeline::resolve_for_viewport(
            UVec2::new(1919, 1079),
            RenderViewportRect::new(UVec2::new(3, 5), UVec2::new(503, 401)),
            RenderResolutionPolicy::with_scales(0.5, 1.0),
            RenderUpscalerKind::Spatial,
        );

        assert_eq!(
            pipeline.resolution().primary_viewport(),
            RenderViewportRect::new(UVec2::new(1, 2), UVec2::new(252, 201))
        );
        assert_eq!(
            pipeline.resolution().primary_allocation_extent(),
            UVec2::new(256, 208)
        );
    }

    #[test]
    fn viewport_depth_range_survives_clamping_and_resolution_scaling() {
        let pipeline = RenderViewFamilyPipeline::resolve_for_viewport(
            UVec2::new(100, 100),
            RenderViewportRect {
                physical_position: UVec2::new(90, 80),
                physical_size: UVec2::new(20, 30),
                depth_min: 0.2,
                depth_max: 0.8,
            },
            RenderResolutionPolicy::with_scales(0.5, 1.0),
            RenderUpscalerKind::Spatial,
        );

        assert_eq!(
            pipeline.resolution().display_viewport(),
            RenderViewportRect {
                physical_position: UVec2::new(90, 80),
                physical_size: UVec2::new(10, 20),
                depth_min: 0.2,
                depth_max: 0.8,
            }
        );
        assert_eq!(
            pipeline.resolution().primary_viewport(),
            RenderViewportRect {
                physical_position: UVec2::new(45, 40),
                physical_size: UVec2::new(5, 10),
                depth_min: 0.2,
                depth_max: 0.8,
            }
        );
    }

    #[test]
    fn phase_targets_keep_logical_rects_separate_from_padded_allocations() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1919, 1079),
            RenderResolutionPolicy::with_scales(0.5, 0.5),
            RenderUpscalerKind::Temporal,
        );

        let scene_targets = pipeline
            .phase_targets(RenderPipelinePhase::SceneLinear)
            .expect("scene linear phase is always present");
        assert_eq!(scene_targets.input(), None);
        let scene_target = scene_targets.output();
        assert_eq!(scene_target.viewport().physical_size, UVec2::new(480, 270));
        assert_eq!(scene_target.allocation_extent(), UVec2::new(480, 272));

        let temporal_targets = pipeline
            .phase_targets(RenderPipelinePhase::TemporalReconstruction)
            .expect("temporal reconstruction owns a secondary target");
        assert_eq!(temporal_targets.input(), Some(scene_target));
        let temporal_target = temporal_targets.output();
        assert_eq!(
            temporal_target.viewport().physical_size,
            UVec2::new(960, 540)
        );
        assert_eq!(temporal_target.allocation_extent(), UVec2::new(960, 544));

        let display_post_target = pipeline
            .output_target_for_phase(RenderPipelinePhase::DisplayPostProcess)
            .expect("display post process is always present");
        assert_eq!(display_post_target, temporal_target);

        let spatial_targets = pipeline
            .phase_targets(RenderPipelinePhase::SpatialUpscale)
            .expect("secondary lowering requires a spatial output transition");
        assert_eq!(spatial_targets.input(), Some(display_post_target));
        let spatial_target = spatial_targets.output();
        assert_eq!(
            spatial_target.viewport().physical_size,
            UVec2::new(1919, 1079)
        );
        assert_eq!(spatial_target.allocation_extent(), UVec2::new(1919, 1079));

        assert_eq!(
            pipeline.output_target_for_phase(RenderPipelinePhase::OutputTransform),
            Some(spatial_target)
        );
    }

    #[test]
    fn spatial_only_phase_targets_keep_post_process_at_primary_resolution() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1919, 1079),
            RenderResolutionPolicy::with_spatial_primary_fraction(0.5),
            RenderUpscalerKind::Spatial,
        );

        let scene_target = pipeline
            .phase_targets(RenderPipelinePhase::SceneLinear)
            .expect("scene linear phase is always present");
        let display_post_target = pipeline
            .phase_targets(RenderPipelinePhase::DisplayPostProcess)
            .expect("display post process is always present");
        assert_eq!(scene_target.input(), None);
        assert_eq!(display_post_target.input(), Some(scene_target.output()));
        let scene_target = scene_target.output();
        let display_post_target = display_post_target.output();
        assert_eq!(scene_target.viewport().physical_size, UVec2::new(960, 540));
        assert_eq!(scene_target.allocation_extent(), UVec2::new(960, 544));
        assert_eq!(display_post_target, scene_target);

        let spatial_targets = pipeline
            .phase_targets(RenderPipelinePhase::SpatialUpscale)
            .expect("primary lowering requires a spatial output transition");
        assert_eq!(spatial_targets.input(), Some(display_post_target));
        let spatial_target = spatial_targets.output();
        assert_eq!(
            spatial_target.viewport().physical_size,
            UVec2::new(1919, 1079)
        );
        assert_eq!(spatial_target.allocation_extent(), UVec2::new(1919, 1079));
        assert_eq!(
            pipeline.output_target_for_phase(RenderPipelinePhase::OutputTransform),
            Some(spatial_target)
        );
        assert_eq!(
            pipeline.output_target_for_phase(RenderPipelinePhase::Present),
            Some(spatial_target)
        );
    }

    #[test]
    fn spatial_only_pipeline_upscales_after_display_phases() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_spatial_primary_fraction(0.5),
            RenderUpscalerKind::Spatial,
        );

        assert_eq!(
            pipeline.phases(),
            &[
                RenderPipelinePhase::SceneLinear,
                RenderPipelinePhase::SceneLinearPostProcess,
                RenderPipelinePhase::DisplayMapping,
                RenderPipelinePhase::DisplayPostProcess,
                RenderPipelinePhase::SpatialUpscale,
                RenderPipelinePhase::OutputTransform,
                RenderPipelinePhase::Present,
            ]
        );
    }

    #[test]
    fn native_resolution_omits_spatial_upscale_and_keeps_present_unpadded() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1919, 1079),
            RenderResolutionPolicy::default(),
            RenderUpscalerKind::Spatial,
        );

        let scene_target = pipeline
            .output_target_for_phase(RenderPipelinePhase::SceneLinear)
            .expect("scene linear phase is always present");
        assert_eq!(
            scene_target.viewport().physical_size,
            UVec2::new(1919, 1079)
        );
        assert_eq!(scene_target.allocation_extent(), UVec2::new(1920, 1080));
        assert_eq!(
            pipeline.output_target_for_phase(RenderPipelinePhase::SpatialUpscale),
            None
        );

        let present_target = pipeline
            .output_target_for_phase(RenderPipelinePhase::Present)
            .expect("present phase is always present");
        assert_eq!(
            present_target.viewport().physical_size,
            UVec2::new(1919, 1079)
        );
        assert_eq!(present_target.allocation_extent(), UVec2::new(1919, 1079));
        assert_eq!(
            pipeline.output_target_for_phase(RenderPipelinePhase::OutputTransform),
            Some(present_target)
        );
    }

    #[test]
    fn direct_spatial_policy_reserves_secondary_fraction_for_the_next_hard_cut() {
        let policy = RenderResolutionPolicy::with_spatial_primary_fraction(0.5);

        assert_eq!(policy.primary_fraction(), 0.5);
        assert_eq!(policy.secondary_fraction(), 1.0);
    }

    #[test]
    fn dynamic_resolution_decision_is_constructible_by_the_runtime_owner() {
        let scope = RenderDynamicResolutionScope::new(17, 4, RenderUpscalerKind::Temporal);
        let decision = RenderDynamicResolutionDecision::new(
            scope,
            101,
            Some(100),
            0.85,
            1.0,
            RenderDynamicResolutionDecisionReason::CompletedGpuSample,
            false,
        );

        assert_eq!(decision.scope(), scope);
        assert_eq!(decision.decision_generation(), 101);
        assert_eq!(decision.source_frame_generation(), Some(100));
        assert_eq!(decision.primary_fraction(), 0.85);
        assert_eq!(decision.primary_upper_bound(), 1.0);
        assert_eq!(
            decision.reason(),
            RenderDynamicResolutionDecisionReason::CompletedGpuSample
        );
        assert!(!decision.requires_temporal_history_reset());
    }

    #[test]
    fn dynamic_resolution_decision_replaces_only_the_primary_view_family_fraction() {
        let decision = RenderDynamicResolutionDecision::new(
            RenderDynamicResolutionScope::new(17, 4, RenderUpscalerKind::Temporal),
            101,
            Some(100),
            0.5,
            1.0,
            RenderDynamicResolutionDecisionReason::CompletedGpuSample,
            false,
        );
        let pipeline =
            RenderViewFamilyPipeline::resolve_for_viewport_with_dynamic_resolution_decision(
                UVec2::new(1920, 1080),
                RenderViewportRect::new(UVec2::ZERO, UVec2::new(1920, 1080)),
                RenderResolutionPolicy::with_temporal_fractions(0.75, 0.75),
                RenderUpscalerKind::Temporal,
                decision,
            );

        assert_eq!(
            pipeline.resolution().secondary_extent(),
            UVec2::new(1440, 810)
        );
        assert_eq!(pipeline.resolution().primary_extent(), UVec2::new(720, 405));
        assert_eq!(
            pipeline.resolution().temporal_history_extent(),
            Some(UVec2::new(1440, 816))
        );
    }

    #[test]
    fn dynamic_resolution_decision_normalizes_the_primary_fraction_to_its_upper_bound() {
        let decision = RenderDynamicResolutionDecision::new(
            RenderDynamicResolutionScope::new(17, 4, RenderUpscalerKind::Temporal),
            101,
            Some(100),
            2.0,
            0.5,
            RenderDynamicResolutionDecisionReason::CompletedGpuSample,
            false,
        );

        assert_eq!(decision.primary_fraction(), 0.5);
        assert_eq!(decision.primary_upper_bound(), 0.5);
    }

    #[test]
    fn dynamic_resolution_controller_converges_with_bounded_square_root_feedback() {
        let controller = RenderDynamicResolutionController::new(0.5, 1.0, 16.0, 0.1, 0.25);

        assert_eq!(controller.next_primary_fraction(1.0, 64.0), 0.9);
        assert_eq!(controller.next_primary_fraction(0.5, 4.0), 0.6);
        assert_eq!(controller.next_primary_fraction(0.75, 16.1), 0.75);
        assert_eq!(controller.next_primary_fraction(0.75, f32::NAN), 0.75);
    }
}

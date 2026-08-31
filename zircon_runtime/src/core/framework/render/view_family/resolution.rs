use crate::core::math::{Real, UVec2};

use super::super::camera::RenderViewportRect;

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

/// A backend-neutral decision made once per view family before graph resource allocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderResolutionPlan {
    pub(super) display_extent: UVec2,
    pub(super) display_viewport: RenderViewportRect,
    pub(super) secondary_viewport: RenderViewportRect,
    pub(super) primary_viewport: RenderViewportRect,
    pub(super) secondary_allocation_extent: UVec2,
    pub(super) primary_allocation_extent: UVec2,
    pub(super) temporal_history_extent: Option<UVec2>,
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
    pub(super) viewport: RenderViewportRect,
    pub(super) allocation_extent: UVec2,
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
    pub(super) input: Option<RenderViewFamilyTarget>,
    pub(super) output: RenderViewFamilyTarget,
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
    pub(super) display_extent: UVec2,
    pub(super) history_viewport_position: UVec2,
    pub(super) history_viewport_size: UVec2,
    pub(super) history_allocation_extent: UVec2,
    pub(super) upscaler: RenderUpscalerKind,
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

pub(super) fn normalize_fraction(fraction: Real) -> Real {
    if fraction.is_finite() {
        fraction.clamp(
            MIN_RENDER_RESOLUTION_FRACTION,
            MAX_RENDER_RESOLUTION_FRACTION,
        )
    } else {
        MAX_RENDER_RESOLUTION_FRACTION
    }
}

//! Responsive shell auto-layout for the Retained workbench host.

pub use zircon_runtime::ui::layout::solve_axis_constraints;
pub use zircon_runtime_interface::ui::layout::{
    AxisConstraint, BoxConstraints as PaneConstraints, ResolvedAxisConstraint, StretchMode,
    UiFrame as ShellFrame, UiSize as ShellSizePx,
};

mod active_tab;
mod axis_constraint_override;
mod constraints;
mod css_like_constraint;
mod floating_window;
mod geometry;
mod layout_tier;
mod pane_constraint_override;
mod region;
mod region_binding;
mod region_state;
mod resolution_context;
mod shell_region_id;
mod shell_regions_asset;
mod workbench_chrome_metrics;
mod workbench_shell_geometry;
mod workbench_skeleton;

pub use axis_constraint_override::AxisConstraintOverride;
pub(crate) use constraints::fixed_axis;
pub use constraints::{default_constraints_for_content, default_region_constraints};
pub use css_like_constraint::{
    CssLikeConstraint, CssLikeConstraintError, CssLikeConstraintProperty, CssLikeDimension,
    CssLikeEdges, CssLikeGap, CssLikeGridTrack, CssLikeGridTrackBreadth, CssLikeOverflow,
    CssLikeOverflowPair, CssLikeSize, family_for_slot_kind,
};
pub(crate) use floating_window::{clamp_floating_window_frame, default_floating_window_frame};
pub(crate) use geometry::{
    balanced_side_widths_for_budget, compact_bottom_height_limit, compact_side_width_limit,
};
pub use geometry::{
    compute_workbench_shell_geometry, compute_workbench_shell_geometry_with_region_defaults,
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode,
    compute_workbench_shell_geometry_with_scale_mode,
};
#[cfg(test)]
pub(crate) use layout_tier::workbench_layout_defaults;
pub(crate) use layout_tier::{
    WorkbenchLayoutTier, compact_bottom_defaults, compact_side_defaults,
    minimum_document_width_fraction, right_drawer_should_collapse_for_logical_width,
    right_drawer_should_collapse_for_physical_width, window_min_height_limit_for_height,
    window_min_width_limit_for_logical_width, window_min_width_limit_for_physical_width,
    workbench_layout_tier_for_logical_width, workbench_layout_tier_for_physical_width,
    workbench_logical_width_for_scale,
};
pub use pane_constraint_override::PaneConstraintOverride;
pub use region_binding::{
    EditorRegion, EditorRegionRole, RegionBinding, RegionBindingError, WorkbenchConstraintTokenName,
};
pub use resolution_context::{ResolutionContext, ResolutionScaleMode};
pub use shell_region_id::ShellRegionId;
pub use shell_regions_asset::{
    WORKBENCH_SHELL_REGIONS_ASSET_ID, WORKBENCH_SHELL_REGIONS_ASSET_KIND,
    WORKBENCH_SHELL_REGIONS_ASSET_VERSION, WorkbenchShellRegionsAsset,
    WorkbenchShellRegionsAssetError, WorkbenchShellRegionsAssetHeader,
};
pub use workbench_chrome_metrics::WorkbenchChromeMetrics;
pub use workbench_shell_geometry::WorkbenchShellGeometry;
pub use workbench_skeleton::WorkbenchSkeleton;

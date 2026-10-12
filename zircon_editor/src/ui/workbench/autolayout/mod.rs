//! Responsive shell auto-layout for the Retained workbench host.

pub use zircon_runtime::ui::layout::solve_axis_constraints;
pub use zircon_runtime_interface::ui::layout::{
    AxisConstraint, BoxConstraints as PaneConstraints, ResolvedAxisConstraint, StretchMode,
    UiFrame as ShellFrame, UiSize as ShellSizePx,
};

#[path = "active_tab.rs"]
mod active_tab;
#[path = "axis_constraint_override.rs"]
mod axis_constraint_override;
#[path = "constraints/mod.rs"]
mod constraints;
#[path = "css_like_constraint.rs"]
mod css_like_constraint;
#[path = "floating_window.rs"]
mod floating_window;
#[path = "geometry/mod.rs"]
mod geometry;
#[path = "layout_tier.rs"]
mod layout_tier;
#[path = "pane_constraint_override.rs"]
mod pane_constraint_override;
#[path = "region/mod.rs"]
mod region;
#[path = "region_binding/mod.rs"]
mod region_binding;
#[path = "region_preferred_extents.rs"]
mod region_preferred_extents;
#[path = "region_state.rs"]
mod region_state;
#[path = "resolution_context.rs"]
mod resolution_context;
#[path = "shell_region_id.rs"]
mod shell_region_id;
#[path = "shell_regions_asset.rs"]
mod shell_regions_asset;
#[path = "workbench_chrome_metrics.rs"]
mod workbench_chrome_metrics;
#[path = "workbench_shell_geometry.rs"]
mod workbench_shell_geometry;
#[path = "workbench_skeleton.rs"]
mod workbench_skeleton;

pub use axis_constraint_override::AxisConstraintOverride;
pub(crate) use constraints::fixed_axis;
pub use constraints::{default_constraints_for_content, default_region_constraints};
pub use css_like_constraint::{
    family_for_slot_kind, CssLikeConstraint, CssLikeConstraintError, CssLikeConstraintProperty,
    CssLikeDimension, CssLikeEdges, CssLikeGap, CssLikeGridTrack, CssLikeGridTrackBreadth,
    CssLikeOverflow, CssLikeOverflowPair, CssLikeSize,
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
    compact_bottom_defaults, compact_side_defaults, minimum_document_width_fraction,
    right_drawer_should_collapse_for_logical_width,
    right_drawer_should_collapse_for_physical_width, window_min_height_limit_for_height,
    window_min_width_limit_for_logical_width, window_min_width_limit_for_physical_width,
    workbench_layout_tier_for_logical_width, workbench_layout_tier_for_physical_width,
    workbench_logical_width_for_scale, WorkbenchLayoutTier,
};
pub use pane_constraint_override::PaneConstraintOverride;
pub use region_binding::{
    EditorRegion, EditorRegionRole, RegionBinding, RegionBindingError, WorkbenchConstraintTokenName,
};
pub(crate) use region_preferred_extents::LogicalRegionPreferredExtents;
pub use resolution_context::{ResolutionContext, ResolutionScaleMode};
pub use shell_region_id::ShellRegionId;
pub use shell_regions_asset::{
    WorkbenchShellRegionsAsset, WorkbenchShellRegionsAssetError, WorkbenchShellRegionsAssetHeader,
    WORKBENCH_SHELL_REGIONS_ASSET_ID, WORKBENCH_SHELL_REGIONS_ASSET_KIND,
    WORKBENCH_SHELL_REGIONS_ASSET_VERSION,
};
pub use workbench_chrome_metrics::WorkbenchChromeMetrics;
pub use workbench_shell_geometry::WorkbenchShellGeometry;
pub use workbench_skeleton::WorkbenchSkeleton;

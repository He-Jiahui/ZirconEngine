//! Responsive shell auto-layout for the Retained workbench host.

pub use zircon_runtime::ui::layout::solve_axis_constraints;
pub use zircon_runtime_interface::ui::layout::{
    AxisConstraint, BoxConstraints as PaneConstraints, ResolvedAxisConstraint, StretchMode,
    UiFrame as ShellFrame, UiSize as ShellSizePx,
};

mod active_tab;
mod axis_constraint_override;
mod constraints;
mod floating_window;
mod geometry;
mod pane_constraint_override;
mod region;
mod region_binding;
mod region_state;
mod shell_region_id;
mod shell_regions_asset;
mod workbench_chrome_metrics;
mod workbench_shell_geometry;
mod workbench_skeleton;

pub use axis_constraint_override::AxisConstraintOverride;
pub use constraints::{default_constraints_for_content, default_region_constraints};
pub(crate) use floating_window::{clamp_floating_window_frame, default_floating_window_frame};
pub use geometry::compute_workbench_shell_geometry;
pub(crate) use geometry::{compact_bottom_height_limit, compact_side_width_limit};
pub use pane_constraint_override::PaneConstraintOverride;
pub use region_binding::{
    EditorRegion, EditorRegionRole, RegionBinding, RegionBindingError, WorkbenchConstraintTokenName,
};
pub use shell_region_id::ShellRegionId;
pub use shell_regions_asset::{
    WorkbenchShellRegionsAsset, WorkbenchShellRegionsAssetError, WorkbenchShellRegionsAssetHeader,
    WORKBENCH_SHELL_REGIONS_ASSET_ID, WORKBENCH_SHELL_REGIONS_ASSET_KIND,
    WORKBENCH_SHELL_REGIONS_ASSET_VERSION,
};
pub use workbench_chrome_metrics::WorkbenchChromeMetrics;
pub use workbench_shell_geometry::WorkbenchShellGeometry;
pub use workbench_skeleton::WorkbenchSkeleton;

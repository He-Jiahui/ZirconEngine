use crate::ui::workbench::autolayout::{
    EditorRegion, EditorRegionRole, RegionBinding, ResolutionScaleMode, ShellFrame, ShellRegionId,
    ShellSizePx, WORKBENCH_SHELL_REGIONS_ASSET_ID, WORKBENCH_SHELL_REGIONS_ASSET_KIND,
    WORKBENCH_SHELL_REGIONS_ASSET_VERSION, WorkbenchChromeMetrics, WorkbenchConstraintTokenName,
    WorkbenchLayoutTier, WorkbenchShellRegionsAsset, WorkbenchShellRegionsAssetError,
    WorkbenchSkeleton, compact_bottom_height_limit, compact_side_width_limit,
    compute_workbench_shell_geometry, compute_workbench_shell_geometry_with_region_defaults,
    compute_workbench_shell_geometry_with_scale_mode, workbench_layout_defaults,
    workbench_layout_tier_for_logical_width, workbench_layout_tier_for_physical_width,
    workbench_logical_width_for_scale,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, LayoutCommand, LayoutCommandError,
    LayoutManager, MainHostPageLayout, MainPageId, SplitAxis, SplitPlacement, WorkbenchLayout,
    WorkspaceTarget,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};
use crate::ui::workbench::{
    FloatingLayer, FloatingWindow, FloatingWindowKind, LayoutPreset, LayoutPresetName,
    PageLayoutTemplate,
};
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

pub(super) const SHELL_REGIONS_ASSET: &str =
    include_str!("../../../../../assets/ui/editor/layout/shell_regions.toml");

mod breakpoints;
mod geometry;
mod layout_commands;
mod region_contracts;

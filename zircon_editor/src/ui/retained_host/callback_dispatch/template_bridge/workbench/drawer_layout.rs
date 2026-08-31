use std::collections::BTreeMap;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    layout::{AxisConstraint, StretchMode, UiFrame, UiSize},
    tree::UiVisibility,
};

use crate::ui::workbench::autolayout::{
    balanced_side_widths_for_budget, compact_bottom_height_limit, compact_side_width_limit,
    minimum_document_width_fraction, right_drawer_should_collapse_for_logical_width,
    workbench_layout_tier_for_logical_width, ShellRegionId, WorkbenchChromeMetrics,
    WorkbenchLayoutTier,
};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ActivityDrawerSnapshot;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::responsive_layout::apply_workbench_responsive_layout;

pub(super) const LEFT_DRAWER_SHELL_CONTROL_ID: &str = "LeftDrawerShellRoot";
pub(super) const LEFT_DRAWER_HEADER_CONTROL_ID: &str = "LeftDrawerHeaderRoot";
pub(super) const LEFT_DRAWER_CONTENT_CONTROL_ID: &str = "LeftDrawerContentRoot";
pub(super) const RIGHT_DRAWER_SHELL_CONTROL_ID: &str = "RightDrawerShellRoot";
pub(super) const RIGHT_DRAWER_HEADER_CONTROL_ID: &str = "RightDrawerHeaderRoot";
pub(super) const RIGHT_DRAWER_CONTENT_CONTROL_ID: &str = "RightDrawerContentRoot";
pub(super) const BOTTOM_DRAWER_SHELL_CONTROL_ID: &str = "BottomDrawerShellRoot";
pub(super) const BOTTOM_DRAWER_HEADER_CONTROL_ID: &str = "BottomDrawerHeaderRoot";
pub(super) const BOTTOM_DRAWER_CONTENT_CONTROL_ID: &str = "BottomDrawerContentRoot";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct WorkbenchDrawerRegionInput {
    visible: bool,
    extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct WorkbenchDrawerLayoutInputs {
    left: WorkbenchDrawerRegionInput,
    right: WorkbenchDrawerRegionInput,
    bottom: WorkbenchDrawerRegionInput,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorkbenchDrawerLayoutAnchors {
    body_height: f32,
}

impl WorkbenchDrawerLayoutInputs {
    fn from_workbench_model(model: &WorkbenchViewModel, metrics: &WorkbenchChromeMetrics) -> Self {
        Self {
            left: drawer_region_input(
                &model.drawer_ring.drawers,
                &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
                metrics.rail_width,
            ),
            right: drawer_region_input(
                &model.drawer_ring.drawers,
                &[
                    ActivityDrawerSlot::RightTop,
                    ActivityDrawerSlot::RightBottom,
                ],
                metrics.rail_width,
            ),
            bottom: drawer_region_input(
                &model.drawer_ring.drawers,
                &[ActivityDrawerSlot::Bottom],
                metrics.panel_header_height,
            ),
        }
    }
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn recompute_layout_with_workbench_model(
        &mut self,
        shell_size: UiSize,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.recompute_layout_with_workbench_model_at_scale(shell_size, 1.0, model, metrics)
    }

    pub(crate) fn recompute_layout_with_workbench_model_at_scale(
        &mut self,
        shell_size: UiSize,
        scale_factor: f32,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.recompute_mounted_layout_with_workbench_model_at_scale(
            UiFrame::new(0.0, 0.0, shell_size.width, shell_size.height),
            scale_factor,
            model,
            metrics,
        )
    }

    pub(crate) fn recompute_mounted_layout_with_workbench_model_at_scale(
        &mut self,
        mount_frame: UiFrame,
        scale_factor: f32,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let physical_shell_size = UiSize::new(mount_frame.width, mount_frame.height);
        let shell_size = self.prepare_layout_at_mount_with_scale(mount_frame, scale_factor);
        let logical_toolbar_width = shell_size.width;
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_bridge_responsive_projection"
            );
            apply_workbench_responsive_layout(
                &mut self.template_surface.surface,
                physical_shell_size,
                scale_factor,
                self.compact_module_details_drawer_open,
            )?;
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_bridge_state_projection"
            );
            self.apply_toolbar_run_state(model)?;
            self.apply_asset_creation_menu_state(model, shell_size)?;
            self.apply_responsive_toolbar_layout(UiSize::new(
                logical_toolbar_width,
                shell_size.height,
            ))?;
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_bridge_drawer_projection"
            );
            apply_workbench_drawer_layout_to_surface(
                &mut self.template_surface.surface,
                shell_size,
                model,
                metrics,
                None,
            )?;
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_bridge_surface_recompute"
            );
            self.template_surface
                .recompute_layout(self.runtime.as_ref(), shell_size)?;
        }
        Ok(())
    }
}

fn apply_workbench_drawer_layout_to_surface(
    surface: &mut UiSurface,
    shell_size: UiSize,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    anchors: Option<WorkbenchDrawerLayoutAnchors>,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    apply_workbench_drawer_layout(
        surface,
        shell_size,
        WorkbenchDrawerLayoutInputs::from_workbench_model(model, metrics),
        *metrics,
        anchors,
    )
}

fn apply_workbench_drawer_layout(
    surface: &mut UiSurface,
    shell_size: UiSize,
    drawer_inputs: WorkbenchDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<WorkbenchDrawerLayoutAnchors>,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let rail_width = metrics.rail_width.max(0.0);
    let left = compacted_side_region_input(
        drawer_inputs.left,
        ShellRegionId::Left,
        shell_size,
        rail_width,
    );
    let right = compacted_side_region_input(
        drawer_inputs.right,
        ShellRegionId::Right,
        shell_size,
        rail_width,
    );
    let bottom = compacted_bottom_region_input(drawer_inputs.bottom, shell_size, metrics, anchors);
    let (left, right) = reserve_document_width(left, right, shell_size, metrics);

    mark_roots_layout_dirty(surface)?;

    apply_fixed_control_width(
        surface,
        LEFT_DRAWER_SHELL_CONTROL_ID,
        resolved_side_panel_extent(left, rail_width),
    )?;
    apply_fixed_control_width(
        surface,
        RIGHT_DRAWER_SHELL_CONTROL_ID,
        resolved_side_panel_extent(right, rail_width),
    )?;
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(bottom),
    )?;
    Ok(())
}

fn reserve_document_width(
    left: WorkbenchDrawerRegionInput,
    right: WorkbenchDrawerRegionInput,
    shell_size: UiSize,
    metrics: WorkbenchChromeMetrics,
) -> (WorkbenchDrawerRegionInput, WorkbenchDrawerRegionInput) {
    let rail_width = metrics.rail_width.max(0.0);
    let visible_panel_count = [left, right]
        .into_iter()
        .filter(|region| resolved_side_panel_extent(*region, rail_width) > f32::EPSILON)
        .count() as f32;
    let fixed_side_chrome =
        visible_panel_count * (rail_width + metrics.separator_thickness.max(0.0) * 2.0);
    let panel_budget = (shell_size.width.max(0.0) * (1.0 - minimum_document_width_fraction())
        - fixed_side_chrome)
        .max(0.0);
    let widths = balanced_side_widths_for_budget(
        resolved_side_panel_extent(left, rail_width),
        resolved_side_panel_extent(right, rail_width),
        panel_budget,
    );

    (
        region_with_panel_width(left, widths.left, rail_width),
        region_with_panel_width(right, widths.right, rail_width),
    )
}

fn region_with_panel_width(
    region: WorkbenchDrawerRegionInput,
    panel_width: f32,
    rail_width: f32,
) -> WorkbenchDrawerRegionInput {
    if !region.visible || resolved_side_panel_extent(region, rail_width) <= f32::EPSILON {
        return region;
    }
    WorkbenchDrawerRegionInput {
        extent: panel_width.max(0.0) + rail_width,
        ..region
    }
}

fn mark_roots_layout_dirty(
    surface: &mut UiSurface,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    for root_index in 0..surface.tree.roots.len() {
        let root_id = surface.tree.roots[root_index];
        surface.tree.mark_layout_dirty(root_id)?;
    }
    Ok(())
}

fn compacted_side_region_input(
    region: WorkbenchDrawerRegionInput,
    side: ShellRegionId,
    shell_size: UiSize,
    rail_width: f32,
) -> WorkbenchDrawerRegionInput {
    if !region.visible {
        return region;
    }

    if side == ShellRegionId::Right
        && right_drawer_should_collapse_for_logical_width(shell_size.width)
    {
        return WorkbenchDrawerRegionInput {
            extent: rail_width,
            ..region
        };
    }

    let extent = compact_side_width_limit(side, shell_size.width)
        .map(|limit| region.extent.min(limit))
        .unwrap_or(region.extent);

    WorkbenchDrawerRegionInput { extent, ..region }
}

fn compacted_bottom_region_input(
    region: WorkbenchDrawerRegionInput,
    shell_size: UiSize,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<WorkbenchDrawerLayoutAnchors>,
) -> WorkbenchDrawerRegionInput {
    if !region.visible {
        return region;
    }

    if matches!(
        workbench_layout_tier_for_logical_width(shell_size.width),
        WorkbenchLayoutTier::Ultra | WorkbenchLayoutTier::Narrow
    ) {
        // The compact bottom drawer keeps its tab strip as the re-open affordance,
        // but yields the remaining vertical budget to the active document surface.
        return WorkbenchDrawerRegionInput {
            extent: metrics.panel_header_height.max(0.0),
            ..region
        };
    }

    let separator = metrics.separator_thickness.max(0.0);
    let available_height = anchors
        .map(|anchors| (anchors.body_height - separator).max(0.0))
        .unwrap_or_else(|| {
            (shell_size.height
                - metrics.top_bar_height.max(0.0)
                - separator
                - metrics.host_bar_height.max(0.0)
                - separator
                - metrics.status_bar_height.max(0.0)
                - separator)
                .max(0.0)
        });
    let extent = compact_bottom_height_limit(available_height)
        .map(|limit| region.extent.min(limit))
        .unwrap_or(region.extent);

    WorkbenchDrawerRegionInput { extent, ..region }
}

fn resolved_drawer_shell_extent(region: WorkbenchDrawerRegionInput) -> f32 {
    if region.visible {
        region.extent.max(0.0)
    } else {
        0.0
    }
}

fn resolved_side_panel_extent(region: WorkbenchDrawerRegionInput, rail_width: f32) -> f32 {
    if region.visible {
        (region.extent - rail_width).max(0.0)
    } else {
        0.0
    }
}

fn drawer_region_input(
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
    slots: &[ActivityDrawerSlot],
    collapsed_extent: f32,
) -> WorkbenchDrawerRegionInput {
    let mut visible = false;
    let mut extent = 0.0_f32;

    for slot in slots {
        let Some(drawer) = drawers.get(slot) else {
            continue;
        };
        if !drawer.visible || drawer.tabs.is_empty() {
            continue;
        }

        visible = true;
        let next_extent = match drawer.mode {
            ActivityDrawerMode::Collapsed => collapsed_extent,
            ActivityDrawerMode::Pinned | ActivityDrawerMode::AutoHide => {
                drawer.extent.max(collapsed_extent)
            }
        };
        extent = extent.max(next_extent);
    }

    WorkbenchDrawerRegionInput {
        visible,
        extent: if visible { extent.max(0.0) } else { 0.0 },
    }
}

fn surface_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Option<zircon_runtime_interface::ui::event_ui::UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn apply_fixed_control_width(
    surface: &mut UiSurface,
    control_id: &str,
    width: f32,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return Ok(());
    };
    let changed = {
        let Some(node) = surface.tree.node_mut(node_id) else {
            return Ok(());
        };
        let next_width = fixed_axis(width);
        let next_visibility = fixed_extent_visibility(width);
        let changed = node.constraints.width != next_width || node.visibility != next_visibility;
        node.constraints.width = next_width;
        node.visibility = next_visibility;
        changed
    };

    if changed {
        surface.tree.mark_layout_dirty(node_id)?;
    }
    Ok(())
}

fn apply_fixed_control_height(
    surface: &mut UiSurface,
    control_id: &str,
    height: f32,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return Ok(());
    };
    let changed = {
        let Some(node) = surface.tree.node_mut(node_id) else {
            return Ok(());
        };
        let next_height = fixed_axis(height);
        let next_visibility = fixed_extent_visibility(height);
        let changed = node.constraints.height != next_height || node.visibility != next_visibility;
        node.constraints.height = next_height;
        node.visibility = next_visibility;
        changed
    };

    if changed {
        surface.tree.mark_layout_dirty(node_id)?;
    }
    Ok(())
}

fn fixed_extent_visibility(size: f32) -> UiVisibility {
    if size > f32::EPSILON {
        UiVisibility::Visible
    } else {
        UiVisibility::Collapsed
    }
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size.max(0.0),
        max: size.max(0.0),
        preferred: size.max(0.0),
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::workbench::fixture::default_preview_fixture;

    #[test]
    fn collapsed_bottom_drawer_input_uses_the_callers_panel_header_metric() {
        let fixture = default_preview_fixture();
        let chrome = fixture.build_chrome();
        let mut model = WorkbenchViewModel::build(
            &crate::core::commands::EditorCommandRegistry::default_workbench(),
            &chrome,
        );
        let bottom = model
            .drawer_ring
            .drawers
            .get_mut(&ActivityDrawerSlot::Bottom)
            .expect("preview fixture should expose a bottom drawer");
        bottom.mode = ActivityDrawerMode::Collapsed;
        bottom.visible = true;
        assert!(!bottom.tabs.is_empty());

        let metrics = WorkbenchChromeMetrics {
            panel_header_height: 31.0,
            ..WorkbenchChromeMetrics::default()
        };
        let inputs = WorkbenchDrawerLayoutInputs::from_workbench_model(&model, &metrics);

        assert!(inputs.bottom.visible);
        assert_eq!(inputs.bottom.extent, metrics.panel_header_height);
    }

    #[test]
    fn narrow_width_collapses_a_visible_bottom_drawer_to_its_tab_strip() {
        let pinned_drawer = WorkbenchDrawerRegionInput {
            visible: true,
            extent: 228.0,
        };
        let metrics = WorkbenchChromeMetrics {
            panel_header_height: 31.0,
            ..WorkbenchChromeMetrics::default()
        };
        let compacted = compacted_bottom_region_input(
            pinned_drawer,
            UiSize::new(640.0, 520.0),
            metrics,
            Some(WorkbenchDrawerLayoutAnchors { body_height: 420.0 }),
        );

        assert_eq!(compacted.extent, metrics.panel_header_height);
    }
}

mod priority;

use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    layout::{AxisConstraint, StretchMode, UiContainerKind, UiSize},
};

use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_logical_width, WorkbenchLayoutTier,
};

use self::priority::resolve_toolbar_priority;
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::module_overflow_menu::{
    WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID, WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
};

const MODULE_COMMAND_GROUP_CONTROL_ID: &str = "WorkbenchModuleCommands";
const FILE_GROUP_CONTROL_ID: &str = "WorkbenchToolbarFileGroup";
const RUN_GROUP_CONTROL_ID: &str = "WorkbenchToolbarRunGroup";
const TOOL_GROUP_DIVIDER_CONTROL_ID: &str = "WorkbenchToolbarToolGroupDivider";
const LAYOUT_GROUP_CONTROL_ID: &str = "WorkbenchToolbarLayoutGroup";
const LAYOUT_GROUP_DIVIDER_CONTROL_ID: &str = "WorkbenchToolbarLayoutGroupDivider";
const MODULE_DETAILS_DRAWER_TOGGLE_CONTROL_ID: &str = "WorkbenchModuleDetailsDrawerToggle";
const MODULE_WORKSPACE_HOST_CONTROL_ID: &str = "WorkbenchMainBandModuleWorkspace";

const COMPACT_HIDDEN_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleBehavior",
    "WorkbenchModuleRender",
    "WorkbenchModuleAssets",
    "WorkbenchModuleVfx",
    "WorkbenchModuleHud",
];

const ULTRA_HIDDEN_FILE_CONTROLS: &[&str] = &[
    "WorkbenchToolbarAssets",
    "WorkbenchToolbarOpen",
    "WorkbenchToolbarSave",
];

const ULTRA_HIDDEN_MODULE_TABS: &[&str] = &["WorkbenchModulePerception", "WorkbenchModuleMaterial"];

const ICON_COMMAND_WIDTH: f32 = 34.0;

const MODULE_PRIMARY_COMMANDS: &[ToolbarCommandDensity] = &[
    ToolbarCommandDensity {
        control_id: "WorkbenchModuleSave",
        label: "Save",
        regular_width: ICON_COMMAND_WIDTH,
        always_icon_only: true,
    },
    ToolbarCommandDensity {
        control_id: "WorkbenchModuleBrowse",
        label: "Browse",
        regular_width: ICON_COMMAND_WIDTH,
        always_icon_only: true,
    },
    ToolbarCommandDensity {
        control_id: "WorkbenchModuleCompile",
        label: "Compile",
        regular_width: 104.0,
        always_icon_only: false,
    },
];

const SECONDARY_MODULE_COMMANDS: &[&str] = &["WorkbenchModuleDiff", "WorkbenchModuleSimulate"];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_responsive_toolbar_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let layout_tier = workbench_layout_tier_for_logical_width(shell_size.width);
        let ultra = layout_tier == WorkbenchLayoutTier::Ultra;
        for command in MODULE_PRIMARY_COMMANDS {
            self.apply_toolbar_command_density(*command, ultra)?;
        }
        self.apply_compact_module_details_toggle_visibility(layout_tier)?;
        let priority = resolve_toolbar_priority(&self.template_surface, shell_size.width);
        let compact = priority.compact_module_tabs;
        for control_id in ULTRA_HIDDEN_FILE_CONTROLS {
            self.set_visible(control_id, !ultra)?;
        }
        for control_id in COMPACT_HIDDEN_MODULE_TABS {
            self.set_visible(control_id, !compact)?;
        }
        for control_id in ULTRA_HIDDEN_MODULE_TABS {
            self.set_visible(control_id, !ultra)?;
        }
        self.set_visible(WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID, compact)?;
        if !compact {
            self.close_workbench_window_menu_control(WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID)?;
        }

        let full_toolbar = priority.full_command_set;
        for control_id in SECONDARY_MODULE_COMMANDS {
            self.set_visible(control_id, full_toolbar)?;
        }
        self.set_visible("WorkbenchToolbarToolGroup", priority.transform_tools)?;
        self.set_visible(TOOL_GROUP_DIVIDER_CONTROL_ID, priority.transform_tools)?;
        // Run and already-iconized layout commands stay directly reachable at every
        // breakpoint; lower-priority module labels and transform tools collapse first.
        self.set_visible(RUN_GROUP_CONTROL_ID, true)?;
        self.set_visible(LAYOUT_GROUP_CONTROL_ID, true)?;
        self.set_visible(LAYOUT_GROUP_DIVIDER_CONTROL_ID, true)?;
        self.apply_horizontal_content_width(FILE_GROUP_CONTROL_ID)?;
        self.apply_horizontal_content_width(MODULE_COMMAND_GROUP_CONTROL_ID)?;
        self.apply_horizontal_content_width(RUN_GROUP_CONTROL_ID)?;
        self.apply_horizontal_content_width(LAYOUT_GROUP_CONTROL_ID)?;
        Ok(())
    }

    pub(super) fn refresh_compact_module_details_toggle_visibility(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let logical_width =
            self.mount_frame.width / self.presentation_scale_factor.max(f32::EPSILON);
        self.apply_compact_module_details_toggle_visibility(
            workbench_layout_tier_for_logical_width(logical_width),
        )?;
        self.apply_horizontal_content_width(LAYOUT_GROUP_CONTROL_ID)
    }

    fn apply_compact_module_details_toggle_visibility(
        &mut self,
        layout_tier: WorkbenchLayoutTier,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let module_workspace_active = self
            .control_node_id(MODULE_WORKSPACE_HOST_CONTROL_ID)
            .and_then(|node_id| self.template_surface.surface.tree.node(node_id))
            .is_some_and(|node| node.visibility.occupies_layout());
        let compact_tier = matches!(
            layout_tier,
            WorkbenchLayoutTier::Narrow | WorkbenchLayoutTier::Regular
        );
        self.set_visible(
            MODULE_DETAILS_DRAWER_TOGGLE_CONTROL_ID,
            module_workspace_active && compact_tier,
        )
    }

    fn apply_toolbar_command_density(
        &mut self,
        command: ToolbarCommandDensity,
        ultra: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let icon_only = command.always_icon_only || ultra;
        self.mutate_control_property(
            command.control_id,
            "text",
            UiValue::String(if icon_only { "" } else { command.label }.to_string()),
        )?;
        self.mutate_control_property(
            command.control_id,
            "label",
            UiValue::String(command.label.to_string()),
        )?;
        self.mutate_control_property(
            command.control_id,
            "icon_placement",
            UiValue::String(if icon_only { "icon_only" } else { "leading" }.to_string()),
        )?;
        self.apply_fixed_control_width(
            command.control_id,
            if icon_only {
                ICON_COMMAND_WIDTH
            } else {
                command.regular_width
            },
        )
    }
}

#[derive(Clone, Copy)]
struct ToolbarCommandDensity {
    control_id: &'static str,
    label: &'static str,
    regular_width: f32,
    always_icon_only: bool,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    fn apply_horizontal_content_width(
        &mut self,
        control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let surface = &self.template_surface.surface;
        let width = {
            let Some(node) = surface.tree.node(node_id) else {
                return Ok(());
            };
            let UiContainerKind::HorizontalBox(config) = &node.container else {
                return Ok(());
            };
            let (content_width, visible_count) = node
                .children
                .iter()
                .filter_map(|child_id| {
                    let child = surface.tree.node(*child_id)?;
                    child
                        .effective_visibility()
                        .occupies_layout()
                        .then_some(child.constraints.width.preferred.max(0.0))
                })
                .fold((0.0_f32, 0_usize), |(width, count), child_width| {
                    (width + child_width, count + 1)
                });
            content_width + config.gap.max(0.0) * visible_count.saturating_sub(1) as f32
        };
        self.apply_content_control_width(node_id, width)
    }

    fn apply_content_control_width(
        &mut self,
        node_id: UiNodeId,
        width: f32,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let surface = &mut self.template_surface.surface;
        let changed = {
            let Some(node) = surface.tree.node_mut(node_id) else {
                return Ok(());
            };
            let next_width = content_axis(node.constraints.width, width);
            let changed = node.constraints.width != next_width;
            node.constraints.width = next_width;
            changed
        };

        if changed {
            surface.tree.mark_layout_dirty(node_id)?;
        }
        Ok(())
    }

    fn apply_fixed_control_width(
        &mut self,
        control_id: &str,
        width: f32,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(control_id) else {
            return Ok(());
        };
        let surface = &mut self.template_surface.surface;
        let changed = {
            let Some(node) = surface.tree.node_mut(node_id) else {
                return Ok(());
            };
            let next_width = AxisConstraint {
                min: width,
                max: width,
                preferred: width,
                priority: node.constraints.width.priority,
                weight: node.constraints.width.weight,
                stretch_mode: StretchMode::Fixed,
            };
            let changed = node.constraints.width != next_width;
            node.constraints.width = next_width;
            changed
        };
        if changed {
            surface.tree.mark_layout_dirty(node_id)?;
        }
        Ok(())
    }
}

fn content_axis(authored: AxisConstraint, size: f32) -> AxisConstraint {
    let size = size.max(authored.min).max(0.0);
    AxisConstraint {
        min: authored.min.max(0.0),
        max: size,
        preferred: size,
        priority: authored.priority,
        weight: authored.weight,
        stretch_mode: StretchMode::Stretch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_width_preserves_authored_priority_and_stretch_contract() {
        let authored = AxisConstraint {
            min: 72.0,
            max: 300.0,
            preferred: 300.0,
            priority: 60,
            weight: 2.0,
            stretch_mode: StretchMode::Stretch,
        };

        assert_eq!(
            content_axis(authored, 180.0),
            AxisConstraint {
                min: 72.0,
                max: 180.0,
                preferred: 180.0,
                priority: 60,
                weight: 2.0,
                stretch_mode: StretchMode::Stretch,
            }
        );
        assert_eq!(content_axis(authored, 12.0).preferred, authored.min);
        assert_eq!(content_axis(authored, 12.0).max, authored.min);
    }
}

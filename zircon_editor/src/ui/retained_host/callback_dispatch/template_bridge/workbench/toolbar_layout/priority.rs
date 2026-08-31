use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiContainerKind};

use crate::ui::workbench::reference::EditorWorkbenchTemplateSurface;

const COMMAND_ROW_CONTROL_ID: &str = "WorkbenchToolbarCommandRow";
const COMMAND_DIVIDERS: &[&str] = &[
    "WorkbenchToolbarFileGroupDivider",
    "WorkbenchToolbarToolGroupDivider",
    "WorkbenchToolbarRunGroupDivider",
    "WorkbenchToolbarLayoutGroupDivider",
];
const FILE_CONTROLS: &[&str] = &[
    "WorkbenchToolbarMenu",
    "WorkbenchToolbarAssets",
    "WorkbenchToolbarOpen",
    "WorkbenchToolbarSave",
];
const PRIMARY_MODULE_COMMANDS: &[&str] = &[
    "WorkbenchModuleSave",
    "WorkbenchModuleBrowse",
    "WorkbenchModuleCompile",
];
const FULL_MODULE_COMMANDS: &[&str] = &[
    "WorkbenchModuleSave",
    "WorkbenchModuleBrowse",
    "WorkbenchModuleCompile",
    "WorkbenchModuleDiff",
    "WorkbenchModuleSimulate",
];
const TRANSFORM_TOOLS: &[&str] = &[
    "WorkbenchToolSelect",
    "WorkbenchToolMove",
    "WorkbenchToolRotate",
    "WorkbenchToolScale",
    "WorkbenchToolSnap",
];
const PRIMARY_RUN_CONTROLS: &[&str] = &["WorkbenchRunPlay", "WorkbenchRunMode"];
const LAYOUT_CONTROLS: &[&str] = &["WorkbenchLayoutGrid", "WorkbenchThemeToggle"];
const LAYOUT_CONTROLS_WITH_DETAILS: &[&str] = &[
    "WorkbenchLayoutGrid",
    "WorkbenchThemeToggle",
    "WorkbenchModuleDetailsDrawerToggle",
];
const FULL_MODULE_TABS: &[&str] = &[
    "WorkbenchModuleScene",
    "WorkbenchModuleEffect",
    "WorkbenchModuleAbility",
    "WorkbenchModuleTags",
    "WorkbenchModulePerception",
    "WorkbenchModuleMaterial",
    "WorkbenchModuleBehavior",
    "WorkbenchModuleRender",
    "WorkbenchModuleAssets",
    "WorkbenchModuleVfx",
    "WorkbenchModuleHud",
];

const COMMAND_CONTENT_MAX_FILL: f32 = 0.62;
const MODULE_TAB_CONTENT_MAX_FILL: f32 = 0.72;

pub(super) struct ToolbarPriorityProjection {
    pub(super) compact_module_tabs: bool,
    pub(super) transform_tools: bool,
    pub(super) full_command_set: bool,
}

pub(super) fn resolve_toolbar_priority(
    template_surface: &EditorWorkbenchTemplateSurface,
    available_width: f32,
) -> ToolbarPriorityProjection {
    let compact = ToolbarPriorityProjection {
        compact_module_tabs: true,
        transform_tools: false,
        full_command_set: false,
    };
    let surface = &template_surface.surface;
    let controls = ToolbarControlSlots { template_surface };
    let Some(command_row_gap) = structural_gap_width(surface, &controls, COMMAND_ROW_CONTROL_ID)
    else {
        return compact;
    };
    let Some(command_divider_width) = control_intrinsic_width(surface, &controls, COMMAND_DIVIDERS)
    else {
        return compact;
    };
    let Some(file_width) = control_sequence_width(surface, &controls, FILE_CONTROLS) else {
        return compact;
    };
    let Some(primary_module_width) =
        control_sequence_width(surface, &controls, PRIMARY_MODULE_COMMANDS)
    else {
        return compact;
    };
    let Some(full_module_width) = control_sequence_width(surface, &controls, FULL_MODULE_COMMANDS)
    else {
        return compact;
    };
    let Some(transform_width) = control_sequence_width(surface, &controls, TRANSFORM_TOOLS) else {
        return compact;
    };
    let Some(primary_run_width) = control_sequence_width(surface, &controls, PRIMARY_RUN_CONTROLS)
    else {
        return compact;
    };
    let layout_controls =
        if control_occupies_layout(surface, &controls, "WorkbenchModuleDetailsDrawerToggle") {
            LAYOUT_CONTROLS_WITH_DETAILS
        } else {
            LAYOUT_CONTROLS
        };
    let Some(layout_width) = control_sequence_width(surface, &controls, layout_controls) else {
        return compact;
    };
    let Some(full_module_tabs_width) = control_sequence_width(surface, &controls, FULL_MODULE_TABS)
    else {
        return compact;
    };

    let command_chrome_width = command_row_gap + command_divider_width;
    let regular_command_width = file_width
        + primary_module_width
        + transform_width
        + primary_run_width
        + layout_width
        + command_chrome_width;
    let full_command_width = file_width
        + full_module_width
        + transform_width
        + primary_run_width
        + layout_width
        + command_chrome_width;

    ToolbarPriorityProjection {
        compact_module_tabs: !fits_fill_ratio(
            full_module_tabs_width,
            available_width,
            MODULE_TAB_CONTENT_MAX_FILL,
        ),
        transform_tools: fits_fill_ratio(
            regular_command_width,
            available_width,
            COMMAND_CONTENT_MAX_FILL,
        ),
        full_command_set: fits_fill_ratio(
            full_command_width,
            available_width,
            COMMAND_CONTENT_MAX_FILL,
        ),
    }
}

fn control_occupies_layout(
    surface: &UiSurface,
    controls: &ToolbarControlSlots<'_>,
    control_id: &str,
) -> bool {
    controls
        .node_id(control_id)
        .and_then(|node_id| surface.tree.node(node_id))
        .is_some_and(|node| node.visibility.occupies_layout())
}

fn control_sequence_width(
    surface: &UiSurface,
    controls: &ToolbarControlSlots<'_>,
    control_ids: &[&str],
) -> Option<f32> {
    let width = control_intrinsic_width(surface, controls, control_ids)?;
    let gap = control_ids
        .first()
        .and_then(|control_id| controls.node_id(control_id))
        .and_then(|node_id| surface.tree.node(node_id))
        .and_then(|node| node.parent)
        .and_then(|parent_id| surface.tree.node(parent_id))
        .and_then(|node| horizontal_gap(&node.container))?;
    Some(width + gap * control_ids.len().saturating_sub(1) as f32)
}

fn control_intrinsic_width(
    surface: &UiSurface,
    controls: &ToolbarControlSlots<'_>,
    control_ids: &[&str],
) -> Option<f32> {
    control_ids.iter().try_fold(0.0, |width, control_id| {
        let node_id = controls.node_id(control_id)?;
        let node = surface.tree.node(node_id)?;
        Some(width + node.constraints.width.preferred.max(0.0))
    })
}

fn structural_gap_width(
    surface: &UiSurface,
    controls: &ToolbarControlSlots<'_>,
    control_id: &str,
) -> Option<f32> {
    let node_id = controls.node_id(control_id)?;
    let node = surface.tree.node(node_id)?;
    let gap = horizontal_gap(&node.container)?;
    Some(gap * node.children.len().saturating_sub(1) as f32)
}

struct ToolbarControlSlots<'a> {
    template_surface: &'a EditorWorkbenchTemplateSurface,
}

impl ToolbarControlSlots<'_> {
    fn node_id(&self, control_id: &str) -> Option<UiNodeId> {
        self.template_surface.control_node_id(control_id)
    }
}

fn horizontal_gap(container: &UiContainerKind) -> Option<f32> {
    let UiContainerKind::HorizontalBox(config) = container else {
        return None;
    };
    Some(config.gap.max(0.0))
}

fn fits_fill_ratio(content_width: f32, available_width: f32, max_fill: f32) -> bool {
    content_width.is_finite()
        && available_width.is_finite()
        && content_width >= 0.0
        && available_width > 0.0
        && content_width <= available_width * max_fill
}

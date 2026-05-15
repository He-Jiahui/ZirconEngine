use crate::ui::workbench::layout::{ActivityDrawerLayout, ActivityDrawerMode, WorkbenchLayout};
use crate::ui::workbench::preset::EditorUiDesignStack;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::editor_subsystems::{
    EditorSubsystemReport, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
};

pub(crate) fn builtin_hybrid_layout() -> WorkbenchLayout {
    builtin_hybrid_layout_for_subsystems(&EditorSubsystemReport::default_enabled())
}

pub(crate) fn builtin_hybrid_layout_for_subsystems(
    subsystems: &EditorSubsystemReport,
) -> WorkbenchLayout {
    let mut layout =
        EditorUiDesignStack::material_fyrox_jetbrains_unreal().default_workbench_layout();
    if !subsystems.is_enabled(EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS) {
        remove_view_from_layout(
            &mut layout,
            &ViewInstanceId::new("editor.runtime_diagnostics#1"),
        );
        remove_view_from_layout(
            &mut layout,
            &ViewInstanceId::new("editor.performance_timeline#1"),
        );
    }
    layout
}

fn remove_view_from_layout(layout: &mut WorkbenchLayout, instance_id: &ViewInstanceId) {
    for drawer in layout.drawers.values_mut() {
        remove_view_from_drawer(drawer, instance_id);
    }
    for window in layout.activity_windows.values_mut() {
        for drawer in window.activity_drawers.values_mut() {
            remove_view_from_drawer(drawer, instance_id);
        }
    }
}

fn remove_view_from_drawer(drawer: &mut ActivityDrawerLayout, instance_id: &ViewInstanceId) {
    drawer.tab_stack.tabs.retain(|tab| tab != instance_id);
    if drawer.tab_stack.active_tab.as_ref() == Some(instance_id) {
        drawer.tab_stack.active_tab = drawer.tab_stack.tabs.first().cloned();
    }
    if drawer.active_view.as_ref() == Some(instance_id) {
        drawer.active_view = drawer.tab_stack.active_tab.clone();
    }
    if drawer.tab_stack.tabs.is_empty() {
        drawer.tab_stack.active_tab = None;
        drawer.active_view = None;
        drawer.mode = ActivityDrawerMode::Collapsed;
    }
}

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot;
use crate::ui::layouts::windows::workbench_host_window::ShellPresentation;
use crate::ui::layouts::windows::workbench_host_window::{
    BuildExportPaneViewData, ModulePluginsPaneViewData,
};
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerSlot, ActivityWindowLayout, WorkbenchLayout,
};
use crate::ui::workbench::model::{ToolWindowStackModel, WorkbenchViewModel};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

pub(in crate::ui::retained_host::app) struct HostLifecyclePanePayloads {
    pub(in crate::ui::retained_host::app) preset_names: Vec<String>,
    pub(in crate::ui::retained_host::app) ui_asset_panes:
        BTreeMap<String, crate::ui::asset_editor::UiAssetEditorPanePresentation>,
    pub(in crate::ui::retained_host::app) animation_panes:
        BTreeMap<String, crate::ui::animation_editor::AnimationEditorPanePresentation>,
    pub(in crate::ui::retained_host::app) runtime_diagnostics: RuntimeDiagnosticsSnapshot,
    pub(in crate::ui::retained_host::app) module_plugins: ModulePluginsPaneViewData,
    pub(in crate::ui::retained_host::app) build_export: BuildExportPaneViewData,
    pub(in crate::ui::retained_host::app) template_v2_data:
        BTreeMap<String, EditorUiTemplatePaneDataSnapshot>,
}

pub(in crate::ui::retained_host::app) struct CommittedShellState {
    pub(in crate::ui::retained_host::app) layout: WorkbenchLayout,
    pub(in crate::ui::retained_host::app) chrome: EditorChromeSnapshot,
    pub(in crate::ui::retained_host::app) model: WorkbenchViewModel,
    pub(in crate::ui::retained_host::app) geometry: WorkbenchShellGeometry,
    pub(in crate::ui::retained_host::app) layout_frames: BuiltinWorkbenchWindowLayoutFrames,
    pub(in crate::ui::retained_host::app) descriptors: Vec<ViewDescriptor>,
    pub(in crate::ui::retained_host::app) pane_payloads: Option<Arc<HostLifecyclePanePayloads>>,
    pub(in crate::ui::retained_host::app) retained_shell_presentation:
        Option<Arc<ShellPresentation>>,
}

impl CommittedShellState {
    pub(in crate::ui::retained_host::app) fn shell_content_scope_for_pane(
        &self,
        pane_id: &str,
    ) -> Option<HostShellContentScope> {
        active_shell_content_scope_for_pane(&self.model.tool_windows, pane_id)
    }

    pub(in crate::ui::retained_host::app) fn patch_shell_content(
        &mut self,
        scope: &HostShellContentScope,
        next_layout: WorkbenchLayout,
    ) -> bool {
        if !validate_shell_content_layout_transition(&self.layout, &next_layout, scope) {
            return false;
        }
        let next_drawers = next_layout.active_activity_window_drawers();
        for slot in ActivityDrawerSlot::ALL
            .into_iter()
            .filter(|slot| slot.shares_region(scope.slot))
        {
            let Some(next_drawer) = next_drawers.get(&slot) else {
                return false;
            };
            let Some(chrome_drawer) = self.chrome.workbench.drawers.get(&slot) else {
                return false;
            };
            if !drawer_tab_ids_match(
                next_drawer,
                chrome_drawer.tabs.iter().map(|tab| &tab.instance_id),
            ) {
                return false;
            }
            let Some(stack) = self.model.tool_windows.get(&slot) else {
                return false;
            };
            if !drawer_tab_ids_match(next_drawer, stack.tabs.iter().map(|tab| &tab.instance_id)) {
                return false;
            }
            if !self.model.drawer_ring.drawers.contains_key(&slot) {
                return false;
            }
        }

        for slot in ActivityDrawerSlot::ALL
            .into_iter()
            .filter(|slot| slot.shares_region(scope.slot))
        {
            let next_drawer = next_drawers
                .get(&slot)
                .expect("shell-content preflight verified the drawer");
            let chrome_drawer = self
                .chrome
                .workbench
                .drawers
                .get_mut(&slot)
                .expect("shell-content preflight verified the chrome drawer");
            chrome_drawer.active_tab = next_drawer.tab_stack.active_tab.clone();
            chrome_drawer.active_view = next_drawer.active_view.clone();
            chrome_drawer.mode = next_drawer.mode;
            chrome_drawer.extent = next_drawer.extent;
            chrome_drawer.visible = next_drawer.visible;

            let stack = self
                .model
                .tool_windows
                .get_mut(&slot)
                .expect("shell-content preflight verified the tool stack");
            stack.active_tab = next_drawer.tab_stack.active_tab.clone();
            stack.mode = next_drawer.mode;
            stack.visible = next_drawer.visible;
            for tab in &mut stack.tabs {
                tab.active = next_drawer.tab_stack.active_tab.as_ref() == Some(&tab.instance_id);
            }

            let model_drawer = self
                .model
                .drawer_ring
                .drawers
                .get_mut(&slot)
                .expect("shell-content preflight verified the model drawer");
            model_drawer.active_tab = next_drawer.tab_stack.active_tab.clone();
            model_drawer.active_view = next_drawer.active_view.clone();
            model_drawer.mode = next_drawer.mode;
            model_drawer.extent = next_drawer.extent;
            model_drawer.visible = next_drawer.visible;
        }
        self.layout = next_layout;
        true
    }
}

fn active_shell_content_scope_for_pane(
    tool_windows: &BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>,
    pane_id: &str,
) -> Option<HostShellContentScope> {
    let mut matches = tool_windows.iter().filter_map(|(slot, stack)| {
        let instance_id = stack.active_tab.as_ref()?;
        (stack.visible && instance_id.0 == pane_id)
            .then(|| HostShellContentScope::new(*slot, instance_id.clone()))
    });
    let scope = matches.next()?;
    matches.next().is_none().then_some(scope)
}

fn validate_shell_content_layout_transition(
    previous: &WorkbenchLayout,
    next: &WorkbenchLayout,
    scope: &HostShellContentScope,
) -> bool {
    let target = scope.slot;
    if previous.active_main_page != next.active_main_page
        || previous.main_pages != next.main_pages
        || previous.floating_windows != next.floating_windows
        || previous.active_activity_window_id() != next.active_activity_window_id()
    {
        return false;
    }

    let previous_windows = previous.activity_windows();
    let next_windows = next.activity_windows();
    if previous_windows.len() != next_windows.len() {
        return false;
    }
    let Some(active_window_id) = next.active_activity_window_id() else {
        return false;
    };
    for (window_id, previous_window) in previous_windows.iter() {
        let Some(next_window) = next_windows.get(window_id) else {
            return false;
        };
        if window_id == &active_window_id {
            if !same_window_outside_drawer_region(previous_window, next_window, target) {
                return false;
            }
        } else if previous_window != next_window {
            return false;
        }
    }

    let previous_drawers = previous.active_activity_window_drawers();
    let next_drawers = next.active_activity_window_drawers();
    if previous_drawers.len() != next_drawers.len() {
        return false;
    }
    let Some(target_drawer) = next_drawers.get(&target) else {
        return false;
    };
    if target_drawer.mode == crate::ui::workbench::layout::ActivityDrawerMode::Collapsed
        || !target_drawer.visible
        || target_drawer.tab_stack.active_tab.as_ref() != Some(&scope.instance_id)
        || target_drawer.active_view.as_ref() != Some(&scope.instance_id)
        || !target_drawer.tab_stack.tabs.contains(&scope.instance_id)
    {
        return false;
    }

    let previous_extent = expanded_region_extent(previous_drawers, target);
    let next_extent = expanded_region_extent(next_drawers, target);
    previous_extent.is_some() && previous_extent == next_extent
}

fn same_window_outside_drawer_region(
    previous: &ActivityWindowLayout,
    next: &ActivityWindowLayout,
    target: ActivityDrawerSlot,
) -> bool {
    if previous.window_id != next.window_id
        || previous.descriptor_id != next.descriptor_id
        || previous.host_mode != next.host_mode
        || previous.content_workspace != next.content_workspace
        || previous.menu_overflow_mode != next.menu_overflow_mode
        || previous.region_overrides != next.region_overrides
        || previous.view_overrides != next.view_overrides
        || previous.activity_drawers.len() != next.activity_drawers.len()
    {
        return false;
    }
    previous.activity_drawers.iter().all(|(slot, drawer)| {
        let Some(next_drawer) = next.activity_drawers.get(slot) else {
            return false;
        };
        if slot.shares_region(target) {
            same_drawer_structure(drawer, next_drawer)
        } else {
            drawer == next_drawer
        }
    })
}

fn same_drawer_structure(previous: &ActivityDrawerLayout, next: &ActivityDrawerLayout) -> bool {
    previous.slot == next.slot
        && previous.tab_stack.tabs == next.tab_stack.tabs
        && previous.extent == next.extent
        && previous.visible == next.visible
}

fn expanded_region_extent(
    drawers: &std::collections::BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    target: ActivityDrawerSlot,
) -> Option<u32> {
    drawers
        .iter()
        .filter(|(slot, drawer)| {
            slot.shares_region(target)
                && drawer.visible
                && drawer.mode != crate::ui::workbench::layout::ActivityDrawerMode::Collapsed
                && !drawer.tab_stack.tabs.is_empty()
        })
        .map(|(_, drawer)| drawer.extent.to_bits())
        .next()
}

fn drawer_tab_ids_match<'a>(
    drawer: &ActivityDrawerLayout,
    ids: impl Iterator<Item = &'a crate::ui::workbench::view::ViewInstanceId>,
) -> bool {
    drawer.tab_stack.tabs.iter().eq(ids)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{active_shell_content_scope_for_pane, validate_shell_content_layout_transition};
    use crate::ui::retained_host::HostShellContentScope;
    use crate::ui::workbench::layout::{
        ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, LayoutManager, WorkbenchLayout,
    };
    use crate::ui::workbench::model::ToolWindowStackModel;
    use crate::ui::workbench::view::ViewInstanceId;

    fn tool_window_stack(
        slot: ActivityDrawerSlot,
        active_tab: Option<ViewInstanceId>,
        visible: bool,
    ) -> ToolWindowStackModel {
        ToolWindowStackModel {
            slot,
            mode: ActivityDrawerMode::Pinned,
            visible,
            tabs: Vec::new(),
            active_tab,
        }
    }

    fn drawer_switch_fixture() -> (WorkbenchLayout, HostShellContentScope) {
        let mut layout = WorkbenchLayout::default();
        let first = ViewInstanceId::new("editor.hierarchy#committed");
        let second = ViewInstanceId::new("editor.project#committed");
        let drawer = layout
            .active_activity_window_mut()
            .expect("active activity window")
            .activity_drawers
            .get_mut(&ActivityDrawerSlot::LeftTop)
            .expect("left drawer");
        drawer.tab_stack.tabs = vec![first.clone(), second.clone()];
        drawer.tab_stack.active_tab = Some(first.clone());
        drawer.active_view = Some(first);
        drawer.mode = ActivityDrawerMode::Pinned;
        (
            layout,
            HostShellContentScope::new(ActivityDrawerSlot::LeftTop, second),
        )
    }

    fn switched_layout(
        previous: &WorkbenchLayout,
        scope: &HostShellContentScope,
    ) -> WorkbenchLayout {
        let mut next = previous.clone();
        LayoutManager::default()
            .apply(
                &mut next,
                LayoutCommand::ActivateDrawerTab {
                    slot: scope.slot,
                    instance_id: scope.instance_id.clone(),
                },
            )
            .expect("switch drawer tab");
        next
    }

    #[test]
    fn accepts_an_exact_tab_switch_inside_the_committed_region() {
        let (previous, scope) = drawer_switch_fixture();
        let next = switched_layout(&previous, &scope);

        assert!(validate_shell_content_layout_transition(
            &previous, &next, &scope
        ));
    }

    #[test]
    fn accepts_content_only_update_for_the_current_active_drawer_pane() {
        let (layout, _) = drawer_switch_fixture();
        let scope = HostShellContentScope::new(
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.hierarchy#committed"),
        );

        assert!(validate_shell_content_layout_transition(
            &layout, &layout, &scope
        ));
    }

    #[test]
    fn rejects_a_region_switch_when_the_mounted_extent_changes() {
        let (previous, scope) = drawer_switch_fixture();
        let mut next = switched_layout(&previous, &scope);
        next.active_activity_window_mut()
            .expect("active activity window")
            .activity_drawers
            .get_mut(&ActivityDrawerSlot::LeftTop)
            .expect("left drawer")
            .extent += 24.0;

        assert!(!validate_shell_content_layout_transition(
            &previous, &next, &scope
        ));
    }

    #[test]
    fn rejects_any_non_target_layout_change_in_the_same_transaction() {
        let (previous, scope) = drawer_switch_fixture();
        let mut next = switched_layout(&previous, &scope);
        next.active_main_page = crate::ui::workbench::layout::MainPageId::new("unexpected");

        assert!(!validate_shell_content_layout_transition(
            &previous, &next, &scope
        ));
    }

    #[test]
    fn active_visible_drawer_pane_resolves_to_shell_content_scope() {
        let instance_id = ViewInstanceId::new("plugin.rows#active");
        let tool_windows = BTreeMap::from([(
            ActivityDrawerSlot::RightTop,
            tool_window_stack(
                ActivityDrawerSlot::RightTop,
                Some(instance_id.clone()),
                true,
            ),
        )]);

        assert_eq!(
            active_shell_content_scope_for_pane(&tool_windows, &instance_id.0),
            Some(HostShellContentScope::new(
                ActivityDrawerSlot::RightTop,
                instance_id
            ))
        );
    }

    #[test]
    fn inactive_or_hidden_drawer_pane_rejects_shell_content_scope() {
        let target = ViewInstanceId::new("plugin.rows#target");
        let other = ViewInstanceId::new("plugin.rows#other");
        let inactive = BTreeMap::from([(
            ActivityDrawerSlot::RightTop,
            tool_window_stack(ActivityDrawerSlot::RightTop, Some(other), true),
        )]);
        let hidden = BTreeMap::from([(
            ActivityDrawerSlot::RightTop,
            tool_window_stack(ActivityDrawerSlot::RightTop, Some(target.clone()), false),
        )]);

        assert_eq!(
            active_shell_content_scope_for_pane(&inactive, &target.0),
            None
        );
        assert_eq!(
            active_shell_content_scope_for_pane(&hidden, &target.0),
            None
        );
    }

    #[test]
    fn duplicate_active_pane_identity_rejects_ambiguous_shell_content_scope() {
        let target = ViewInstanceId::new("plugin.rows#duplicate");
        let tool_windows = BTreeMap::from([
            (
                ActivityDrawerSlot::LeftTop,
                tool_window_stack(ActivityDrawerSlot::LeftTop, Some(target.clone()), true),
            ),
            (
                ActivityDrawerSlot::RightTop,
                tool_window_stack(ActivityDrawerSlot::RightTop, Some(target.clone()), true),
            ),
        ]);

        assert_eq!(
            active_shell_content_scope_for_pane(&tool_windows, &target.0),
            None
        );
    }
}

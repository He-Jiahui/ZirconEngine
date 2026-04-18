use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;
use crate::ui::workbench::model::{FloatingWindowModel, WorkbenchViewModel};
use crate::{LayoutCommand, MainPageId, ViewInstanceId};

use super::dispatch_layout_command;

pub(crate) fn resolve_floating_window_focus_instance(
    window: &FloatingWindowModel,
) -> Option<ViewInstanceId> {
    window.focus_target_instance().cloned()
}

pub(crate) fn resolve_floating_window_close_instances(
    window: &FloatingWindowModel,
) -> Option<Vec<ViewInstanceId>> {
    if window.tabs.is_empty() || window.tabs.iter().any(|tab| !tab.closeable) {
        return None;
    }
    Some(
        window
            .tabs
            .iter()
            .map(|tab| tab.instance_id.clone())
            .collect(),
    )
}

pub(crate) fn dispatch_builtin_floating_window_focus(
    runtime: &EditorEventRuntime,
    window_id: &MainPageId,
) -> Option<Result<SlintDispatchEffects, String>> {
    let chrome = runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let window = model
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    let instance_id = resolve_floating_window_focus_instance(window)?;
    Some(dispatch_layout_command(
        runtime,
        LayoutCommand::FocusView { instance_id },
    ))
}

pub(crate) fn dispatch_builtin_floating_window_focus_for_source(
    runtime: &EditorEventRuntime,
    source_window_id: Option<&MainPageId>,
    last_focused_window_id: Option<&MainPageId>,
) -> Option<Result<SlintDispatchEffects, String>> {
    let window_id = source_window_id?;
    if Some(window_id) == last_focused_window_id {
        return None;
    }
    dispatch_builtin_floating_window_focus(runtime, window_id)
}

pub(crate) fn resolve_builtin_floating_window_close_instances(
    runtime: &EditorEventRuntime,
    window_id: &MainPageId,
) -> Option<Vec<ViewInstanceId>> {
    let chrome = runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let window = model
        .floating_windows
        .iter()
        .find(|window| &window.window_id == window_id)?;
    resolve_floating_window_close_instances(window)
}

#[cfg(test)]
mod tests {
    use crate::layout::{MainPageId, WorkspaceTarget};
    use crate::snapshot::ViewContentKind;
    use crate::ui::workbench::model::{DocumentTabModel, FloatingWindowModel};
    use crate::{ViewDescriptorId, ViewInstanceId};

    use super::{resolve_floating_window_close_instances, resolve_floating_window_focus_instance};

    #[test]
    fn resolve_floating_window_focus_instance_prefers_explicit_focused_view() {
        let focused_view = ViewInstanceId::new("editor.prefab#2");
        let window = floating_window(
            MainPageId::new("window:prefab"),
            Some(focused_view.clone()),
            &[
                tab_spec("editor.scene#1", true),
                tab_spec("editor.prefab#2", false),
            ],
        );

        assert_eq!(
            resolve_floating_window_focus_instance(&window),
            Some(focused_view)
        );
    }

    #[test]
    fn resolve_floating_window_focus_instance_falls_back_to_active_tab() {
        let active_view = ViewInstanceId::new("editor.game#1");
        let window = floating_window(
            MainPageId::new("window:game"),
            None,
            &[
                tab_spec("editor.scene#1", false),
                tab_spec("editor.game#1", true),
            ],
        );

        assert_eq!(
            resolve_floating_window_focus_instance(&window),
            Some(active_view)
        );
    }

    #[test]
    fn resolve_floating_window_focus_instance_falls_back_to_first_tab_when_none_active() {
        let first_view = ViewInstanceId::new("editor.console#1");
        let window = floating_window(
            MainPageId::new("window:console"),
            None,
            &[
                tab_spec("editor.console#1", false),
                tab_spec("editor.assets#1", false),
            ],
        );

        assert_eq!(
            resolve_floating_window_focus_instance(&window),
            Some(first_view)
        );
    }

    #[test]
    fn resolve_floating_window_focus_instance_ignores_stale_focused_view_and_uses_active_tab() {
        let active_view = ViewInstanceId::new("editor.game#1");
        let window = floating_window(
            MainPageId::new("window:preview"),
            Some(ViewInstanceId::new("editor.missing#1")),
            &[
                tab_spec("editor.scene#1", false),
                tab_spec("editor.game#1", true),
            ],
        );

        assert_eq!(
            resolve_floating_window_focus_instance(&window),
            Some(active_view)
        );
    }

    #[test]
    fn resolve_floating_window_close_instances_returns_all_tabs_when_every_tab_is_closeable() {
        let window = floating_window(
            MainPageId::new("window:assets"),
            None,
            &[
                close_tab_spec("editor.assets#1", true, true),
                close_tab_spec("editor.prefab#2", false, true),
            ],
        );

        assert_eq!(
            resolve_floating_window_close_instances(&window),
            Some(vec![
                ViewInstanceId::new("editor.assets#1"),
                ViewInstanceId::new("editor.prefab#2"),
            ])
        );
    }

    #[test]
    fn resolve_floating_window_close_instances_blocks_when_any_tab_is_non_closeable() {
        let window = floating_window(
            MainPageId::new("window:scene"),
            None,
            &[
                close_tab_spec("editor.scene#1", true, false),
                close_tab_spec("editor.prefab#2", false, true),
            ],
        );

        assert_eq!(resolve_floating_window_close_instances(&window), None);
    }

    fn floating_window(
        window_id: MainPageId,
        focused_view: Option<ViewInstanceId>,
        tabs: &[(&str, bool, bool)],
    ) -> FloatingWindowModel {
        FloatingWindowModel {
            window_id: window_id.clone(),
            title: window_id.0.clone(),
            requested_frame: crate::ShellFrame::default(),
            focused_view,
            tabs: tabs
                .iter()
                .enumerate()
                .map(
                    |(index, (instance_id, active, closeable))| DocumentTabModel {
                        workspace: WorkspaceTarget::FloatingWindow(window_id.clone()),
                        workspace_path: vec![index],
                        instance_id: ViewInstanceId::new(*instance_id),
                        descriptor_id: ViewDescriptorId::new("editor.placeholder"),
                        title: (*instance_id).to_string(),
                        icon_key: "placeholder".to_string(),
                        content_kind: ViewContentKind::Placeholder,
                        active: *active,
                        closeable: *closeable,
                        empty_state: None,
                    },
                )
                .collect(),
        }
    }

    fn tab_spec(instance_id: &'static str, active: bool) -> (&'static str, bool, bool) {
        close_tab_spec(instance_id, active, true)
    }

    fn close_tab_spec(
        instance_id: &'static str,
        active: bool,
        closeable: bool,
    ) -> (&'static str, bool, bool) {
        (instance_id, active, closeable)
    }
}

//! 菜单动作与宿主事件之间的无窗口编解码契约；这些测试不执行项目、Play或视图命令。
use crate::core::editor_event::{
    ConsoleMessageFilter, ConsoleSourceFilter, MenuAction, ViewDescriptorId,
};
use crate::core::play::PlayKind;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::workbench::event::{
    dispatch_editor_host_binding, menu_action_binding, EditorHostEvent,
};
use zircon_runtime::scene::components::NodeKind;

#[test]
/// 普通视图打开意图经绑定恢复为同一typed菜单事件；后续布局应用不在此测试范围。
fn menu_action_binding_roundtrips_through_headless_dispatch() {
    let action = MenuAction::OpenView(ViewDescriptorId::new("editor.scene"));
    let binding = menu_action_binding(&action);

    assert_eq!(
        binding.native_binding(),
        r#"WorkbenchMenuBar/OpenView.editor.scene:onClick(MenuAction("workbench.view.open.editor.scene"))"#
    );
    assert_eq!(
        dispatch_editor_host_binding(&binding).unwrap(),
        EditorHostEvent::Menu(action)
    );
}

#[test]
/// 调试窗口沿通用打开视图链编码，避免为其引入另一套菜单事件语义。
fn debug_observatory_window_menu_binding_roundtrips_through_headless_dispatch() {
    let action = MenuAction::OpenView(ViewDescriptorId::new("editor.debug_observatory"));
    let binding = menu_action_binding(&action);

    assert_eq!(
        binding.native_binding(),
        r#"WorkbenchMenuBar/OpenView.editor.debug_observatory:onClick(MenuAction("workbench.view.open.editor.debug_observatory"))"#
    );
    assert_eq!(
        dispatch_editor_host_binding(&binding).unwrap(),
        EditorHostEvent::Menu(action)
    );
}

#[test]
/// Play模式选择与生命周期动作保持各自事件身份；未创建Play世界或保留变更。
fn play_mode_menu_action_bindings_roundtrip_through_headless_dispatch() {
    for (action, expected_binding) in [
        (
            MenuAction::SelectPlayMode(PlayKind::Play),
            r#"WorkbenchMenuBar/SelectPlayMode.Play:onClick(MenuAction("workbench.play_mode.select.play"))"#,
        ),
        (
            MenuAction::SelectPlayMode(PlayKind::Simulate),
            r#"WorkbenchMenuBar/SelectPlayMode.Simulate:onClick(MenuAction("workbench.play_mode.select.simulate"))"#,
        ),
        (
            MenuAction::EnterPlayMode,
            r#"WorkbenchMenuBar/EnterPlayMode:onClick(MenuAction("workbench.play_mode.enter"))"#,
        ),
        (
            MenuAction::KeepPlayChanges,
            r#"WorkbenchMenuBar/KeepPlayChanges:onClick(MenuAction("workbench.play_mode.keep_changes"))"#,
        ),
        (
            MenuAction::ExitPlayMode,
            r#"WorkbenchMenuBar/ExitPlayMode:onClick(MenuAction("workbench.play_mode.exit"))"#,
        ),
    ] {
        let binding = menu_action_binding(&action);

        assert_eq!(binding.native_binding(), expected_binding);
        assert_eq!(
            dispatch_editor_host_binding(&binding).unwrap(),
            EditorHostEvent::Menu(action)
        );
    }
}

#[test]
/// 关闭项目意图能经菜单绑定返回；资源释放和会话关闭仍属事件执行器契约。
fn project_close_menu_action_binding_roundtrips_through_headless_dispatch() {
    let action = MenuAction::CloseProject;
    let binding = menu_action_binding(&action);

    assert_eq!(
        binding.native_binding(),
        r#"WorkbenchMenuBar/CloseProject:onClick(MenuAction("workbench.project.close"))"#
    );
    assert_eq!(
        dispatch_editor_host_binding(&binding).unwrap(),
        EditorHostEvent::Menu(action)
    );
}

#[test]
/// 消息级别过滤保留typed值，供宿主后续刷新console投影。
fn console_filter_menu_action_bindings_roundtrip_through_headless_dispatch() {
    for filter in [
        ConsoleMessageFilter::All,
        ConsoleMessageFilter::Info,
        ConsoleMessageFilter::Warning,
        ConsoleMessageFilter::Error,
    ] {
        let action = MenuAction::SetConsoleMessageFilter(filter);
        let binding = menu_action_binding(&action);

        assert_eq!(
            dispatch_editor_host_binding(&binding).unwrap(),
            EditorHostEvent::Menu(action)
        );
    }
}

#[test]
/// 来源过滤与消息级别过滤分别编码，防止不同过滤维度误用同一事件。
fn console_source_filter_bindings_roundtrip_through_headless_dispatch() {
    for filter in [
        ConsoleSourceFilter::All,
        ConsoleSourceFilter::Editor,
        ConsoleSourceFilter::Runtime,
        ConsoleSourceFilter::Play,
        ConsoleSourceFilter::Plugin,
        ConsoleSourceFilter::Import,
        ConsoleSourceFilter::ScriptBuild,
    ] {
        let action = MenuAction::SetConsoleSourceFilter(filter);
        let binding = menu_action_binding(&action);

        assert_eq!(
            dispatch_editor_host_binding(&binding).unwrap(),
            EditorHostEvent::Menu(action)
        );
    }
}

#[test]
/// 明确当前规范ID和仍支持的旧菜单拼写都恢复为同一动作，限制兼容范围由枚举用例决定。
fn dotted_menu_action_ids_roundtrip_through_headless_dispatch() {
    for (action_id, expected_action) in [
        (
            "workbench.scene.node.create.cube",
            MenuAction::CreateNode(NodeKind::Cube),
        ),
        (
            "workbench.view.open.editor.scene",
            MenuAction::OpenView(ViewDescriptorId::new("editor.scene")),
        ),
        ("CreateNode.Cube", MenuAction::CreateNode(NodeKind::Cube)),
        (
            "OpenView.editor.scene",
            MenuAction::OpenView(ViewDescriptorId::new("editor.scene")),
        ),
        (
            "menu_action.workbench.project.save",
            MenuAction::SaveProject,
        ),
        ("SaveProject", MenuAction::SaveProject),
        (
            "menu_action.workbench.project.close",
            MenuAction::CloseProject,
        ),
        ("CloseProject", MenuAction::CloseProject),
    ] {
        let binding = EditorUiBinding::new(
            "WorkbenchMenuBar",
            action_id,
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action(action_id),
        );

        assert_eq!(
            dispatch_editor_host_binding(&binding).unwrap(),
            EditorHostEvent::Menu(expected_action)
        );
    }
}

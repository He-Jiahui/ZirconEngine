use super::super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::core::i18n::EditorLocale;
use crate::ui::binding::EditorUiBindingPayload;
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiSize};

#[test]
fn icon_library_find_usage_click_reports_unavailable_without_fake_results() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("the built-in workbench surface should mount");

    let binding = bridge
        .dispatch_control_state(
            "WorkbenchExtensionIconLibraryFindUsageButton",
            UiEventKind::Click,
        )
        .expect("the installed Click binding should dispatch")
        .expect("Find Usage should have an installed Click binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.extension.icon_library.find_usage.invoke"
    ));

    assert_eq!(
        bridge
            .control_string("WorkbenchStatusReady", "text")
            .as_deref(),
        Some("Icon usage search unavailable")
    );
    let output = bridge
        .control_string("WorkbenchExtensionIconLibraryOutputRow", "value_text")
        .expect("Find Usage should report its current capability");
    assert_eq!(output, "No icon reference scanner is connected.");
    assert!(!output.to_ascii_lowercase().contains("queued"));
    assert!(!output.contains("14 references"));

    bridge
        .i18n
        .set_active_locale(EditorLocale::parse("zh-CN").expect("valid locale"))
        .expect("the test locale should be available");
    bridge
        .dispatch_control_state(
            "WorkbenchExtensionIconLibraryFindUsageButton",
            UiEventKind::Click,
        )
        .expect("the installed Click binding should dispatch")
        .expect("Find Usage should have an installed Click binding");

    assert_eq!(
        bridge
            .control_string("WorkbenchStatusReady", "text")
            .as_deref(),
        Some("图标使用情况查询不可用")
    );
    let output = bridge
        .control_string("WorkbenchExtensionIconLibraryOutputRow", "value_text")
        .expect("Find Usage should report its current capability");
    assert_eq!(output, "尚未连接图标引用扫描器。");
    assert!(!output.to_ascii_lowercase().contains("queued"));
    assert!(!output.contains("14 references"));
}

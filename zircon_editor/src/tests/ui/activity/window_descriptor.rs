//! 验证UI 资源编辑器窗口描述符与共享资产编辑器的宿主、标题和挂载路径一致。

use crate::ui::asset_editor::ui_asset_editor_window_descriptor;
use zircon_runtime_interface::ui::event_ui::UiNodePath;

#[test]
fn ui_asset_editor_window_descriptor_matches_shared_asset_editor_contract() {
    let descriptor = ui_asset_editor_window_descriptor();

    assert_eq!(descriptor.window_id, "editor.ui_asset");
    assert_eq!(descriptor.title, "UI Asset Editor");
    assert!(descriptor.multi_instance);
    assert!(descriptor.supports_document_tab);
    assert!(descriptor.supports_exclusive_page);
    assert!(descriptor.supports_floating_window);
    assert_eq!(
        descriptor.reflection_root,
        UiNodePath::new("editor/windows/editor.ui_asset")
    );
}

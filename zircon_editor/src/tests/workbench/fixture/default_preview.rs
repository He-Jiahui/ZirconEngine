//! 共享预览材料的默认布局、描述符和实例集合契约。
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};

#[test]
/// 固定参考预览具备workbench页、语义抽屉及基本视图；不以此证明项目恢复成功。
fn default_preview_fixture_loads_shared_workbench_state() {
    let fixture = default_preview_fixture();

    assert_eq!(fixture.layout.active_main_page, MainPageId::workbench());
    assert_eq!(fixture.layout.active_activity_window_drawers().len(), 5);
    assert!(fixture
        .layout
        .active_activity_window_drawers()
        .contains_key(&ActivityDrawerSlot::LeftTop));
    assert!(fixture
        .descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.scene"));
    assert!(fixture
        .descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.game"));
    assert!(fixture
        .descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.assets"));
    assert!(fixture
        .instances
        .iter()
        .any(|instance| instance.descriptor_id.0 == "editor.scene"));
    assert!(fixture
        .instances
        .iter()
        .any(|instance| instance.descriptor_id.0 == "editor.game"));
    assert!(fixture
        .instances
        .iter()
        .any(|instance| instance.descriptor_id.0 == "editor.assets"));
}

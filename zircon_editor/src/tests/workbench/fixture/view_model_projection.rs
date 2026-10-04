//! 共享fixture经过chrome到视图模型的壳层投影检查。
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
/// 检查页、抽屉和状态行来自同一fixture材料；命令注册表只参与模型构建。
fn default_preview_fixture_projects_into_workbench_view_model() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let view_model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    assert_eq!(view_model.host_strip.active_page, MainPageId::workbench());
    assert!(view_model.drawer_ring.visible);
    assert_eq!(
        view_model.status_bar.primary_text,
        fixture.editor.status_line
    );
}

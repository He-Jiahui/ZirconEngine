use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn default_preview_fixture_projects_into_workbench_view_model() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let view_model = WorkbenchViewModel::build(&chrome);

    assert_eq!(view_model.host_strip.active_page, MainPageId::workbench());
    assert!(view_model.drawer_ring.visible);
    assert_eq!(
        view_model.status_bar.primary_text,
        fixture.editor.status_line
    );
}

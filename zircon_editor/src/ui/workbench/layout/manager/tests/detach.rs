use super::*;
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::view::ViewHost;

#[test]
fn closing_the_active_exclusive_page_reports_change_and_restores_a_valid_active_page() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let instance_id = ViewInstanceId::new("editor.asset_browser#1");
    let page_id = MainPageId::new("page:editor.asset_browser#1");
    manager
        .apply(
            &mut layout,
            LayoutCommand::AttachView {
                instance_id: instance_id.clone(),
                target: ViewHost::ExclusivePage(page_id.clone()),
                anchor: None,
            },
        )
        .expect("exclusive page should attach");
    assert_eq!(layout.active_main_page, page_id);

    let close = manager
        .apply(&mut layout, LayoutCommand::CloseView { instance_id })
        .expect("exclusive page should close");

    assert!(close.changed);
    assert_eq!(layout.active_main_page, MainPageId::workbench());
    assert!(layout.main_pages.iter().all(|page| page.id() != &page_id));
}

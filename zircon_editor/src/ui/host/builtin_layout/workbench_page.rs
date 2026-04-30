use crate::ui::workbench::layout::{
    ActivityWindowId, DocumentNode, MainHostPageLayout, MainPageId, TabStackLayout,
};
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn builtin_workbench_page() -> MainHostPageLayout {
    MainHostPageLayout::WorkbenchPage {
        id: MainPageId::workbench(),
        title: "Workbench".to_string(),
        activity_window: ActivityWindowId::workbench(),
        document_workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                ViewInstanceId::new("editor.scene#1"),
                ViewInstanceId::new("editor.game#1"),
            ],
            active_tab: Some(ViewInstanceId::new("editor.scene#1")),
        }),
    }
}

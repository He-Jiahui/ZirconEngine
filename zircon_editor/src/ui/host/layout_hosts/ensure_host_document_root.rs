use crate::ui::workbench::layout::{
    ActivityWindowId, DocumentNode, MainHostPageLayout, MainPageId, WorkbenchLayout,
};

pub(super) fn ensure_host_document_root(layout: &mut WorkbenchLayout) -> &mut DocumentNode {
    if let Some(index) = layout
        .main_pages
        .iter()
        .position(|page| matches!(page, MainHostPageLayout::WorkbenchPage { .. }))
    {
        match &mut layout.main_pages[index] {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => document_workspace,
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => unreachable!(),
        }
    } else {
        layout.main_pages.insert(
            0,
            MainHostPageLayout::WorkbenchPage {
                id: MainPageId::workbench(),
                title: "Workbench".to_string(),
                activity_window: ActivityWindowId::workbench(),
                document_workspace: DocumentNode::default(),
            },
        );
        match &mut layout.main_pages[0] {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => document_workspace,
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => unreachable!(),
        }
    }
}

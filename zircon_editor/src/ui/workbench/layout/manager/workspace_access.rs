use super::super::{DocumentNode, LayoutManager, MainPageId, WorkbenchLayout, WorkspaceTarget};

impl LayoutManager {
    pub(crate) fn document_node_mut<'a>(
        &self,
        layout: &'a mut WorkbenchLayout,
        page_id: &MainPageId,
        path: &[usize],
    ) -> Option<&'a mut DocumentNode> {
        let workspace = layout.content_workspace_for_page_mut(page_id)?;
        workspace.node_at_path_mut(path)
    }

    pub(crate) fn workspace_node_mut<'a>(
        &self,
        layout: &'a mut WorkbenchLayout,
        workspace: &WorkspaceTarget,
        path: &[usize],
    ) -> Option<&'a mut DocumentNode> {
        match workspace {
            WorkspaceTarget::MainPage(page_id) => self.document_node_mut(layout, page_id, path),
            WorkspaceTarget::FloatingWindow(window_id) => layout
                .floating_windows
                .iter_mut()
                .find(|window| &window.window_id == window_id)
                .and_then(|window| window.workspace.node_at_path_mut(path)),
        }
    }
}

use super::super::{DocumentNode, LayoutManager, MainPageId, WorkbenchLayout, WorkspaceTarget};

impl LayoutManager {
    /// 将主页面关联到活动窗口内容树后解析路径；独占页或缺失owner返回空。
    pub(crate) fn document_node_mut<'a>(
        &self,
        layout: &'a mut WorkbenchLayout,
        page_id: &MainPageId,
        path: &[usize],
    ) -> Option<&'a mut DocumentNode> {
        let workspace = layout.content_workspace_for_page_mut(page_id)?;
        workspace.node_at_path_mut(path)
    }

    /// 连同工作区owner解析路径，避免把不同窗口的同一子路径混用。
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

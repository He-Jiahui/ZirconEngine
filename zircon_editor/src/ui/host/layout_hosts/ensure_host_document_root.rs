use crate::ui::workbench::layout::{DocumentNode, WorkbenchLayout};

pub(super) fn ensure_host_document_root(layout: &mut WorkbenchLayout) -> &mut DocumentNode {
    layout.ensure_workbench_content_workspace()
}

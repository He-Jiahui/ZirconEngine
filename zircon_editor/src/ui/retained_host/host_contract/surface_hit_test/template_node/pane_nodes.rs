use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{PaneData, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract) fn pane_template_nodes(
    pane: &PaneData,
) -> Option<&ModelRc<TemplatePaneNodeData>> {
    pane.template_nodes()
}

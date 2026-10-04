use crate::ui::retained_host::host_contract::data::{PaneData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn pane_template_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    pane.template_nodes()
}

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::super::data::{PaneData, TemplatePaneNodeData};

pub(super) fn select_pane_template_nodes(
    pane: &PaneData,
) -> Option<&ModelRc<TemplatePaneNodeData>> {
    pane.template_nodes()
}

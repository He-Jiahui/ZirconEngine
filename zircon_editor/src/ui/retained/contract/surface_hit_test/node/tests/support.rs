use std::rc::Rc;

use crate::ui::retained_host::host_contract::data::{
    HostWindowPresentationData, TemplatePaneNodeData, TemplatePaneOptionData,
};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::{
    hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
    TemplateNodePointerHit,
};

pub(super) fn hit_test_workbench_window_template_node(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    hit_test_workbench_window_template_node_with_index(presentation, &index, x, y)
}

pub(super) fn option(id: &str, disabled: bool) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: id.into(),
        disabled,
        ..TemplatePaneOptionData::default()
    }
}

pub(super) fn workbench_node(
    presentation: &HostWindowPresentationData,
    control_id: &str,
) -> TemplatePaneNodeData {
    (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

pub(super) fn model<T: Clone>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}

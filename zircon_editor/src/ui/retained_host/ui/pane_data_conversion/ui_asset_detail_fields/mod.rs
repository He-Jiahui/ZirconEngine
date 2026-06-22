use crate::ui::asset_editor;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;

use super::super::template_node_conversion::to_host_contract_template_node_owned;

mod binding;
mod layout;
mod row_model;
mod section_nodes;
mod sections;
mod slot;
mod widget;

pub(super) fn to_host_contract_ui_asset_template_nodes(
    items: Vec<ViewTemplateNodeData>,
    data: &asset_editor::UiAssetEditorPanePresentation,
    prop_state_rows: &[asset_editor::UiAssetEditorWidgetPropStateItem],
    instance_id: &str,
) -> ModelRc<host_contract::TemplatePaneNodeData> {
    let mut nodes = items
        .into_iter()
        .map(to_host_contract_template_node_owned)
        .collect::<Vec<_>>();
    for section in sections::ui_asset_detail_field_sections(data, prop_state_rows) {
        section_nodes::append_detail_section_nodes(&mut nodes, &section, instance_id);
    }
    model_rc(nodes)
}

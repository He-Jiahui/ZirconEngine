mod actions;
mod collections;
mod designer_tools;
mod header;
mod inspector;
mod palette_drag;
mod preview;
mod runtime_report;
mod source;
mod string_selection;
mod style;

use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

use self::actions::to_host_contract_ui_asset_actions;
use self::collections::to_host_contract_ui_asset_collections;
use self::designer_tools::to_host_contract_ui_asset_designer_tools;
use self::header::to_host_contract_ui_asset_header;
use self::inspector::to_host_contract_ui_asset_inspector_panel;
use self::palette_drag::to_host_contract_ui_asset_palette_drag;
use self::preview::to_host_contract_ui_asset_preview_panel;
use self::runtime_report::to_host_contract_ui_asset_runtime_report;
use self::source::to_host_contract_ui_asset_source;
use self::style::to_host_contract_ui_asset_style_panel;
use super::super::template_node_conversion::to_host_contract_template_node_owned;
use super::ui_asset_detail_fields::to_host_contract_ui_asset_template_nodes;

pub(in super::super) fn to_host_contract_ui_asset_pane(
    mut data: asset_editor::UiAssetEditorPanePresentation,
    instance_id: &str,
) -> host_contract::UiAssetEditorPaneData {
    let template_nodes = to_host_contract_ui_asset_template_nodes(
        std::mem::take(&mut data.nodes),
        &data,
        &data.inspector_widget_prop_state_rows,
        instance_id,
    );
    let preview = to_host_contract_ui_asset_preview_panel(&mut data);
    let style = to_host_contract_ui_asset_style_panel(&mut data);
    let inspector = to_host_contract_ui_asset_inspector_panel(&mut data);
    let runtime_report = to_host_contract_ui_asset_runtime_report(&mut data);
    let designer_tools = to_host_contract_ui_asset_designer_tools(&mut data);
    let palette_drag = to_host_contract_ui_asset_palette_drag(&mut data);
    let header = to_host_contract_ui_asset_header(&mut data);
    let actions = to_host_contract_ui_asset_actions(&data);
    let collections = to_host_contract_ui_asset_collections(&mut data);
    let source = to_host_contract_ui_asset_source(&mut data);

    host_contract::UiAssetEditorPaneData {
        nodes: template_nodes,
        center_column_node: to_host_contract_template_node_owned(data.center_column_node),
        designer_panel_node: to_host_contract_template_node_owned(data.designer_panel_node),
        designer_canvas_panel_node: to_host_contract_template_node_owned(
            data.designer_canvas_panel_node,
        ),
        inspector_panel_node: to_host_contract_template_node_owned(data.inspector_panel_node),
        stylesheet_panel_node: to_host_contract_template_node_owned(data.stylesheet_panel_node),
        header,
        actions,
        collections,
        source,
        preview,
        runtime_report,
        designer_tools,
        palette_drag,
        style,
        inspector,
    }
}

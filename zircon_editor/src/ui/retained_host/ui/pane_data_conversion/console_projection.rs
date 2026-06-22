use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    ConsolePaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::template_node_conversion::to_host_contract_template_node_owned;
use super::pane_component_projection::host_template_node;
use super::pane_template_runtime;
use super::pane_value_conversion::value_as_string;
use super::template_node_projection::project_nodes;

fn to_host_contract_console_pane(data: ConsolePaneViewData) -> host_contract::ConsolePaneData {
    host_contract::ConsolePaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_console_legacy_node),
        status_text: data.status_text,
    }
}

pub(crate) fn to_host_contract_console_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, None)
        .unwrap_or_else(|| to_host_contract_console_pane(data.native_body.console.clone()))
}

pub(crate) fn to_host_contract_console_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, Some(runtime))
        .unwrap_or_else(|| to_host_contract_console_pane(data.native_body.console.clone()))
}

fn console_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<host_contract::ConsolePaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(&presentation.body.payload, PanePayload::ConsoleV1(_)) {
        return None;
    }

    let runtime = pane_template_runtime(runtime)?;
    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;
    let status_text = projection
        .root
        .attributes
        .get("payload_status_text")
        .and_then(value_as_string)
        .or_else(|| match &presentation.body.payload {
            PanePayload::ConsoleV1(payload) => Some(payload.status_text.clone()),
            _ => None,
        })
        .unwrap_or_default();

    Some(host_contract::ConsolePaneData {
        nodes: model_rc(
            host_model
                .nodes
                .into_iter()
                .filter_map(host_template_node)
                .collect(),
        ),
        status_text: status_text.into(),
    })
}

fn to_host_contract_console_legacy_node(
    data: crate::ui::layouts::views::ViewTemplateNodeData,
) -> host_contract::TemplatePaneNodeData {
    let mut node = to_host_contract_template_node_owned(data);
    if node.control_id == "ConsoleTextPanel" {
        node.control_id = "ConsoleBodySection".into();
    }
    node
}

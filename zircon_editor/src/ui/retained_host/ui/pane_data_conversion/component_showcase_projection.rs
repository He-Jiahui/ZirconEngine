use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{PaneContentSize, PaneData, PanePayload};
use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime_interface::ui::layout::UiSize;

use super::pane_component_projection::host_template_node;

pub(crate) fn to_host_contract_component_showcase_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ProjectOverviewPaneData {
    super::builtin_host_runtime()
        .and_then(|runtime| component_showcase_template_projection(data, content_size, runtime))
        .unwrap_or_default()
}

pub(crate) fn to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::ProjectOverviewPaneData {
    component_showcase_template_projection(data, content_size, runtime).unwrap_or_default()
}

fn component_showcase_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> Option<host_contract::ProjectOverviewPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        PanePayload::UiComponentShowcaseV1(_)
    ) {
        return None;
    }

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

    Some(host_contract::ProjectOverviewPaneData {
        nodes: model_rc(
            host_model
                .nodes
                .into_iter()
                .filter_map(host_template_node)
                .collect(),
        ),
    })
}

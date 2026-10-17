//! 生成的底部 Pane 尝试消费当前模板文档；模板无法投影时沿用原生节点，但状态文本继续来自同一视图数据。
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{PaneContentSize, PaneData};
use crate::ui::retained_host as host_contract;

use super::super::template_node_conversion::to_host_contract_template_node;
use super::template_node_projection::project_nodes;

pub(crate) fn to_host_contract_generated_bottom_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::GeneratedBottomPaneData {
    let nodes = data
        .pane_presentation
        .as_ref()
        .and_then(|presentation| {
            super::project_pane_template_nodes(&presentation.body, content_size)
        })
        .map(model_rc)
        .unwrap_or_else(|| {
            project_nodes(
                &data.native_body.generated_bottom.nodes,
                to_host_contract_template_node,
            )
        });

    host_contract::GeneratedBottomPaneData {
        nodes,
        status: data.native_body.generated_bottom.status.clone(),
        ..host_contract::GeneratedBottomPaneData::default()
    }
}

pub(crate) fn to_host_contract_generated_bottom_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&crate::ui::template_runtime::EditorUiHostRuntime>,
) -> host_contract::GeneratedBottomPaneData {
    let Some(runtime) = runtime else {
        return to_host_contract_generated_bottom_pane_from_host_pane(data, content_size);
    };
    let Some(presentation) = data.pane_presentation.as_ref() else {
        return host_contract::GeneratedBottomPaneData::default();
    };
    let Ok(published) = runtime.project_generated_bottom_pane(
        data.id.as_str(),
        &presentation.body,
        zircon_runtime_interface::ui::layout::UiSize::new(content_size.width, content_size.height),
    ) else {
        return host_contract::GeneratedBottomPaneData::default();
    };
    // The adapter carries this published holder's leased actions in routes. The shared
    // converter preserves those actions, typed identity and strict Source frame binding.
    let nodes = super::super::workbench_window_projection::to_host_contract_workbench_window_nodes(
        Some(&published.projection),
    );
    let source = published.projection.source_surface_frame;
    host_contract::GeneratedBottomPaneData {
        nodes,
        status: data.native_body.generated_bottom.status.clone(),
        source_surface_frame: source,
        accepts_text_input: published.accepts_text_input,
        source_width: content_size.width,
        source_height: content_size.height,
        input_result: published.input_result,
    }
}

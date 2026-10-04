use crate::ui::retained_host::host_contract::data::{PaneData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn ui_asset_pane(id: &str, nodes: ModelRc<TemplatePaneNodeData>) -> PaneData {
    PaneData {
        id: id.into(),
        kind: "UiAssetEditor".into(),
        ui_asset: crate::ui::retained_host::host_contract::UiAssetEditorPaneData {
            nodes,
            ..Default::default()
        },
        ..PaneData::default()
    }
}

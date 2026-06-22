use crate::ui::retained_host::host_contract::data::PaneData;

#[derive(Clone, Copy)]
pub(super) enum PaneRouteMode {
    Default,
    PointerMove,
}

impl PaneRouteMode {
    pub(super) fn allows_template_hit_for_move(self, pane: &PaneData) -> bool {
        match self {
            Self::Default => true,
            Self::PointerMove => !matches!(
                pane.kind.as_str(),
                "Hierarchy" | "Welcome" | "Assets" | "AssetBrowser"
            ),
        }
    }
}

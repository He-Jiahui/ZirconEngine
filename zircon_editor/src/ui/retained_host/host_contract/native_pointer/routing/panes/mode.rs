use crate::ui::retained_host::host_contract::data::PaneData;

#[derive(Clone, Copy)]
pub(super) enum PaneRouteMode {
    Default,
    PointerMove,
    PointerScroll,
}

impl PaneRouteMode {
    pub(super) fn allows_template_hit(self, pane: &PaneData) -> bool {
        match self {
            Self::Default => true,
            Self::PointerMove => !matches!(
                pane.kind.as_str(),
                "Hierarchy" | "Welcome" | "Assets" | "AssetBrowser"
            ),
            Self::PointerScroll => pane.kind.as_str() != "Console",
        }
    }

    pub(super) fn uses_console_output_viewport(self) -> bool {
        matches!(self, Self::PointerScroll)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuntimeUiFixture {
    HudOverlay,
    PauseMenu,
    SettingsDialog,
    InventoryList,
}

impl RuntimeUiFixture {
    pub(crate) fn asset_id(self) -> &'static str {
        match self {
            Self::HudOverlay => "runtime.ui.hud_overlay",
            Self::PauseMenu => "runtime.ui.pause_menu",
            Self::SettingsDialog => "runtime.ui.settings_dialog",
            Self::InventoryList => "runtime.ui.inventory_list",
        }
    }

    pub(crate) fn source(self) -> &'static str {
        match self {
            Self::HudOverlay => include_str!("fixtures/hud_overlay.ui.toml"),
            Self::PauseMenu => include_str!("fixtures/pause_menu.ui.toml"),
            Self::SettingsDialog => include_str!("fixtures/settings_dialog.ui.toml"),
            Self::InventoryList => include_str!("fixtures/inventory_list.ui.toml"),
        }
    }
}

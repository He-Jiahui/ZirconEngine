use std::path::{Path, PathBuf};

use crate::asset::runtime_asset_path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum RuntimeUiFixture {
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

    pub(crate) fn relative_asset_path(self) -> &'static Path {
        match self {
            Self::HudOverlay => Path::new("ui/runtime/fixtures/hud_overlay.ui.toml"),
            Self::PauseMenu => Path::new("ui/runtime/fixtures/pause_menu.ui.toml"),
            Self::SettingsDialog => Path::new("ui/runtime/fixtures/settings_dialog.ui.toml"),
            Self::InventoryList => Path::new("ui/runtime/fixtures/inventory_list.ui.toml"),
        }
    }

    pub(crate) fn asset_path(self) -> PathBuf {
        runtime_asset_path(self.relative_asset_path())
    }
}

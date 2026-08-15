use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::template::{UiAssetKind, UiNodeDefinitionKind};
use zircon_runtime_interface::ui::v2::{UI_V2_ASSET_SCHEMA_VERSION, UiV2AssetKind};

use crate::ui::host::EditorManager;
use crate::ui::host::module::EDITOR_MANAGER_NAME;

use super::support::*;

mod component_promotion;
mod reference_navigation;
mod theme;
mod tree_and_component_transforms;

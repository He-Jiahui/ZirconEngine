use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::UiAssetDocument;
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

#[derive(Default)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetImportDocuments {
    pub(in crate::ui::host::asset_editor_sessions) widgets: BTreeMap<String, UiAssetDocument>,
    pub(in crate::ui::host::asset_editor_sessions) styles: BTreeMap<String, UiAssetDocument>,
    pub(in crate::ui::host::asset_editor_sessions) v2_widgets: BTreeMap<String, UiV2AssetDocument>,
    pub(in crate::ui::host::asset_editor_sessions) v2_styles: BTreeMap<String, UiV2AssetDocument>,
}

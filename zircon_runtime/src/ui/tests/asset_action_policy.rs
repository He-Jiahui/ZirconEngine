use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use zircon_runtime_interface::ui::template::{
    UiActionSideEffectClass, UiCompiledAssetPackageProfile,
};

const LOCAL_ACTION_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.local"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "LocalAction"

[[root.bindings]]
id = "Local/OpenPopup"
event = "Click"
route = "UiComponentShowcase.SelectCategory.All"
"##;

const ASSET_IO_ACTION_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.asset_io"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "SaveAsset"

[[root.bindings]]
id = "Asset/Save"
event = "Click"
route = "Asset.Save"
"##;

const NETWORK_ACTION_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.action_policy.network"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "FetchRemote"

[[root.bindings]]
id = "Network/Fetch"
event = "Click"
route = "Network.Fetch"
"##;

#[test]
fn action_policy_allows_local_ui_actions_in_runtime_profile() {
    let document = UiAssetLoader::load_toml_str(LOCAL_ACTION_LAYOUT).unwrap();
    let report = UiDocumentCompiler::default()
        .validate_package(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();

    assert!(report.action_policy_report.is_allowed());
}

#[test]
fn action_policy_reports_asset_io_rejected_by_runtime_profile() {
    let document = UiAssetLoader::load_toml_str(ASSET_IO_ACTION_LAYOUT).unwrap();
    let report = UiDocumentCompiler::default()
        .validate_package(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();

    assert_eq!(report.action_policy_report.diagnostics.len(), 1);
    let diagnostic = &report.action_policy_report.diagnostics[0];
    assert_eq!(diagnostic.node_id, "root");
    assert_eq!(diagnostic.binding_id, "Asset/Save");
    assert_eq!(diagnostic.side_effect, UiActionSideEffectClass::AssetIo);
}

#[test]
fn action_policy_allows_asset_io_in_editor_profile_but_not_network() {
    let asset_document = UiAssetLoader::load_toml_str(ASSET_IO_ACTION_LAYOUT).unwrap();
    let network_document = UiAssetLoader::load_toml_str(NETWORK_ACTION_LAYOUT).unwrap();
    let compiler = UiDocumentCompiler::default();

    let asset_report = compiler
        .validate_package(&asset_document, UiCompiledAssetPackageProfile::Editor)
        .unwrap();
    let network_report = compiler
        .validate_package(&network_document, UiCompiledAssetPackageProfile::Editor)
        .unwrap();

    assert!(asset_report.action_policy_report.is_allowed());
    assert_eq!(network_report.action_policy_report.diagnostics.len(), 1);
    assert_eq!(
        network_report.action_policy_report.diagnostics[0].side_effect,
        UiActionSideEffectClass::Network
    );
}

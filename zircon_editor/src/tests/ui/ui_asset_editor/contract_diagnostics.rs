use crate::ui::asset_editor::{UiAssetEditorDiagnosticSeverity, UiAssetEditorSession};
use zircon_runtime::ui::template::UiAssetLoader;
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

use super::support::open_design_session;

const PUBLIC_CARD_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.card"
version = 1

[root]
node_id = "widget_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "WidgetPreviewRoot"

[components.Card]

[components.Card.contract]
api_version = "1.2.0"

[components.Card.contract.public_parts.label]
node_id = "card_label"
control_id = "CardLabel"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"

[[components.Card.root.children]]
[components.Card.root.children.node]
node_id = "card_label"
kind = "native"
type = "Label"
control_id = "CardLabel"
props = { text = "Title" }

[[components.Card.root.children]]
[components.Card.root.children.node]
node_id = "card_secret"
kind = "native"
type = "Label"
control_id = "SecretLabel"
props = { text = "Internal" }
"##;

const CARD_LAYOUT_WITH_PRIVATE_SELECTOR: &str = r##"
[asset]
kind = "layout"
id = "editor.card_host"
version = 1

[imports]
widgets = ["asset://ui/common/card.ui#Card"]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/common/card.ui#Card"
control_id = "CardInstance"

[[stylesheets]]
id = "host_styles"

[[stylesheets.rules]]
selector = "#SecretLabel"
set = { self = { text = "Leaked" } }
"##;

const CARD_LAYOUT_WITH_INCOMPATIBLE_API: &str = r##"
[asset]
kind = "layout"
id = "editor.card_host"
version = 1

[imports]
widgets = ["asset://ui/common/card.ui#Card"]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/common/card.ui#Card"
component_api_version = "2.0.0"
control_id = "CardInstance"
"##;

const CLOSED_ROOT_CLASS_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.closed_card"
version = 1

[root]
node_id = "closed_card_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "ClosedCardPreviewRoot"

[components.Card]

[components.Card.contract]
root_class_policy = "closed"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"
"##;

const CLOSED_ROOT_CLASS_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.closed_card_host"
version = 1

[imports]
widgets = ["asset://ui/common/closed_card.ui#Card"]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/common/closed_card.ui#Card"
control_id = "ClosedCardInstance"
classes = ["host-shell"]
"##;

#[test]
fn ui_asset_editor_projects_private_selector_contract_diagnostic() {
    let mut session = open_design_session(
        "asset://ui/card_host.ui.toml",
        CARD_LAYOUT_WITH_PRIVATE_SELECTOR,
    );
    session
        .register_widget_import(
            "asset://ui/common/card.ui#Card",
            UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap(),
        )
        .unwrap();

    let diagnostic = session
        .structured_diagnostics()
        .first()
        .expect("private selector diagnostic");
    assert_eq!(diagnostic.code, "private_selector");
    assert_eq!(diagnostic.severity, UiAssetEditorDiagnosticSeverity::Error);
    assert_eq!(
        diagnostic.source_path,
        "stylesheets.host_styles.rules.0.selector"
    );
    assert_eq!(diagnostic.target_control_id.as_deref(), Some("SecretLabel"));

    let pane = session.pane_presentation();
    assert!(pane
        .structured_diagnostic_items
        .iter()
        .any(|item| item.contains("private_selector")));
}

#[test]
fn ui_asset_editor_projects_api_mismatch_to_source_outline_target() {
    let mut session = open_design_session(
        "asset://ui/card_host.ui.toml",
        CARD_LAYOUT_WITH_INCOMPATIBLE_API,
    );
    session
        .register_widget_import(
            "asset://ui/common/card.ui#Card",
            UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap(),
        )
        .unwrap();

    let diagnostic = session
        .structured_diagnostics()
        .first()
        .expect("API mismatch diagnostic");
    assert_eq!(diagnostic.code, "api_mismatch");
    assert_eq!(diagnostic.target_node_id.as_deref(), Some("root"));

    let pane = session.pane_presentation();
    assert_eq!(pane.source_outline_selected_index, 0);
}

#[test]
fn ui_asset_editor_projects_closed_root_class_contract_diagnostic() {
    let route = crate::ui::asset_editor::UiAssetEditorRoute::new(
        "asset://ui/closed_card_host.ui.toml",
        UiAssetKind::Layout,
        crate::ui::asset_editor::UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        CLOSED_ROOT_CLASS_LAYOUT,
        UiSize::new(1280.0, 720.0),
    )
    .unwrap();
    session
        .register_widget_import(
            "asset://ui/common/closed_card.ui#Card",
            UiAssetLoader::load_toml_str(CLOSED_ROOT_CLASS_WIDGET).unwrap(),
        )
        .unwrap();

    let diagnostic = session
        .structured_diagnostics()
        .first()
        .expect("closed root class diagnostic");
    assert_eq!(diagnostic.code, "closed_root_class");
    assert_eq!(diagnostic.source_path, "nodes.root.classes");
    assert_eq!(diagnostic.target_node_id.as_deref(), Some("root"));
}

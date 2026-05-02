use crate::ui::template::{component_contract_diagnostic, UiAssetLoader, UiDocumentCompiler};
use zircon_runtime_interface::ui::template::{
    UiComponentApiVersion, UiComponentContractDiagnosticCode, UiSelector, UiSelectorToken,
};

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

const CARD_LAYOUT_WITH_PUBLIC_PART_SELECTOR: &str = r##"
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
component_api_version = "1.1.0"
control_id = "CardInstance"

[[stylesheets]]
id = "host_styles"

[[stylesheets.rules]]
selector = ":part(label)"
set = { self = { text = "Projected" } }
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

const PUBLIC_BUTTON_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.button"
version = 1

[root]
node_id = "button_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "ButtonPreviewRoot"

[components.Button]

[components.Button.contract.public_parts.icon]
node_id = "button_icon"
control_id = "ButtonIcon"

[components.Button.root]
node_id = "button_root"
kind = "native"
type = "Button"
control_id = "ButtonRoot"

[[components.Button.root.children]]
[components.Button.root.children.node]
node_id = "button_icon"
kind = "native"
type = "Icon"
control_id = "ButtonIcon"
"##;

const MULTI_WIDGET_LAYOUT_WITH_SCOPED_PART_SELECTORS: &str = r##"
[asset]
kind = "layout"
id = "editor.multi_widget_host"
version = 1

[imports]
widgets = ["asset://ui/common/card.ui#Card", "asset://ui/common/button.ui#Button"]

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
control_id = "Root"

[[root.children]]
[root.children.node]
node_id = "card_instance"
kind = "reference"
component_ref = "asset://ui/common/card.ui#Card"
control_id = "CardInstance"

[[root.children]]
[root.children.node]
node_id = "button_instance"
kind = "reference"
component_ref = "asset://ui/common/button.ui#Button"
control_id = "ButtonInstance"

[[stylesheets]]
id = "scoped_parts"

[[stylesheets.rules]]
selector = "Card:part(label)"
set = { self = { text = "Card Label" } }

[[stylesheets.rules]]
selector = "Button:part(icon)"
set = { self = { icon = "bolt" } }
"##;

const CARD_LAYOUT_WITH_IMPORTED_PRIVATE_STYLE: &str = r##"
[asset]
kind = "layout"
id = "editor.card_host"
version = 1

[imports]
widgets = ["asset://ui/common/card.ui#Card"]
styles = ["asset://ui/common/private_card_style.ui"]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/common/card.ui#Card"
control_id = "CardInstance"
"##;

const PRIVATE_CARD_STYLE: &str = r##"
[asset]
kind = "style"
id = "ui.common.private_card_style"
version = 1

[[stylesheets]]
id = "private_card_style"

[[stylesheets.rules]]
selector = "#SecretLabel"
set = { self = { text = "Leaked" } }
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

const INVALID_PUBLIC_PART_CONTROL_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.bad_part_control"
version = 1

[root]
node_id = "bad_part_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "BadPartPreviewRoot"

[components.Card]

[components.Card.contract.public_parts.label]
node_id = "card_label"
control_id = "SecretLabel"

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

[[components.Card.root.children]]
[components.Card.root.children.node]
node_id = "card_secret"
kind = "native"
type = "Label"
control_id = "SecretLabel"
"##;

const INVALID_BINDING_CONTRACT_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.bad_binding"
version = 1

[root]
node_id = "bad_binding_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "BadBindingPreviewRoot"

[components.Card]

[components.Card.contract.bindings.public_actions.Open]
target = "SecretLabel"
payload_kind = "unit"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"

[[components.Card.root.children]]
[components.Card.root.children.node]
node_id = "card_secret"
kind = "native"
type = "Label"
control_id = "SecretLabel"
"##;

const INVALID_API_VERSION_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.bad_api"
version = 1

[root]
node_id = "bad_api_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "BadApiPreviewRoot"

[components.Card]

[components.Card.contract]
api_version = "1.0"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"
"##;

const INVALID_FOCUS_CONTRACT_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.common.bad_focus"
version = 1

[root]
node_id = "bad_focus_preview_root"
kind = "native"
type = "VerticalBox"
control_id = "BadFocusPreviewRoot"

[components.Card]

[components.Card.contract.focus]
initial_focus = "SecretLabel"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
control_id = "CardRoot"

[[components.Card.root.children]]
[components.Card.root.children.node]
node_id = "card_secret"
kind = "native"
type = "Label"
control_id = "SecretLabel"
"##;

#[test]
fn component_contract_defaults_to_closed_public_surface_and_api_version() {
    let document = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let card = document.components.get("Card").unwrap();

    assert_eq!(
        card.contract.api_version,
        UiComponentApiVersion {
            major: 1,
            minor: 2,
            patch: 0
        }
    );
    assert_eq!(card.contract.public_parts["label"].node_id, "card_label");
    assert_eq!(
        card.contract.public_parts["label"].control_id.as_deref(),
        Some("CardLabel")
    );
}

#[test]
fn selector_parser_materializes_public_part_tokens() {
    let selector = UiSelector::parse("Card:part(label)").unwrap();
    assert_eq!(
        selector.segments[0].tokens,
        vec![
            UiSelectorToken::Type("Card".to_string()),
            UiSelectorToken::Part("label".to_string())
        ]
    );
}

#[test]
fn component_contract_allows_public_part_selector_on_imported_widget() {
    let widget = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT_WITH_PUBLIC_PART_SELECTOR).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/card.ui#Card", widget)
        .unwrap();

    compiler
        .compile(&layout)
        .expect("public part selectors should satisfy the imported component contract");
}

#[test]
fn component_contract_rejects_private_selector_target_on_imported_widget() {
    let widget = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT_WITH_PRIVATE_SELECTOR).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/card.ui#Card", widget)
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("private imported component controls must not be targetable");
    assert!(
        error
            .to_string()
            .contains("private component internals SecretLabel"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_reports_private_selector_diagnostic_code_and_path() {
    let widget = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT_WITH_PRIVATE_SELECTOR).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/card.ui#Card", widget.clone())
        .unwrap();

    let mut widgets = std::collections::BTreeMap::new();
    widgets.insert("asset://ui/common/card.ui#Card".to_string(), widget);
    let diagnostic =
        component_contract_diagnostic(&layout, &widgets, &std::collections::BTreeMap::new())
            .unwrap()
            .expect("private selector should produce structured diagnostic");

    assert_eq!(
        diagnostic.code,
        UiComponentContractDiagnosticCode::PrivateSelector
    );
    assert_eq!(diagnostic.path, "stylesheets.host_styles.rules.0.selector");
    assert_eq!(diagnostic.target_control_id.as_deref(), Some("SecretLabel"));
    assert!(diagnostic.target_node_id.is_none());
}

#[test]
fn component_contract_rejects_incompatible_imported_component_api_version() {
    let widget = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT_WITH_INCOMPATIBLE_API).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/card.ui#Card", widget)
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("major API mismatches must be rejected");
    assert!(
        error.to_string().contains("requires component API 2.0.0"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_reports_api_mismatch_diagnostic_target_node() {
    let widget = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT_WITH_INCOMPATIBLE_API).unwrap();
    let mut widgets = std::collections::BTreeMap::new();
    widgets.insert("asset://ui/common/card.ui#Card".to_string(), widget);

    let diagnostic =
        component_contract_diagnostic(&layout, &widgets, &std::collections::BTreeMap::new())
            .unwrap()
            .expect("API mismatch should produce structured diagnostic");

    assert_eq!(
        diagnostic.code,
        UiComponentContractDiagnosticCode::ApiMismatch
    );
    assert_eq!(diagnostic.path, "nodes.root.component_api_version");
    assert_eq!(diagnostic.target_node_id.as_deref(), Some("root"));
}

#[test]
fn component_contract_scopes_public_part_selectors_to_matching_imports() {
    let card = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let button = UiAssetLoader::load_toml_str(PUBLIC_BUTTON_WIDGET).unwrap();
    let layout =
        UiAssetLoader::load_toml_str(MULTI_WIDGET_LAYOUT_WITH_SCOPED_PART_SELECTORS).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/card.ui#Card", card)
        .unwrap()
        .register_widget_import("asset://ui/common/button.ui#Button", button)
        .unwrap();

    compiler
        .compile(&layout)
        .expect("scoped public part selectors should not leak across imported components");
}

#[test]
fn component_contract_rejects_imported_stylesheet_private_selector_target() {
    let widget = UiAssetLoader::load_toml_str(PUBLIC_CARD_WIDGET).unwrap();
    let style = UiAssetLoader::load_toml_str(PRIVATE_CARD_STYLE).unwrap();
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT_WITH_IMPORTED_PRIVATE_STYLE).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/card.ui#Card", widget)
        .unwrap()
        .register_style_import("asset://ui/common/private_card_style.ui", style)
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("imported styles must not target private widget internals");
    assert!(
        error
            .to_string()
            .contains("private component internals SecretLabel"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_rejects_closed_root_class_policy_appends() {
    let widget = UiAssetLoader::load_toml_str(CLOSED_ROOT_CLASS_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CLOSED_ROOT_CLASS_LAYOUT).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/closed_card.ui#Card", widget)
        .unwrap();

    let error = compiler
        .compile(&layout)
        .expect_err("closed component roots must reject instance class appends");
    assert!(
        error.to_string().contains("root class policy is closed"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_reports_closed_root_class_diagnostic_target_node() {
    let widget = UiAssetLoader::load_toml_str(CLOSED_ROOT_CLASS_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(CLOSED_ROOT_CLASS_LAYOUT).unwrap();
    let mut widgets = std::collections::BTreeMap::new();
    widgets.insert("asset://ui/common/closed_card.ui#Card".to_string(), widget);

    let diagnostic =
        component_contract_diagnostic(&layout, &widgets, &std::collections::BTreeMap::new())
            .unwrap()
            .expect("closed root class should produce structured diagnostic");

    assert_eq!(
        diagnostic.code,
        UiComponentContractDiagnosticCode::ClosedRootClass
    );
    assert_eq!(diagnostic.path, "nodes.root.classes");
    assert_eq!(diagnostic.target_node_id.as_deref(), Some("root"));
}

#[test]
fn component_contract_rejects_public_part_control_from_another_node() {
    let widget = UiAssetLoader::load_toml_str(INVALID_PUBLIC_PART_CONTROL_WIDGET).unwrap();
    let error = UiDocumentCompiler::default()
        .compile(&widget)
        .expect_err("public parts must bind control ids on their exported node");
    assert!(
        error
            .to_string()
            .contains("control SecretLabel does not belong to node card_label"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_rejects_private_binding_targets_inside_component_contract() {
    let widget = UiAssetLoader::load_toml_str(INVALID_BINDING_CONTRACT_WIDGET).unwrap();
    let error = UiDocumentCompiler::default()
        .compile(&widget)
        .expect_err("public binding contract must not name private internals");
    assert!(
        error.to_string().contains("target SecretLabel is private"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_rejects_invalid_api_version_strings() {
    let error = UiAssetLoader::load_toml_str(INVALID_API_VERSION_WIDGET)
        .expect_err("component API versions must use major.minor.patch syntax");
    assert!(
        error
            .to_string()
            .contains("invalid ui component api version"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn component_contract_rejects_private_focus_targets_inside_component_contract() {
    let widget = UiAssetLoader::load_toml_str(INVALID_FOCUS_CONTRACT_WIDGET).unwrap();
    let error = UiDocumentCompiler::default()
        .compile(&widget)
        .expect_err("public focus contract must not name private internals");
    assert!(
        error.to_string().contains("target SecretLabel is private"),
        "unexpected error: {error:?}"
    );
}

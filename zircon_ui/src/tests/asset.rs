use toml::Value;

use crate::{
    UiAssetKind, UiAssetLoader, UiDocumentCompiler, UiLegacyTemplateAdapter, UiSize,
    UiTemplateLoader, UiTemplateSurfaceBuilder, UiTreeId,
};

const IMPORTED_BUTTON_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.buttons"
version = 1
display_name = "Common Buttons"

[root]
node = "button_root"

[components.ToolbarButton]
root = "button_root"
style_scope = "closed"

[components.ToolbarButton.params.label]
type = "string"
default = "Toolbar"

[components.ToolbarButton.params.icon]
type = "string"
default = "ellipse-outline"

[nodes.button_root]
kind = "native"
type = "Button"
classes = ["toolbar-button"]
props = { text = "$param.label", icon = "$param.icon" }
layout = { width = { min = 96.0, preferred = 96.0, max = 96.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }
"##;

const IMPORTED_TOOLBAR_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.toolbar"
version = 1
display_name = "Toolbar Shell"

[root]
node = "toolbar_root"

[components.ToolbarShell]
root = "toolbar_root"
style_scope = "closed"

[components.ToolbarShell.slots.leading]
required = false
multiple = true

[nodes.toolbar_root]
kind = "native"
type = "HorizontalBox"
control_id = "ToolbarRoot"
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" }, container = { kind = "HorizontalBox", gap = 4.0 } }
children = [{ child = "leading_slot" }]

[nodes.leading_slot]
kind = "slot"
slot_name = "leading"
"##;

const IMPORTED_STYLE_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.editor_base"
version = 1
display_name = "Editor Base"

[tokens]
accent = "#4488ff"
open_text = "Open Styled"

[[stylesheets]]
id = "editor_base"

[[stylesheets.rules]]
selector = ".toolbar > Button.primary"
set = { self = { background = { color = "$accent" }, layout = { width = { preferred = 144.0 } } } }

[[stylesheets.rules]]
selector = "#OpenButton"
set = { self = { text = "$open_text" }, slot = { layout = { height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" } } } }
"##;

const LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.ui_asset_editor"
version = 2
display_name = "UI Asset Editor"

[imports]
widgets = [
  "asset://ui/common/toolbar.ui#ToolbarShell",
  "asset://ui/common/buttons.ui#ToolbarButton",
]
styles = ["asset://ui/theme/editor_base.ui"]

[tokens]
panel_gap = 12.0

[root]
node = "editor_root"

[nodes.editor_root]
kind = "native"
type = "VerticalBox"
control_id = "EditorRoot"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 12.0 } }
children = [{ child = "toolbar" }]

[nodes.toolbar]
kind = "reference"
component_ref = "asset://ui/common/toolbar.ui#ToolbarShell"
control_id = "ToolbarHost"
classes = ["toolbar"]
children = [{ child = "open_button", mount = "leading", slot = { layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" } } } }]

[nodes.open_button]
kind = "reference"
component_ref = "asset://ui/common/buttons.ui#ToolbarButton"
control_id = "OpenButton"
classes = ["primary"]
params = { label = "Open", icon = "folder-open-outline" }
style_overrides = { self = { text = "Open Override" } }
"##;

const LEGACY_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "VerticalBox"
control_id = "LegacyRoot"
attributes = { layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } } }
children = [
  { component = "Button", control_id = "LegacyButton", bindings = [{ id = "Legacy/Button", event = "Click", route = "MenuAction.OpenProject" }], attributes = { text = "Open" } }
]
"#;

#[test]
fn ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets() {
    let button_asset = UiAssetLoader::load_toml_str(IMPORTED_BUTTON_ASSET_TOML).unwrap();
    let toolbar_asset = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_ASSET_TOML).unwrap();
    let style_asset = UiAssetLoader::load_toml_str(IMPORTED_STYLE_ASSET_TOML).unwrap();
    let layout_asset = UiAssetLoader::load_toml_str(LAYOUT_ASSET_TOML).unwrap();

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/buttons.ui#ToolbarButton", button_asset)
        .unwrap();
    compiler
        .register_widget_import("asset://ui/common/toolbar.ui#ToolbarShell", toolbar_asset)
        .unwrap();
    compiler
        .register_style_import("asset://ui/theme/editor_base.ui", style_asset)
        .unwrap();

    let compiled = compiler.compile(&layout_asset).unwrap();
    assert_eq!(compiled.asset.kind, UiAssetKind::Layout);
    assert_eq!(compiled.asset.id, "editor.ui_asset_editor");

    let instance = compiled.clone().into_template_instance();
    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(instance.root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(instance.root.children.len(), 1);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("HorizontalBox")
    );
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("ToolbarHost")
    );

    let open_button = instance.root.children[0]
        .children
        .iter()
        .find(|child| child.control_id.as_deref() == Some("OpenButton"))
        .unwrap();
    assert_eq!(open_button.component.as_deref(), Some("Button"));
    assert_eq!(
        open_button.attributes.get("text").unwrap().as_str(),
        Some("Open Override")
    );
    assert_eq!(
        open_button.attributes.get("icon").unwrap().as_str(),
        Some("folder-open-outline")
    );
    assert_eq!(open_button.classes, vec!["toolbar-button", "primary"]);
    assert_eq!(
        open_button
            .attributes
            .get("background")
            .unwrap()
            .get("color")
            .unwrap()
            .as_str(),
        Some("#4488ff")
    );
    assert_eq!(
        open_button
            .attributes
            .get("layout")
            .unwrap()
            .get("width")
            .unwrap()
            .get("preferred")
            .unwrap()
            .as_float(),
        Some(144.0)
    );
    assert_eq!(
        open_button
            .slot_attributes
            .get("layout")
            .unwrap()
            .get("height")
            .unwrap()
            .get("preferred")
            .unwrap()
            .as_float(),
        Some(40.0)
    );

    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("ui.asset.layout"),
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(800.0, 600.0)).unwrap();

    let open_button_node = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OpenButton")
        })
        .unwrap();
    assert_eq!(open_button_node.layout_cache.frame.width, 144.0);
    assert_eq!(open_button_node.layout_cache.frame.height, 40.0);
}

#[test]
fn ui_legacy_template_adapter_converts_template_documents_into_asset_documents() {
    let legacy = UiTemplateLoader::load_toml_str(LEGACY_TEMPLATE_TOML).unwrap();

    let asset_document =
        UiLegacyTemplateAdapter::layout_document("legacy.workbench", "Legacy Workbench", &legacy)
            .unwrap();

    assert_eq!(asset_document.asset.kind, UiAssetKind::Layout);
    assert_eq!(asset_document.asset.id, "legacy.workbench");
    assert_eq!(asset_document.asset.display_name, "Legacy Workbench");
    assert_eq!(asset_document.root.as_ref().unwrap().node, "root");
    assert!(asset_document.nodes.contains_key("root"));
    assert!(asset_document.nodes.contains_key("root_0"));

    let compiler = UiDocumentCompiler::default();
    let compiled = compiler.compile(&asset_document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(instance.root.control_id.as_deref(), Some("LegacyRoot"));
    assert_eq!(instance.root.children.len(), 1);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("Button")
    );
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("LegacyButton")
    );
    assert_eq!(
        instance.root.children[0].attributes.get("text"),
        Some(&Value::String("Open".to_string()))
    );
    assert_eq!(instance.root.children[0].bindings[0].id, "Legacy/Button");
}

#[test]
fn ui_legacy_template_adapter_emits_canonical_asset_source_that_roundtrips() {
    let legacy = UiTemplateLoader::load_toml_str(LEGACY_TEMPLATE_TOML).unwrap();

    let source =
        UiLegacyTemplateAdapter::layout_source("legacy.workbench", "Legacy Workbench", &legacy)
            .unwrap();
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(document.asset.id, "legacy.workbench");
    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("LegacyButton")
    );
}

#[test]
fn ui_asset_compiler_is_split_into_folder_backed_pipeline_modules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("template")
        .join("asset")
        .join("compiler");

    for relative in [
        "mod.rs",
        "ui_document_compiler.rs",
        "compile.rs",
        "node_expander.rs",
        "component_instance_expander.rs",
        "ui_style_resolver.rs",
        "style_apply.rs",
        "value_normalizer.rs",
        "shape_validator.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected compiler pipeline module {relative} under {:?}",
            root
        );
    }
}

use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};
use zircon_runtime_interface::ui::{
    accessibility::UiA11yRole, navigation::UiTabIndex, picking::UiPickMode,
};

#[test]
fn ui_asset_compiler_preserves_m1_contract_sections_from_source_nodes() {
    const CONTRACT_SECTIONS_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.contract_sections"
version = 1
display_name = "Contract Sections"

[root]
node_id = "button_instance"
kind = "component"
component = "ContractButton"
control_id = "Launch"

[root.widget]
disabled = true

[components.ContractButton]
style_scope = "closed"

[components.ContractButton.root]
node_id = "button_root"
kind = "native"
type = "Button"

[components.ContractButton.root.focus]
focusable = true
autofocus = true

[components.ContractButton.root.navigation.tab_index]
order = 4
tabbable = true

[components.ContractButton.root.picking]
pointer = "receive"
focus = "receive"
accessibility = "receive"

[components.ContractButton.root.a11y]
role = "button"
name = "Launch"

[components.ContractButton.root.widget]
disabled = false
tooltip = "Launch action"
"#;

    let document = UiAssetLoader::load_toml_str(CONTRACT_SECTIONS_TOML).unwrap();
    assert!(
        document
            .root
            .as_ref()
            .unwrap()
            .widget
            .as_ref()
            .unwrap()
            .disabled
    );

    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(instance.root.component.as_deref(), Some("Button"));
    assert_eq!(instance.root.control_id.as_deref(), Some("Launch"));
    assert!(instance.root.focus.focusable);
    assert!(instance.root.focus.autofocus);
    assert_eq!(instance.root.navigation.tab_index, Some(UiTabIndex::new(4)));
    assert_eq!(instance.root.picking.pointer, UiPickMode::Receive);
    assert_eq!(instance.root.picking.accessibility, UiPickMode::Receive);
    assert_eq!(instance.root.a11y.role, UiA11yRole::Button);
    assert_eq!(instance.root.a11y.name.as_deref(), Some("Launch"));
    assert!(
        instance.root.widget.disabled,
        "instance-authored widget contract should override the component root default"
    );
    assert_eq!(
        instance.root.widget.tooltip.as_deref(),
        None,
        "component-instance widget override is an explicit replacement contract"
    );
}

#[test]
fn legacy_template_conversion_only_authors_non_default_contract_sections() {
    const LEGACY_TEMPLATE_TOML: &str = r#"
version = 1

[components.ContractButton.root]
component = "Button"

[components.ContractButton.root.focus]
focusable = true

[components.ContractButton.root.widget]
tooltip = "Launch action"

[root]
template = "ContractButton"
control_id = "Launch"
"#;

    let migrated = crate::ui::template::UiAssetSchemaMigrator::migrate_legacy_template_str(
        "legacy.contract_sections",
        "Legacy Contract Sections",
        LEGACY_TEMPLATE_TOML,
    )
    .unwrap()
    .document;

    let root = migrated.root.as_ref().unwrap();
    assert_eq!(root.component.as_deref(), Some("ContractButton"));
    assert!(root.focus.is_none());
    assert!(root.navigation.is_none());
    assert!(root.picking.is_none());
    assert!(root.a11y.is_none());
    assert!(root.widget.is_none());

    let component_root = &migrated.components.get("ContractButton").unwrap().root;
    assert!(component_root.focus.as_ref().unwrap().focusable);
    assert_eq!(
        component_root.widget.as_ref().unwrap().tooltip.as_deref(),
        Some("Launch action")
    );

    let compiled = UiDocumentCompiler::default().compile(&migrated).unwrap();
    assert!(compiled.template_instance().root.focus.focusable);
    assert_eq!(
        compiled.template_instance().root.widget.tooltip.as_deref(),
        Some("Launch action")
    );
}

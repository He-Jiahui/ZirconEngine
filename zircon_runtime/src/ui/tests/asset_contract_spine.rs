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

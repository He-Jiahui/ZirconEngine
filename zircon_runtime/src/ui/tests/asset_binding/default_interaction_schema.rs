use super::*;
use zircon_runtime_interface::ui::widget::UiWidgetBehavior;

fn compile_single_component(
    component_id: &str,
    role: &str,
    widget: Option<&str>,
) -> crate::ui::template::UiCompiledDocument {
    let widget = widget
        .map(|widget| format!("widget = {widget}\n"))
        .unwrap_or_default();
    let source = format!(
        r#"
[asset]
kind = "layout"
id = "runtime.default_interaction.semantic_role"
version = 3

[root]
node_id = "root"
kind = "native"
type = {component_id:?}
control_id = "SemanticRoleRoot"
{widget}"#
    );
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let mut registry = UiComponentDescriptorRegistry::new();
    registry
        .register(UiComponentDescriptor::new(
            component_id,
            component_id,
            UiComponentCategory::Input,
            role,
        ))
        .unwrap();

    UiDocumentCompiler::default()
        .with_component_registry(registry)
        .compile(&document)
        .unwrap()
}

#[test]
fn descriptor_role_projects_typed_default_interaction_behavior() {
    let compiled = compile_single_component("ProductPrimaryAction", "button", None);

    assert_eq!(
        compiled
            .template_instance()
            .root
            .attributes
            .get("component_role")
            .and_then(toml::Value::as_str),
        Some("button")
    );
    assert_eq!(
        compiled.template_instance().root.widget.behavior,
        UiWidgetBehavior::Button
    );
}

#[test]
fn explicit_widget_behavior_overrides_descriptor_role() {
    let compiled = compile_single_component(
        "ProductPrimaryAction",
        "button",
        Some(r#"{ behavior = "passive" }"#),
    );

    assert_eq!(
        compiled.template_instance().root.widget.behavior,
        UiWidgetBehavior::Passive
    );
}

#[test]
fn component_name_does_not_supply_default_interaction_behavior() {
    let compiled = compile_single_component("Button", "status", None);

    assert_eq!(
        compiled.template_instance().root.widget.behavior,
        UiWidgetBehavior::Passive
    );
}

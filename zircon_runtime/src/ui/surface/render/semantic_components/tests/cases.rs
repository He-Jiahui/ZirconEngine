use std::collections::BTreeMap;

use toml::Value;
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

use super::semantic_component_suppresses_owner_text;

#[test]
fn semantic_component_text_is_owned_by_the_specialized_painter() {
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata.component = "AgentPlan".into();
    assert!(semantic_component_suppresses_owner_text(Some(&metadata)));

    metadata.component = "DataGrid".into();
    assert!(semantic_component_suppresses_owner_text(Some(&metadata)));

    metadata.component = "Panel".into();
    metadata.attributes = BTreeMap::from([(
        "component_role".into(),
        Value::String("mui-x-agent-plan".into()),
    )]);
    assert!(semantic_component_suppresses_owner_text(Some(&metadata)));
}

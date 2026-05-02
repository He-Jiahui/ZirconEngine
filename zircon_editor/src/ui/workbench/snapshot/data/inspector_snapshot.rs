use zircon_runtime::scene::NodeId;

#[derive(Clone, Debug)]
pub struct InspectorSnapshot {
    pub id: NodeId,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
    pub plugin_components: Vec<InspectorPluginComponentSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorPluginComponentSnapshot {
    pub component_id: String,
    pub display_name: String,
    pub plugin_id: String,
    pub drawer_available: bool,
    pub diagnostic: Option<String>,
    pub properties: Vec<InspectorPluginComponentPropertySnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorPluginComponentPropertySnapshot {
    pub field_id: String,
    pub name: String,
    pub label: String,
    pub value: String,
    pub value_kind: String,
    pub editable: bool,
}

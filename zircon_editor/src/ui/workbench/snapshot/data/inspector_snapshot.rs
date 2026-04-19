use zircon_runtime::scene::NodeId;

#[derive(Clone, Debug)]
pub struct InspectorSnapshot {
    pub id: NodeId,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
}

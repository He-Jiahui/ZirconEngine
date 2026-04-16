use zircon_scene::NodeId;

#[derive(Clone, Debug)]
pub struct SceneEntry {
    pub id: NodeId,
    pub name: String,
    pub depth: usize,
    pub selected: bool,
}

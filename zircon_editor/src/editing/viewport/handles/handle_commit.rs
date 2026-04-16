use zircon_math::Transform;
use zircon_scene::SceneViewportTool;

#[derive(Clone, Debug)]
pub(crate) struct HandleCommit {
    pub(crate) node_id: u64,
    pub(crate) tool: SceneViewportTool,
    pub(crate) initial_transform: Transform,
}

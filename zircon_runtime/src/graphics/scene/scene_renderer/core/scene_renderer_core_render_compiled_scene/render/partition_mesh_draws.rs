use crate::graphics::scene::scene_renderer::mesh::MeshDraw;

pub(super) fn partition_mesh_draws(mesh_draws: &[MeshDraw]) -> (Vec<&MeshDraw>, Vec<&MeshDraw>) {
    mesh_draws.iter().partition(|draw| !draw.is_transparent())
}

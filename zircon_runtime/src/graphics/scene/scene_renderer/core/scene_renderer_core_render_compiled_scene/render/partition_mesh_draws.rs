use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshDrawQueuePhase};

pub(super) struct MeshDrawPartitions<'a> {
    pub(super) opaque: Vec<&'a MeshDraw>,
    pub(super) alpha_mask: Vec<&'a MeshDraw>,
    pub(super) transparent: Vec<&'a MeshDraw>,
}

impl<'a> MeshDrawPartitions<'a> {
    pub(super) fn non_transparent(&self) -> Vec<&'a MeshDraw> {
        self.opaque
            .iter()
            .copied()
            .chain(self.alpha_mask.iter().copied())
            .collect()
    }
}

pub(super) fn partition_mesh_draws(mesh_draws: &[MeshDraw]) -> MeshDrawPartitions<'_> {
    let mut partitions = MeshDrawPartitions {
        opaque: Vec::new(),
        alpha_mask: Vec::new(),
        transparent: Vec::new(),
    };
    for draw in mesh_draws {
        match draw.queue_profile().phase() {
            MeshDrawQueuePhase::Opaque => partitions.opaque.push(draw),
            MeshDrawQueuePhase::AlphaMask => partitions.alpha_mask.push(draw),
            MeshDrawQueuePhase::Transparent => partitions.transparent.push(draw),
        }
    }
    partitions
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderParticleGpuReadbackOutputs {
    pub alive_count: u32,
    pub spawned_total: u32,
    pub debug_flags: u32,
    pub per_emitter_spawned: Vec<u32>,
    pub indirect_draw_args: [u32; 4],
}

impl RenderParticleGpuReadbackOutputs {
    pub fn is_empty(&self) -> bool {
        self.alive_count == 0
            && self.spawned_total == 0
            && self.debug_flags == 0
            && self.per_emitter_spawned.is_empty()
            && self.indirect_draw_args == [0; 4]
    }
}

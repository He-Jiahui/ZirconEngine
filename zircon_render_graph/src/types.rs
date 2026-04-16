#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPassId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransientTexture(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransientBuffer(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExternalResource(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueLane {
    Graphics,
    AsyncCompute,
    AsyncCopy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassFlags {
    pub allow_culling: bool,
    pub has_side_effects: bool,
}

impl Default for PassFlags {
    fn default() -> Self {
        Self {
            allow_culling: true,
            has_side_effects: false,
        }
    }
}

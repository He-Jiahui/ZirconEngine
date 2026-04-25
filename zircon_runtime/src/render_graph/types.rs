use crate::rhi::{BufferDesc, TextureDesc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPassId(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransientTexture(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransientBuffer(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExternalResource(pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResource {
    TransientTexture(TransientTexture),
    TransientBuffer(TransientBuffer),
    External(ExternalResource),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceKind {
    TransientTexture,
    TransientBuffer,
    External,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderGraphResourceDesc {
    Texture(TextureDesc),
    Buffer(BufferDesc),
    External,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderGraphResourceLifetime {
    pub name: String,
    pub kind: RenderGraphResourceKind,
    pub first_pass: usize,
    pub last_pass: usize,
    pub imported: bool,
}

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

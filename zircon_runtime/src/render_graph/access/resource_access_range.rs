use super::{RenderGraphBufferRange, RenderGraphTextureSubresourceRange};

/// Logical resource scope addressed by one graph access.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceAccessRange {
    Texture(RenderGraphTextureSubresourceRange),
    Buffer(RenderGraphBufferRange),
    /// Report-only external imports have no physical type contract yet.
    ///
    /// Such accesses remain legacy-only and cannot be lowered into a typed
    /// texture view or buffer slice by the future binding table.
    UnresolvedExternal,
}

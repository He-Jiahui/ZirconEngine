/// Plane selection for a texture subresource access.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderGraphTextureAspect {
    #[default]
    All,
    Color,
    Depth,
    Stencil,
}

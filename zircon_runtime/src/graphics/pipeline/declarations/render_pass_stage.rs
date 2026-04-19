#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderPassStage {
    DepthPrepass,
    Shadow,
    GBuffer,
    AmbientOcclusion,
    Lighting,
    Opaque,
    Transparent,
    PostProcess,
    Overlay,
}

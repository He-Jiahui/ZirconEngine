#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialDomain {
    Surface,
    PostProcess,
    DebugOverlay,
    LightFunction,
}

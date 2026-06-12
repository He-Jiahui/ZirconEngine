#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshDrawGeometrySource {
    Prepared,
    Dynamic,
    DynamicGpuSkinningSource,
}

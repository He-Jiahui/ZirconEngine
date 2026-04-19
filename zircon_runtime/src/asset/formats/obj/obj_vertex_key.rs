#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct ObjVertexKey {
    pub(super) position: usize,
    pub(super) uv: Option<usize>,
    pub(super) normal: Option<usize>,
}

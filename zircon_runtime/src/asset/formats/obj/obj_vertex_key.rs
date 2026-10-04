/// 顶点复用以三个索引的组合为单位；相同位置上的 UV 接缝和法线硬边不能合并。
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) struct ObjVertexKey {
    pub(super) position: usize,
    pub(super) uv: Option<usize>,
    pub(super) normal: Option<usize>,
}

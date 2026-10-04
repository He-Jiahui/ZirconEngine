#[derive(Clone, Debug, PartialEq, Eq)]
/// 某一页世代的完整 CPU 像素镜像；局部补丁只能更新已建立的完整镜像。
pub(in crate::text::atlas) struct GlyphAtlasBitmapPageShadow {
    pub(super) generation: u64,
    pub(super) bytes: Vec<u8>,
}

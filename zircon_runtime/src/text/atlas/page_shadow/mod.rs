//! 保存已确认上传的位图页 CPU 副本，供后续脏区合并和 GPU 资源重建重放。
//! 提交记录由渲染端汇总成功与失败结果，存储端再按当前驻留页及世代接纳。

mod commit;
mod patch;
mod shadow;
mod store;

pub(crate) use commit::GlyphAtlasBitmapPageShadowCommit;
pub(crate) use patch::GlyphAtlasBitmapPageShadowPatch;
pub(super) use shadow::GlyphAtlasBitmapPageShadow;
pub(crate) use store::{GlyphAtlasBitmapPageShadowReport, GlyphAtlasBitmapPageShadowStore};

//! 字体后端的失败诊断值；文本范围和后备处置由上层整形流程补充。
//! 操作标签与错误原因按值传递，不承担字体资源租约或后备执行。

use crate::text::font::FontDatabaseError;
use crate::text::FontFaceId;

/// 标识一次失败的字体数据库调用，供失败回执映射阶段。
///
/// 由水平、垂直后端填入；标签本身不触发字体访问或选择后备策略。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::text::shaping) enum BackendFontOperation {
    /// 计算字体面的有效可变轴坐标失败。
    ResolveVariations,
    /// 获取字体面字节失败。
    LoadFaceBytes,
    /// 查询字体集合中的字体面索引失败。
    ResolveFaceIndex,
}

#[derive(Debug, thiserror::Error)]
/// 这些错误只描述字体后端边界；上层把它们包装成带 source range 的 `DirectShapeError`，再决定是否尝试 alternate backend。
pub(in crate::text::shaping) enum BackendShapeError {
    /// 保留字体数据库失败的操作、字体面身份及原始错误。
    #[error("font backend {operation:?} failed for {face:?}: {source}")]
    FontDatabase {
        /// 失败发生的字体库调用阶段。
        operation: BackendFontOperation,
        /// 本次调用使用的字体面身份，仍依赖对应字体数据库上下文。
        face: FontFaceId,
        /// 按值保存的原始错误，供标准错误来源链追溯。
        #[source]
        source: FontDatabaseError,
    },
    /// 记录后端解析失败的字体面及其字体集合索引，不验证两者是否对应。
    #[error("font backend could not parse face {face:?} at collection index {face_index}")]
    FaceParseFailed { face: FontFaceId, face_index: u32 },
    /// 水平、垂直后端在排除空文本后记录空 glyph；本变体不存储或校验文本。
    #[error("font backend returned no glyphs for non-empty input using {face:?}")]
    EmptyGlyphOutput { face: FontFaceId },
}

impl BackendShapeError {
    /// 将三个入参原样封装，供后端的错误映射闭包调用。
    ///
    /// # 参数
    /// - `operation`：失败的调用标签；`face`：该调用使用的字体面身份。
    /// - `source`：按值移入的字体数据库错误，保留为来源链。
    ///
    /// # 返回与约束
    /// 返回 `FontDatabase` 变体，不核验三个入参是否对应。
    /// 不访问字体、不附加文本范围，也不决定或执行后备策略。
    ///
    /// 参考：`horizontal/backend.rs`、`vertical/backend.rs` 与 `failure_receipt.rs`。
    pub(in crate::text::shaping) fn font_database(
        operation: BackendFontOperation,
        face: FontFaceId,
        source: FontDatabaseError,
    ) -> Self {
        Self::FontDatabase {
            operation,
            face,
            source,
        }
    }
}

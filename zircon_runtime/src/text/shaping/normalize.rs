//! 为成形后端保留原始 UTF-8 字节身份的借用视图。
//!
//! 本地区间以输入文本起点为零；消费端负责边界验证和绝对源偏移的叠加。

use std::ops::Range;

/// 在一次成形请求中共享原始文本及其本地字节坐标。
///
/// # 使用与生命周期
/// 请求入口创建视图，分析过程读取同一源文本；备用后端的行与字形投影沿用这些坐标。
/// 输入存储由调用者持有，视图及其返回的文本借用均受输入生命周期 `'a` 约束。
pub(super) struct ShapingTextView<'a> {
    /// 输入存储的共享借用，不保存文本副本或取得其所有权。
    original: &'a str,
}

impl<'a> ShapingTextView<'a> {
    /// 直接借用本次请求的源文本，创建不改变字节的视图。
    ///
    /// # 参数与返回
    /// `original` 是由调用者保持存储有效的 UTF-8 文本；返回值借用该输入，期限为 `'a`。
    ///
    /// V1 keeps the exact source UTF-8 for both shaping and all published offsets.
    ///
    /// NFC/NFD must not be introduced here until a versioned bidirectional source map is
    /// available to selection, IME, accessibility, cache, and glyph projection consumers.
    pub(super) const fn source_preserving(original: &'a str) -> Self {
        Self { original }
    }

    /// 取得分析和成形实际读取的原始文本。
    ///
    /// # 返回与生命周期
    /// 返回输入的 `&'a str`；可长于本次 `&self` 借用，但不能长于原输入。
    pub(super) const fn shaping_text(&self) -> &'a str {
        self.original
    }

    /// 将成形文本的本地区间限制到源文本的字节长度内。
    ///
    /// # 参数
    /// - `range`：以本视图文本起点为零的半开字节区间，允许越界或反向。
    ///
    /// # 返回
    /// 起点截到文本末尾，终点限制在该起点与文本末尾之间；反向区间退化为空区间。
    ///
    /// # 约束与参考
    /// 区间对应 [`Self::shaping_text`] 的字节；这里只夹取整数，不验证字符或字素边界。
    /// 现有行与字形消费端先验证边界，再以检查加法叠加请求的绝对源起点。
    pub(super) fn source_range_for_shaping_range(&self, range: Range<usize>) -> Range<usize> {
        let start = range.start.min(self.original.len());
        let end = range.end.clamp(start, self.original.len());
        start..end
    }
}

#[cfg(test)]
#[path = "tests/normalize.rs"]
mod tests;

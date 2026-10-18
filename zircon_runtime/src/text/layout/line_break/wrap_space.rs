//! 普通空格的折行裁剪量与借用切片，供上层同步维护源字节范围。

const WRAP_SPACE: char = ' ';

/// 软折行后的行首裁剪只处理普通空格，并同步推进源字节起点。
/// 不间断空格等粘连字符仍由原源区间保留，供后续命中测试和光标投影使用。
///
/// # 参数、返回与约束
/// `text` 是完整源区间对应的切片，`source_start` 是该区间的逻辑字节起点；返回文本继续借用 `text`。
/// 返回的新起点仅推进被移除的 U+0020 字节数，不建立或验证独立的来源映射。
/// 当前加减按左结合求值，调用端须保证整个 `source_start + text.len()` 可表示，不能只保证最终起点可表示。
/// 需要行尾裁剪量时使用 [`trailing_wrap_space_byte_len`]，它只返回数量。
pub(crate) fn trim_leading_wrap_spaces(text: &str, source_start: usize) -> (&str, usize) {
    let trimmed = text.trim_start_matches(WRAP_SPACE);
    (trimmed, source_start + text.len() - trimmed.len())
}

/// 返回视觉候选末尾可裁剪的普通空格字节数；调用端据此调整行宽而不误删 NBSP。
///
/// 返回值是 UTF-8 字节数；函数只读取 `text`，行文本及 run 范围的改写由消费方完成。
pub(crate) fn trailing_wrap_space_byte_len(text: &str) -> usize {
    text.len() - text.trim_end_matches(WRAP_SPACE).len()
}

#[cfg(test)]
#[path = "tests/wrap_space.rs"]
mod tests;

const WRAP_SPACE: char = ' ';

/// 软折行后的行首裁剪只处理普通空格，并同步推进源字节起点。
/// 不间断空格等粘连字符仍由原源区间保留，供后续命中测试和光标投影使用。
pub(crate) fn trim_leading_wrap_spaces(text: &str, source_start: usize) -> (&str, usize) {
    let trimmed = text.trim_start_matches(WRAP_SPACE);
    (trimmed, source_start + text.len() - trimmed.len())
}

/// 返回视觉候选末尾可裁剪的普通空格字节数；调用端据此调整行宽而不误删 NBSP。
pub(crate) fn trailing_wrap_space_byte_len(text: &str) -> usize {
    text.len() - text.trim_end_matches(WRAP_SPACE).len()
}

#[cfg(test)]
#[path = "tests/wrap_space.rs"]
mod tests;

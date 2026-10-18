//! 按固定字符集合调整相邻折行块的禁则边界。
//!
//! 调整只遍历一轮；块中的文本仍借用原输入，范围仍以该输入的 UTF-8 字节为基准。
//! 这里只限制细分折行许可，不执行重新成形、测量或字体覆盖选择。

use super::line_break::LineBreakChunk;
use crate::text::TextRange;

/// 对已生成的折行块执行一轮相邻合并及尾部开标点拆分。
///
/// `text` 是块引用的原输入；`chunks` 转移容器所有权，块文本和两类范围须对应同一输入，
/// 并按逻辑顺序提供。返回新的块容器，文本切片继续借用 `text`，不会复制原文。
/// 局部合并检查只保护本次切片，不验证所有块的文本身份、连续性或源映射。
///
/// 强制边界保护该块之后的位置，带此标记的右块仍可并入左块并传递标记。
/// 合并优先保留左块的断行后缀，不重算后缀锚点；重切原文也可能重新纳入先前省略的软连字符。
/// 这不是反复求稳定结果的过程，固定规则也不构成完整的日文排版规范验证。
/// 参见 [`LineBreakChunk`] 的文本借用与范围字段。
pub(super) fn apply_kinsoku_start_rules<'a>(
    text: &'a str,
    chunks: Vec<LineBreakChunk<'a>>,
) -> Vec<LineBreakChunk<'a>> {
    let mut adjusted: Vec<LineBreakChunk<'a>> = Vec::with_capacity(chunks.len());
    let mut chunk_iter = chunks.into_iter().peekable();

    while let Some(mut chunk) = chunk_iter.next() {
        // 先尝试并入已输出的左块；只检查左块后的强制边界，成功后直接处理下一输入块。
        if starts_with_forbidden_line_start(chunk.text)
            || completes_jlreq_inseparable_pair(adjusted.last(), chunk.text)
        {
            if let Some(previous) = adjusted.last_mut().filter(|chunk| !chunk.mandatory_break) {
                let start = previous.visual_range.start;
                let end = chunk.visual_range.end;
                if start < end
                    && end <= text.len()
                    && text.is_char_boundary(start)
                    && text.is_char_boundary(end)
                {
                    previous.text = &text[start..end];
                    previous.visual_range.end = end;
                    previous.source_range.end = chunk.source_range.end;
                    if previous.break_suffix.is_none() {
                        previous.break_suffix = chunk.break_suffix;
                    }
                    previous.mandatory_break |= chunk.mandatory_break;
                    previous.allow_glyph_fallback = false;
                    continue;
                }
            }
        }

        // 块首为开标点时尝试取下一个块；只有切片边界检查通过后才消费迭代器。
        if starts_with_forbidden_line_end(chunk.text) && !chunk.mandatory_break {
            if let Some(next) = chunk_iter.peek() {
                let start = chunk.visual_range.start;
                let end = next.visual_range.end;
                if start < end
                    && end <= text.len()
                    && text.is_char_boundary(start)
                    && text.is_char_boundary(end)
                {
                    if let Some(next) = chunk_iter.next() {
                        chunk.text = &text[start..end];
                        chunk.visual_range.end = end;
                        chunk.source_range.end = next.source_range.end;
                        if chunk.break_suffix.is_none() {
                            chunk.break_suffix = next.break_suffix;
                        }
                        chunk.mandatory_break |= next.mandatory_break;
                        chunk.allow_glyph_fallback = false;
                    }
                }
            }
        }
        // 将尾部开标点留给后续内容；拆出的前缀先输出，尾块最多再向右合并一次。
        if ends_with_forbidden_line_end(chunk.text) {
            if let Some((prefix, suffix)) = split_forbidden_line_end_suffix(text, chunk) {
                adjusted.push(prefix);
                chunk = suffix;
                if !chunk.mandatory_break {
                    if let Some(next) = chunk_iter.peek() {
                        let start = chunk.visual_range.start;
                        let end = next.visual_range.end;
                        if start < end
                            && end <= text.len()
                            && text.is_char_boundary(start)
                            && text.is_char_boundary(end)
                        {
                            if let Some(next) = chunk_iter.next() {
                                chunk.text = &text[start..end];
                                chunk.visual_range.end = end;
                                chunk.source_range.end = next.source_range.end;
                                if chunk.break_suffix.is_none() {
                                    chunk.break_suffix = next.break_suffix;
                                }
                                chunk.mandatory_break |= next.mandatory_break;
                            }
                        }
                    }
                }
                chunk.allow_glyph_fallback = false;
            }
        }

        // Let closing punctuation overhang with the preceding glyph instead of
        // allowing glyph fallback to put it at the start of the next line.
        // 此处关闭的是过宽块的细分折行许可，与选择备用字体无关。
        if has_protected_forbidden_prefix(chunk.text)
            || has_protected_forbidden_suffix(chunk.text)
            || has_jlreq_inseparable_pair(chunk.text)
        {
            chunk.allow_glyph_fallback = false;
        }

        adjusted.push(chunk);
    }

    adjusted
}

fn starts_with_forbidden_line_end(text: &str) -> bool {
    text.chars().next().is_some_and(is_forbidden_line_end)
}

/// 至少有两个 Unicode 标量时才把末标量视为可拆出的开标点；空串和单标量保持原块。
fn ends_with_forbidden_line_end(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(last) = chars.next_back() else {
        return false;
    };
    chars.next_back().is_some() && is_forbidden_line_end(last)
}

/// 从尾部拆出最后一个标量；调用方已确认它属于开标点集合。
///
/// 原文和块范围须成对有效；这里只检查拆分点，不为任意外部块补做全部端点验证。
/// 前缀保留原细分许可但清除强制边界及后缀；尾块继承两者并关闭细分许可。
/// 空串、单标量或无效拆分点返回 `None`，其余情况返回仍借用原文的两个块。
fn split_forbidden_line_end_suffix<'a>(
    text: &'a str,
    chunk: LineBreakChunk<'a>,
) -> Option<(LineBreakChunk<'a>, LineBreakChunk<'a>)> {
    let (suffix_offset, _) = chunk.text.char_indices().next_back()?;
    if suffix_offset == 0 {
        return None;
    }

    let suffix_start = chunk.visual_range.start + suffix_offset;
    if !text.is_char_boundary(suffix_start) {
        return None;
    }

    let prefix = LineBreakChunk {
        text: &text[chunk.visual_range.start..suffix_start],
        visual_range: TextRange {
            start: chunk.visual_range.start,
            end: suffix_start,
        },
        source_range: TextRange {
            start: chunk.source_range.start,
            end: suffix_start,
        },
        allow_glyph_fallback: chunk.allow_glyph_fallback,
        mandatory_break: false,
        break_suffix: None,
    };
    let suffix = LineBreakChunk {
        text: &text[suffix_start..chunk.visual_range.end],
        visual_range: TextRange {
            start: suffix_start,
            end: chunk.visual_range.end,
        },
        source_range: TextRange {
            start: suffix_start,
            end: chunk.source_range.end,
        },
        allow_glyph_fallback: false,
        mandatory_break: chunk.mandatory_break,
        break_suffix: chunk.break_suffix,
    };
    Some((prefix, suffix))
}

fn starts_with_forbidden_line_start(text: &str) -> bool {
    text.chars().next().is_some_and(is_forbidden_line_start)
}

/// 检查左块末标量和当前块首标量是否构成固定连写对；左块后的强制边界优先保留。
fn completes_jlreq_inseparable_pair(
    previous: Option<&LineBreakChunk<'_>>,
    current_text: &str,
) -> bool {
    if previous.is_some_and(|chunk| chunk.mandatory_break) {
        return false;
    }
    let Some(previous_char) = previous.and_then(|chunk| chunk.text.chars().next_back()) else {
        return false;
    };
    let Some(current_char) = current_text.chars().next() else {
        return false;
    };

    is_jlreq_inseparable_pair(previous_char, current_char)
}

/// 顺序检查相邻 Unicode 标量；这里只识别下面列出的固定连写对。
fn has_jlreq_inseparable_pair(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(mut previous_char) = chars.next() else {
        return false;
    };

    for current_char in chars {
        if is_jlreq_inseparable_pair(previous_char, current_char) {
            return true;
        }
        previous_char = current_char;
    }

    false
}

/// 固定的五类成对规则，不代表全部日文连写规则或字素簇属性。
fn is_jlreq_inseparable_pair(previous_char: char, current_char: char) -> bool {
    matches!(
        (previous_char, current_char),
        ('—', '—') | ('…', '…') | ('‥', '‥') | ('〳', '〵') | ('〴', '〵')
    )
}

/// 从开头最多读取两个标量，区分单独开标点与已带后续内容的受保护前缀。
fn has_protected_forbidden_prefix(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    chars.next().is_some() && is_forbidden_line_end(first)
}

/// 从末尾最多读取两个标量，区分单独禁行首字符与已有前导内容的受保护后缀。
fn has_protected_forbidden_suffix(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(last) = chars.next_back() else {
        return false;
    };
    chars.next_back().is_some() && is_forbidden_line_start(last)
}

/// 按当前显式标量集合分类，不查询完整 Unicode 属性或字体数据。
fn is_forbidden_line_start(ch: char) -> bool {
    matches!(
        ch,
        // Closing punctuation.
        '、' | '。' | '，' | '．' | '・' | '：' | '；' | '！' | '？' | '）' | '］'
            | '｝' | '｠' | '】' | '〕' | '〉' | '》' | '」' | '』' | '〗' | '〙' | '〛'
            | '’' | '”' | '〟' | '〞'
            // Small kana.
            | 'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ' | 'っ' | 'ゃ' | 'ゅ' | 'ょ' | 'ゎ' | 'ゕ'
            | 'ゖ' | 'ァ' | 'ィ' | 'ゥ' | 'ェ' | 'ォ' | 'ッ' | 'ャ' | 'ュ' | 'ョ' | 'ヮ'
            | 'ヵ' | 'ヶ' | 'ㇰ' | 'ㇱ' | 'ㇲ' | 'ㇳ' | 'ㇴ' | 'ㇵ' | 'ㇶ' | 'ㇷ' | 'ㇸ'
            | 'ㇹ' | 'ㇺ' | 'ㇻ' | 'ㇼ' | 'ㇽ' | 'ㇾ' | 'ㇿ'
            // Half-width punctuation and kana.
            | '｡' | '｣' | '､' | '･' | 'ｧ' | 'ｨ' | 'ｩ' | 'ｪ' | 'ｫ' | 'ｯ' | 'ｬ'
            | 'ｭ' | 'ｮ' | 'ｰ' | 'ﾞ' | 'ﾟ'
            // Japanese non-starters, spacing marks, and JLREQ hyphens.
            | 'ー' | '々' | '〻' | 'ゝ' | 'ゞ' | 'ヽ' | 'ヾ' | '゛' | '゜' | '‐' | '〜'
            | '゠' | '–'
    )
}

/// 当前开标点集合；分类本身不测量字符，也不验证调用方的范围。
fn is_forbidden_line_end(ch: char) -> bool {
    matches!(
        ch,
        '（' | '｛'
            | '｟'
            | '［'
            | '【'
            | '〔'
            | '〈'
            | '《'
            | '「'
            | '『'
            | '〖'
            | '〘'
            | '〚'
            | '‘'
            | '“'
            | '〝'
            | '｢'
    )
}

#[cfg(test)]
#[path = "kinsoku/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "kinsoku/tests/match_dispatch_tests.rs"]
mod match_dispatch_tests;

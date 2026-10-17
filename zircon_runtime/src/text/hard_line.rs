//! 共享文本服务的规范硬行扫描与源偏移导航。
//!
//! 硬行由源分隔符确定，软换行和成形分块不在这里产生新行。
//! 行描述中的范围是 UTF-8 字节偏移，窗口参数则是从零开始的行序号范围。
//! 返回值只拥有范围描述；据此切片时仍须使用生成这些范围的同一份源文本。
//! 本模块执行扫描和范围物化，资源准入由调用方及缓存入口另行决定。

use std::ops::Range;

/// 一条规范硬行在源文本中的内容与分隔符位置。
///
/// # 约束与生命周期
/// 扫描器生成的两个半开字节范围依次覆盖内容和紧随其后的分隔符。
/// 本类型不持有源文本，也不校验 crate 内直接构造的字段；使用范围切片时须保留对应文本。
/// 参见 [visit_hard_lines] 的顺序扫描及 [HardLine::source_range] 的范围合并。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HardLine {
    /// 不含分隔符的内容字节范围；空源和末尾分隔符后的终止行允许为空。
    pub(crate) content: Range<usize>,
    /// 内容后分隔符的字节范围；扫描器将 CRLF 合为一段，终止行使用空范围。
    pub(crate) separator: Range<usize>,
}

impl HardLine {
    /// 合并本行内容与分隔符的覆盖范围，返回拥有式字节范围。
    ///
    /// 该计算取两个终点的较大值，不验证手工构造的端点或源文本边界。
    /// 返回值不借用本行；将其用作切片仍依赖合法字段及对应源文本。
    pub(crate) fn source_range(&self) -> Range<usize> {
        self.content.start..self.separator.end.max(self.content.end)
    }
}

/// 在调用方需要完整列表时，同步物化源文本的所有规范硬行。
///
/// # 参数、返回与资源
/// `text` 仅在扫描期间借用；返回向量拥有范围值，不保留文本。
/// 空文本产生一条终止行，末尾分隔符之后也保留一条空的终止行。
/// 分配随行数增长；只需逐行处理时可使用 [visit_hard_lines]。
pub(crate) fn hard_lines(text: &str) -> Vec<HardLine> {
    let mut lines = Vec::new();
    visit_hard_lines(text, |line| {
        lines.push(line);
    });
    lines
}

/// Visits canonical hard lines without retaining a document-sized line vector.
///
/// 同步按源顺序调用 `visit`，每次移交一个拥有式范围描述。
///
/// # 调用约束与生命周期
/// 回调按值接收并只在本次调用内同步执行，不存入模块状态；`text` 仅在调用期间借用。
/// 回调没有停止返回值；扫描结束时会访问终止行，包括空文本或尾随分隔符产生的空行。
/// 本入口不保留全量行向量；回调自行保留结果时仍会占用相应资源。
pub(crate) fn visit_hard_lines(text: &str, mut visit: impl FnMut(HardLine)) {
    for_each_hard_line(text, |line| {
        visit(line);
        true
    });
}

/// Counts canonical hard lines without retaining a per-line allocation.
///
/// 对借用的 `text` 完整扫描，返回包含终止行的硬行数，空文本的结果为一。
/// 计数使用饱和加法；本入口不物化逐行向量，也不执行工作预算准入。
pub(crate) fn hard_line_count(text: &str) -> usize {
    let mut count: usize = 0;
    for_each_hard_line(text, |_| {
        count = count.saturating_add(1);
        true
    });
    count
}

/// Returns whether source segmentation can produce more than one hard line.
///
/// This is the allocation-free rejection path for viewport virtualization. Only a source
/// separator changes the document's hard-line identity; backend execution policy must not become
/// a layout line.
///
/// 对 `text` 查找任一规范分隔符并提前返回；找到时结果为真。
/// 即使分隔符位于文本末尾，它也会产生额外的终止行；长而无分隔符的文本仍为假。
/// 调用方据此拒绝单硬行的虚拟化候选，具体虚拟化资格仍由上层决定。
pub(crate) fn has_multiple_hard_lines(text: &str) -> bool {
    text.chars().any(is_hard_line_separator)
}

/// Materializes only the requested canonical hard-line range.
///
/// # 参数、返回与扫描边界
/// `range` 是从零开始的半开行序号范围，不是源字节范围；空或反向范围直接返回空向量。
/// 扫描到窗口上界或文本结尾即结束，结果仅保留实际存在且落入窗口的行描述。
/// 返回范围不保留借用的 `text`；窗口大小仍由调用方决定，不构成分配峰值上限。
/// 同时需要总行数时使用 [hard_line_count_and_window]。
pub(crate) fn hard_line_window(text: &str, range: Range<usize>) -> Vec<HardLine> {
    if range.start >= range.end {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut line_index = 0;
    // The viewport counts with the same scanner first; stop this pass as soon as its window ends.
    for_each_hard_line(text, |line| {
        if range.contains(&line_index) {
            lines.push(line);
        }
        line_index = line_index.saturating_add(1);
        line_index < range.end
    });
    lines
}

/// Counts all canonical hard lines while retaining only the requested window.
///
/// This is the bounded viewport path: document height still needs the full count, but the
/// layout only needs candidate lines around the visible region.
///
/// # 参数、返回与资源
/// `text` 仅在调用期间借用；`range` 选择半开的硬行序号范围。
/// 返回完整扫描得到的总行数及拥有式窗口；空或反向窗口仍扫描并返回总数。
/// 仅窗口内的描述进入返回向量，扫描工作量和窗口分配不由本函数执行预算准入。
/// 有键会话可由缓存入口保留索引，本函数自身不保留文本或索引状态。
pub(crate) fn hard_line_count_and_window(
    text: &str,
    range: Range<usize>,
) -> (usize, Vec<HardLine>) {
    let mut count = 0usize;
    let mut lines = Vec::new();
    for_each_hard_line(text, |line| {
        if range.contains(&count) {
            lines.push(line);
        }
        count = count.saturating_add(1);
        true
    });
    (count, lines)
}

/// Returns the source start of the canonical hard line containing `offset`.
///
/// # 参数与返回
/// `offset` 是源字节偏移；超长值夹到文本末尾，字符内部位置向前退至 UTF-8 边界。
/// 返回前一个完整分隔符之后的位置，找不到时为零；位于 CRLF 的 LF 字节时仍归前一行。
/// 结果只描述该 `text` 的位置，不保留文本或计算软换行边界。
pub(crate) fn hard_line_start(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    text[..offset]
        .char_indices()
        .rev()
        .find(|(index, character)| {
            is_hard_line_separator(*character)
                && !is_crlf_prefix_boundary(text, *index, *character, offset)
        })
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0)
}

/// Returns the source end, excluding the separator, of the canonical hard line at `offset`.
///
/// # 参数与返回
/// 按 [hard_line_start] 相同规则夹取字节偏移，再查找后续分隔符的起点；没有分隔符时返回文本长度。
/// 若偏移正落在 CRLF 的 LF 字节，返回前一个 CR 的位置，避免把这一对分隔符拆入内容。
/// 返回的是内容终点；需要下一行起点时可传给 [next_hard_line_start]。
pub(crate) fn hard_line_end(text: &str, offset: usize) -> usize {
    let offset = clamp_utf8_boundary(text, offset);
    if text.as_bytes().get(offset) == Some(&b'\n')
        && offset > 0
        && text.as_bytes().get(offset - 1) == Some(&b'\r')
    {
        return offset - 1;
    }
    text[offset..]
        .char_indices()
        .find(|(_, character)| is_hard_line_separator(*character))
        .map(|(index, _)| offset + index)
        .unwrap_or(text.len())
}

/// Returns the source start of the next canonical hard line after a line end.
///
/// # 参数、返回与约束
/// `line_end` 通常来自 [hard_line_end]，按字节偏移夹到有效 UTF-8 边界后检查当前位置。
/// 位于分隔符时返回其后的字节位置，CRLF 一起跨过；当前位置不是分隔符或已到文本末尾时返回空值。
/// 本函数不会向后搜索分隔符，返回值与输入文本使用同一源偏移空间。
pub(crate) fn next_hard_line_start(text: &str, line_end: usize) -> Option<usize> {
    let line_end = clamp_utf8_boundary(text, line_end);
    let separator = text.get(line_end..)?.chars().next()?;
    if separator == '\r' {
        return Some(
            line_end
                + if text.as_bytes().get(line_end + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                },
        );
    }
    is_hard_line_separator(separator).then_some(line_end + separator.len_utf8())
}

/// 以字符索引维护源字节位置，将 CRLF 合并后向同步回调移交行描述。
///
/// 回调返回假时立即结束，不再访问剩余内容或终止行；否则扫描末尾再访问终止行。
/// `line_start` 仅在分隔符回调允许继续后推进，终止行回调的返回值不再影响后续状态。
/// 此内部停止协议用于窗口扫描；逐行访问与完整计数包装始终要求继续。
fn for_each_hard_line(text: &str, mut visit: impl FnMut(HardLine) -> bool) {
    let mut line_start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        let mut next_start = index + ch.len_utf8();
        let is_break = match ch {
            '\r' => {
                if let Some((next_index, '\n')) = chars.peek().copied() {
                    chars.next();
                    next_start = next_index + '\n'.len_utf8();
                }
                true
            }
            _ => is_hard_line_separator(ch),
        };
        if is_break {
            if !visit(HardLine {
                content: line_start..index,
                separator: index..next_start,
            }) {
                return;
            }
            line_start = next_start;
        }
    }
    let _ = visit(HardLine {
        content: line_start..text.len(),
        separator: text.len()..text.len(),
    });
}

/// 判断单个字符是否属于这里列出的规范硬行分隔符。
///
/// 此函数只识别字符；CRLF 配对由扫描与导航入口处理，其他字符不会因空白属性而自动成为分隔符。
pub(crate) fn is_hard_line_separator(character: char) -> bool {
    matches!(
        character,
        '\r' | '\n' | '\u{000b}' | '\u{000c}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

/// A prefix ending at the LF byte of CRLF contains only an incomplete separator.
/// Treating its CR as a completed break would place reverse line navigation inside CRLF.
///
/// 反向查找的前缀尚未包含 LF 时，阻止将已看到的 CR 作为完整分隔符。
/// 输入位置来自同一文本的字符索引；此处只判断 CRLF 前缀关系，不替代偏移夹取。
fn is_crlf_prefix_boundary(text: &str, index: usize, character: char, offset: usize) -> bool {
    character == '\r'
        && text.as_bytes().get(index + 1) == Some(&b'\n')
        && index.saturating_add(1) >= offset
}

/// 将任意字节偏移夹在文本范围内，并向前退到最近的 UTF-8 字符边界。
///
/// 返回值可用于本次借用文本的切片端点；只处理编码边界，不处理字素或硬行归属。
fn clamp_utf8_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
#[path = "tests/hard_line.rs"]
mod tests;

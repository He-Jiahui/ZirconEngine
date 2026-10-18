use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

use crate::text::layout_geometry::{finite_geometry, finite_sum};
use crate::text::{compiled_joining_type_map, TextJoiningTypeMap};

/// 余量不大于此阈值时跳过拉伸，避免为微小正余量分配。
const JUSTIFY_EPSILON: f32 = 0.01;

/// 按扩展字素顺序拉伸一行 advance，成功时返回独立的新数组。
///
/// # 参数与返回
/// `text` 与 `advances` 须描述同一行，每项 advance 对应一个扩展字素；这里只检查数量。
/// `natural_width` 与 `target_width` 由调用方提供，本函数不重新测量。
/// 余量不大于阈值、数量不匹配、少于两个字素或没有可分配位置时返回 `None`。
///
/// # 调用与约束
/// 布局和 SDF 绘制入口在决定两端对齐后调用；不修改文本、输入数组或生成延展字形。
/// 被调整项的局部数值恢复不等于完整几何校验，也不保证结果和精确等于目标宽度。
/// 参见 [`justification_opportunities`] 与 [`finite_sum`]。
pub(crate) fn justify_line_advances(
    text: &str,
    advances: &[f32],
    natural_width: f32,
    target_width: f32,
) -> Option<Vec<f32>> {
    let extra = target_width - natural_width;
    if extra <= JUSTIFY_EPSILON {
        return None;
    }

    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != advances.len() || graphemes.len() < 2 {
        return None;
    }

    let opportunities = justification_opportunities(&graphemes);
    if opportunities.is_empty() {
        return None;
    }

    let mut adjusted = advances.to_vec();
    let per_opportunity = extra / opportunities.len() as f32;
    let mut assigned = 0.0_f32;
    for (position, index) in opportunities.iter().copied().enumerate() {
        // 最后一项吸收前面累计后的剩余差值；有限恢复仍可能舍入或夹取。
        let delta = if position + 1 == opportunities.len() {
            finite_geometry(f64::from(extra) - f64::from(assigned))
        } else {
            per_opportunity
        };
        adjusted[index] = finite_sum([adjusted[index], delta]).max(0.0);
        assigned = finite_sum([assigned, delta]);
    }

    Some(adjusted)
}

/// Returns logical byte offsets immediately after Arabic joining pairs that are candidates for a
/// virtual tatweel. The shaping backend must still validate that a candidate is safe for the
/// selected face and language before materializing it.
///
/// # 坐标与使用
/// `text` 只借用于本次扫描；拥有的返回列表记录右侧字素的逻辑 UTF-8 字节起点。
/// 位置处于相邻字素之间，不是视觉坐标；字体和语言的插入安全性仍须后续塑形验证。
/// 需限制返回项数时使用 [`arabic_kashida_insertion_offsets_bounded`]。
pub(crate) fn arabic_kashida_insertion_offsets(text: &str) -> Vec<usize> {
    let joining_types = compiled_joining_type_map();
    arabic_kashida_insertion_offsets_with_map(text, joining_types)
}

/// 使用同一张连写属性表扫描相邻扩展字素，保留右侧字素在原输入中的字节起点。
fn arabic_kashida_insertion_offsets_with_map(
    text: &str,
    joining_types: TextJoiningTypeMap,
) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut previous = None;
    for (start, grapheme) in text.grapheme_indices(true) {
        if previous.is_some_and(|left| is_arabic_kashida_pair(left, grapheme, joining_types)) {
            offsets.push(start);
        }
        previous = Some(grapheme);
    }
    offsets
}

/// Returns at most `limit` evenly distributed insertion offsets without allocating one entry for
/// every joining pair in a long line.
///
/// # 参数、返回与代价
/// `limit == 0` 直接返回空列表；其余情形先数候选，再重新扫描取回偏移。
/// 候选过多时按候选序号分桶并取各桶中点；上限约束返回项数，不限制扫描量或保证视觉等距。
/// 返回坐标和安全性约束同 [`arabic_kashida_insertion_offsets`]，不改动输入文本。
pub(crate) fn arabic_kashida_insertion_offsets_bounded(text: &str, limit: usize) -> Vec<usize> {
    if limit == 0 {
        return Vec::new();
    }

    let joining_types = compiled_joining_type_map();
    let pair_count = arabic_kashida_pair_count(text, joining_types);
    if pair_count <= limit {
        return arabic_kashida_insertion_offsets_with_map(text, joining_types);
    }

    let mut offsets = Vec::with_capacity(limit);
    let mut next_slot = 0;
    let mut next_pair_index = Some(evenly_spaced_pair_index(next_slot, pair_count, limit));
    let mut pair_index = 0;
    let mut previous = None;
    for (start, grapheme) in text.grapheme_indices(true) {
        if previous.is_some_and(|left| is_arabic_kashida_pair(left, grapheme, joining_types)) {
            if next_pair_index == Some(pair_index) {
                offsets.push(start);
                next_slot += 1;
                next_pair_index = if next_slot < limit {
                    Some(evenly_spaced_pair_index(next_slot, pair_count, limit))
                } else {
                    None
                };
            }
            pair_index += 1;
        }
        previous = Some(grapheme);
    }
    offsets
}

/// 第一遍只统计符合相邻阿拉伯字素连写条件的候选，不保存每个偏移。
fn arabic_kashida_pair_count(text: &str, joining_types: TextJoiningTypeMap) -> usize {
    let mut count = 0;
    let mut previous = None;
    for (_, grapheme) in text.grapheme_indices(true) {
        if previous.is_some_and(|left| is_arabic_kashida_pair(left, grapheme, joining_types)) {
            count += 1;
        }
        previous = Some(grapheme);
    }
    count
}

/// 在 `pair_count > slot_count > 0` 且 `slot < slot_count` 时取候选序号桶的中点。
/// 前提由有界入口的过多候选分支建立；这里不验证任意 usize 参数的算术安全。
fn evenly_spaced_pair_index(slot: usize, pair_count: usize, slot_count: usize) -> usize {
    let bucket_start =
        slot * (pair_count / slot_count) + slot * (pair_count % slot_count) / slot_count;
    let next_slot = slot + 1;
    let bucket_end =
        next_slot * (pair_count / slot_count) + next_slot * (pair_count % slot_count) / slot_count;
    bucket_start + (bucket_end - bucket_start) / 2
}

/// 排除首尾 U+0020/U+3000 后，在内容区间内收集可拉伸项；下标仍对应原数组。
/// 内部词间空格优先，其次是列举范围的 CJK 对；阿拉伯对只接受已物化的 tatweel。
fn justification_opportunities(graphemes: &[&str]) -> Vec<usize> {
    let Some((content_start, content_end)) = content_grapheme_range(graphemes) else {
        return Vec::new();
    };

    let mut opportunities = Vec::new();
    let joining_types = compiled_joining_type_map();
    for index in content_start..content_end.saturating_sub(1) {
        if is_word_space(graphemes[index]) {
            opportunities.push(index);
            continue;
        }
        if is_cjk_justifiable_pair(graphemes[index], graphemes[index + 1]) {
            opportunities.push(index);
            continue;
        }
        if is_arabic_kashida_pair(graphemes[index], graphemes[index + 1], joining_types) {
            if let Some(tatweel_index) =
                arabic_tatweel_opportunity_index(graphemes, index, joining_types)
            {
                opportunities.push(tatweel_index);
            }
        }
    }
    opportunities
}

/// 相邻对的右侧须已经是 tatweel，才返回它在原 advance 数组中的下标。
fn arabic_tatweel_opportunity_index(
    graphemes: &[&str],
    index: usize,
    joining_types: TextJoiningTypeMap,
) -> Option<usize> {
    if is_tatweel(graphemes[index + 1], joining_types) {
        return Some(index + 1);
    }
    None
}

/// 取首尾非词间空白的半开字素下标区间；空输入或全为这些空白时没有内容。
fn content_grapheme_range(graphemes: &[&str]) -> Option<(usize, usize)> {
    let start = graphemes
        .iter()
        .position(|grapheme| !is_word_space(grapheme))?;
    let end = graphemes
        .iter()
        .rposition(|grapheme| !is_word_space(grapheme))?
        + 1;
    Some((start, end))
}

/// 这里只识别 U+0020 与 U+3000，不代表全部 Unicode 空白。
fn is_word_space(grapheme: &str) -> bool {
    matches!(grapheme, " " | "\u{3000}")
}

/// 两侧均须是下方列举范围内的单个标量，才提供 CJK 间隙机会。
fn is_cjk_justifiable_pair(left: &str, right: &str) -> bool {
    cjk_char(left).is_some() && cjk_char(right).is_some()
}

/// 按逻辑顺序核对左侧向后、右侧向前的连写能力，两端均须先通过阿拉伯字素筛选。
fn is_arabic_kashida_pair(left: &str, right: &str, joining_types: TextJoiningTypeMap) -> bool {
    let Some(left) = arabic_grapheme_base(left, joining_types) else {
        return false;
    };
    let Some(right) = arabic_grapheme_base(right, joining_types) else {
        return false;
    };

    joining_types
        .get(left)
        .joins_with_following_logical_character()
        && joining_types
            .get(right)
            .joins_with_preceding_logical_character()
}

/// 基字符须为阿拉伯脚本或 tatweel，其后仅允许透明连写标量或 ZWJ。
/// 此局部形态筛选不判断选定字体和语言是否能安全塑形。
fn arabic_grapheme_base(grapheme: &str, joining_types: TextJoiningTypeMap) -> Option<char> {
    let mut chars = grapheme.chars();
    let base = chars.next()?;
    ((base.script() == Script::Arabic || base == '\u{0640}')
        && chars.all(|ch| is_arabic_grapheme_continuation(ch, joining_types)))
    .then_some(base)
}

/// 透明连写标量和 ZWJ 可随基字符保留；ZWNJ 不在显式许可项中。
fn is_arabic_grapheme_continuation(ch: char, joining_types: TextJoiningTypeMap) -> bool {
    joining_types.get(ch).is_transparent() || matches!(ch, '\u{200d}')
}

/// 仅返回列举 Unicode 区间内的单标量字素；附带额外标量的字素不命中。
fn cjk_char(grapheme: &str) -> Option<char> {
    single_char(grapheme).filter(|ch| {
        matches!(
            *ch as u32,
            0x3040..=0x30FF
                | 0x31F0..=0x31FF
                | 0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xAC00..=0xD7AF
                | 0xF900..=0xFAFF
                | 0x20000..=0x2FA1F
        )
    })
}

/// 为恰含一个 Unicode 标量的非空字素返回字符，不把字节数当字符数。
fn single_char(grapheme: &str) -> Option<char> {
    let mut chars = grapheme.chars();
    let ch = chars.next()?;
    chars.next().is_none().then_some(ch)
}

/// 用相同的基字符和延续规则识别 U+0640，使已有 advance 成为可拉伸目标。
fn is_tatweel(grapheme: &str, joining_types: TextJoiningTypeMap) -> bool {
    matches!(
        arabic_grapheme_base(grapheme, joining_types),
        Some('\u{0640}')
    )
}

#[cfg(test)]
#[path = "tests/align.rs"]
mod tests;

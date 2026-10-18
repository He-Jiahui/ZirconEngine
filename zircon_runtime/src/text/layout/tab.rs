//! 按扩展字素的推进量计算从零游标起的制表停靠位置。
//! 此模块只调整几何；文本、推进量和样式由调用方持有。

use crate::text::layout_geometry::{finite_f32_or_geometry, finite_sum, FiniteGeometryAccumulator};
use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

// 制表大小至少按一个空格计，单次推进下限用于避免停在原位置。
const MIN_TAB_SIZE: f32 = 1.0;
const MIN_TAB_ADVANCE: f32 = 0.01;

/// 借用文本、推进量与样式，按值接收空格宽度，返回拥有的推进量副本。
/// `advances` 应与字素一一对应；缺少制表字符或数量不符时原样复制，不裁掉负值。
/// 匹配时使用 `style` 与 `space_width` 计算间距，非制表推进量按非负几何发布。
/// 规则参见 [`TextStyle`]；行框测量取得空格宽度后调用，不回写输入或缓存。
pub(crate) fn tab_aligned_advances(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> Vec<f32> {
    // 该回退保持调用方提供的原始推进量，包括负值。
    if !has_matching_tab_graphemes(text, advances.len()) {
        return advances.to_vec();
    }

    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    // 发布值沿用正常精度运算；宽精度历史供溢出后的几何恢复使用。
    let mut cursor = FiniteGeometryAccumulator::default();
    let mut adjusted = Vec::with_capacity(advances.len());
    for (grapheme, advance) in text.graphemes(true).zip(advances.iter().copied()) {
        let resolved_advance = if grapheme == "\t" {
            next_tab_advance(
                cursor.value(),
                cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        cursor.add(resolved_advance);
        adjusted.push(resolved_advance);
    }
    adjusted
}

/// 借用文本、推进量与样式，按值接收空格宽度，返回整段几何宽度。
/// 扫描时同时核对字素数量并累计原推进量，数量不符即退回原数组的有限和。
/// 完全匹配且含制表字符时返回调整后宽度，否则返回原累计值，可保留负推进。
/// 与向量入口共用 [`TextStyle`] 的制表规则；每次调用的起点为零。
pub(crate) fn tab_aligned_width(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> f32 {
    let all_advances = advances;
    let mut advances = all_advances.iter().copied();
    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut raw_cursor = FiniteGeometryAccumulator::default();
    let mut tabbed_cursor = FiniteGeometryAccumulator::default();
    let mut has_tab = false;

    // 单遍扫描同时保留原始与调整后的累计值，以便最后决定返回路径。
    for grapheme in text.graphemes(true) {
        // 推进量不足时不返回已算前缀，而是累计整个原数组。
        let Some(advance) = advances.next() else {
            return finite_sum(all_advances.iter().copied());
        };
        raw_cursor.add(advance);
        let resolved_advance = if grapheme == "\t" {
            has_tab = true;
            next_tab_advance(
                tabbed_cursor.value(),
                tabbed_cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        tabbed_cursor.add(resolved_advance);
    }

    // 还剩推进量同样视为错配；完整数组的回退与不足情况一致。
    if advances.next().is_some() {
        return finite_sum(all_advances.iter().copied());
    }
    // 没有制表字符时保留原推进量和，不采用扫描中计算的非负替代值。
    if has_tab {
        tabbed_cursor.value()
    } else {
        raw_cursor.value()
    }
}

/// Streams a tabbed sequence whose grapheme count and tab presence were already validated.
/// 调用方须已确认含制表字符且字素数量与 `advances` 长度相等。
/// 此入口直接并行遍历两个序列，不复查数量；违反前提时会只处理配对前缀。
/// 返回从零起点累计的宽度，输入借用只延续到本次调用。
/// 测量模块先投影字素几何，再用空格测量结果调用此入口。
pub(super) fn tab_aligned_width_for_matching_graphemes(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> f32 {
    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut cursor = FiniteGeometryAccumulator::default();
    for (grapheme, advance) in text.graphemes(true).zip(advances.iter().copied()) {
        let resolved_advance = if grapheme == "\t" {
            next_tab_advance(
                cursor.value(),
                cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        cursor.add(resolved_advance);
    }
    cursor.value()
}

/// 由借用样式与按值空格宽度计算停靠间距，空格宽度至少取推进下限。
/// 非有限候选由宽精度结果恢复；有限候选维持原运算结果。
pub(crate) fn tab_interval_width(style: &TextStyle, space_width: f32) -> f32 {
    finite_f32_or_geometry(
        space_width.max(MIN_TAB_ADVANCE) * resolved_tab_size(style),
        tab_interval_exact_width(style, space_width),
    )
}

/// 按相同下限在 f64 中计算间距，供恢复路径使用；不承诺数学精确值。
fn tab_interval_exact_width(style: &TextStyle, space_width: f32) -> f64 {
    f64::from(space_width.max(MIN_TAB_ADVANCE)) * f64::from(resolved_tab_size(style))
}

/// 有限的制表大小夹到最小值，非有限值使用 [`TextStyle`] 的默认值。
/// 仅返回本次计算使用的值，不回写样式字段。
fn resolved_tab_size(style: &TextStyle) -> f32 {
    if style.tab_size.is_finite() {
        style.tab_size.max(MIN_TAB_SIZE)
    } else {
        TextStyle::DEFAULT_TAB_SIZE
    }
}

/// 由当前发布位置与宽精度历史计算严格下一个停靠位置的推进量。
/// 两种精度各自计算后交恢复入口选择，返回推进量至少按下限处理。
fn next_tab_advance(
    cursor: f32,
    cursor_exact: f64,
    tab_interval: f32,
    tab_interval_exact: f64,
) -> f32 {
    let tab_interval = tab_interval.max(MIN_TAB_ADVANCE);
    // 即使当前位置恰在停靠点，也前进到下一个间距。
    let next_stop = ((cursor / tab_interval).floor() + 1.0) * tab_interval;
    let next_stop_exact = ((cursor_exact / tab_interval_exact).floor() + 1.0) * tab_interval_exact;
    finite_f32_or_geometry(
        (next_stop - cursor).max(MIN_TAB_ADVANCE),
        (next_stop_exact - cursor_exact).max(f64::from(MIN_TAB_ADVANCE)),
    )
}

/// 用一次折叠同时统计扩展字素数量并记录是否出现制表字符。
/// 仅两项条件都满足才许可向量调整路径；不收集字素列表。
fn has_matching_tab_graphemes(text: &str, advance_count: usize) -> bool {
    let (grapheme_count, has_tab) = text
        .graphemes(true)
        .fold((0_usize, false), |(count, has_tab), grapheme| {
            (count + 1, has_tab || grapheme == "\t")
        });
    grapheme_count == advance_count && has_tab
}

#[cfg(test)]
#[path = "tests/tab.rs"]
mod tests;

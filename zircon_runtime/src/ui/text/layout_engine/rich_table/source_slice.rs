//! 对表格周围文本和单元内容建立局部解析投影，并回移布局的源字节范围。
//! 投影共享编译产物的引用计数所有权，但仍会建立新的投影描述与索引，不承诺无分配。

use crate::core::framework::text::TextLayoutError;
use crate::text::shaping::TextLayoutOutcome;
use crate::text::shaping::TextShapingOutcome;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout},
};

use super::super::super::rich_text::UiParsedText;
use super::super::layout_parsed_text_with_provider_outcome;

/// 在同一文本会话中布局解析视图的局部字节范围。
///
/// # 参数与返回
/// `range` 相对当前 `parsed` 视图；`frame` 与 `clip` 是本次物理布局框和显式裁剪框。
/// 成功返回拥有的布局，并把局部源范围回移到当前视图坐标；延迟或失败结果保持原样传播。
///
/// # 借用与约束
/// 投影失败直接返回布局失败结果；会话借用贯穿布局调用，可累计缓存、工作量和诊断状态。
/// 返回布局不借用局部投影视图；范围回移依赖投影和布局已满足局部范围约束。
pub(super) fn layout_range_with_provider(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiResolvedTextLayout> {
    let local = match slice_parsed(parsed, range.clone()) {
        Ok(local) => local,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    layout_parsed_text_with_provider_outcome(&local, style, frame, Some(clip), provider).map(
        |mut layout| {
            shift_layout_source_ranges(&mut layout, range.start);
            layout
        },
    )
}

/// 不额外指定父表深度地建立局部投影；范围错误由解析投影层返回。
pub(super) fn slice_parsed(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
) -> Result<UiParsedText, TextLayoutError> {
    slice_parsed_with_table_depth(parsed, range, None)
}

/// 将局部字节范围及可选父表深度交给解析视图建立拥有的投影描述。
/// 范围顺序、边界和整数转换由投影层检查；父深度用于选择单元投影及后续表根深度。
/// 成功结果共享编译产物，可独立于本次输入借用继续存活；详见 `UiParsedText::project_range`。
pub(super) fn slice_parsed_with_table_depth(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    parent_table_depth: Option<u16>,
) -> Result<UiParsedText, TextLayoutError> {
    parsed.project_range(range, parent_table_depth)
}

/// 将布局、装饰盒子、行及行内运行的局部源范围原地加上相同偏移，不改变几何。
/// 调用者须保证所有范围加偏移后仍可表示，且与父视图一致；此处使用普通加法，不重新校验或返回错误。
pub(super) fn shift_layout_source_ranges(layout: &mut UiResolvedTextLayout, offset: usize) {
    layout.source_range.start += offset;
    layout.source_range.end += offset;
    for text_box in &mut layout.boxes {
        text_box.range.start += offset;
        text_box.range.end += offset;
    }
    for line in &mut layout.lines {
        line.source_range.start += offset;
        line.source_range.end += offset;
        for run in &mut line.runs {
            run.source_range.start += offset;
            run.source_range.end += offset;
        }
    }
}

#[cfg(test)]
#[path = "tests/source_slice.rs"]
mod tests;

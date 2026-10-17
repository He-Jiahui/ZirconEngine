//! 为已解析文本生成选区、预编辑与光标的装饰几何，供绘制命令组装载荷。
//!
//! 输入布局与编辑状态只在本次调用中借用；输出装饰拥有颜色、范围和行框值。
//! 源字节范围先经行映射投影为视觉字节范围，再将推进量换算为布局坐标。
//! 临时映射只在一次生成内按触及行复用；本层不修改文档或执行字体整形。
//! 视口裁剪由后续绘制路径处理，装饰生成本身不裁剪行框。
//! 参考：[`UiTextLineSourceMap`]、[`UiTextPaintDecoration`]。

use std::{collections::HashMap, ops::Range};

use crate::ui::{layout::UiFrame, surface::UiTextPreeditClauseKind};

use super::{
    UiEditableTextState, UiResolvedTextLayout, UiResolvedTextLine, UiTextCaret,
    UiTextCaretAffinity, UiTextPaintDecoration, UiTextPaintDecorationKind, UiTextRange,
    UiTextWritingMode,
};

#[path = "source_map/mod.rs"]
mod source_map;

pub use source_map::{UiTextLineSourceMap, UiTextVisualBoundaryBias, UiTextVisualSpan};

#[cfg(test)]
#[path = "tests/source_map.rs"]
mod source_map_tests;

#[cfg(test)]
#[path = "tests/caret_perf.rs"]
mod caret_performance_tests;

#[cfg(test)]
#[path = "../tests/decoration_perf.rs"]
mod decoration_performance_tests;

const TEXT_SELECTION_COLOR: &str = "#4d89ff66";
const TEXT_CARET_COLOR: &str = "#e8eef7";
const TEXT_COMPOSITION_HIGHLIGHT_COLOR: &str = "#4d89ff24";
const TEXT_COMPOSITION_UNDERLINE_COLOR: &str = "#4d89ff";
const TEXT_COMPOSITION_CONVERTED_UNDERLINE_COLOR: &str = "#72b7f2";
const TEXT_COMPOSITION_TARGET_CONVERTED_UNDERLINE_COLOR: &str = "#42bf77";
const TEXT_COMPOSITION_TARGET_NOT_CONVERTED_UNDERLINE_COLOR: &str = "#e05a5a";
const TEXT_CARET_WIDTH: f32 = 1.0;
const TEXT_COMPOSITION_UNDERLINE_HEIGHT: f32 = 2.0;

/// 将编辑状态转换为范围装饰，并在范围装饰之后追加光标。
///
/// # 参数
/// - `layout`：本次绘制采用的已解析行和书写方向；几何沿用各行的行框坐标。
/// - `editable`：源文本字节偏移与预编辑子句的只读状态快照。
///
/// # 返回
/// 拥有颜色和几何值的装饰列表，不保留输入引用。
///
/// # 约束
/// 子句偏移相对组合范围；饱和加法和端点限制只约束数值，不验证字符边界。
/// 范围装饰与光标可共用本次惰性行映射，纯光标路径则单独构造所选行映射。
/// 本函数不应用编辑操作，也不裁剪到视口。
///
/// # 参考
/// [`UiEditableTextState`]、[`UiResolvedTextLayout`]、[`UiTextLineSourceMap`]。
pub(super) fn editable_text_decorations(
    layout: &UiResolvedTextLayout,
    editable: &UiEditableTextState,
) -> Vec<UiTextPaintDecoration> {
    let mut decorations = Vec::new();
    let composition = editable.composition.as_ref();
    let mut range_decorations = Vec::with_capacity(
        usize::from(editable.selection.is_some())
            + composition.map_or(0, |value| value.preedit_clauses.len().max(1) + 1),
    );

    if let Some(composition) = composition {
        range_decorations.push(TextRangeDecoration::composition_highlight(
            composition.range,
        ));
    }

    if let Some(selection) = editable.selection.as_ref() {
        let range = selection.range();
        if range.start < range.end {
            range_decorations.push(TextRangeDecoration::selection(range));
        }
    }

    if let Some(composition) = composition {
        if composition.preedit_clauses.is_empty() {
            range_decorations.push(TextRangeDecoration::composition_underline(
                composition.range,
                TEXT_COMPOSITION_UNDERLINE_COLOR,
            ));
        } else {
            for clause in &composition.preedit_clauses {
                let start = composition
                    .range
                    .start
                    .saturating_add(clause.range.start_byte as usize)
                    .min(composition.range.end);
                let range = UiTextRange {
                    start,
                    end: composition
                        .range
                        .start
                        .saturating_add(clause.range.end_byte as usize)
                        .min(composition.range.end)
                        .max(start),
                };
                range_decorations.push(TextRangeDecoration::composition_underline(
                    range,
                    composition_underline_color(clause.kind),
                ));
            }
        }
    }

    let mut source_maps =
        (!range_decorations.is_empty()).then(|| TextDecorationLineSourceMaps::new(&layout.lines));
    if let Some(source_maps) = source_maps.as_mut() {
        append_range_decorations_with_source_maps(
            &mut decorations,
            layout,
            &range_decorations,
            source_maps,
        );
    }

    let caret_frame = match source_maps.as_mut() {
        Some(source_maps) => caret_frame_with_source_maps(layout, &editable.caret, source_maps),
        None => caret_frame(layout, &editable.caret),
    };
    if let Some(frame) = caret_frame {
        decorations.push(UiTextPaintDecoration {
            kind: UiTextPaintDecorationKind::Caret,
            range: UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            },
            frame,
            color: TEXT_CARET_COLOR.to_string(),
            thickness: TEXT_CARET_WIDTH,
        });
    }
    decorations
}

/// 范围装饰的几何度量类别；背景使用整行次轴厚度，下划线使用固定厚度。
#[derive(Clone, Copy)]
enum TextDecorationMetric {
    Selection,
    CompositionUnderline,
}

/// 待投影的绘制种类；高亮与选区共享几何度量，但保留不同绘制标记。
#[derive(Clone, Copy)]
enum TextRangeDecorationKind {
    Selection,
    CompositionHighlight,
    CompositionUnderline,
}

/// 投影之前的源字节范围与绘制意图值，不持有文本或行映射。
///
/// 范围、静态颜色和种类只负责组装；创建时不校验字符边界。
/// 生成绘制条目时再把颜色转换为拥有值，几何由调用者提供。
#[derive(Clone, Copy)]
struct TextRangeDecoration {
    /// 源文本中的字节端点；具体可见片段由行映射投影。
    /// 每个片段的绘制条目仍保留这一原请求范围。
    range: UiTextRange,
    /// 本模块静态调色值，直到组装绘制条目才创建拥有的字符串。
    color: &'static str,
    /// 同时决定几何度量选择与最终绘制种类。
    kind: TextRangeDecorationKind,
}

impl TextRangeDecoration {
    const fn selection(range: UiTextRange) -> Self {
        Self {
            range,
            color: TEXT_SELECTION_COLOR,
            kind: TextRangeDecorationKind::Selection,
        }
    }

    const fn composition_underline(range: UiTextRange, color: &'static str) -> Self {
        Self {
            range,
            color,
            kind: TextRangeDecorationKind::CompositionUnderline,
        }
    }

    const fn composition_highlight(range: UiTextRange) -> Self {
        Self {
            range,
            color: TEXT_COMPOSITION_HIGHLIGHT_COLOR,
            kind: TextRangeDecorationKind::CompositionHighlight,
        }
    }

    /// 高亮和选区共用行厚度；绘制种类仍由原请求保留。
    const fn metric(self) -> TextDecorationMetric {
        match self.kind {
            TextRangeDecorationKind::Selection | TextRangeDecorationKind::CompositionHighlight => {
                TextDecorationMetric::Selection
            }
            TextRangeDecorationKind::CompositionUnderline => {
                TextDecorationMetric::CompositionUnderline
            }
        }
    }

    /// 将给定布局坐标行框与意图组装为拥有颜色字符串的绘制值。
    fn paint(self, frame: UiFrame) -> UiTextPaintDecoration {
        match self.kind {
            TextRangeDecorationKind::Selection => {
                UiTextPaintDecoration::selection(self.range, frame, self.color)
            }
            TextRangeDecorationKind::CompositionHighlight => {
                UiTextPaintDecoration::composition_highlight(self.range, frame, self.color)
            }
            TextRangeDecorationKind::CompositionUnderline => {
                UiTextPaintDecoration::composition_underline(self.range, frame, self.color)
            }
        }
    }
}

/// 同一次编辑装饰生成中按触及行建立的临时映射缓存。
///
/// 行切片在缓存存在期间只读借用，映射始终与构造时的切片配套。
/// 返回的映射引用还受当前缓存借用约束；缓存不跨本次生成共享。
/// 参考：[`UiTextLineSourceMap`]、[`editable_text_decorations`]。
struct TextDecorationLineSourceMaps<'a> {
    /// 构造时借用的行切片，行下标均相对于它解释。
    lines: &'a [UiResolvedTextLine],
    /// 只保存实际触及行的映射，键为上述切片下标。
    maps: HashMap<usize, UiTextLineSourceMap<'a>>,
    /// 仅记录源起点、终点各自是否不递减，用于选择范围遍历方式。
    source_ranges_are_ordered: bool,
    /// 仅测试保留本缓存首次构造映射的次数，不是运行期统计。
    #[cfg(test)]
    initialized_count: usize,
}

impl<'a> TextDecorationLineSourceMaps<'a> {
    /// 借用行切片并准备空缓存，同时记录源端点顺序。
    ///
    /// 首次触及某行时才构造映射；此处不验证字符边界或行框几何。
    fn new(lines: &'a [UiResolvedTextLine]) -> Self {
        Self {
            lines,
            maps: HashMap::new(),
            // `UiResolvedTextLayout` is an interface DTO, so callers may
            // provide lines whose source ranges are not monotonic. Keep the
            // binary range path for the normal shaped-layout contract and
            // remember when a safe linear fallback is required.
            source_ranges_are_ordered: lines.windows(2).all(|pair| {
                pair[0].source_range.start <= pair[1].source_range.start
                    && pair[0].source_range.end <= pair[1].source_range.end
            }),
            #[cfg(test)]
            initialized_count: 0,
        }
    }

    /// 取得行与其复用映射；下标越界时返回空且不创建缓存项。
    ///
    /// `line_index` 是构造时行切片的下标，首次访问才建立映射。
    /// 返回行沿用输入借用期，映射引用受本次可变缓存借用约束。
    fn for_line(
        &mut self,
        line_index: usize,
    ) -> Option<(&'a UiResolvedTextLine, &UiTextLineSourceMap<'a>)> {
        let line = self.lines.get(line_index)?;
        let source_map = match self.maps.entry(line_index) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                #[cfg(test)]
                {
                    self.initialized_count += 1;
                }
                entry.insert(UiTextLineSourceMap::new(line))
            }
        };
        Some((line, source_map))
    }

    #[cfg(test)]
    fn initialized_count(&self) -> usize {
        self.initialized_count
    }
}

/// 将源字节范围装饰追加到输出，保留装饰声明顺序。
///
/// # 参数
/// - `decorations`：追加目标，原有条目保留。
/// - `layout` 与 `source_maps`：须使用同一组行，映射按行下标配对。
/// - `range_decorations`：本次只读范围意图值切片。
///
/// # 约束
/// 端点有序时只遍历候选区间，否则逐行筛选；随后仍投影到视觉字节范围。
/// 此处只生成几何条目，不执行视口裁剪。
fn append_range_decorations_with_source_maps(
    decorations: &mut Vec<UiTextPaintDecoration>,
    layout: &UiResolvedTextLayout,
    range_decorations: &[TextRangeDecoration],
    source_maps: &mut TextDecorationLineSourceMaps<'_>,
) {
    // Reuse each touched line's cluster projection and exact-advance cache
    // across the selection and every IME clause while retaining declaration order.
    for decoration in range_decorations {
        if source_maps.source_ranges_are_ordered {
            for line_index in intersecting_line_range(source_maps.lines, decoration.range) {
                append_range_decoration_for_line(
                    decorations,
                    layout,
                    *decoration,
                    source_maps,
                    line_index,
                );
            }
        } else {
            // Keep declaration and source order for malformed/foreign DTOs;
            // unlike a guessed binary search, this cannot silently skip a
            // line whose range was published out of order.
            let line_count = source_maps.lines.len();
            for line_index in 0..line_count {
                let intersects = {
                    let line = &source_maps.lines[line_index];
                    decoration.range.start < line.source_range.end
                        && line.source_range.start < decoration.range.end
                };
                if intersects {
                    append_range_decoration_for_line(
                        decorations,
                        layout,
                        *decoration,
                        source_maps,
                        line_index,
                    );
                }
            }
        }
    }
}

/// 为已确认与范围相交的一行追加各个视觉片段。
///
/// 下标越界时不追加；推进量由配套映射给出，结果沿用行框的布局坐标。
fn append_range_decoration_for_line(
    decorations: &mut Vec<UiTextPaintDecoration>,
    layout: &UiResolvedTextLayout,
    decoration: TextRangeDecoration,
    source_maps: &mut TextDecorationLineSourceMaps<'_>,
    line_index: usize,
) {
    // Both callers have already established intersection: the ordered path
    // derives `line_index` from `intersecting_line_range`, while the fallback
    // checks the DTO line explicitly. Avoid repeating that range predicate
    // before touching the lazy source-map cache.
    let Some((line, source_map)) = source_maps.for_line(line_index) else {
        return;
    };
    for span in source_map.visual_spans_for_source_range(decoration.range) {
        let start = source_map.advance_to_visual_offset(span.visual_range.start);
        let end = source_map.advance_to_visual_offset(span.visual_range.end);
        decorations.push(decoration.paint(decoration_frame(
            layout,
            line,
            start,
            end,
            decoration.metric(),
        )));
    }
}

/// 把源字节范围定位为可能相交的行下标半开区间。
///
/// # 约束
/// 行的源起点和终点分别不递减，调用者负责检查该前提。
/// 这里只生成候选下标，不校验输入范围或执行字符切片。
fn intersecting_line_range(lines: &[UiResolvedTextLine], range: UiTextRange) -> Range<usize> {
    let start = lines.partition_point(|line| line.source_range.end <= range.start);
    let end = lines.partition_point(|line| line.source_range.start < range.end);
    start.min(end)..end
}

fn composition_underline_color(kind: UiTextPreeditClauseKind) -> &'static str {
    match kind {
        UiTextPreeditClauseKind::Input => TEXT_COMPOSITION_UNDERLINE_COLOR,
        UiTextPreeditClauseKind::Converted => TEXT_COMPOSITION_CONVERTED_UNDERLINE_COLOR,
        UiTextPreeditClauseKind::TargetConverted => {
            TEXT_COMPOSITION_TARGET_CONVERTED_UNDERLINE_COLOR
        }
        UiTextPreeditClauseKind::TargetNotConverted => {
            TEXT_COMPOSITION_TARGET_NOT_CONVERTED_UNDERLINE_COLOR
        }
    }
}

/// 以行框及主轴推进量组装范围装饰几何。
///
/// # 参数
/// - `layout`：提供书写方向。
/// - `line`：提供几何原点与次轴厚度，结果使用同一布局坐标系。
/// - `start`、`end`：行内推进量，不是字节偏移。
/// - `metric`：选择整行厚度或固定下划线厚度。
///
/// # 返回
/// 主轴尺寸至少为光标宽度的行框值；竖排交换两轴，下划线贴右侧或底侧。
/// 此处不裁剪结果，也不补做几何有效性验证。
fn decoration_frame(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    start: f32,
    end: f32,
    metric: TextDecorationMetric,
) -> UiFrame {
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        let (x, width) = match metric {
            TextDecorationMetric::Selection => (line.frame.x, line.frame.width),
            TextDecorationMetric::CompositionUnderline => (
                line.frame.right() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                TEXT_COMPOSITION_UNDERLINE_HEIGHT,
            ),
        };
        return UiFrame::new(
            x,
            line.frame.y + start.min(end),
            width,
            (end - start).abs().max(TEXT_CARET_WIDTH),
        );
    }

    let (y, height) = match metric {
        TextDecorationMetric::Selection => (line.frame.y, line.frame.height),
        TextDecorationMetric::CompositionUnderline => (
            line.frame.bottom() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
            TEXT_COMPOSITION_UNDERLINE_HEIGHT,
        ),
    };
    UiFrame::new(
        line.frame.x + start.min(end),
        y,
        (end - start).abs().max(TEXT_CARET_WIDTH),
        height,
    )
}

/// 独立构造所选行映射并生成光标几何；无可选行时返回空。
///
/// 返回的是行框值，不保留布局或临时映射引用。
fn caret_frame(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<UiFrame> {
    let line = caret_line(layout, caret)?;
    let map = UiTextLineSourceMap::new(line);
    caret_frame_from_source_map(layout, caret, line, &map)
}

/// 复用范围装饰的行映射生成光标几何。
///
/// `source_maps` 须与 `layout` 的行配套；无可选下标或缓存行时返回空。
/// 输出只保留几何值，缓存借用在调用完成后结束。
fn caret_frame_with_source_maps(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    source_maps: &mut TextDecorationLineSourceMaps<'_>,
) -> Option<UiFrame> {
    let line_index = caret_line_index(layout, caret)?;
    let (line, map) = source_maps.for_line(line_index)?;
    caret_frame_from_source_map(layout, caret, line, map)
}

/// 将光标的视觉字节位置换算为行内推进量，再组装光标行框。
///
/// `line` 与 `map` 须配套；书写方向决定光标厚度所在的轴。
/// 本层不再选择行，也不验证字符边界；两个书写方向分支均返回行框值。
fn caret_frame_from_source_map(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    line: &UiResolvedTextLine,
    map: &UiTextLineSourceMap<'_>,
) -> Option<UiFrame> {
    let main_offset = map.advance_to_visual_offset(map.visual_offset_for_caret(caret));
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        return Some(UiFrame::new(
            line.frame.x,
            line.frame.y + main_offset,
            line.frame.width.max(TEXT_CARET_WIDTH),
            TEXT_CARET_WIDTH,
        ));
    }
    Some(UiFrame::new(
        line.frame.x + main_offset,
        line.frame.y,
        TEXT_CARET_WIDTH,
        line.frame.height.max(TEXT_CARET_WIDTH),
    ))
}

/// 按源字节光标与亲和性借用布局中的行。
///
/// # 返回
/// 借用来自 `layout`；无行时为空。共享边界可由亲和性选取不同相邻行。
///
/// # 约束
/// 快速候选依赖行源端点顺序，只有候选不覆盖偏移时才回退完整遍历。
/// 源范围右端点也可承载光标；此处不验证全文或字符边界。
/// 参考：[`UiTextCaret`]、[`UiResolvedTextLine`]。
fn caret_line<'a>(
    layout: &'a UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<&'a UiResolvedTextLine> {
    let candidate = match caret.affinity {
        UiTextCaretAffinity::Upstream => {
            let index = layout
                .lines
                .partition_point(|line| line.source_range.end < caret.offset);
            layout.lines.get(index)
        }
        UiTextCaretAffinity::Downstream => {
            let end = layout
                .lines
                .partition_point(|line| line.source_range.start <= caret.offset);
            end.checked_sub(1).and_then(|index| layout.lines.get(index))
        }
    };
    candidate
        .filter(|line| line_contains_caret(line, caret.offset))
        .or_else(|| linear_caret_line(layout, caret))
}

/// 采用同样的候选与回退策略返回行下标，供共享映射缓存定位。
/// 下标相对于 `layout.lines`，不持有行引用。
fn caret_line_index(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<usize> {
    let candidate = match caret.affinity {
        UiTextCaretAffinity::Upstream => {
            let index = layout
                .lines
                .partition_point(|line| line.source_range.end < caret.offset);
            Some(index).filter(|index| {
                layout
                    .lines
                    .get(*index)
                    .is_some_and(|line| line_contains_caret(line, caret.offset))
            })
        }
        UiTextCaretAffinity::Downstream => {
            let end = layout
                .lines
                .partition_point(|line| line.source_range.start <= caret.offset);
            end.checked_sub(1).filter(|index| {
                layout
                    .lines
                    .get(*index)
                    .is_some_and(|line| line_contains_caret(line, caret.offset))
            })
        }
    };
    candidate.or_else(|| linear_caret_line_index(layout, caret))
}

fn linear_caret_line<'a>(
    layout: &'a UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<&'a UiResolvedTextLine> {
    linear_caret_line_index(layout, caret).and_then(|index| layout.lines.get(index))
}

/// 完整遍历时，上游取首个匹配行，下游取最后一个。
///
/// 未匹配时，首行之前的偏移选首行，其余偏移选末行；空布局返回空。
fn linear_caret_line_index(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<usize> {
    match caret.affinity {
        UiTextCaretAffinity::Upstream => layout
            .lines
            .iter()
            .position(|line| line_contains_caret(line, caret.offset)),
        UiTextCaretAffinity::Downstream => layout
            .lines
            .iter()
            .rposition(|line| line_contains_caret(line, caret.offset)),
    }
    .or_else(|| {
        layout
            .lines
            .first()
            .filter(|line| caret.offset < line.source_range.start)
            .map(|_| 0)
    })
    .or_else(|| layout.lines.len().checked_sub(1))
}

/// 光标匹配包含源范围右端点，使相邻行交界可由亲和性选择。
fn line_contains_caret(line: &UiResolvedTextLine, offset: usize) -> bool {
    offset >= line.source_range.start && offset <= line.source_range.end
}

#[cfg(test)]
#[path = "tests/mod_perf.rs"]
mod performance_tests;

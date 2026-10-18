//! 从成形簇断点构造供上层换行消费的源序块。
//!
//! 本层按硬行请求成形并应用禁则与词尾规则；候选行宽限制和最终排布由调用方处理。
//! 块借用本次输入文本，不保留 provider 或成形 run；返回三态不构成 provider 工作回滚。

pub(crate) use self::boundary_correction::{
    corrected_glyph_ranges_with_provider, corrected_index_advance_with_provider,
    corrected_metric_ranges, BOUNDARY_SHAPING_CONTEXT_GRAPHEMES,
};
use self::glue::allows_glyph_fallback;
pub(crate) use self::greedy::{line_text_fits_with_provider, should_wrap_before_accumulated};
pub(crate) use self::soft_hyphen::{
    break_suffix_at as soft_hyphen_break_suffix_at, DiscretionaryHyphenDecision,
};
pub(crate) use self::wrap_space::{trailing_wrap_space_byte_len, trim_leading_wrap_spaces};
use super::kinsoku::apply_kinsoku_start_rules;
use crate::core::framework::text::{TextDirection, TextLayoutError};
#[cfg(test)]
use crate::text::shaping::DirectTextShapeRunProvider;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{TextRange, TextStyle};

#[path = "boundary_correction.rs"]
mod boundary_correction;
#[path = "glue.rs"]
mod glue;
#[path = "glyph_fallback.rs"]
mod glyph_fallback;
#[path = "greedy.rs"]
mod greedy;
#[path = "smart.rs"]
mod smart;
#[path = "soft_hyphen.rs"]
mod soft_hyphen;
#[path = "wrap_space.rs"]
mod wrap_space;

/// 借用本次输入原文的换行候选块，供上层选断点和拼接候选行。
///
/// # 坐标与生命周期
/// 两个范围均使用本次输入的 UTF-8 字节坐标；`visual_range` 不是屏幕或视觉排序坐标。
/// `text` 的借用随输入存活，块本身不持有成形 run、provider 或字体数据库。
/// 较大文档中的位置由调用方加回片段偏移；字段本身不完成整份成形数据验证。
/// 参见 [line_break_chunks_with_provider] 与 [LineBreakChunk::new]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineBreakChunk<'a> {
    /// 借用原文中的当前片段；后续合并仍从同一输入重新切片。
    pub text: &'a str,
    /// 当前文本片段在本次输入中的字节范围，不表示最终视觉位置。
    pub visual_range: TextRange,
    /// 供上层映射回输入源位置的字节范围，附加断点决策另见 `break_suffix`。
    pub source_range: TextRange,
    /// 允许过宽块进一步按字素折行；这是折行许可，和字体覆盖回退无关。
    pub allow_glyph_fallback: bool,
    /// 块后的强制断点标志，供后续合并和排布保留断行语义。
    pub mandatory_break: bool,
    /// 可选软连字符断点决策；本层保存元数据，由消费方决定断点处后缀的使用。
    pub break_suffix: Option<DiscretionaryHyphenDecision>,
}

/// 测试用同步入口，创建直接 provider 后要求结果为 Ready。
/// 非 Ready 结果经 `into_result` 使原有 `expect` 失败；生产调用应使用三态入口。
/// 返回块仍借用 `text`，不借用临时 provider。
#[cfg(test)]
pub(crate) fn line_break_chunks<'a>(text: &'a str, style: &TextStyle) -> Vec<LineBreakChunk<'a>> {
    let mut provider = DirectTextShapeRunProvider::default();
    line_break_chunks_with_provider(text, style, &mut provider)
        .into_result()
        .expect("test shaping request must be valid")
}

/// 按硬行成形并返回经过禁则处理、按逻辑源顺序物化的候选块。
///
/// # 参数、返回与时机
/// `text` 是本次输入，`style` 用于各段请求，`provider` 在本次调用内可变借用。
/// 非空输入按硬行的内容及分隔符范围调用水平成形 trait 方法，方向为 Auto 并启用 kerning。
/// 空输入直接返回 Ready 空向量；任一段 Deferred/Failed 都原样提前返回，不输出局部块。
/// 这不撤销 provider 已完成的请求、计账或缓存工作。
///
/// # 约束与生命周期
/// 返回块仅借用 `text`；局部范围检查只针对本层消费的簇首数据，并非完整 run 验证。
/// 本入口不按行宽挑选断点；词尾策略另见 [word_smart_line_break_chunks_with_provider]。
pub(crate) fn line_break_chunks_with_provider<'a, P>(
    text: &'a str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<LineBreakChunk<'a>>>
where
    P: TextShapeRunProvider + ?Sized,
{
    if text.is_empty() {
        return TextShapingOutcome::Ready(Vec::new());
    }

    let mut chunks = Vec::new();
    for hard_line in crate::text::hard_lines(text) {
        let source_range = hard_line.source_range();
        let Some(paragraph_text) = text.get(source_range.clone()) else {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        };
        let shaped = match provider.shape_horizontal_range_with_kerning(
            paragraph_text,
            style,
            TextDirection::Auto,
            TextRange {
                start: source_range.start,
                end: source_range.end,
            },
            true,
        ) {
            TextShapingOutcome::Ready(shaped) => shaped,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        match append_shaped_line_break_chunks(text, &shaped, source_range, &mut chunks) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    }

    TextShapingOutcome::Ready(apply_kinsoku_start_rules(text, chunks))
}

/// 将一个硬行的成形断点追加为借用原文的候选块。
///
/// `source_range` 使用 `text` 的半开 UTF-8 字节坐标；`shaped` 只在调用期间借用。
/// 仅检查簇首字形，空源范围的虚拟簇首验证位置后跳过；普通簇首即使无断点也检查源范围。
/// 断点按结束、起始偏移整理，同簇重复断点合并强制标志，矛盾或重叠边界返回失败。
/// 硬行范围无效返回 LayoutFailed，簇范围或边界矛盾返回 BidiInvariant。
/// `chunks` 可已含前段；后续失败前可能已追加本段部分块，外层会丢弃整个局部结果。
fn append_shaped_line_break_chunks<'a>(
    text: &'a str,
    shaped: &crate::text::ShapedGlyphRun,
    source_range: std::ops::Range<usize>,
    chunks: &mut Vec<LineBreakChunk<'a>>,
) -> TextLayoutOutcome<()> {
    if source_range.start > source_range.end
        || source_range.end > text.len()
        || !text.is_char_boundary(source_range.start)
        || !text.is_char_boundary(source_range.end)
    {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    }

    // Shaping backends may expose glyphs in visual order (notably RTL runs). Collect only
    // logical break boundaries first, then materialize chunks in source order.
    let mut break_boundaries = Vec::new();
    for line in &shaped.lines {
        for glyph in &line.glyphs {
            if !glyph.cluster_flags.cluster_start {
                continue;
            }

            let glyph_range = glyph.source_range;
            if glyph_range.start == glyph_range.end && glyph.cluster_flags.virtual_glyph {
                if glyph_range.start < source_range.start
                    || glyph_range.start > source_range.end
                    || !text.is_char_boundary(glyph_range.start)
                {
                    return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
                }
                continue;
            }
            if glyph_range.start < source_range.start
                || glyph_range.end > source_range.end
                || glyph_range.start >= glyph_range.end
                || !text.is_char_boundary(glyph_range.start)
                || !text.is_char_boundary(glyph_range.end)
            {
                return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
            }

            if !glyph.cluster_flags.soft_break && !glyph.cluster_flags.mandatory_break {
                continue;
            }

            break_boundaries.push((
                glyph_range.start,
                glyph_range.end,
                glyph.cluster_flags.mandatory_break,
            ));
        }
    }

    if break_boundaries.windows(2).any(|boundaries| {
        boundaries[0].1 > boundaries[1].1
            || (boundaries[0].1 == boundaries[1].1 && boundaries[0].0 > boundaries[1].0)
    }) {
        break_boundaries.sort_unstable_by_key(|boundary| (boundary.1, boundary.0));
    }
    let mut chunk_start = source_range.start;
    let mut previous_cluster_end = source_range.start;
    let mut boundary_index = 0;
    while boundary_index < break_boundaries.len() {
        let (cluster_start, chunk_end, mut mandatory_break) = break_boundaries[boundary_index];
        if cluster_start < previous_cluster_end {
            return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
        }
        boundary_index += 1;
        // 同一簇可由多个字形携带断点；只在起止一致时合并标志，拒绝同终点的矛盾范围。
        while let Some(&(next_start, next_end, next_mandatory)) =
            break_boundaries.get(boundary_index)
        {
            if next_end != chunk_end {
                break;
            }
            if next_start != cluster_start {
                return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
            }
            mandatory_break |= next_mandatory;
            boundary_index += 1;
        }
        if chunk_end <= chunk_start {
            return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
        }
        soft_hyphen::push_chunks(text, chunk_start, chunk_end, chunks);
        if mandatory_break {
            if let Some(chunk) = chunks.last_mut() {
                chunk.mandatory_break = true;
            }
        }
        chunk_start = chunk_end;
        previous_cluster_end = chunk_end;
    }

    if chunk_start < source_range.end {
        soft_hyphen::push_chunks(text, chunk_start, source_range.end, chunks);
    }
    TextShapingOutcome::Ready(())
}

/// 测试用词尾规则入口，先经同步测试分块，再调整已生成的候选块。
/// 基础分块的非 Ready 结果仍由测试入口的 `expect` 处理。
#[cfg(test)]
pub(crate) fn word_smart_line_break_chunks<'a>(
    text: &'a str,
    style: &TextStyle,
) -> Vec<LineBreakChunk<'a>> {
    smart::apply_word_smart_rules(text, line_break_chunks(text, style))
}

/// 在基础分块 Ready 时追加词尾标点策略。
/// `text` 与结果块共享输入借用，`style` 和 `provider` 只供基础成形使用。
/// `map` 保留 Deferred/Failed，不对这些结果执行词尾调整；实际行宽选择仍由上层负责。
pub(crate) fn word_smart_line_break_chunks_with_provider<'a, P>(
    text: &'a str,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<Vec<LineBreakChunk<'a>>>
where
    P: TextShapeRunProvider + ?Sized,
{
    line_break_chunks_with_provider(text, style, provider)
        .map(|chunks| smart::apply_word_smart_rules(text, chunks))
}

impl<'a> LineBreakChunk<'a> {
    /// 使用调用方已有的候选宽度，判断是否允许更细的字素折行。
    ///
    /// `candidate_text` 与 `candidate_advance` 的计量基准由调用方配对；本方法不重新测量。
    /// UI 消费方也可能传入整条候选行的修正宽度，不能一概视为文本单独测宽。
    /// 返回值还受本块许可、候选 advance 有限性、阈值及第二个字素存在性约束。
    /// 参见 [glyph_fallback::should_fallback_to_glyph_wrap_with_advance]。
    pub(crate) fn should_fallback_to_glyph_wrap_with_advance(
        &self,
        candidate_text: &str,
        candidate_advance: f32,
        max_width: f32,
    ) -> bool {
        glyph_fallback::should_fallback_to_glyph_wrap_with_advance(
            self.allow_glyph_fallback,
            candidate_text,
            candidate_advance,
            max_width,
        )
    }

    /// 测试用重新测量入口，委派给对应 helper，沿用其结果转布尔值及异常处理。
    /// 该路径不同于生产调用传入已有 advance 的谓词。
    #[cfg(test)]
    pub(crate) fn should_fallback_to_glyph_wrap(
        &self,
        candidate_text: &str,
        max_width: f32,
        style: &TextStyle,
    ) -> bool {
        glyph_fallback::should_fallback_to_glyph_wrap(
            self.allow_glyph_fallback,
            candidate_text,
            max_width,
            style,
        )
    }

    /// 组装借用片段及范围，按当前文本计算一次 glue 折行许可。
    /// 强制断点初始为假，后续簇断点处理或合并可更新标志。
    /// 参数范围由内部调用方准备；本构造器不独立验证端点或执行行宽决策。
    fn new(
        text: &'a str,
        visual_range: TextRange,
        source_range: TextRange,
        break_suffix: Option<DiscretionaryHyphenDecision>,
    ) -> Self {
        Self {
            text,
            visual_range,
            source_range,
            allow_glyph_fallback: allows_glyph_fallback(text),
            mandatory_break: false,
            break_suffix,
        }
    }
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

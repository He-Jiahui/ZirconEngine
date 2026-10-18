use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::font::FontCollectionRevision;
use crate::text::shaping::{
    analyze_bidi_line, BidiInvariantError, BidiLineOrder, TextLayoutOutcome, TextShapeRunProvider,
    TextShapingOutcome,
};
use crate::text::{ShapedGlyphRun, TextRange, TextStyle};

use super::{HorizontalLineFragmentGeometry, MeasuredGlyphCluster, TextLineMetrics};

/// One final physical source slice shaped once for layout metrics, advances, and later artifact
/// projection.
///
/// This owner retains the absolute source range selected by the caller. It does not infer that
/// range from visual text, so synthetic ellipsis and tatweel remain distinct requests.
/// 保存最终物理行的共享塑形结果及派生几何，供布局与字形产物投影复用。
///
/// 源范围沿用调用方的绝对坐标；视觉顺序由当前文本的局部字素范围解析。
/// 本对象拥有几何和顺序，借用受本对象限制；克隆仍共享塑形 Arc。
#[derive(Clone, Debug)]
pub(crate) struct CanonicalPhysicalLineFragment {
    /// 同一塑形结果及其行指标、字素 advance 和簇几何。
    geometry: HorizontalLineFragmentGeometry,
    /// 请求前取得并在 Ready 后核对的集合身份与代次；不是资源租约。
    font_revision: FontCollectionRevision,
    /// 非空文本的局部视觉顺序；空文本保持 None。
    visual_order: Option<BidiLineOrder>,
}

impl CanonicalPhysicalLineFragment {
    /// 借用共享塑形句柄；调用方需独立持有时自行克隆 Arc。
    pub(crate) fn shaped(&self) -> &Arc<ShapedGlyphRun> {
        self.geometry.shaped()
    }

    /// 返回代次投影；跨字体集合匹配应使用完整 revision。
    pub(crate) const fn font_generation(&self) -> u64 {
        self.font_revision.generation()
    }

    /// 复制集合身份与代次，供保留结果的复用校验。
    pub(crate) const fn font_collection_revision(&self) -> FontCollectionRevision {
        self.font_revision
    }

    /// 复制由保留塑形结果派生的行指标。
    pub(crate) const fn metrics(&self) -> TextLineMetrics {
        self.geometry.metrics()
    }

    /// 借用字素 advance 数组，其有效期不超过本对象的借用。
    pub(crate) fn grapheme_advances(&self) -> &[f32] {
        self.geometry.grapheme_advances()
    }

    /// 借用与同一塑形结果配对的簇几何，供后续文本映射使用。
    pub(crate) fn glyph_clusters(&self) -> &[MeasuredGlyphCluster] {
        self.geometry.glyph_clusters()
    }

    /// 借用局部文本的视觉顺序；空文本返回 None。
    pub(crate) fn visual_order(&self) -> Option<&BidiLineOrder> {
        self.visual_order.as_ref()
    }
}

/// 将调用方选定的最终水平物理行请求整理为可保留的片段。
///
/// # 参数与返回
/// `text` 是源片段文本，`source_range` 是调用方坐标系中的绝对 UTF-8 字节范围；
/// 本入口不从视觉文本反推或修正它。`style`、`direction` 与可变 `provider`
/// 用于一次带 kerning 的请求；返回值拥有成功结果，不保留这些输入借用。
///
/// # 时机与约束
/// 用于断行后的最终物理行。仅 Ready 后比较请求前后的完整字体 revision；
/// 变化返回 Deferred，原 Deferred/Failed 直接传播，不保证 provider 副作用回滚。
/// 非空文本按局部 UTF-8 字素范围解析 Bidi，失败或几何投影失败不发布片段。
///
/// # 参考
/// [`TextShapeRunProvider`]、[`CanonicalPhysicalLineFragment`]。
pub(crate) fn shape_horizontal_physical_line_fragment_with_provider<P>(
    text: &str,
    style: &TextStyle,
    direction: TextDirection,
    source_range: TextRange,
    provider: &mut P,
) -> TextLayoutOutcome<CanonicalPhysicalLineFragment>
where
    P: TextShapeRunProvider + ?Sized,
{
    let font_revision = provider.font_collection_revision();
    provider
        .shape_horizontal_range_with_kerning(text, style, direction, source_range, true)
        .and_then(|shaped| {
            if provider.font_collection_revision() != font_revision {
                return TextShapingOutcome::deferred(
                    crate::core::framework::text::TextLayoutError::FontGenerationChanged,
                );
            }
            let visual_order = if text.is_empty() {
                None
            } else {
                match resolve_physical_line_visual_order(text, direction) {
                    Ok(order) => Some(order),
                    Err(_) => {
                        return TextShapingOutcome::failed(
                            crate::core::framework::text::TextLayoutError::BidiInvariant,
                        );
                    }
                }
            };
            HorizontalLineFragmentGeometry::from_shaped(shaped, text, style).map(|geometry| {
                CanonicalPhysicalLineFragment {
                    font_revision,
                    geometry,
                    visual_order,
                }
            })
        })
}

/// 以当前文本的局部 UTF-8 字节范围分字素，返回这些逻辑簇的 Bidi 排序回执。
/// 输入方向交给解析器；返回的排列索引针对本次字素列表，不改写绝对源范围。
/// 参考 [`analyze_bidi_line`]；错误由调用入口统一映射。
fn resolve_physical_line_visual_order(
    text: &str,
    direction: TextDirection,
) -> Result<BidiLineOrder, BidiInvariantError> {
    crate::profile_scope!("runtime", "text.layout", "resolve_physical_line_bidi_order");
    let logical_ranges = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| TextRange {
            start,
            end: start.saturating_add(grapheme.len()),
        })
        .collect::<Vec<_>>();
    analyze_bidi_line(
        text,
        direction,
        TextRange {
            start: 0,
            end: text.len(),
        },
        &logical_ranges,
    )
}

#[cfg(test)]
#[path = "tests/physical_line_fragment.rs"]
mod tests;

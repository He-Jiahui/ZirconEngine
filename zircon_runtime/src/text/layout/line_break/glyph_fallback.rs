//! 折行候选的字素兜底判定。
//! 生产入口消费已有行进距离；测试入口重新测量文本，二者数值边界不同。
//! 判定不执行拆分，后续由折行调用方处理；所有文本与提供者借用仅用于同步调用。

#[cfg(test)]
use super::super::measure::measure_line_width;
#[cfg(test)]
use super::super::measure::measure_line_width_with_provider;
#[cfg(test)]
use crate::text::shaping::DirectTextShapeRunProvider;
#[cfg(test)]
use crate::text::shaping::TextShapeRunProvider;
#[cfg(test)]
use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

/// 测试便捷入口：建立默认提供者，重新测量候选后判定。
/// `allow_glyph_fallback` 为候选许可，`max_width` 为测量比较上限，`style` 决定测量样式；
/// 文本与样式仅在本次调用中借用。非就绪结果转为错误后由现有 `expect` 引发 panic。
#[cfg(test)]
pub(super) fn should_fallback_to_glyph_wrap(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    max_width: f32,
    style: &TextStyle,
) -> bool {
    let mut provider = DirectTextShapeRunProvider::default();
    should_fallback_to_glyph_wrap_with_provider(
        allow_glyph_fallback,
        candidate_text,
        max_width,
        style,
        &mut provider,
    )
    .into_result()
    .expect("test shaping request must be valid")
}

/// 测试用可注入提供者的判定；候选、样式和提供者仅借用于本次测量。
/// 即使 `allow_glyph_fallback` 为 `false` 仍先测量，随后只对就绪宽度做布尔映射；
/// 延后或失败的整形结果原样向上传播，不当作“不需要兜底”。
#[cfg(test)]
pub(super) fn should_fallback_to_glyph_wrap_with_provider<P>(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> crate::text::shaping::TextLayoutOutcome<bool>
where
    P: TextShapeRunProvider + ?Sized,
{
    line_text_fits_with_provider(candidate_text, max_width, style, provider)
        .map(|fits| allow_glyph_fallback && !fits && has_more_than_one_grapheme(candidate_text))
}

/// 使用调用方已有的行进距离，判断是否请求字素兜底折行。
///
/// # 参数
/// - `allow_glyph_fallback`：候选是否允许继续拆分；本函数不重新检查连接字符。
/// - `candidate_text`：仅用来判断是否存在第二个字素分段，借用不离开本次调用。
/// - `candidate_advance`：调用方选取的候选行或块的测量值，本函数不重新整形。
/// - `max_width`：当前行宽限制，按 `max_width.max(0.0)` 参与阈值比较。
///
/// # 返回与约束
/// 许可开启、行进距离有限、严格超过宽度限制加 `0.01` 且存在第二个字素分段时返回 `true`。
/// 这里只返回拆分意向；[`super::LineBreakChunk`] 的调用方负责实际折行与结果传播。
pub(super) fn should_fallback_to_glyph_wrap_with_advance(
    allow_glyph_fallback: bool,
    candidate_text: &str,
    candidate_advance: f32,
    max_width: f32,
) -> bool {
    allow_glyph_fallback
        && candidate_advance.is_finite()
        && candidate_advance > max_width.max(0.0) + 0.01
        && has_more_than_one_grapheme(candidate_text)
}

#[cfg(test)]
fn line_text_fits(text: &str, max_width: f32, style: &TextStyle) -> bool {
    measure_line_width(text, style)
        .into_result()
        .expect("measure test line width")
        <= max_width + 0.01
}

/// 将提供者的测量宽度映射为阈值内判定，保留非就绪结果。
/// 此测试路径直接比较 `max_width + 0.01`，与生产入口的宽度下限处理和有限值条件分开。
#[cfg(test)]
fn line_text_fits_with_provider<P>(
    text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> crate::text::shaping::TextLayoutOutcome<bool>
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_line_width_with_provider(text, style, provider).map(|width| width <= max_width + 0.01)
}

// 只探测第二个字素分段；不是按 UTF-8 字节数或 Unicode 标量值计数。
fn has_more_than_one_grapheme(text: &str) -> bool {
    text.graphemes(true).nth(1).is_some()
}

#[cfg(test)]
#[path = "tests/glyph_fallback.rs"]
mod tests;

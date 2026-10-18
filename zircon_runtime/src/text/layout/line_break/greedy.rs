//! 折行的标量宽度预判与带成形 provider 的候选文本测量。
//!
//! 两个入口分别消费已有 advance 与重新测量结果；异常宽度的阈值规则以各自入口为准。

use super::super::measure::measure_line_width_with_provider;
use crate::text::layout_geometry::finite_sum;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider};
use crate::text::TextStyle;

const LINE_FIT_EPSILON: f32 = 0.01;

/// 根据已有 advance 判断是否应在追加内容前换行，不执行文本测量或修改行。
///
/// # 参数与返回
/// `current_is_empty` 为真时直接返回假，让调用端先接纳首项；其余情况比较当前与下一项的累计宽度。
/// `current_advance`、`next_advance` 的负值和非有限值按零处理；`max_width` 的 NaN 按零，其余值先取非负值。
/// 有限阈值在此基础上加 0.01，累计值严格大于阈值才返回真；正无穷宽度允许有限累计值。
///
/// # 约束与参考
/// 累加溢出通过 [`finite_sum`] 发布有限值，不是精确和或预算准入；调用端仍负责真正的换行和源范围更新。
/// [`line_text_fits_with_provider`] 直接使用测量阈值，对 NaN 和负无穷的处理与本入口不同。
pub(crate) fn should_wrap_before_accumulated(
    current_is_empty: bool,
    current_advance: f32,
    next_advance: f32,
    max_width: f32,
) -> bool {
    if current_is_empty {
        return false;
    }
    let current_advance = finite_non_negative(current_advance);
    let next_advance = finite_non_negative(next_advance);
    let max_width = if max_width.is_nan() {
        0.0
    } else {
        max_width.max(0.0)
    };
    finite_sum([current_advance, next_advance]) > fit_limit(max_width)
}

/// 借用本次候选文本、样式和可变 provider 测量行宽，再判断是否满足阈值。
///
/// # 参数与返回
/// `text`、`style` 只在本次调用中借用；`provider` 允许泛型或 trait object，由测量路径同步使用。
/// Ready 只携带布尔比较结果；Deferred、Failed 原样传递。
/// `max_width` 为 NaN 时阈值是零，正负无穷保持原值；有限负值以零为底再加 0.01。
///
/// # 使用与约束
/// 空文本由测量层返回零宽，因此负无穷阈值仍得到 Ready(false)，不等同于 UI 空行溢出捷径。
/// 此接口不承诺回滚 provider 内部工作、缓存效果或分配次数；返回值不保留上述借用。
/// 参考 [`measure_line_width_with_provider`]；仅需已有 advance 的判断时使用 [`should_wrap_before_accumulated`]。
pub(crate) fn line_text_fits_with_provider<P>(
    text: &str,
    max_width: f32,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<bool>
where
    P: TextShapeRunProvider + ?Sized,
{
    measure_line_width_with_provider(text, style, provider)
        .map(|width| width <= fit_limit(max_width))
}

// 预判路径把非有限 advance 舍为零；这与 finite_sum 的有限溢出恢复承担不同职责。
fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

// 本函数不统一归一化无穷值；累计入口会先处理 max_width，测量入口则直接调用。
fn fit_limit(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else if value.is_infinite() {
        value
    } else {
        finite_sum([value.max(0.0), LINE_FIT_EPSILON])
    }
}

#[cfg(test)]
#[path = "tests/greedy.rs"]
mod tests;

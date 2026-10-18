//! 省略策略对预计算字素宽度的保留数量选择与尾部空白裁剪。
//!
//! 本模块不成形、不插入省略号，也不改写文本或 advance；UI 调用者另行预留标记宽度并重建输出。

use crate::text::layout_geometry::finite_sum;
use crate::text::WordBoundaryMap;

/// 由调用者单独测量并插入的省略标记；不包含在保留数量或宽度判定中。
pub(crate) const ELLIPSIS: &str = "…";

/// 逻辑前后缀选择策略，不决定双向文字的视觉顺序。
///
/// End/EndWord 选择前缀，后者额外查询完整词边界；Start 选择后缀。
/// Middle 从后缀优先交替尝试两端，按单遍停止规则选择，并非全局最优分割。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EllipsisPlacement {
    End,
    EndWord,
    Start,
    Middle,
}

/// 按 `placement` 返回 `(前缀字素数, 后缀字素数)`，供调用者拼接省略后的内容。
///
/// # 参数与返回
/// `text` 与 `graphemes` 应来自同一原文；范围按源字节位置有序且落在合法 UTF-8 边界，`advances` 与字素一一对应。
/// `available` 是调用者为保留文本留下的宽度；此入口不扣省略标记宽度，也不统一验证异常数值。
/// 返回数量不是字节偏移或像素宽度；这里只借用输入，不保存引用或改写文本、宽度。
///
/// # 约束与参考
/// EndWord 使用零拷贝 [`WordBoundaryMap`] 查询完整词前缀；下方 BUG 所述零拟合情形会影响结果。
/// 累加使用 [`finite_sum`] 的有限值恢复规则，不构成始终满足预算的保证；尾部空白由 [`trim_end_ellipsis_trailing_graphemes`] 另行裁剪。
pub(crate) fn retained_grapheme_counts(
    text: &str,
    graphemes: &[(usize, usize)],
    advances: &[f32],
    available: f32,
    placement: EllipsisPlacement,
) -> (usize, usize) {
    match placement {
        EllipsisPlacement::Start => (0, fitting_suffix_count(advances, available)),
        EllipsisPlacement::Middle => middle_retained_grapheme_counts(advances, available),
        EllipsisPlacement::EndWord => {
            let fitted = fitting_prefix_count(advances, available);
            // BUG: [CR-ZCA-R113-0001] text="a"、范围[(0,1)]、advances=[1.0]、available=0 且 EndWord 时，fitted=0，
            // 饱和减法仍使 get 取首项 end=1，随后完整词查询使结果成为 (1,0)，而不是零保留。
            // UI 可产生此可用宽度，且会保留这个非空白前缀的 advance 再加标记；这是静态路径证明，未运行字体或绘制。
            let fitted_end = graphemes
                .get(fitted.saturating_sub(1))
                .map(|(_, end)| *end)
                .unwrap_or_default();
            let word_end = WordBoundaryMap::new(text).completed_prefix_end(fitted_end);
            (graphemes.partition_point(|&(_, end)| end <= word_end), 0)
        }
        EllipsisPlacement::End => (fitting_prefix_count(advances, available), 0),
    }
}

/// 从后缀优先交替保留两端；后缀放不下时尝试当前前缀，前缀放不下即停止。
///
/// 不重排或修改 `advances`，也不搜索所有分割方案；返回量用于调用者的字素边界。
fn middle_retained_grapheme_counts(advances: &[f32], available: f32) -> (usize, usize) {
    let mut prefix_count = 0;
    let mut suffix_count = 0;
    let mut retained_width = 0.0;
    let mut prefer_suffix = true;

    loop {
        let remaining = advances.len().saturating_sub(prefix_count + suffix_count);
        if remaining == 0 {
            break;
        }

        let next_index = if prefer_suffix {
            advances.len() - suffix_count - 1
        } else {
            prefix_count
        };
        let next_width = advances[next_index];
        if finite_sum([retained_width, next_width]) <= available {
            retained_width = finite_sum([retained_width, next_width]);
            if prefer_suffix {
                suffix_count += 1;
            } else {
                prefix_count += 1;
            }
            prefer_suffix = !prefer_suffix;
            continue;
        }

        // 后缀不适可转试当前前缀；前缀不适时不再搜索更远项。
        if !prefer_suffix {
            break;
        }

        let prefix_width = advances[prefix_count];
        if finite_sum([retained_width, prefix_width]) > available {
            break;
        }
        retained_width = finite_sum([retained_width, prefix_width]);
        prefix_count += 1;
        prefer_suffix = false;
    }

    (prefix_count, suffix_count)
}

/// 对 End/EndWord 原地减少前缀计数，移除末尾整字素均为 Unicode whitespace 的部分。
///
/// # 参数、约束与生命周期
/// `prefix_count` 必须不超过 `graphemes.len()`，各范围须能直接切片同一 `text`；此处不执行边界校验。
/// 其它策略保持计数不变；返回单位值，不裁剪文本或 advance，所有借用仅用于本次同步调用。
/// 参考 [`retained_grapheme_counts`]；调用者仍负责按新计数更新自己的输出。
pub(crate) fn trim_end_ellipsis_trailing_graphemes(
    text: &str,
    graphemes: &[(usize, usize)],
    prefix_count: &mut usize,
    placement: EllipsisPlacement,
) {
    if !matches!(
        placement,
        EllipsisPlacement::End | EllipsisPlacement::EndWord
    ) {
        return;
    }
    while *prefix_count > 0 {
        let (start, end) = graphemes[*prefix_count - 1];
        if !text[start..end].chars().all(char::is_whitespace) {
            break;
        }
        *prefix_count -= 1;
    }
}

/// 从头累加到第一个不满足 `available` 的字素为止；相等仍可保留，输入数值不在此归一化。
fn fitting_prefix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .take_while(|advance| {
            let fits = finite_sum([width, **advance]) <= available;
            if fits {
                width = finite_sum([width, **advance]);
            }
            fits
        })
        .count()
}

/// 与前缀入口使用相同累计规则，但从末尾反向消费；返回的是尾部保留数量而非起始下标。
fn fitting_suffix_count(advances: &[f32], available: f32) -> usize {
    let mut width = 0.0;
    advances
        .iter()
        .rev()
        .take_while(|advance| {
            let fits = finite_sum([width, **advance]) <= available;
            if fits {
                width = finite_sum([width, **advance]);
            }
            fits
        })
        .count()
}

#[cfg(test)]
#[path = "tests/overflow.rs"]
mod tests;

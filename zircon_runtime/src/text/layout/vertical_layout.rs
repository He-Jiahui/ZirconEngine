use crate::text::layout_geometry::{finite_f32_or_geometry, finite_geometry};

/// 已测得单列的几何值；右到左位置由 [`layout_vertical_rl_columns`] 生成。
/// 后续 UI 排版仍可替换纵向位置与高度，本结构不执行裁剪或塑形。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct VerticalColumnFrame {
    /// 列框左坐标；列宽与列步进不同时，首列右边未必等于容器右边。
    pub(crate) x: f32,
    /// 本次调用归一后的容器纵向原点。
    pub(crate) y: f32,
    /// 本次调用归一后的列框宽度，不等同于列步进。
    pub(crate) width: f32,
    /// 当前输入项归一后的列高。
    pub(crate) height: f32,
}

/// 本次列排布的自有结果；高度切片不被保留，列框随结果一起存活。
/// UI 可先用空高度切片查询容量，再用已测得列高生成几何；裁剪由调用方完成。
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct VerticalColumnLayout {
    /// 横向容纳列数的估计；正常调用至少为 1，默认值为 0，不限制列框数量。
    pub(crate) column_capacity: usize,
    /// 按输入顺序拥有全部列框；理论左坐标按步进递减，发布时仍可能钳制。
    pub(crate) frames: Vec<VerticalColumnFrame>,
    /// 列数乘列步进的有限发布值，不是列框几何并集的宽度。
    pub(crate) measured_width: f32,
    /// 归一后列高的最大值；空输入为 0。
    pub(crate) measured_height: f32,
}

/// 将已测得列高排成从右向左的列框，供测量后的几何发布使用。
///
/// # 参数
/// `frame_x`、`frame_y` 是容器原点，`frame_width` 参与容量估计和右端点计算。
/// `column_width` 是列框宽度，`column_advance` 是相邻列的横向步进，两者可不同。
/// `column_heights` 仅在本次调用借用；每项产生一个列框，结果拥有独立数组。
///
/// # 返回与约束
/// 容量只供调用方决策，不截断输入；UI 的溢出处理及纵向几何调整另行完成。
/// 非有限坐标归零；宽高负值或非有限值归零，无效步进回退到归一列宽与 1 的较大值。
/// 常规有限的 f32 中间值沿用；溢出时用较宽计算恢复有限发布值，不保证精确目标宽度。
/// 首列左坐标从右端点减一个步进，因此首列右边未必与容器右边重合。
///
/// # 参考
/// 统计字段见 [`VerticalColumnLayout`]，数值发布见 [`finite_f32_or_geometry`]。
pub(crate) fn layout_vertical_rl_columns(
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    column_width: f32,
    column_advance: f32,
    column_heights: &[f32],
) -> VerticalColumnLayout {
    let column_width = finite_non_negative(column_width);
    let column_advance = finite_positive(column_advance).unwrap_or(column_width.max(1.0));
    let frame_width = finite_non_negative(frame_width);
    let column_capacity = (frame_width.max(column_advance) / column_advance)
        .floor()
        .max(1.0) as usize;
    let frame_x = finite_coordinate(frame_x);
    let frame_right_exact = f64::from(frame_x) + f64::from(frame_width);
    let frame_right_candidate = frame_x + frame_width;
    let frame_right_overflowed = !frame_right_candidate.is_finite();
    let frame_right = finite_f32_or_geometry(frame_right_candidate, frame_right_exact);
    let frame_y = finite_coordinate(frame_y);
    // 每项高度只在建框时归一并汇总；容量估计不参与这个遍历。
    let mut frames = Vec::with_capacity(column_heights.len());
    let mut measured_height = 0.0_f32;
    for (index, height) in column_heights.iter().copied().enumerate() {
        let height = finite_non_negative(height);
        measured_height = measured_height.max(height);
        let column_index = (index + 1) as f32;
        let x_candidate = frame_right - column_index * column_advance;
        let x_exact = frame_right_exact - (index + 1) as f64 * f64::from(column_advance);
        frames.push(VerticalColumnFrame {
            // 右端点曾溢出时，所有列都从原始 f64 端点重算，不能混用已钳制端点。
            x: if frame_right_overflowed {
                finite_geometry(x_exact)
            } else {
                finite_f32_or_geometry(x_candidate, x_exact)
            },
            y: frame_y,
            width: column_width,
            height,
        });
    }

    VerticalColumnLayout {
        column_capacity,
        measured_width: finite_f32_or_geometry(
            frames.len() as f32 * column_advance,
            frames.len() as f64 * f64::from(column_advance),
        ),
        measured_height,
        frames,
    }
}

#[cfg(test)]
#[path = "vertical_layout/tests/single_pass_frame_tests.rs"]
mod single_pass_frame_tests;

/// 保留有限坐标（含负数），将非有限坐标恢复为零。
fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

/// 保留非负有限尺寸；负数和非有限值均归零。
fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// 仅对有限且严格为正的输入返回原值，其他输入返回 `None`。
fn finite_positive(value: f32) -> Option<f32> {
    (value.is_finite() && value > 0.0).then_some(value)
}

#[cfg(test)]
#[path = "tests/vertical_layout.rs"]
mod tests;

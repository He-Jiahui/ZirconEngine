//! SDF 文本装饰线的字体度量换算；缺少字体指标时使用可见的兜底尺寸。
use std::collections::HashMap;

use crate::asset::{FontAssetFaceMetrics, FontAssetLineMetrics};
use crate::core::framework::text::TextWritingMode;
use crate::text::FontFaceId;
use crate::text::TextFrame;

use super::FontDatabase;

const FALLBACK_UNDERLINE_POSITION_EM: f32 = -0.1;
const FALLBACK_LINE_THICKNESS_EM: f32 = 0.05;
const FALLBACK_STRIKEOUT_POSITION_EM: f32 = 0.3;
const FALLBACK_ASCENDER_EM: f32 = 0.8;
const MIN_VISIBLE_TEXT_DECORATION_PX: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// 单种装饰线的显示像素度量；位置符号保留到基线坐标换算阶段。
pub(crate) struct TextDecorationLineMetrics {
    pub(crate) position_px: f32,
    pub(crate) thickness_px: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// 主字体的升部与两类装饰线度量，作为后续帧生成的值快照。
pub(crate) struct TextDecorationMetrics {
    pub(crate) ascender_px: f32,
    pub(crate) underline: TextDecorationLineMetrics,
    pub(crate) strikeout: TextDecorationLineMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDecorationKind {
    Underline,
    Strikethrough,
}

/// 由 SDF bake 持有、按 face 与显示字号复用；字体代际变化时 owner 必须丢弃本缓存。
#[derive(Default)]
pub(crate) struct TextDecorationMetricsCache {
    // 身份键不包含数据库或字体代次；失效由持有本缓存的 SDF owner 处理。
    entries: HashMap<(FontFaceId, u32), TextDecorationMetrics>,
}

impl TextDecorationMetrics {
    /// 将字体单位指标换算成指定显示字号下的装饰度量。
    ///
    /// # 参数与返回
    /// `metrics` 是字体指标，`display_px` 是显示字号；字号先归一为至少 `1`，非有限值用 `1`。
    /// 返回升部及装饰线位置、厚度；缺失的线指标使用对应的 em 兜底值。
    ///
    /// # 约束
    /// UPEM 为零时仅升部改用兜底；已有线指标仍按 `max(UPEM, 1)` 缩放。
    /// 即使显示字号有限，后续浮点乘积也不保证有限。
    pub(crate) fn from_font_units(metrics: FontAssetFaceMetrics, display_px: f32) -> Self {
        let display_px = normalized_display_px(display_px);
        let units_per_em = f32::from(metrics.units_per_em.max(1));
        let unit_scale = display_px / units_per_em;
        Self {
            ascender_px: if metrics.units_per_em == 0 {
                display_px * FALLBACK_ASCENDER_EM
            } else {
                f32::from(metrics.ascender) * unit_scale
            },
            underline: scaled_line_metrics(
                metrics.underline,
                unit_scale,
                display_px * FALLBACK_UNDERLINE_POSITION_EM,
                display_px * FALLBACK_LINE_THICKNESS_EM,
            ),
            strikeout: scaled_line_metrics(
                metrics.strikeout,
                unit_scale,
                display_px * FALLBACK_STRIKEOUT_POSITION_EM,
                display_px * FALLBACK_LINE_THICKNESS_EM,
            ),
        }
    }

    /// 从空字体指标取得兜底度量；字号归一规则与 [`Self::from_font_units`] 相同。
    pub(crate) fn fallback(display_px: f32) -> Self {
        Self::from_font_units(FontAssetFaceMetrics::default(), display_px)
    }

    /// 一行混用回退字体时取较粗的装饰线，避免字符跨字体后线条忽粗忽细。
    /// 只增大两类厚度，不改主字体的升部或装饰位置；空迭代器保留原值。
    pub(crate) fn aggregate_fallback_thicknesses(
        mut self,
        fallback_metrics: impl IntoIterator<Item = Self>,
    ) -> Self {
        for metrics in fallback_metrics {
            self.underline.thickness_px = self
                .underline
                .thickness_px
                .max(metrics.underline.thickness_px);
            self.strikeout.thickness_px = self
                .strikeout
                .thickness_px
                .max(metrics.strikeout.thickness_px);
        }
        self
    }

    pub(crate) fn line(self, kind: TextDecorationKind) -> TextDecorationLineMetrics {
        match kind {
            TextDecorationKind::Underline => self.underline,
            TextDecorationKind::Strikethrough => self.strikeout,
        }
    }
}

impl TextDecorationMetricsCache {
    /// 按字体面与归一显示字号复用本缓存中的度量值。
    ///
    /// # 参数与返回
    /// `font_database` 提供 `face` 的指标；`display_px` 归一后以浮点位模式参与键。
    /// 命中时复制返回；查询报错或没有指标时返回并缓存兜底度量。
    ///
    /// # 生命周期与约束
    /// 缓存不持有数据库，键也不区分数据库或字体代次；owner 必须在身份语境改变时失效缓存。
    pub(crate) fn resolve(
        &mut self,
        font_database: &FontDatabase,
        face: FontFaceId,
        display_px: f32,
    ) -> TextDecorationMetrics {
        let display_px = normalized_display_px(display_px);
        let key = (face, display_px.to_bits());
        if let Some(metrics) = self.entries.get(&key) {
            return *metrics;
        }
        let metrics = font_database
            .face_metrics(face)
            .ok()
            .flatten()
            .map(|metrics| TextDecorationMetrics::from_font_units(metrics, display_px))
            .unwrap_or_else(|| TextDecorationMetrics::fallback(display_px));
        self.entries.insert(key, metrics);
        metrics
    }
}

/// 由基线坐标和装饰度量组装一条装饰线的帧。
///
/// # 参数与返回
/// `run_frame` 提供文字行在主轴的跨度，`baseline_coordinate` 是该坐标系内的基线。
/// `writing_mode` 选择横排或竖排轴，`kind` 从 `metrics` 选择位置和厚度。
/// 返回以基线减去指标位置为中心、厚度至少 `1` 的帧；不执行视口裁剪或变换。
/// 输入帧和基线沿用调用方坐标系，本函数不校验所有浮点值或修复负跨度。
pub(crate) fn text_decoration_frame(
    run_frame: TextFrame,
    baseline_coordinate: f32,
    writing_mode: TextWritingMode,
    metrics: TextDecorationMetrics,
    kind: TextDecorationKind,
) -> TextFrame {
    let line = metrics.line(kind);
    let thickness = line.thickness_px.abs().max(MIN_VISIBLE_TEXT_DECORATION_PX);
    let center = baseline_coordinate - line.position_px;
    if matches!(writing_mode, TextWritingMode::VerticalRightToLeft) {
        TextFrame::new(
            center - thickness * 0.5,
            run_frame.y,
            thickness,
            run_frame.height,
        )
    } else {
        TextFrame::new(
            run_frame.x,
            center - thickness * 0.5,
            run_frame.width,
            thickness,
        )
    }
}

// 有指标时保留位置符号并对字体厚度取绝对值；只有 None 使用两项兜底值。
fn scaled_line_metrics(
    metrics: Option<FontAssetLineMetrics>,
    unit_scale: f32,
    fallback_position_px: f32,
    fallback_thickness_px: f32,
) -> TextDecorationLineMetrics {
    metrics.map_or(
        TextDecorationLineMetrics {
            position_px: fallback_position_px,
            thickness_px: fallback_thickness_px,
        },
        |metrics| TextDecorationLineMetrics {
            position_px: f32::from(metrics.position) * unit_scale,
            thickness_px: f32::from(metrics.thickness).abs() * unit_scale,
        },
    )
}

// 归一只约束字号本身，不能据此推导调用方全部几何运算均为有限值。
fn normalized_display_px(display_px: f32) -> f32 {
    if display_px.is_finite() {
        display_px.max(1.0)
    } else {
        1.0
    }
}

#[cfg(test)]
#[path = "decoration_metrics/tests/cases.rs"]
mod tests;

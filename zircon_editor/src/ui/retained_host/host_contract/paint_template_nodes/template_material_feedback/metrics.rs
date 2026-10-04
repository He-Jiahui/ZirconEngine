//! 将宿主密度指标投影为反馈控件尺寸约定；不确定环仅使用固定示意进度，当前不驱动动画。

use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

const CIRCULAR_INDETERMINATE_PERCENT: f32 = 0.58;

// 与线形/环形呈现共用的快照投影；半径的最终帧边界仍由绘制几何层限制。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct MaterialFeedbackMetrics {
    pub linear_radius_floor: f32,
    pub circular_indeterminate_percent: f32,
}

pub(super) fn material_feedback_metrics() -> MaterialFeedbackMetrics {
    material_feedback_metrics_from_host(current_host_metrics())
}

pub(super) fn material_feedback_metrics_from_host(
    metrics: HostControlMetrics,
) -> MaterialFeedbackMetrics {
    MaterialFeedbackMetrics {
        linear_radius_floor: (metrics.border_width * 2.0).max(1.0),
        circular_indeterminate_percent: CIRCULAR_INDETERMINATE_PERCENT,
    }
}

pub(super) fn linear_progress_radius(
    authored_radius: f32,
    track_height: f32,
    metrics: MaterialFeedbackMetrics,
) -> f32 {
    authored_radius
        .max((track_height * 0.5).min(metrics.linear_radius_floor))
        .max(0.0)
}

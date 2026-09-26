use serde::{Deserialize, Serialize};

/// 绘制规划阶段的统计，供可视化和批处理诊断使用；它不读取 RHI 提交计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchStats {
    pub element_count: usize,
    pub batch_count: usize,
    // BUG: [CR-SURFACE-0002] Editor 的 Render Visualizer 将该估计值直接标作 draw calls，
    // 与其 Render 区域的 estimated draw calls 用词及 RHI 实测计数不一致。
    /// 由规划批次数推算的预计调用数，不等于 GPU 实际提交的绘制数。
    pub draw_call_count: usize,
}

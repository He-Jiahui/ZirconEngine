/// 场景中可见发射器汇总的 GPU 粒子帧统计与间接绘制参数。
/// 当前渲染侧也把它用于诊断展示；它是提取结果而非 GPU 同步凭据。
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RenderParticleGpuFrameExtract {
    pub alive_count: u32,
    pub spawned_total: u32,
    pub per_emitter_spawned: Vec<u32>,
    pub indirect_draw_args: [u32; 4],
}

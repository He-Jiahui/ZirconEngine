use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsTimer, OffscreenTarget, ProductDiagnosticQueryFrameScope,
    ProductDiagnosticReadbackFrameScope,
};
use crate::graphics::scene::scene_renderer::graph_execution::FrameCommandEncoderSet;
use crate::graphics::types::GraphicsError;

/// 汇集图执行后才可确定的尾部工作；所有命令缓冲仍须由帧提交事务统一提交。
pub(super) struct TerminalFramePacketContext<'frame, 'diagnostic, 'timer> {
    pub(super) device: &'frame wgpu::Device,
    pub(super) target: &'frame OffscreenTarget,
    pub(super) command_encoders: FrameCommandEncoderSet,
    pub(super) history_initialization_command_buffer: Option<wgpu::CommandBuffer>,
    pub(super) viewport_product_copy: Option<&'frame zr_rhi_wgpu::WgpuUiExternalImageCopyTarget>,
    pub(super) product_diagnostic_frame_scope:
        Option<ProductDiagnosticReadbackFrameScope<'diagnostic>>,
    pub(super) product_diagnostic_query_scope:
        Option<ProductDiagnosticQueryFrameScope<'diagnostic>>,
    pub(super) gpu_pass_timer: Option<&'timer mut GpuPassTimer>,
    pub(super) gpu_pipeline_statistics_timer: Option<&'timer mut GpuPipelineStatisticsTimer>,
    pub(super) timer_frame_generation: u64,
}

pub(super) struct PreparedTerminalFramePacket {
    pub(super) command_buffers: Vec<wgpu::CommandBuffer>,
    pub(super) product_diagnostic_frame: Option<zr_rhi_wgpu::WgpuNativeDiagnosticReadbackFrame>,
    pub(super) product_diagnostic_query_frame: Option<zr_rhi_wgpu::WgpuNativeDiagnosticQueryFrame>,
}

/// 在 submit 前封闭命令流：历史初始化先行，图命令保持拓扑顺序，诊断复制随后跟随场景输出。
/// 此函数只准备 packet，不取得 queue 提交权。
pub(super) fn prepare_terminal_frame_packet(
    mut context: TerminalFramePacketContext<'_, '_, '_>,
) -> Result<PreparedTerminalFramePacket, GraphicsError> {
    if let Some(viewport_product_copy) = context.viewport_product_copy {
        viewport_product_copy.encode_copy(
            context.command_encoders.serial_encoder(context.device),
            &context.target.final_color,
        );
    }
    let product_diagnostic_frame = match context.product_diagnostic_frame_scope.take() {
        Some(scope) => match scope.prepare(
            "product-diagnostic-readback",
            context.command_encoders.serial_encoder(context.device),
        ) {
            Ok(frame) => frame,
            Err(error) => {
                defer_gpu_timers(&mut context);
                return Err(error);
            }
        },
        None => None,
    };
    // TODO: [CR-GRAPHICS-SCENECORE-0001] 确认 query 计划登记或 readback 准备失败是否需要上报诊断状态；当前 Err 被丢弃，调用方仍提交场景 packet，缺少失败可见性的契约测试。
    let product_diagnostic_query_frame =
        context
            .product_diagnostic_query_scope
            .take()
            .and_then(|scope| {
                scope
                    .finish_and_prepare(
                        context.command_encoders.serial_encoder(context.device),
                        context.gpu_pass_timer.as_deref_mut(),
                        context.gpu_pipeline_statistics_timer.as_deref_mut(),
                    )
                    .ok()
                    .flatten()
            });

    let mut command_buffers = context.command_encoders.finish();
    if let Some(history_initialization) = context.history_initialization_command_buffer {
        command_buffers.insert(0, history_initialization);
    }

    Ok(PreparedTerminalFramePacket {
        command_buffers,
        product_diagnostic_frame,
        product_diagnostic_query_frame,
    })
}

fn defer_gpu_timers(context: &mut TerminalFramePacketContext<'_, '_, '_>) {
    if let Some(timer) = context.gpu_pass_timer.as_deref_mut() {
        timer.defer_frame(context.timer_frame_generation);
    }
    if let Some(timer) = context.gpu_pipeline_statistics_timer.as_deref_mut() {
        timer.finish_product_frame();
    }
}

#[cfg(test)]
#[path = "tests/terminal_frame_packet.rs"]
mod tests;

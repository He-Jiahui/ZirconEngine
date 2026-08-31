use crate::graphics::backend::{
    GpuPassTimer, GpuPipelineStatisticsTimer, OffscreenTarget, ProductDiagnosticQueryFrameScope,
    ProductDiagnosticReadbackFrameScope,
};
use crate::graphics::scene::scene_renderer::graph_execution::FrameCommandEncoderSet;
use crate::graphics::types::GraphicsError;

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
mod tests {
    #[test]
    fn terminal_tail_defers_surface_blit_to_the_compiled_graph() {
        let source = include_str!("terminal_frame_packet.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("terminal packet test boundary");
        let product = source
            .find("viewport_product_copy.encode_copy(")
            .expect("retained product copy");
        let diagnostic = source.find("scope.prepare(").expect("copy diagnostic tail");
        let finish = source
            .find("context.command_encoders.finish()")
            .expect("terminal packet finish");
        let history_initialization = source
            .find("command_buffers.insert(0, history_initialization)")
            .expect("history initialization must lead the scene packet");

        assert!(product < diagnostic);
        assert!(diagnostic < finish);
        assert!(finish < history_initialization);
        assert!(source.contains("history_initialization_command_buffer"));
        assert!(!source.contains("submit_graphics_command_buffers("));
        assert!(!source.contains("record_frame_target_blit("));
        assert!(!source.contains("encode_output_target_writeback("));
        assert!(!source.contains("skip_output_target_writeback_after_direct_import("));
        assert!(!source.contains("suppress_output_target_writeback("));
        assert!(!source.contains("queue.submit("));
    }
}

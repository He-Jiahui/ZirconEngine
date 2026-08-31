use zr_rhi::{
    CommandList, CompareFunction, DepthStencilStateDesc, PipelineDesc, PipelineHandle,
    PipelineKind, PipelineLayoutDesc, PipelineLayoutHandle, RasterPipelineStateDesc,
    RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc, RenderPassStoreOp, RhiError,
    ShaderModuleDesc, ShaderModuleHandle, ShaderStage, TextureFormat, TextureHandle,
};

const TRIANGLE_VERTEX_SHADER: &str = "@vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {\n  let positions = array<vec2<f32>, 3>(vec2<f32>(0.0, 0.75), vec2<f32>(-0.75, -0.75), vec2<f32>(0.75, -0.75));\n  return vec4<f32>(positions[index], 0.0, 1.0);\n}";
const TRIANGLE_FRAGMENT_SHADER: &str =
    "@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.1, 0.8, 0.3, 1.0); }";

/// Reusable neutral pipeline state for the smallest raster product frame.
///
/// The owner carries no native WGPU references. Both offscreen and acquired-surface frames record
/// this state through `RenderDevice`, so they share the same pipeline and submission semantics.
pub(super) struct MvpTrianglePipeline {
    layout: PipelineLayoutHandle,
    vertex_shader: ShaderModuleHandle,
    fragment_shader: ShaderModuleHandle,
    pipeline: PipelineHandle,
}

impl MvpTrianglePipeline {
    pub(super) fn new(
        device: &dyn RenderDevice,
        color_format: TextureFormat,
    ) -> Result<Self, RhiError> {
        let layout = device.create_pipeline_layout(&PipelineLayoutDesc::new(
            "zircon-mvp-triangle-layout",
            Vec::new(),
        ))?;
        let vertex_shader = match device.create_shader_module(&ShaderModuleDesc::new(
            "zircon-mvp-triangle-vertex",
            ShaderStage::Vertex,
            "vs_main",
            TRIANGLE_VERTEX_SHADER,
        )) {
            Ok(handle) => handle,
            Err(error) => {
                let _ = device.destroy_pipeline_layout(layout);
                return Err(error);
            }
        };
        let fragment_shader = match device.create_shader_module(&ShaderModuleDesc::new(
            "zircon-mvp-triangle-fragment",
            ShaderStage::Fragment,
            "fs_main",
            TRIANGLE_FRAGMENT_SHADER,
        )) {
            Ok(handle) => handle,
            Err(error) => {
                let _ = device.destroy_shader_module(vertex_shader);
                let _ = device.destroy_pipeline_layout(layout);
                return Err(error);
            }
        };
        let pipeline = match device.create_pipeline(
            &PipelineDesc::new("zircon-mvp-triangle", PipelineKind::Raster)
                .with_layout(layout)
                .with_vertex_shader(vertex_shader)
                .with_fragment_shader(fragment_shader)
                .with_raster_state(
                    RasterPipelineStateDesc::single_color(color_format).with_depth_stencil(
                        DepthStencilStateDesc::new(
                            TextureFormat::Depth24Plus,
                            true,
                            CompareFunction::LessEqual,
                        ),
                    ),
                ),
        ) {
            Ok(handle) => handle,
            Err(error) => {
                let _ = device.destroy_shader_module(fragment_shader);
                let _ = device.destroy_shader_module(vertex_shader);
                let _ = device.destroy_pipeline_layout(layout);
                return Err(error);
            }
        };

        Ok(Self {
            layout,
            vertex_shader,
            fragment_shader,
            pipeline,
        })
    }

    pub(super) fn record_draw(
        &self,
        command_list: &mut dyn CommandList,
        color_attachment: RenderPassColorAttachmentDesc,
        depth_target: TextureHandle,
    ) {
        command_list.push_debug_group("zircon-mvp-triangle-frame");
        command_list.begin_render_pass(
            "zircon-mvp-triangle-pass",
            vec![color_attachment],
            Some(RenderPassDepthStencilAttachmentDesc::depth(
                depth_target,
                RenderPassDepthLoadOp::Clear(1.0),
                RenderPassStoreOp::Discard,
            )),
        );
        command_list.push_debug_marker("zircon-mvp-triangle-draw");
        command_list.set_pipeline(self.pipeline);
        command_list.draw(0, 3, 0, 1);
        command_list.end_render_pass();
        command_list.pop_debug_group();
    }

    pub(super) fn color_attachment(target: TextureHandle) -> RenderPassColorAttachmentDesc {
        RenderPassColorAttachmentDesc::new(
            target,
            RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
            RenderPassStoreOp::Store,
        )
    }

    pub(super) fn destroy(self, device: &dyn RenderDevice) -> Result<(), RhiError> {
        device.destroy_pipeline(self.pipeline)?;
        device.destroy_shader_module(self.fragment_shader)?;
        device.destroy_shader_module(self.vertex_shader)?;
        device.destroy_pipeline_layout(self.layout)
    }
}

use std::sync::Arc;

use crate::core::framework::render::RenderViewportSurfaceDescriptor;
use crate::core::math::UVec2;
use crate::graphics::types::GraphicsError;
use crate::rhi::{
    PresentMode, RenderDevice, RenderSurfaceDescriptor, SubmissionTicket, SurfaceAcquireOutcome,
    SurfaceSessionCreateOutcome, SurfaceSessionReceipt, SwapchainDesc, TextureFormat,
};
use zr_rhi_wgpu::{WgpuNativeSurfaceFrameTarget, WgpuRenderDevice};

use super::render_backend::RenderBackend;

const PRESENT_BLIT_SHADER: &str = r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(2.0, 1.0),
        vec2<f32>(0.0, -1.0)
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = uvs[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, input.uv);
}
"#;

pub(crate) struct ViewportSurface {
    render_device: Arc<WgpuRenderDevice>,
    session: SurfaceSessionReceipt,
    blit: SurfaceBlitResources,
}

pub(crate) struct ViewportSurfacePresentFailure {
    source: GraphicsError,
    submission: Option<SubmissionTicket>,
}

impl ViewportSurfacePresentFailure {
    pub(crate) fn before_submission(source: impl Into<GraphicsError>) -> Self {
        Self {
            source: source.into(),
            submission: None,
        }
    }

    pub(crate) fn after_submission(
        source: impl Into<GraphicsError>,
        submission: SubmissionTicket,
    ) -> Self {
        Self {
            source: source.into(),
            submission: Some(submission),
        }
    }

    pub(crate) fn into_parts(self) -> (GraphicsError, Option<SubmissionTicket>) {
        (self.source, self.submission)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ViewportSurfacePresentOutcome {
    Presented(SubmissionTicket),
    Reconfigured,
    DeferredTimeout,
    DeferredOccluded,
}

pub(crate) enum ViewportSurfaceFrameAcquire {
    Acquired(WgpuNativeSurfaceFrameTarget),
    NoSubmit(ViewportSurfacePresentOutcome),
}

impl ViewportSurfacePresentOutcome {
    pub(crate) const fn submission_ticket(self) -> Option<SubmissionTicket> {
        match self {
            Self::Presented(ticket) => Some(ticket),
            Self::Reconfigured | Self::DeferredTimeout | Self::DeferredOccluded => None,
        }
    }
}

impl ViewportSurface {
    pub(crate) fn size(&self) -> UVec2 {
        UVec2::new(self.session.swapchain.width, self.session.swapchain.height)
    }

    pub(crate) fn acquire_frame_target(
        &mut self,
    ) -> Result<ViewportSurfaceFrameAcquire, ViewportSurfacePresentFailure> {
        let frame = match self
            .render_device
            .acquire_surface_frame(self.session.session())
            .map_err(ViewportSurfacePresentFailure::before_submission)?
        {
            SurfaceAcquireOutcome::Acquired(frame) => frame,
            SurfaceAcquireOutcome::Retryable { reason, .. } => {
                let outcome = match reason {
                    crate::rhi::SurfaceRetryReason::Timeout => {
                        ViewportSurfacePresentOutcome::DeferredTimeout
                    }
                    crate::rhi::SurfaceRetryReason::Occluded => {
                        ViewportSurfacePresentOutcome::DeferredOccluded
                    }
                };
                return Ok(ViewportSurfaceFrameAcquire::NoSubmit(outcome));
            }
            SurfaceAcquireOutcome::ReconfigureRequired { .. } => {
                self.reconfigure_session()?;
                return Ok(ViewportSurfaceFrameAcquire::NoSubmit(
                    ViewportSurfacePresentOutcome::Reconfigured,
                ));
            }
            SurfaceAcquireOutcome::NonRenderable { .. } => {
                return Ok(ViewportSurfaceFrameAcquire::NoSubmit(
                    ViewportSurfacePresentOutcome::Reconfigured,
                ));
            }
        };
        self.render_device
            .prepare_native_surface_frame_target(frame)
            .map(ViewportSurfaceFrameAcquire::Acquired)
            .map_err(ViewportSurfacePresentFailure::before_submission)
    }

    pub(crate) fn record_frame_target_blit(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        target: &WgpuNativeSurfaceFrameTarget,
    ) -> Result<(), GraphicsError> {
        target.record(
            self.render_device.as_ref(),
            encoder,
            |device, target_view, encoder| {
                self.blit.record(
                    device,
                    encoder,
                    source_view,
                    target_view,
                    self.size(),
                    self.session.swapchain.format,
                )
            },
        )
    }

    pub(crate) fn present_frame_target(
        &self,
        mut target: WgpuNativeSurfaceFrameTarget,
        submission: SubmissionTicket,
    ) -> Result<ViewportSurfacePresentOutcome, ViewportSurfacePresentFailure> {
        match target.present(submission) {
            Ok(_) => Ok(ViewportSurfacePresentOutcome::Presented(submission)),
            Err(source) => {
                let source = GraphicsError::from(source);
                let source = match target.discard() {
                    Ok(()) => source,
                    Err(cleanup) => GraphicsError::SurfaceFrameCleanupFailed {
                        cleanup: cleanup.to_string(),
                        source: Box::new(source),
                    },
                };
                Err(ViewportSurfacePresentFailure::after_submission(
                    source, submission,
                ))
            }
        }
    }

    pub(crate) fn discard_frame_target(
        &self,
        target: WgpuNativeSurfaceFrameTarget,
        source: GraphicsError,
    ) -> GraphicsError {
        match target.discard() {
            Ok(()) => source,
            Err(cleanup) => GraphicsError::SurfaceFrameCleanupFailed {
                cleanup: cleanup.to_string(),
                source: Box::new(source),
            },
        }
    }

    fn reconfigure_session(&mut self) -> Result<(), ViewportSurfacePresentFailure> {
        let outcome = self
            .render_device
            .reconfigure_surface_session(self.session.session(), &self.session.swapchain)
            .map_err(ViewportSurfacePresentFailure::before_submission)?;
        self.session = surface_session_receipt(outcome);
        Ok(())
    }
}

impl Drop for ViewportSurface {
    fn drop(&mut self) {
        let _ = self
            .render_device
            .destroy_surface_session(self.session.session());
    }
}

impl RenderBackend {
    pub(crate) fn create_viewport_surface(
        &self,
        descriptor: RenderViewportSurfaceDescriptor,
    ) -> Result<ViewportSurface, GraphicsError> {
        let size = clamp_surface_size(descriptor.size);
        let surface_descriptor = RenderSurfaceDescriptor::new(
            "zircon-viewport-surface",
            descriptor.target,
            SwapchainDesc {
                width: size.x,
                height: size.y,
                present_mode: PresentMode::Fifo,
                format: TextureFormat::Bgra8UnormSrgb,
            },
        );
        let session = surface_session_receipt(
            self.render_device
                .create_surface_session(&surface_descriptor)?,
        );
        let blit = SurfaceBlitResources::new(&self.device)?;
        Ok(ViewportSurface {
            render_device: Arc::clone(&self.render_device),
            session,
            blit,
        })
    }
}

struct SurfaceBlitResources {
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bgra_pipeline: wgpu::RenderPipeline,
    rgba_pipeline: wgpu::RenderPipeline,
}

impl SurfaceBlitResources {
    fn new(device: &wgpu::Device) -> Result<Self, GraphicsError> {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-present-blit-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-present-blit-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-present-blit-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-present-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(PRESENT_BLIT_SHADER.into()),
        });
        let create_pipeline = |target_format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("zircon-present-blit-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let bgra_pipeline = create_pipeline(wgpu_surface_format(TextureFormat::Bgra8UnormSrgb)?);
        let rgba_pipeline = create_pipeline(wgpu_surface_format(TextureFormat::Rgba8UnormSrgb)?);

        Ok(Self {
            sampler,
            bind_group_layout,
            bgra_pipeline,
            rgba_pipeline,
        })
    }

    fn record(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source_view: &wgpu::TextureView,
        target_view: &wgpu::TextureView,
        size: UVec2,
        target_format: TextureFormat,
    ) -> Result<(), GraphicsError> {
        let pipeline = match target_format {
            TextureFormat::Bgra8UnormSrgb => &self.bgra_pipeline,
            TextureFormat::Rgba8UnormSrgb => &self.rgba_pipeline,
            _ => {
                return Err(GraphicsError::SurfaceStatus(
                    "neutral viewport surface negotiated a non-SDR format",
                ));
            }
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-present-blit-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("zircon-present-blit-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_viewport(0.0, 0.0, size.x as f32, size.y as f32, 0.0, 1.0);
            pass.draw(0..3, 0..1);
        }
        Ok(())
    }
}

fn surface_session_receipt(outcome: SurfaceSessionCreateOutcome) -> SurfaceSessionReceipt {
    match outcome {
        SurfaceSessionCreateOutcome::Renderable(receipt)
        | SurfaceSessionCreateOutcome::NonRenderable(receipt) => receipt,
    }
}

fn clamp_surface_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}

fn wgpu_surface_format(format: TextureFormat) -> Result<wgpu::TextureFormat, GraphicsError> {
    match format {
        TextureFormat::Bgra8UnormSrgb => Ok(wgpu::TextureFormat::Bgra8UnormSrgb),
        TextureFormat::Rgba8UnormSrgb => Ok(wgpu::TextureFormat::Rgba8UnormSrgb),
        _ => Err(GraphicsError::SurfaceStatus(
            "neutral viewport surface negotiated a non-SDR format",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{ViewportSurfacePresentOutcome, clamp_surface_size, wgpu_surface_format};
    use crate::core::math::UVec2;
    use crate::rhi::{
        DeviceGeneration, DeviceId, RenderQueueClass, SubmissionTicket, TextureFormat,
    };

    #[test]
    fn graphics_surface_backend_clamps_zero_descriptor_size() {
        assert_eq!(clamp_surface_size(UVec2::new(0, 0)), UVec2::new(1, 1));
        assert_eq!(clamp_surface_size(UVec2::new(640, 0)), UVec2::new(640, 1));
        assert_eq!(clamp_surface_size(UVec2::new(0, 480)), UVec2::new(1, 480));
    }

    #[test]
    fn graphics_surface_backend_accepts_only_neutral_srgb_formats() {
        assert_eq!(
            wgpu_surface_format(TextureFormat::Bgra8UnormSrgb).unwrap(),
            wgpu::TextureFormat::Bgra8UnormSrgb
        );
        assert_eq!(
            wgpu_surface_format(TextureFormat::Rgba8UnormSrgb).unwrap(),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
        assert!(wgpu_surface_format(TextureFormat::Rgba16Float).is_err());
    }

    #[test]
    fn graphics_surface_backend_uses_the_neutral_surface_transaction_owner() {
        let source = include_str!("viewport_surface.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("surface source must retain its test-module boundary");

        assert!(production.contains(".create_surface_session(&surface_descriptor)"));
        assert!(production.contains("SurfaceBlitResources::new(&self.device)?"));
        assert!(!production.contains("BGRA8 sRGB is part of the neutral surface contract"));
        assert!(!production.contains("RGBA8 sRGB is part of the neutral surface contract"));
        assert!(production.contains(".acquire_surface_frame(self.session.session())"));
        assert!(production.contains("ViewportSurfacePresentOutcome::Presented(submission)"));
        assert!(!production.contains("queue.submit("));
        assert!(!production.contains("get_current_texture"));
        assert!(production.contains("prepare_native_surface_frame_target(frame)"));
        assert!(production.contains("target.record("));
        assert!(production.contains("self.render_device.as_ref(),"));
        assert!(production.contains("target.present(submission)"));
        assert!(production.contains("match target.discard()"));
        assert!(production.contains("fn discard_frame_target("));
        assert!(production.contains("GraphicsError::SurfaceFrameCleanupFailed"));
        assert!(!production.contains("present_texture("));
        assert!(!production.contains("submit_native_surface_recording_packet"));
        assert!(!production.contains("surface.configure"));
        assert!(!production.contains("surface_texture.present"));
    }

    #[test]
    fn only_presented_surface_outcomes_expose_a_submission_ticket() {
        let ticket = SubmissionTicket::new(
            DeviceId::new(3),
            DeviceGeneration::new(2),
            RenderQueueClass::Graphics,
            41,
        );

        assert_eq!(
            ViewportSurfacePresentOutcome::Presented(ticket).submission_ticket(),
            Some(ticket)
        );
        assert_eq!(
            ViewportSurfacePresentOutcome::Reconfigured.submission_ticket(),
            None
        );
        assert_eq!(
            ViewportSurfacePresentOutcome::DeferredTimeout.submission_ticket(),
            None
        );
        assert_eq!(
            ViewportSurfacePresentOutcome::DeferredOccluded.submission_ticket(),
            None
        );
    }
}

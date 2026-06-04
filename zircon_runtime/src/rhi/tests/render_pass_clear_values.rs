use crate::rhi::{
    RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc, RenderPassColorLoadOp,
    RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp,
    RenderPassStoreOp, RenderQueueClass, RhiError, TextureDesc, TextureFormat, TextureHandle,
    TextureUsage,
};
use crate::rhi_wgpu::WgpuRenderDevice;

fn create_render_attachment(
    device: &WgpuRenderDevice,
    label: &str,
    format: TextureFormat,
) -> TextureHandle {
    device
        .create_texture(&TextureDesc::new(
            label,
            32,
            32,
            format,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap()
}

#[test]
fn command_list_records_clear_values_for_color_depth_and_stencil() {
    let device = WgpuRenderDevice::new_headless();
    let color = create_render_attachment(&device, "clear-color", TextureFormat::Rgba8UnormSrgb);
    let depth_stencil =
        create_render_attachment(&device, "clear-depth", TextureFormat::Depth24PlusStencil8);
    let color_attachment = RenderPassColorAttachmentDesc::new(
        color,
        RenderPassColorLoadOp::Clear(RenderClearColor::new(0.2, 0.3, 0.4, 0.5)),
        RenderPassStoreOp::Store,
    );
    let depth_attachment = RenderPassDepthStencilAttachmentDesc::depth(
        depth_stencil,
        RenderPassDepthLoadOp::Clear(0.25),
        RenderPassStoreOp::Store,
    )
    .with_stencil(
        RenderPassStencilLoadOp::Clear(7),
        RenderPassStoreOp::Discard,
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "clear-values")
        .unwrap();
    command_list.begin_render_pass(
        "clear-values",
        vec![color_attachment],
        Some(depth_attachment),
    );
    command_list.end_render_pass();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            crate::rhi::CommandListCommand::BeginRenderPass {
                label: "clear-values".to_string(),
                color_attachments: vec![color_attachment],
                depth_stencil_attachment: Some(depth_attachment),
            },
            crate::rhi::CommandListCommand::EndRenderPass,
        ]
    );
    assert!(device
        .is_fence_complete(device.submit(command_list).unwrap())
        .unwrap());
}

#[test]
fn command_list_render_pass_submit_validates_color_clear_values_are_finite() {
    let device = WgpuRenderDevice::new_headless();
    let color = create_render_attachment(&device, "nan-clear-color", TextureFormat::Rgba8UnormSrgb);
    let color_attachment = RenderPassColorAttachmentDesc::new(
        color,
        RenderPassColorLoadOp::Clear(RenderClearColor::new(0.0, f32::NAN, 0.0, 1.0)),
        RenderPassStoreOp::Store,
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "nan-clear-color")
        .unwrap();
    command_list.begin_render_pass("nan-clear-color", vec![color_attachment], None);

    assert_eq!(
        device.submit(command_list).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "color attachment 0 clear color values must be finite".to_string(),
        }
    );
}

#[test]
fn command_list_render_pass_submit_validates_depth_clear_range() {
    let device = WgpuRenderDevice::new_headless();
    let depth = create_render_attachment(&device, "bad-depth-clear", TextureFormat::Depth24Plus);
    let depth_attachment = RenderPassDepthStencilAttachmentDesc::depth(
        depth,
        RenderPassDepthLoadOp::Clear(1.25),
        RenderPassStoreOp::Store,
    );

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "bad-depth-clear")
        .unwrap();
    command_list.begin_render_pass("bad-depth-clear", Vec::new(), Some(depth_attachment));

    assert_eq!(
        device.submit(command_list).unwrap_err(),
        RhiError::InvalidRenderPass {
            reason: "depth clear value must stay within 0.0..=1.0".to_string(),
        }
    );
}

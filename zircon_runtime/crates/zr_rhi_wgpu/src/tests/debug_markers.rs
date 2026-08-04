use crate::DeterministicRhiContractDevice;
use zr_rhi::{
    CommandList, CommandListCommand, RenderClearColor, RenderDevice, RenderPassColorAttachmentDesc,
    RenderPassColorLoadOp, RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc,
    RenderPassStoreOp, RenderQueueClass, RhiError, TextureDesc, TextureFormat, TextureHandle,
    TextureUsage,
};

fn create_render_attachment(
    device: &DeterministicRhiContractDevice,
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

fn color_attachment(texture: TextureHandle) -> RenderPassColorAttachmentDesc {
    RenderPassColorAttachmentDesc::new(
        texture,
        RenderPassColorLoadOp::Clear(RenderClearColor::BLACK),
        RenderPassStoreOp::Store,
    )
}

fn depth_attachment(texture: TextureHandle) -> RenderPassDepthStencilAttachmentDesc {
    RenderPassDepthStencilAttachmentDesc::depth(
        texture,
        RenderPassDepthLoadOp::Clear(1.0),
        RenderPassStoreOp::Store,
    )
}

fn begin_default_render_pass(
    command_list: &mut dyn CommandList,
    color: TextureHandle,
    depth: TextureHandle,
) {
    command_list.begin_render_pass(
        "debug-marker-pass",
        vec![color_attachment(color)],
        Some(depth_attachment(depth)),
    );
}

#[test]
fn command_list_records_debug_markers_and_groups() {
    let device = DeterministicRhiContractDevice::new_headless();

    let mut command_list = device
        .create_command_list(RenderQueueClass::Graphics, "debug-groups")
        .unwrap();
    command_list.push_debug_marker("frame-begin");
    command_list.push_debug_group("render-graph-pass");
    command_list.push_debug_marker("inside-pass");
    command_list.pop_debug_group();

    assert_eq!(
        command_list.recorded_commands(),
        &[
            CommandListCommand::DebugMarker {
                label: "frame-begin".to_string(),
            },
            CommandListCommand::PushDebugGroup {
                label: "render-graph-pass".to_string(),
            },
            CommandListCommand::DebugMarker {
                label: "inside-pass".to_string(),
            },
            CommandListCommand::PopDebugGroup,
        ]
    );
    assert!(device
        .is_fence_complete(device.submit(command_list).unwrap())
        .unwrap());
}

#[test]
fn command_list_submit_validates_debug_marker_labels_and_group_lifetime() {
    let device = DeterministicRhiContractDevice::new_headless();

    let mut empty_marker = device
        .create_command_list(RenderQueueClass::Graphics, "empty-debug-marker")
        .unwrap();
    empty_marker.push_debug_marker("");
    assert_eq!(
        device.submit(empty_marker).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason: "debug marker label must not be empty".to_string(),
        }
    );

    let mut empty_group = device
        .create_command_list(RenderQueueClass::Graphics, "empty-debug-group")
        .unwrap();
    empty_group.push_debug_group("");
    assert_eq!(
        device.submit(empty_group).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason: "debug group label must not be empty".to_string(),
        }
    );

    let mut stray_pop = device
        .create_command_list(RenderQueueClass::Graphics, "stray-debug-pop")
        .unwrap();
    stray_pop.pop_debug_group();
    assert_eq!(
        device.submit(stray_pop).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason: "pop_debug_group requires an active debug group".to_string(),
        }
    );

    let mut unclosed_group = device
        .create_command_list(RenderQueueClass::Graphics, "unclosed-debug-group")
        .unwrap();
    unclosed_group.push_debug_group("frame");
    assert_eq!(
        device.submit(unclosed_group).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason: "command list ended with an active debug group".to_string(),
        }
    );
}

#[test]
fn command_list_submit_validates_render_pass_debug_group_scope() {
    let device = DeterministicRhiContractDevice::new_headless();
    let color =
        create_render_attachment(&device, "debug-scope-color", TextureFormat::Rgba8UnormSrgb);
    let depth = create_render_attachment(&device, "debug-scope-depth", TextureFormat::Depth24Plus);

    let mut valid = device
        .create_command_list(RenderQueueClass::Graphics, "render-pass-debug-group")
        .unwrap();
    valid.push_debug_group("frame");
    begin_default_render_pass(&mut *valid, color, depth);
    valid.push_debug_group("pass");
    valid.push_debug_marker("draw-setup");
    valid.pop_debug_group();
    valid.end_render_pass();
    valid.pop_debug_group();
    assert!(device
        .is_fence_complete(device.submit(valid).unwrap())
        .unwrap());

    let mut unclosed_pass_group = device
        .create_command_list(
            RenderQueueClass::Graphics,
            "unclosed-render-pass-debug-group",
        )
        .unwrap();
    begin_default_render_pass(&mut *unclosed_pass_group, color, depth);
    unclosed_pass_group.push_debug_group("pass");
    unclosed_pass_group.end_render_pass();
    assert_eq!(
        device.submit(unclosed_pass_group).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason: "render pass ended with an active debug group".to_string(),
        }
    );

    let mut wrong_scope_pop = device
        .create_command_list(RenderQueueClass::Graphics, "wrong-scope-debug-pop")
        .unwrap();
    wrong_scope_pop.push_debug_group("frame");
    begin_default_render_pass(&mut *wrong_scope_pop, color, depth);
    wrong_scope_pop.pop_debug_group();
    assert_eq!(
        device.submit(wrong_scope_pop).unwrap_err(),
        RhiError::InvalidDebugMarker {
            reason:
                "pop_debug_group must close a debug group recorded outside the active render pass"
                    .to_string(),
        }
    );
}

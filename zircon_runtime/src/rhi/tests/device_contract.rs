use crate::rhi::{
    BufferDesc, BufferHandle, BufferUsage, CommandList, FenceValue, PipelineDesc, PipelineHandle,
    PipelineKind, RenderDevice, RenderQueueClass, SamplerDesc, SamplerHandle, ShaderModuleDesc,
    ShaderModuleHandle, ShaderStage, TextureDesc, TextureFormat, TextureHandle, TextureUsage,
};
use crate::rhi_wgpu::{WgpuCommandList, WgpuRenderDevice};
use std::path::Path;

#[test]
fn rhi_handles_are_stable_raw_identifiers() {
    assert_eq!(BufferHandle::new(11).raw(), 11);
    assert_eq!(TextureHandle::new(12).raw(), 12);
    assert_eq!(SamplerHandle::new(13).raw(), 13);
    assert_eq!(ShaderModuleHandle::new(14).raw(), 14);
    assert_eq!(PipelineHandle::new(15).raw(), 15);
}

#[test]
fn buffer_and_texture_usage_flags_are_composable() {
    let buffer_usage = BufferUsage::UNIFORM | BufferUsage::STORAGE | BufferUsage::COPY_DST;
    assert!(buffer_usage.contains(BufferUsage::UNIFORM));
    assert!(buffer_usage.contains(BufferUsage::STORAGE));
    assert!(buffer_usage.contains(BufferUsage::COPY_DST));
    assert!(!buffer_usage.contains(BufferUsage::INDEX));

    let texture_usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC;
    assert!(texture_usage.contains(TextureUsage::RENDER_ATTACHMENT));
    assert!(texture_usage.contains(TextureUsage::SAMPLED));
    assert!(texture_usage.contains(TextureUsage::COPY_SRC));
    assert!(!texture_usage.contains(TextureUsage::PRESENT));
}

#[test]
fn wgpu_rhi_device_allocates_stable_resource_handles_and_fences() {
    let device = WgpuRenderDevice::new_headless();

    let buffer = device
        .create_buffer(&BufferDesc::new(
            "frame-uniform",
            256,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        ))
        .unwrap();
    let texture = device
        .create_texture(&TextureDesc::new(
            "scene-color",
            64,
            64,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
        ))
        .unwrap();
    let sampler = device
        .create_sampler(&SamplerDesc::linear("scene-linear"))
        .unwrap();
    let shader = device
        .create_shader_module(&ShaderModuleDesc {
            label: Some("fullscreen".to_string()),
            source: "@compute @workgroup_size(1) fn main() {}".to_string(),
            stage: ShaderStage::Compute,
            entry_point: "main".to_string(),
        })
        .unwrap();
    let pipeline = device
        .create_pipeline(&PipelineDesc::new("compute", PipelineKind::Compute))
        .unwrap();

    assert_ne!(buffer.raw(), texture.raw());
    assert_ne!(sampler.raw(), shader.raw());
    assert_ne!(pipeline.raw(), 0);

    let command_list = device
        .create_command_list(RenderQueueClass::Copy, "copy-upload")
        .unwrap();
    assert_eq!(command_list.queue_class(), RenderQueueClass::Copy);
    assert_eq!(command_list.label(), Some("copy-upload"));

    let fence = device.submit(command_list).unwrap();
    assert_eq!(fence, FenceValue(1));
    assert!(device.is_fence_complete(fence).unwrap());

    let bytes = device.read_buffer(buffer, 0, 16).unwrap();
    assert_eq!(bytes.len(), 16);

    device.destroy_pipeline(pipeline).unwrap();
    device.destroy_shader_module(shader).unwrap();
    device.destroy_sampler(sampler).unwrap();
    device.destroy_texture(texture).unwrap();
    device.destroy_buffer(buffer).unwrap();
}

#[test]
fn command_list_keeps_queue_class_and_label() {
    let command_list = WgpuCommandList::new(RenderQueueClass::Graphics, "main");
    assert_eq!(command_list.queue_class(), RenderQueueClass::Graphics);
    assert_eq!(command_list.label(), Some("main"));
}

#[test]
fn app_editor_and_core_framework_sources_do_not_import_wgpu() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime should live under the workspace root");
    let boundary_roots = [
        runtime_root.join("src").join("core").join("framework"),
        workspace_root.join("zircon_app").join("src"),
        workspace_root.join("zircon_editor").join("src"),
    ];
    let mut offenders = Vec::new();
    for root in boundary_roots {
        collect_wgpu_imports(&root, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "app/editor/framework sources must stay behind RenderFramework/RHI boundaries: {offenders:?}"
    );
}

fn collect_wgpu_imports(path: &Path, offenders: &mut Vec<String>) {
    let entries = std::fs::read_dir(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_wgpu_imports(&path, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (line_index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            let imports_wgpu = trimmed.starts_with("use wgpu")
                || trimmed.starts_with("use ::wgpu")
                || (trimmed.contains("wgpu::") && !trimmed.contains('"'));
            if imports_wgpu {
                offenders.push(format!("{}:{}", path.display(), line_index + 1));
            }
        }
    }
}

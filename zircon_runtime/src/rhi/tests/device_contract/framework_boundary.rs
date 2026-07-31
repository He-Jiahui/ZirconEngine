use std::path::Path;

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

#[test]
fn deterministic_rhi_contract_device_stays_test_only_and_out_of_product_call_graphs() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime should live under the workspace root");
    let rhi_wgpu_module = include_str!("../../../rhi_wgpu/mod.rs");
    let compact_rhi_wgpu_module = rhi_wgpu_module.split_whitespace().collect::<String>();
    let render_backend = include_str!("../../../graphics/backend/render_backend/render_backend.rs");
    let request_device = include_str!("../../../graphics/backend/render_backend/request_device.rs");
    let scene_renderer = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer/scene_renderer.rs"
    );
    let scene_renderer_construct = include_str!(
        "../../../graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs"
    );

    assert!(
        compact_rhi_wgpu_module.contains("#[cfg(test)]moddevice;"),
        "the deterministic contract device module must not compile into production builds"
    );
    assert!(
        !rhi_wgpu_module.contains("pub use device::"),
        "the deterministic contract device must not be a public WGPU backend export"
    );
    assert!(
        !rhi_wgpu_module.contains("as WgpuRenderDevice")
            && !rhi_wgpu_module.contains("as WgpuCommandList"),
        "the deterministic contract test types must not retain production-shaped WGPU aliases"
    );
    assert!(
        render_backend.contains("pub(crate) struct RenderBackend")
            && render_backend.contains("pub(crate) device: wgpu::Device")
            && render_backend.contains("pub(crate) queue: wgpu::Queue")
            && request_device.contains("adapter.request_device(")
            && scene_renderer.contains("backend: RenderBackend")
            && scene_renderer_construct
                .contains("crate::graphics::backend::RenderBackend::new_offscreen()?"),
        "SceneRenderer must construct graphics/backend's real wgpu device/queue owner"
    );

    let product_roots = [
        runtime_root.join("src"),
        workspace_root.join("zircon_app").join("src"),
        workspace_root.join("zircon_editor").join("src"),
    ];
    let mut offenders = Vec::new();
    for root in product_roots {
        for symbol in [
            "DeterministicRhiContractDevice",
            "DeterministicRhiContractCommandList",
            "WgpuRenderDevice",
            "WgpuCommandList",
        ] {
            collect_product_symbol_mentions(&root, symbol, &mut offenders);
        }
    }

    assert!(
        offenders.is_empty(),
        "product sources must use graphics/backend's real wgpu owner, not the deterministic RHI contract device: {offenders:?}"
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

fn collect_product_symbol_mentions(path: &Path, symbol: &str, offenders: &mut Vec<String>) {
    let entries = std::fs::read_dir(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let directory_name = path.file_name().and_then(|name| name.to_str());
            if directory_name == Some("tests") || directory_name == Some("rhi_wgpu") {
                continue;
            }
            collect_product_symbol_mentions(&path, symbol, offenders);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs")
            || path.file_name().and_then(|name| name.to_str()) == Some("tests.rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        if source.contains(symbol) {
            offenders.push(format!("{}:{symbol}", path.display()));
        }
    }
}

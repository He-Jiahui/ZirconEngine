use std::path::{Path, PathBuf};

#[test]
fn app_editor_and_core_framework_sources_do_not_import_wgpu() {
    let workspace_root = workspace_root();
    let runtime_root = workspace_root.join("zircon_runtime");
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
fn deterministic_test_backend_and_neutral_product_owner_remain_distinct() {
    let workspace_root = workspace_root();
    let runtime_root = workspace_root.join("zircon_runtime");
    let rhi_wgpu_module = std::fs::read_to_string(
        runtime_root
            .join("crates")
            .join("zr_rhi_wgpu")
            .join("src")
            .join("lib.rs"),
    )
    .expect("read zr_rhi_wgpu crate root");
    let compact_rhi_wgpu_module = rhi_wgpu_module.split_whitespace().collect::<String>();
    let neutral_mvp_renderer = std::fs::read_to_string(
        runtime_root
            .join("src")
            .join("graphics")
            .join("backend")
            .join("render_backend")
            .join("neutral_mvp_renderer.rs"),
    )
    .expect("read neutral MVP renderer");

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
        neutral_mvp_renderer.contains("device: WgpuRenderDevice")
            && neutral_mvp_renderer.contains("WgpuRenderDeviceContext::new(")
            && neutral_mvp_renderer.contains("WgpuRenderDevice::new(context, profile)")
            && neutral_mvp_renderer.contains("self.frame.submit(&self.device)")
            && !neutral_mvp_renderer.contains("RenderBackend::new_offscreen")
            && !neutral_mvp_renderer.contains("queue.submit"),
        "the neutral MVP product path must own and submit through the production RHI device"
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
        ] {
            collect_product_symbol_mentions(&root, symbol, &mut offenders);
        }
    }

    assert!(
        offenders.is_empty(),
        "product sources must never use the deterministic RHI test backend: {offenders:?}"
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("zr_rhi_wgpu should live under zircon_runtime/crates")
        .to_path_buf()
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

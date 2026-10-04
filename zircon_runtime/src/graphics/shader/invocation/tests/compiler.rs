use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

use super::*;

fn shader_ref() -> AssetReference {
    AssetReference::from_locator(
        ResourceLocator::parse("builtin://shaders/compute/particles").unwrap(),
    )
}

fn entry(name: &str, stage: RenderShaderStage) -> RenderShaderEntryPointDescriptor {
    RenderShaderEntryPointDescriptor {
        name: name.to_string(),
        stage,
    }
}

fn resource(
    name: &str,
    kind: ShaderResourceKind,
    access: ShaderResourceAccess,
) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name: name.to_string(),
        kind,
        access: Some(access),
    }
}

#[test]
fn render_compute_dispatch_builder_emits_kernel_resource_abi_and_cache_key() {
    let kernel = ComputeKernelRef::new(shader_ref(), "cs_main");
    let builder = ComputeDispatchBuilder::new(kernel.clone())
        .with_workgroup_size([64, 0, 1])
        .with_option_bits(0x3)
        .with_content_hash(0x55aa)
        .set_f32("delta_time", 1.0 / 60.0)
        .bind_storage("particles")
        .bind_storage_read("alive_list")
        .dispatch_groups([32, 1, 1]);

    let plan = builder
        .build(
            ShaderAssetKind::Compute,
            &[entry("cs_main", RenderShaderStage::Compute)],
            &[
                resource(
                    "particles",
                    ShaderResourceKind::StorageBuffer,
                    ShaderResourceAccess::ReadWrite,
                ),
                resource(
                    "alive_list",
                    ShaderResourceKind::StorageBuffer,
                    ShaderResourceAccess::Read,
                ),
            ],
        )
        .unwrap();

    assert_eq!(plan.kernel, kernel);
    assert_eq!(plan.workgroup_size, [64, 1, 1]);
    assert_eq!(
        plan.dispatch_extent,
        ShaderDispatchExtent::Fixed([32, 1, 1])
    );
    assert_eq!(
        plan.parameters.get("delta_time"),
        Some(&ShaderParameterValue::F32 { value: 1.0 / 60.0 })
    );
    assert_eq!(plan.resources.len(), 2);
    assert_eq!(plan.resources[0].name, "particles");
    assert_eq!(
        plan.resources[0].abi,
        ShaderAbiBinding {
            group: 0,
            binding: 1
        }
    );
    assert_eq!(plan.resources[1].name, "alive_list");
    assert_eq!(
        plan.resources[1].abi,
        ShaderAbiBinding {
            group: 0,
            binding: 2
        }
    );
    assert_eq!(COMPUTE_SHADER_PARAMS_BINDING.group, 0);
    assert_eq!(COMPUTE_SHADER_PARAMS_BINDING.binding, 0);
    assert_eq!(
        plan.pipeline_key.canonical_string(),
        format!(
            "shader_compute_pipeline_v1|shader={}|kernel=cs_main|options=0x00000003|content=0x00000000000055aa",
            shader_ref()
        )
    );
    assert_eq!(plan.pipeline_label, plan.pipeline_key.canonical_string());
}

#[test]
fn render_compute_dispatch_builder_reports_named_binding_diagnostics() {
    let builder = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader_ref(), "main"))
        .bind_texture("particles")
        .bind_storage_read("unknown")
        .dispatch_groups([1, 1, 1]);

    let diagnostics = builder
        .build(
            ShaderAssetKind::Surface,
            &[entry("main", RenderShaderStage::Fragment)],
            &[
                resource(
                    "particles",
                    ShaderResourceKind::StorageBuffer,
                    ShaderResourceAccess::Write,
                ),
                resource(
                    "params",
                    ShaderResourceKind::UniformBuffer,
                    ShaderResourceAccess::Read,
                ),
            ],
        )
        .unwrap_err();

    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidShaderKind {
            expected: ShaderAssetKind::Compute,
            actual: ShaderAssetKind::Surface,
        })
    );
    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidEntryPointStage {
            entry_point: "main".to_string(),
            stage: RenderShaderStage::Fragment,
            expected_stage: RenderShaderStage::Compute,
        })
    );
    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
            name: "particles".to_string(),
            expected: ShaderResourceKind::StorageBuffer,
            actual: ShaderResourceKind::Texture,
        })
    );
    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::MissingResource {
            name: "params".to_string(),
        })
    );
    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::UnknownResource {
            name: "unknown".to_string(),
        })
    );
}

#[test]
fn render_compute_dispatch_builder_requires_dispatch_groups() {
    let builder = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader_ref(), "cs_main"));

    let diagnostics = builder
        .build(
            ShaderAssetKind::Compute,
            &[entry("cs_main", RenderShaderStage::Compute)],
            &[],
        )
        .unwrap_err();

    assert_eq!(
        diagnostics,
        vec![ShaderDispatchBuildDiagnostic::MissingDispatchGroups {
            kernel: "cs_main".to_string(),
        }]
    );
}

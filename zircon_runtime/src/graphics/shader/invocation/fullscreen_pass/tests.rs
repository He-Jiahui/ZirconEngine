use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

use super::super::compiler::{
    ShaderAbiBinding, ShaderDispatchBuildDiagnostic, ShaderParameterValue,
};
use super::super::{
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};
use super::*;

fn shader_ref() -> AssetReference {
    AssetReference::from_locator(
        ResourceLocator::parse("builtin://shaders/fullscreen/tonemap").unwrap(),
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
fn render_fullscreen_pass_builder_emits_pass_input_and_params_abi() {
    let shader = FullscreenShaderRef::new(shader_ref(), "fs_main");
    let builder = FullscreenPassBuilder::new(shader.clone())
        .with_option_bits(0x8)
        .with_content_hash(0xf00d)
        .set_f32("exposure", 1.25)
        .bind_texture("source_color")
        .bind_sampler("linear_sampler");

    let plan = builder
        .build(
            ShaderAssetKind::Fullscreen,
            &[entry("fs_main", RenderShaderStage::Fragment)],
            &[
                resource(
                    "source_color",
                    ShaderResourceKind::Texture,
                    ShaderResourceAccess::Read,
                ),
                resource(
                    "linear_sampler",
                    ShaderResourceKind::Sampler,
                    ShaderResourceAccess::Read,
                ),
            ],
        )
        .unwrap();

    assert_eq!(plan.shader, shader);
    assert_eq!(plan.vertex_entry, FULLSCREEN_TRIANGLE_VERTEX_ENTRY);
    assert_eq!(
        plan.parameters.get("exposure"),
        Some(&ShaderParameterValue::F32 { value: 1.25 })
    );
    assert_eq!(FULLSCREEN_FRAME_GROUP, 0);
    assert_eq!(FULLSCREEN_PASS_INPUT_GROUP, 1);
    assert_eq!(
        FULLSCREEN_PARAMS_BINDING,
        ShaderAbiBinding {
            group: 2,
            binding: 0
        }
    );
    assert_eq!(
        plan.resources[0].abi,
        ShaderAbiBinding {
            group: 1,
            binding: 0
        }
    );
    assert_eq!(
        plan.resources[1].abi,
        ShaderAbiBinding {
            group: 1,
            binding: 1
        }
    );
    assert_eq!(
        plan.pipeline_key.canonical_string(),
        format!(
            "shader_fullscreen_pipeline_v1|shader={}|fragment=fs_main|options=0x00000008|content=0x000000000000f00d",
            shader_ref()
        )
    );
}

#[test]
fn render_fullscreen_pass_parameters_use_stable_vec4_slots() {
    let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
        .set_vec4("tint", [0.25, 0.5, 0.75, 1.0])
        .set_bool("enabled", true)
        .set_f32("exposure", 1.25)
        .build(
            ShaderAssetKind::Fullscreen,
            &[entry("fs_main", RenderShaderStage::Fragment)],
            &[],
        )
        .expect("fullscreen parameter-only plan should build");

    assert_eq!(plan.parameter_slot("enabled"), Some(0));
    assert_eq!(plan.parameter_slot("exposure"), Some(1));
    assert_eq!(plan.parameter_slot("tint"), Some(2));
    assert_eq!(plan.parameter_byte_len(), 48);
    assert_eq!(
        plan.parameter_bytes(),
        [
            1_u32,
            0,
            0,
            0,
            1.25_f32.to_bits(),
            0,
            0,
            0,
            0.25_f32.to_bits(),
            0.5_f32.to_bits(),
            0.75_f32.to_bits(),
            1.0_f32.to_bits(),
        ]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>(),
    );
}

#[test]
fn render_fullscreen_pass_reencodes_parameters_into_a_reused_buffer() {
    let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
        .set_f32("exposure", 1.25)
        .set_vec4("tint", [0.25, 0.5, 0.75, 1.0])
        .build(
            ShaderAssetKind::Fullscreen,
            &[entry("fs_main", RenderShaderStage::Fragment)],
            &[],
        )
        .expect("fullscreen parameter-only plan should build");
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(&[0xff; 8]);
    let capacity = bytes.capacity();

    plan.write_parameter_bytes(&mut bytes);

    assert_eq!(bytes.capacity(), capacity);
    assert_eq!(bytes.len(), 32);
    assert_eq!(
        bytes,
        [
            1.25_f32.to_bits(),
            0,
            0,
            0,
            0.25_f32.to_bits(),
            0.5_f32.to_bits(),
            0.75_f32.to_bits(),
            1.0_f32.to_bits(),
        ]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>(),
    );

    plan.write_parameter_bytes(&mut bytes);
    assert_eq!(bytes.capacity(), capacity);
}

#[test]
fn render_fullscreen_pass_builder_encodes_every_parameter_value_shape() {
    let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
        .set_i32("signed", -2)
        .set_u32("unsigned", 7)
        .set_vec2("uv", [0.25, 0.5])
        .set_vec3("normal", [0.0, 1.0, 0.0])
        .build(
            ShaderAssetKind::Fullscreen,
            &[entry("fs_main", RenderShaderStage::Fragment)],
            &[],
        )
        .expect("all fullscreen parameter value shapes should build");

    assert_eq!(plan.parameter_slot("normal"), Some(0));
    assert_eq!(plan.parameter_slot("signed"), Some(1));
    assert_eq!(plan.parameter_slot("unsigned"), Some(2));
    assert_eq!(plan.parameter_slot("uv"), Some(3));
    assert_eq!(
        plan.parameter_bytes(),
        [
            0.0_f32.to_bits(),
            1.0_f32.to_bits(),
            0.0_f32.to_bits(),
            0,
            (-2_i32) as u32,
            0,
            0,
            0,
            7,
            0,
            0,
            0,
            0.25_f32.to_bits(),
            0.5_f32.to_bits(),
            0,
            0,
        ]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>(),
    );
}

#[test]
fn render_fullscreen_pass_parameter_abi_is_explicitly_little_endian() {
    let source = include_str!("plan.rs");
    assert!(source.contains("word.to_le_bytes()"));
    assert!(!source.contains("to_ne_bytes"));
}

#[test]
fn render_fullscreen_pass_builder_reports_stage_and_resource_errors() {
    let builder = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader_ref(), "fs_main"))
        .bind_storage_read("source_color");

    let diagnostics = builder
        .build(
            ShaderAssetKind::Compute,
            &[entry("fs_main", RenderShaderStage::Compute)],
            &[resource(
                "source_color",
                ShaderResourceKind::Texture,
                ShaderResourceAccess::Read,
            )],
        )
        .unwrap_err();

    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidShaderKind {
            expected: ShaderAssetKind::Fullscreen,
            actual: ShaderAssetKind::Compute,
        })
    );
    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::InvalidEntryPointStage {
            entry_point: "fs_main".to_string(),
            stage: RenderShaderStage::Compute,
            expected_stage: RenderShaderStage::Fragment,
        })
    );
    assert!(
        diagnostics.contains(&ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
            name: "source_color".to_string(),
            expected: ShaderResourceKind::Texture,
            actual: ShaderResourceKind::StorageBuffer,
        })
    );
}

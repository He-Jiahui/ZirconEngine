use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

use super::*;
use crate::graphics::shader::invocation::{
    FullscreenPassBuilder, FullscreenShaderRef, RenderShaderEntryPointDescriptor,
    RenderShaderStage, ShaderAssetKind, FULLSCREEN_PARAMS_BINDING,
};

#[test]
fn fullscreen_parameter_layout_projects_nonempty_parameters_to_group_two_uniform() {
    let shader = AssetReference::from_locator(
        ResourceLocator::parse("builtin://shaders/fullscreen/parameterized").unwrap(),
    );
    let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader, "fs_main"))
        .set_vec4("tile_scale", [1.0, 1.0, 0.0, 0.0])
        .build(
            ShaderAssetKind::Fullscreen,
            &[RenderShaderEntryPointDescriptor {
                name: "fs_main".to_string(),
                stage: RenderShaderStage::Fragment,
            }],
            &[],
        )
        .expect("parameterized fullscreen plan should build");

    let entry = fullscreen_pass_parameter_layout_entry(&plan)
        .expect("nonempty parameters require a group-two uniform layout entry");

    assert_eq!(entry.binding, FULLSCREEN_PARAMS_BINDING.binding);
    assert_eq!(entry.visibility, wgpu::ShaderStages::FRAGMENT);
    assert!(matches!(
        entry.ty,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            ..
        }
    ));
}

#[test]
fn fullscreen_parameter_layout_omits_group_two_for_empty_parameters() {
    let shader = AssetReference::from_locator(
        ResourceLocator::parse("builtin://shaders/fullscreen/no-parameters").unwrap(),
    );
    let plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader, "fs_main"))
        .build(
            ShaderAssetKind::Fullscreen,
            &[RenderShaderEntryPointDescriptor {
                name: "fs_main".to_string(),
                stage: RenderShaderStage::Fragment,
            }],
            &[],
        )
        .expect("parameter-free fullscreen plan should build");

    assert!(fullscreen_pass_parameter_layout_entry(&plan).is_none());
}

#[test]
fn fullscreen_parameter_bindings_have_no_dynamic_queue_write_authority() {
    let source = include_str!("../fullscreen_pass_parameters.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("fullscreen parameter production source");

    assert_eq!(production.matches("create_buffer_init(").count(), 1);
    assert!(!production.contains("wgpu::Queue"));
    assert!(!production.contains("write_buffer("));
    assert!(!production.contains("COPY_DST"));
    assert!(!production.contains("pub(crate) fn write("));
}

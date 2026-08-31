use crate::graphics::shader::invocation::{FULLSCREEN_PARAMS_BINDING, FullscreenPassPlan};
use wgpu::util::DeviceExt;

pub(crate) struct FullscreenPassParameterBindings {
    bind_group: wgpu::BindGroup,
    _buffer: wgpu::Buffer,
}

impl FullscreenPassParameterBindings {
    pub(crate) fn new(
        device: &wgpu::Device,
        plan: &FullscreenPassPlan,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<Self> {
        if plan.parameter_byte_len() == 0 {
            return None;
        }

        let mut upload_bytes = Vec::with_capacity(usize::try_from(plan.parameter_byte_len()).ok()?);
        plan.write_parameter_bytes(&mut upload_bytes);
        debug_assert_eq!(upload_bytes.len() as u64, plan.parameter_byte_len());
        let buffer_label = format!("{}-params-buffer", plan.pipeline_label);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&buffer_label),
            contents: &upload_bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group_label = format!("{}-params-bind-group", plan.pipeline_label);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&bind_group_label),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: FULLSCREEN_PARAMS_BINDING.binding,
                resource: buffer.as_entire_binding(),
            }],
        });

        Some(Self {
            bind_group,
            _buffer: buffer,
        })
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub(crate) fn fullscreen_pass_parameter_layout_entry(
    plan: &FullscreenPassPlan,
) -> Option<wgpu::BindGroupLayoutEntry> {
    let min_binding_size = std::num::NonZeroU64::new(plan.parameter_byte_len())?;
    Some(wgpu::BindGroupLayoutEntry {
        binding: FULLSCREEN_PARAMS_BINDING.binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(min_binding_size),
        },
        count: None,
    })
}

pub(crate) fn create_fullscreen_pass_parameter_bind_group_layout(
    device: &wgpu::Device,
    plan: &FullscreenPassPlan,
) -> Option<wgpu::BindGroupLayout> {
    let entry = fullscreen_pass_parameter_layout_entry(plan)?;
    let label = format!("{}-params-layout", plan.pipeline_label);
    Some(
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&label),
            entries: &[entry],
        }),
    )
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

    use super::*;
    use crate::graphics::shader::invocation::{
        FULLSCREEN_PARAMS_BINDING, FullscreenPassBuilder, FullscreenShaderRef,
        RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind,
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
        let source = include_str!("fullscreen_pass_parameters.rs");
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
}

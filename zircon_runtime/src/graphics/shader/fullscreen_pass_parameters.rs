use crate::core::framework::render::{
    FullscreenPassPlan, ShaderParameterValue, FULLSCREEN_PARAMS_BINDING,
};

type FullscreenParameterLayout = Vec<(String, std::mem::Discriminant<ShaderParameterValue>)>;

pub(crate) struct FullscreenPassParameterBindings {
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    parameter_layout: FullscreenParameterLayout,
    upload_bytes: Vec<u8>,
}

impl FullscreenPassParameterBindings {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &FullscreenPassPlan,
        layout: &wgpu::BindGroupLayout,
    ) -> Option<Self> {
        if plan.parameter_byte_len() == 0 {
            return None;
        }

        let buffer_label = format!("{}-params-buffer", plan.pipeline_label);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&buffer_label),
            size: plan.parameter_byte_len(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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

        let mut bindings = Self {
            bind_group,
            buffer,
            parameter_layout: fullscreen_parameter_layout(plan),
            upload_bytes: Vec::with_capacity(usize::try_from(plan.parameter_byte_len()).ok()?),
        };
        let initial_upload_applied = bindings.write(queue, plan);
        debug_assert!(initial_upload_applied);
        Some(bindings)
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(crate) fn write(&mut self, queue: &wgpu::Queue, plan: &FullscreenPassPlan) -> bool {
        if !fullscreen_parameter_layout_matches(plan, &self.parameter_layout) {
            return false;
        }
        plan.write_parameter_bytes(&mut self.upload_bytes);
        queue.write_buffer(&self.buffer, 0, &self.upload_bytes);
        true
    }
}

fn fullscreen_parameter_layout(plan: &FullscreenPassPlan) -> FullscreenParameterLayout {
    plan.parameters
        .iter()
        .map(|(name, value)| (name.clone(), std::mem::discriminant(value)))
        .collect()
}

fn fullscreen_parameter_layout_matches(
    plan: &FullscreenPassPlan,
    expected_layout: &FullscreenParameterLayout,
) -> bool {
    plan.parameters.len() == expected_layout.len()
        && plan.parameters.iter().zip(expected_layout).all(
            |((name, value), (expected_name, expected_kind))| {
                name == expected_name && std::mem::discriminant(value) == *expected_kind
            },
        )
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
    use crate::core::framework::render::{
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
    fn fullscreen_parameter_layout_distinguishes_value_kinds_with_the_same_name() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("builtin://shaders/fullscreen/typed-parameters").unwrap(),
        );
        let entries = [RenderShaderEntryPointDescriptor {
            name: "fs_main".to_string(),
            stage: RenderShaderStage::Fragment,
        }];
        let float_plan =
            FullscreenPassBuilder::new(FullscreenShaderRef::new(shader.clone(), "fs_main"))
                .set_f32("threshold", 0.5)
                .build(ShaderAssetKind::Fullscreen, &entries, &[])
                .expect("float fullscreen plan should build");
        let integer_plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader, "fs_main"))
            .set_u32("threshold", 1)
            .build(ShaderAssetKind::Fullscreen, &entries, &[])
            .expect("integer fullscreen plan should build");

        assert_ne!(
            fullscreen_parameter_layout(&float_plan),
            fullscreen_parameter_layout(&integer_plan),
        );
    }

    #[test]
    fn fullscreen_parameter_layout_match_uses_the_cached_layout_contract() {
        let shader = AssetReference::from_locator(
            ResourceLocator::parse("builtin://shaders/fullscreen/typed-parameters").unwrap(),
        );
        let entries = [RenderShaderEntryPointDescriptor {
            name: "fs_main".to_string(),
            stage: RenderShaderStage::Fragment,
        }];
        let float_plan =
            FullscreenPassBuilder::new(FullscreenShaderRef::new(shader.clone(), "fs_main"))
                .set_f32("threshold", 0.5)
                .build(ShaderAssetKind::Fullscreen, &entries, &[])
                .expect("float fullscreen plan should build");
        let integer_plan = FullscreenPassBuilder::new(FullscreenShaderRef::new(shader, "fs_main"))
            .set_u32("threshold", 1)
            .build(ShaderAssetKind::Fullscreen, &entries, &[])
            .expect("integer fullscreen plan should build");
        let layout = fullscreen_parameter_layout(&float_plan);

        assert!(fullscreen_parameter_layout_matches(&float_plan, &layout));
        assert!(!fullscreen_parameter_layout_matches(&integer_plan, &layout));
    }
}

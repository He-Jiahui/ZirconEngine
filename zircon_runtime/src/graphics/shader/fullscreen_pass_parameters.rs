use crate::graphics::shader::invocation::{FullscreenPassPlan, FULLSCREEN_PARAMS_BINDING};
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
#[path = "tests/fullscreen_pass_parameters.rs"]
mod tests;

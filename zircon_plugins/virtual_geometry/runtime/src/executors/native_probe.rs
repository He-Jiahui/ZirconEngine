//! Post-frame gathers over the actual retained buffers. Host code selects only
//! the available ABI; every live count and returned record comes from GPU bytes.
use std::{num::NonZeroU64, ops::Range};

use zircon_runtime::{
    graphics::{
        RenderPassGpuExecutionContext, RenderPassGpuRecordingContext, RenderPassGpuResourceFactory,
    },
    render_graph::RenderGraphResourceAccessKind,
};

const ROLE_COUNT: usize = 14;
const MAX_RECORDS: u32 = 65_536;
const MAX_RESULT_STRIDE: u64 = 14;

#[repr(usize)]
#[derive(Clone, Copy, Debug)]
pub(super) enum ProbeSourceRole {
    SharedArgs4 = 0,
    SharedAuthority14 = 1,
    SharedSubmission = 2,
    DrawRefs = 3,
    Segments = 4,
    ExecutionArgs = 5,
    ExecutionAuthority = 6,
    ExecutionSubmission = 7,
    DedicatedExecutionIndices = 8,
    DedicatedExecutionRecords = 9,
    IndexedArgs5 = 10,
    IndexedSidecar32 = 11,
    IndexedStatus1 = 12,
    AdmittedRunSlots = 13,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
enum RawReadKind {
    SubmissionRecordsWithTokens = 0,
    DrawRefIndices = 1,
    ExecutionRecords = 2,
    SharedArgs = 3,
    AuthorSegments = 4,
    TerminalAuthorityRows = 5,
}

impl TryFrom<u32> for RawReadKind {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, String> {
        match value {
            0 => Ok(Self::SubmissionRecordsWithTokens),
            1 => Ok(Self::DrawRefIndices),
            2 => Ok(Self::ExecutionRecords),
            3 => Ok(Self::SharedArgs),
            4 => Ok(Self::AuthorSegments),
            5 => Ok(Self::TerminalAuthorityRows),
            _ => Err("unknown virtual geometry native probe read kind".into()),
        }
    }
}

type SourceSpec<'a> = Option<(&'a str, RenderGraphResourceAccessKind, u32, u32)>;
type SourceView<'a> = Option<(wgpu::BufferBinding<'a>, u32, u32)>;

/// The caller supplies only proven producers. In particular, the two dedicated
/// legacy execution buffers remain None; missing declared bindings are errors.
pub(super) fn retain(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    sources: [SourceSpec<'_>; ROLE_COUNT],
    output_capacity: u32,
) -> Result<bool, String> {
    let staging_bytes = result_bytes(output_capacity)?;
    gpu.retain_virtual_geometry_native_probe(
        &sources,
        staging_bytes,
        move |recording, retained, kind| {
            gather(
                recording,
                retained,
                RawReadKind::try_from(kind)?,
                output_capacity,
            )
        },
    )
}

fn result_bytes(capacity: u32) -> Result<u64, String> {
    if capacity > MAX_RECORDS {
        return Err("native gather result capacity exceeds the bounded ABI".into());
    }
    u64::from(capacity)
        .checked_mul(MAX_RESULT_STRIDE)
        .and_then(|n| n.checked_add(1))
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "native gather result allocation overflow".into())
}

fn source<'slice, 'buffer>(
    sources: &'slice [SourceView<'buffer>],
    role: ProbeSourceRole,
) -> Option<&'slice (wgpu::BufferBinding<'buffer>, u32, u32)> {
    sources.get(role as usize)?.as_ref()
}

fn validate_binding(
    binding: &wgpu::BufferBinding<'_>,
    capacity: u32,
    actual_stride: u32,
    expected_stride: u32,
    prefix_words: u64,
) -> Result<u64, String> {
    let required = u64::from(capacity)
        .checked_mul(u64::from(expected_stride))
        .and_then(|n| n.checked_add(prefix_words))
        .and_then(|n| n.checked_mul(4))
        .ok_or("native gather input ABI size overflow")?;
    let available = binding
        .size
        .map(NonZeroU64::get)
        .unwrap_or_else(|| binding.buffer.size().saturating_sub(binding.offset));
    if capacity > MAX_RECORDS
        || actual_stride != expected_stride
        || binding.offset % 256 != 0
        || available % 4 != 0
        || required > available
        || binding
            .offset
            .checked_add(available)
            .is_none_or(|end| end > binding.buffer.size())
        || !binding.buffer.usage().contains(wgpu::BufferUsages::STORAGE)
    {
        return Err("native gather input binding does not satisfy its real ABI".into());
    }
    Ok(available)
}

fn gather(
    recording: &mut dyn RenderPassGpuRecordingContext<
        ResourceFactory = dyn RenderPassGpuResourceFactory + '_,
    >,
    sources: &[SourceView<'_>],
    kind: RawReadKind,
    output_capacity: u32,
) -> Result<(wgpu::Buffer, Range<u64>), String> {
    if sources.len() != ROLE_COUNT {
        return Err("native gather source role table has changed".into());
    }
    // Selection depends on Option presence alone. A present zero GPU header is
    // consumed by the shader and returns empty; it never activates another path.
    let primary = (
        source(sources, ProbeSourceRole::ExecutionArgs),
        source(sources, ProbeSourceRole::ExecutionAuthority),
    );
    let (first, second, mode, first_stride, prefix) =
        if matches!(kind, RawReadKind::SharedArgs | RawReadKind::AuthorSegments) {
            let segments = source(sources, ProbeSourceRole::Segments)
                .ok_or("author segment source was dropped")?;
            if matches!(kind, RawReadKind::SharedArgs) {
                let args = source(sources, ProbeSourceRole::SharedArgs4)
                    .ok_or("raw args source was dropped")?;
                (args, segments, 2u32, 4u32, 1u64)
            } else {
                (segments, segments, 2u32, 12u32, 4u64)
            }
        } else {
            match primary {
                (Some(args), Some(authority)) => (args, authority, 0u32, 4u32, 1u64),
                _ => {
                    let submission = source(sources, ProbeSourceRole::ExecutionSubmission).ok_or(
                "native gather has neither execution args/authority nor submission fallback",
            )?;
                    let authority = source(sources, ProbeSourceRole::SharedAuthority14).ok_or(
                        "native gather submission fallback requires retained shared authority",
                    )?;
                    (submission, authority, 1u32, 17u32, 1u64)
                }
            }
        };
    validate_binding(&first.0, first.1, first.2, first_stride, prefix)?;
    if mode == 2 {
        validate_binding(&second.0, second.1, second.2, 12, 4)?;
    } else {
        validate_binding(&second.0, second.1, second.2, 14, 0)?;
    }
    let bytes = result_bytes(output_capacity)?;
    let words = usize::try_from(bytes / 4)
        .map_err(|_| "native gather result cannot fit host allocation length")?;
    let zeros = vec![0u32; words];
    let params = [
        mode,
        kind as u32,
        output_capacity,
        first.1,
        second.1,
        first_stride,
        second.2,
        0,
    ];
    let factory = recording.resource_factory();
    let output = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-native-gather-output"),
        contents: bytemuck::cast_slice(&zeros),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let staging = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-native-gather-staging"),
        contents: bytemuck::cast_slice(&zeros),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    });
    let uniform = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-vg-native-gather-layout"),
        contents: bytemuck::cast_slice(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-vg-native-gather-layout"),
        entries: &[
            storage_entry(0, true),
            storage_entry(1, true),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(32),
                },
                count: None,
            },
            storage_entry(3, false),
        ],
    });
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-vg-native-gather"),
        source: wgpu::ShaderSource::Wgsl(include_str!("native_probe.wgsl").into()),
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-vg-native-gather-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-vg-native-gather"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("gather"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bindings = factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-vg-native-gather-bindings"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(first.0.clone()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(second.0.clone()),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: output.as_entire_binding(),
            },
        ],
    });
    {
        let mut pass =
            recording
                .command_encoder()
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("zircon-vg-native-gather"),
                    timestamp_writes: None,
                });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    recording
        .command_encoder()
        .copy_buffer_to_buffer(&output, 0, &staging, 0, bytes);
    Ok((staging, 0..bytes))
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: NonZeroU64::new(4),
        },
        count: None,
    }
}

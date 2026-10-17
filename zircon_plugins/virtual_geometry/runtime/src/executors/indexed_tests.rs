use super::*;
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

struct TestRecordingContext<'a> {
    device: &'a wgpu::Device,
    encoder: &'a mut wgpu::CommandEncoder,
}

impl RenderPassGpuRecordingContext for TestRecordingContext<'_> {
    type ResourceFactory = wgpu::Device;
    fn resource_factory(&self) -> &Self::ResourceFactory {
        self.device
    }
    fn command_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder
    }
}

fn words_buffer(device: &wgpu::Device, words: &[u32], usage: wgpu::BufferUsages) -> wgpu::Buffer {
    DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("indexed-adapter-contract"),
            contents: bytemuck::cast_slice(words),
            usage,
        },
    )
}

fn binding(buffer: &wgpu::Buffer) -> GpuBufferSlice<'_> {
    GpuBufferSlice::new(buffer, 0, buffer.size()).expect("whole storage buffer")
}

fn read_words(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    submission: wgpu::SubmissionIndex,
) -> Vec<u32> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).expect("map receiver");
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(Duration::from_secs(30)),
        })
        .expect("actual indexed adapter submission completes");
    receiver
        .recv_timeout(Duration::from_secs(30))
        .expect("actual map callback")
        .expect("actual map succeeds");
    let mapped = slice.get_mapped_range();
    let words = bytemuck::cast_slice::<u8, u32>(&mapped).to_vec();
    drop(mapped);
    buffer.unmap();
    words
}

fn identity(row: u32) -> RenderVirtualGeometryExecutionIdentity {
    RenderVirtualGeometryExecutionIdentity {
        original_index: row + 10,
        draw_ref_index: row + 20,
        stable_instance_key: 0x1234_5678_0000_0000 + u64::from(row),
        instance_index: Some(row + 30),
        entity: 0x8765_4321_0000_0000 + u64::from(row),
        page_id: row + 300,
        submission_index: Some(row + 40),
        draw_ref_rank: Some(row + 50),
        cluster_start_ordinal: row + 60,
        cluster_span_count: 1,
        submission_slot: if row == 0 { None } else { Some(row + 1) },
    }
}

fn authority(
    id: RenderVirtualGeometryExecutionIdentity,
    candidate: u32,
    cluster: u32,
) -> [u32; 14] {
    [
        id.instance_index.unwrap(),
        id.entity as u32,
        (id.entity >> 32) as u32,
        id.page_id,
        cluster,
        id.cluster_start_ordinal,
        id.draw_ref_index,
        id.original_index,
        id.submission_index.unwrap(),
        id.draw_ref_rank.unwrap(),
        id.submission_slot.unwrap_or(u32::MAX),
        candidate,
        77,
        88,
    ]
}

fn run(
    row: u32,
    range: Range<u32>,
    ordinal: u32,
    id: RenderVirtualGeometryExecutionIdentity,
) -> IndexedAuthorRun {
    let (candidate, cluster, first_instance, base_vertex) = match row {
        0 => (42, 700, 17, -2),
        1 => (99, 701, 23, 0),
        _ => (42, 700, 29, 5),
    };
    IndexedAuthorRun::try_new(IndexedAuthorRunInput {
        source_row: row,
        candidate_index: candidate,
        cluster_id: cluster,
        cluster_ordinal: id.cluster_start_ordinal,
        index_range: range,
        source_index_count: 12,
        base_vertex,
        gpu_scene_instance_span: first_instance..first_instance + 1,
        run_ordinal: ordinal,
        identity: id,
    })
    .expect("admitted author run")
}

#[test]
fn indexed_adapter_consumes_gpu_authority_and_preserves_run_identity() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()))
        .into_iter()
        .find(|adapter| adapter.get_info().name.contains("RTX 3060"))
        .expect("required RTX 3060 adapter");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("indexed adapter real GPU contract"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .expect("required GPU device");
    let identities = [identity(0), identity(1), identity(2)];
    // The existing production kernel writes the visibility and fourteen-word authority.
    let records = [
        super::super::SourceRecord {
            cluster_id: 700,
            candidate_index: 42,
            first_vertex: 0,
            vertex_count: 6,
            first_instance: 0x10001,
            authority: authority(identities[0], 42, 700),
        },
        super::super::SourceRecord {
            cluster_id: 701,
            candidate_index: 99,
            first_vertex: 6,
            vertex_count: 6,
            first_instance: 0x20002,
            authority: authority(identities[1], 99, 701),
        },
        super::super::SourceRecord {
            cluster_id: 700,
            candidate_index: 42,
            first_vertex: 12,
            vertex_count: 6,
            first_instance: 0x30003,
            authority: authority(identities[2], 42, 700),
        },
    ];
    let mut source_words = vec![records.len() as u32];
    source_words.extend_from_slice(bytemuck::cast_slice(&records));
    let visible = words_buffer(&device, &[1, 42], wgpu::BufferUsages::STORAGE);
    let source = words_buffer(&device, &source_words, wgpu::BufferUsages::STORAGE);
    let old_args = words_buffer(
        &device,
        &[0; 13],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let old_authority = words_buffer(
        &device,
        &[0; 42],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let indexed_args = words_buffer(
        &device,
        &[0xdead_beef; 30],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
    );
    let sidecar = words_buffer(
        &device,
        &[0xdead_beef; 192],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let status = words_buffer(
        &device,
        &[0xdead_beef; 6],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let pipeline = super::super::create_pipeline(&device);
    let layout = pipeline.get_bind_group_layout(0);
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("existing indirect producer"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: visible.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: source.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: old_args.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: old_authority.as_entire_binding(),
            },
        ],
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("producer adapter readback"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("real existing producer"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    let mut wrong_identity = identities[0];
    wrong_identity.stable_instance_key ^= 1;
    let runs = [
        run(0, 0..3, 0, identities[0]),
        run(0, 9..12, 1, identities[0]),
        run(1, 6..9, 0, identities[1]),
        run(2, 3..6, 0, identities[2]),
        run(0, 0..3, 2, wrong_identity),
    ];
    encode(
        &mut TestRecordingContext {
            device: &device,
            encoder: &mut encoder,
        },
        IndexedAdapterInputs {
            source_args: binding(&old_args),
            source_authority: binding(&old_authority),
            source_identities: &identities,
            runs: &runs,
            output_capacity: 6,
        },
        IndexedAdapterOutputs {
            args: binding(&indexed_args),
            sidecar: binding(&sidecar),
            status: binding(&status),
        },
    )
    .expect("encode real adapter");
    let buffers = [&old_args, &old_authority, &indexed_args, &sidecar, &status];
    let staging: Vec<_> = buffers
        .iter()
        .map(|buffer| {
            let staging = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("actual indexed mapped bytes"),
                size: buffer.size(),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, buffer.size());
            staging
        })
        .collect();
    let submission = queue.submit([encoder.finish()]);
    let actual: Vec<_> = staging
        .iter()
        .map(|buffer| read_words(&device, buffer, submission.clone()))
        .collect();
    assert_eq!(actual[0][0], 2);
    assert_eq!(actual[0][2], 1);
    assert_eq!(actual[0][6], 0);
    assert_eq!(actual[0][10], 1);
    for (row, record) in records.iter().enumerate() {
        assert_eq!(&actual[1][row * 14..row * 14 + 14], &record.authority);
    }
    let expected = [
        [3, 1, 0, (-2i32) as u32, 17],
        [3, 1, 9, (-2i32) as u32, 17],
        [0; 5],
        [3, 1, 3, 5, 29],
        [0; 5],
        [0; 5],
    ];
    assert_eq!(actual[2], expected.concat());
    for (slot, row) in [(0, 0), (1, 0), (2, 1), (3, 2)] {
        let data = &actual[3][slot * 32..slot * 32 + 32];
        assert_eq!(&data[..14], &actual[1][row * 14..row * 14 + 14]);
        assert_eq!(&data[14..28], &identity_words(identities[row]));
        assert_eq!(data[28], row as u32);
        assert_eq!(data[29], records[row].candidate_index);
        assert_eq!(data[30], u32::from(slot == 1));
        assert_eq!(data[31], u32::from(row != 1));
    }
    assert_eq!(&actual[3][128..], &[0; 64]);
    assert_eq!(actual[4], [0, 0, 0, 0, 2, 0]);
    assert_ne!(actual[2][4], actual[0][4]);
    assert_ne!(actual[2][19], actual[0][12]);
}

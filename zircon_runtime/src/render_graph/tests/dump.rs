use super::RenderGraphDump;
use crate::render_graph::{QueueLane, RenderGraphBuilder};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

#[test]
fn render_graph_dump_resource_rows_preserve_transient_bucket_identity() {
    let mut builder = RenderGraphBuilder::new("dump-bucket-identity");
    let r8 = builder.create_texture(TextureDesc::new(
        "r8-color",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let r16 = builder.create_texture(TextureDesc::new(
        "r16-color",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("output");

    let write_r8 = builder.add_pass("write-r8", QueueLane::Graphics);
    let write_r16 = builder.add_pass("write-r16", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);
    builder.write_texture(write_r8, r8).unwrap();
    builder.write_texture(write_r16, r16).unwrap();
    builder.read_texture(present, r8).unwrap();
    builder.read_texture(present, r16).unwrap();
    builder.write_external(present, output).unwrap();

    let dump = RenderGraphDump::from_graph(&builder.compile().unwrap());
    let r8_row = dump
        .resource_rows
        .iter()
        .find(|resource| resource.name == "r8-color")
        .unwrap();
    let r16_row = dump
        .resource_rows
        .iter()
        .find(|resource| resource.name == "r16-color")
        .unwrap();

    assert_eq!(r8_row.transient_slot, Some(0));
    assert_eq!(r16_row.transient_slot, Some(0));
    assert_ne!(
        r8_row.transient_bucket_key_hash,
        r16_row.transient_bucket_key_hash
    );

    let text = dump.to_text();
    assert!(text.contains(
        "resource_state_transitions=0 queue_state_transitions=0 legacy_state_accesses=5"
    ));
    assert!(text.contains(&format!(
        "resource name=r8-color kind=TransientTexture imported=false usage=- live=true lifetime=0..2 slot=0 bucket={}",
        r8_row.transient_bucket_key_hash.unwrap()
    )));
    assert!(text.contains(&format!(
        "resource name=r16-color kind=TransientTexture imported=false usage=- live=true lifetime=1..2 slot=0 bucket={}",
        r16_row.transient_bucket_key_hash.unwrap()
    )));
}

#[test]
fn render_graph_dump_reports_executable_topology_layers() {
    let mut builder = RenderGraphBuilder::new("dump-topology-layers");
    let left = builder.create_texture(TextureDesc::new(
        "left",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let right = builder.create_texture(TextureDesc::new(
        "right",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let unused = builder.create_texture(TextureDesc::new(
        "unused",
        32,
        32,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let output = builder.import_present_external_resource("output");

    let write_left = builder.add_pass("write-left", QueueLane::Graphics);
    let write_right = builder.add_pass("write-right", QueueLane::AsyncCompute);
    let write_unused = builder.add_pass("write-unused", QueueLane::Graphics);
    let composite = builder.add_pass("composite", QueueLane::Graphics);
    builder.write_texture(write_left, left).unwrap();
    builder.write_texture(write_right, right).unwrap();
    builder.write_texture(write_unused, unused).unwrap();
    builder.read_texture(composite, left).unwrap();
    builder.read_texture(composite, right).unwrap();
    builder.write_external(composite, output).unwrap();

    let dump = RenderGraphDump::from_graph(&builder.compile().unwrap());

    assert_eq!(dump.executable_topology_layer_count, 2);
    assert_eq!(dump.executable_topology_peak_width, 2);
    assert_eq!(
        dump.pass_rows
            .iter()
            .find(|pass| pass.name == "write-left")
            .unwrap()
            .executable_topology_layer,
        Some(0)
    );
    assert_eq!(
        dump.pass_rows
            .iter()
            .find(|pass| pass.name == "write-right")
            .unwrap()
            .executable_topology_layer,
        Some(0)
    );
    assert_eq!(
        dump.pass_rows
            .iter()
            .find(|pass| pass.name == "composite")
            .unwrap()
            .executable_topology_layer,
        Some(1)
    );
    assert_eq!(
        dump.pass_rows
            .iter()
            .find(|pass| pass.name == "write-unused")
            .unwrap()
            .executable_topology_layer,
        None
    );

    let text = dump.to_text();
    assert!(text.starts_with(
        "render_graph name=dump-topology-layers passes=4 executable=3 culled=1 resources=3 topology_layers=2 topology_peak_width=2\n"
    ));
    assert!(text.contains("pass[0] id=0 name=write-left layer=0"));
    assert!(text.contains("pass[1] id=1 name=write-right layer=0"));
    assert!(text.contains("name=write-unused layer=-"));
    assert!(text.contains("name=composite layer=1"));
}

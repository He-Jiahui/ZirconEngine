use super::*;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    RenderGraphResource,
};
use crate::rhi::{BufferDesc, BufferUsage};

#[test]
fn external_access_packet_preserves_live_access_identity_and_typed_descriptor() {
    let mut builder = RenderGraphBuilder::new("external-access-packet");
    let buffer = builder.import_present_external_buffer_with_binding(
        "external-buffer",
        BufferDesc::new("external-buffer", 256, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = builder.add_pass("external-writer", QueueLane::AsyncCompute);
    builder.write_storage_external(pass, buffer).unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let access_id = graph.access_id_at(pass, 0).unwrap();
    let entry = graph
        .external_access_packet()
        .access(access_id)
        .expect("live external access must be packetized");
    assert_eq!(entry.key.resource, RenderGraphResource::External(buffer));
    assert_eq!(entry.access_id, access_id);
    assert!(matches!(&entry.desc, RenderGraphResourceDesc::Buffer(_)));
    assert_eq!(graph.external_access_packet().accesses().len(), 1);
}

#[test]
fn external_access_packet_looks_up_multiple_resources_by_sorted_name() {
    let mut builder = RenderGraphBuilder::new("external-access-packet-multiple-resources");
    let zeta_desc = BufferDesc::new("zeta-buffer", 32, BufferUsage::STORAGE);
    let alpha_desc = BufferDesc::new("alpha-buffer", 64, BufferUsage::STORAGE);
    let mu_desc = BufferDesc::new("mu-buffer", 96, BufferUsage::STORAGE);
    let zeta_binding = RenderGraphExternalResourceBinding::required_buffer();
    let alpha_binding = RenderGraphExternalResourceBinding::report_only_buffer();
    let mu_binding = RenderGraphExternalResourceBinding::required_buffer();
    let zeta_buffer = builder.import_present_external_buffer_with_binding(
        "zeta-buffer",
        zeta_desc.clone(),
        zeta_binding,
    );
    let alpha_buffer = builder.import_present_external_buffer_with_binding(
        "alpha-buffer",
        alpha_desc.clone(),
        alpha_binding,
    );
    let mu_buffer = builder.import_present_external_buffer_with_binding(
        "mu-buffer",
        mu_desc.clone(),
        mu_binding,
    );

    let zeta_pass = builder.add_pass("zeta-writer", QueueLane::AsyncCompute);
    builder
        .write_storage_external(zeta_pass, zeta_buffer)
        .unwrap();
    builder
        .set_pass_flags(
            zeta_pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let alpha_pass = builder.add_pass("alpha-writer", QueueLane::AsyncCompute);
    builder
        .write_storage_external(alpha_pass, alpha_buffer)
        .unwrap();
    builder
        .set_pass_flags(
            alpha_pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let mu_pass = builder.add_pass("mu-writer", QueueLane::AsyncCompute);
    builder.write_storage_external(mu_pass, mu_buffer).unwrap();
    builder
        .set_pass_flags(
            mu_pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let expected = [
        (
            zeta_pass,
            RenderGraphResource::External(zeta_buffer),
            zeta_binding,
            RenderGraphResourceDesc::Buffer(zeta_desc),
        ),
        (
            alpha_pass,
            RenderGraphResource::External(alpha_buffer),
            alpha_binding,
            RenderGraphResourceDesc::Buffer(alpha_desc),
        ),
        (
            mu_pass,
            RenderGraphResource::External(mu_buffer),
            mu_binding,
            RenderGraphResourceDesc::Buffer(mu_desc),
        ),
    ];
    let expected_access_count = expected.len();

    for (pass, resource, binding, desc) in expected {
        let access_id = graph.access_id_at(pass, 0).unwrap();
        let entry = graph
            .external_access_packet()
            .access(access_id)
            .expect("each live external resource access must be packetized");
        assert_eq!(entry.access_id, access_id);
        assert_eq!(entry.key.resource, resource);
        assert_eq!(entry.binding, binding);
        assert_eq!(entry.desc, desc);
    }
    assert_eq!(
        graph.external_access_packet().accesses().len(),
        expected_access_count
    );
}

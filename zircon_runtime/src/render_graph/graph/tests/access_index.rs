use std::collections::HashMap;

use super::CompiledRenderGraphAccessIndex;
use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphError, RenderGraphExternalResourceBinding,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessId,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessMetadata, RenderGraphResourceAccessRange,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceUsageFlags, RenderGraphResourceVersion, RenderPassId, RgBufferHandle,
    RgTextureHandle,
};
use crate::rhi::{BufferDesc, BufferUsage};

fn compiled_pass(
    resources: Vec<RenderGraphPassResourceAccess>,
) -> super::super::CompiledRenderPass {
    super::super::CompiledRenderPass {
        id: RenderPassId::from_index(0, 1),
        name: "pass".to_owned(),
        declared_queue: QueueLane::Graphics,
        queue: QueueLane::Graphics,
        flags: PassFlags::default(),
        dependencies: Vec::new(),
        culled: false,
        executor_id: None,
        compute_workload: None,
        compute_pass_metadata: None,
        resources,
    }
}

#[test]
fn legacy_lookup_refuses_multiple_same_kind_accesses() {
    let pass = RenderPassId::from_index(0, 1);
    let resource = RenderGraphResource::TransientTexture(RgTextureHandle::from_index(0, 1));
    let mut index = CompiledRenderGraphAccessIndex::default();

    index.record_legacy_access(
        RenderGraphResourceAccessId::new(pass, 0),
        resource,
        RenderGraphResourceAccessKind::Write,
    );
    index.record_legacy_access(
        RenderGraphResourceAccessId::new(pass, 1),
        resource,
        RenderGraphResourceAccessKind::Write,
    );

    assert_eq!(
        index.access_id_for(pass, resource, RenderGraphResourceAccessKind::Write),
        None
    );
}

#[test]
fn access_index_rejects_missing_compiler_table_rows() {
    let pass = compiled_pass(Vec::new());

    let error = CompiledRenderGraphAccessIndex::new(&[pass], &[], &HashMap::new(), &[], &[], &[])
        .expect_err("each compiler table needs one row for every compiled pass");

    assert!(matches!(
        error,
        RenderGraphError::CompiledAccessTablePassCountMismatch {
            table: "produced versions",
            expected: 1,
            actual: 0,
        }
    ));
}

#[test]
fn access_index_rejects_per_pass_access_table_cardinality_drift() {
    let pass = compiled_pass(vec![RenderGraphPassResourceAccess {
        name: "resource".to_owned(),
        kind: RenderGraphResourceKind::TransientTexture,
        access: RenderGraphResourceAccessKind::Write,
        attachment_ops: None,
    }]);

    let error = CompiledRenderGraphAccessIndex::new(
        &[pass],
        &[],
        &HashMap::new(),
        &[Vec::new()],
        &[Vec::new()],
        &[Vec::new()],
    )
    .expect_err("each compiler table row must align with compiled pass accesses");

    assert!(matches!(
        error,
        RenderGraphError::CompiledAccessTableAccessCountMismatch {
            table: "produced versions",
            ref pass,
            expected: 1,
            actual: 0,
        } if pass == "pass"
    ));
}

#[test]
fn access_index_rejects_compiled_access_kind_that_conflicts_with_declaration() {
    let resource = RenderGraphResource::TransientBuffer(RgBufferHandle::from_index(0, 1));
    let pass = compiled_pass(vec![RenderGraphPassResourceAccess {
        name: "resource".to_owned(),
        kind: RenderGraphResourceKind::TransientTexture,
        access: RenderGraphResourceAccessKind::Write,
        attachment_ops: None,
    }]);
    let declaration = RenderGraphResourceDeclaration {
        resource,
        name: "resource".to_owned(),
        kind: RenderGraphResourceKind::TransientBuffer,
        desc: RenderGraphResourceDesc::Buffer(BufferDesc::new(
            "resource",
            16,
            BufferUsage::STORAGE,
        )),
        external_binding: RenderGraphExternalResourceBinding::report_only(),
        external_texture_desc: None,
        external_buffer_desc: None,
        texture_view_alias: None,
        imported: false,
        usage: RenderGraphResourceUsageFlags::default(),
    };
    let metadata = RenderGraphResourceAccessMetadata::new(
        RenderGraphResourceAccessRange::UnresolvedExternal,
        RenderGraphResourceAccessIntent::Legacy,
    );

    let error = CompiledRenderGraphAccessIndex::new(
        &[pass],
        &[declaration],
        &HashMap::from([("resource".to_owned(), 0)]),
        &[vec![RenderGraphResourceVersion::new(resource, 1)]],
        &[vec![None]],
        &[vec![metadata]],
    )
    .expect_err("compiled access kind must match the named resource declaration");

    assert!(matches!(
        error,
        RenderGraphError::CompiledAccessResourceKindMismatch {
            ref pass,
            ref resource,
            access_kind: RenderGraphResourceKind::TransientTexture,
            declaration_kind: RenderGraphResourceKind::TransientBuffer,
        } if pass == "pass" && resource == "resource"
    ));
}

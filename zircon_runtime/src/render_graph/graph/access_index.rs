use std::collections::HashMap;

use super::super::access::{
    RenderGraphResourceAccessId, RenderGraphResourceAccessMetadata, RenderGraphVersionedAccessKey,
};
use super::super::error::RenderGraphError;
use super::super::types::{
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceVersion, RenderPassId,
};
use super::CompiledRenderPass;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum LegacyAccessKind {
    Read,
    Write,
}

impl From<RenderGraphResourceAccessKind> for LegacyAccessKind {
    fn from(access: RenderGraphResourceAccessKind) -> Self {
        match access {
            RenderGraphResourceAccessKind::Read => Self::Read,
            RenderGraphResourceAccessKind::Write => Self::Write,
        }
    }
}

type LegacyAccessKey = (RenderPassId, RenderGraphResource, LegacyAccessKind);

/// Immutable index for access-level compiler facts.
///
/// Access IDs are the authority. The legacy pass/resource/kind lookup is
/// deliberately retained as `Option` only while its key resolves to one row.
/// This prevents a future multi-range pass from silently selecting the last
/// same-kind declaration inserted into a hash map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CompiledRenderGraphAccessIndex {
    positions: HashMap<RenderGraphResourceAccessId, (usize, usize)>,
    legacy_access_ids: HashMap<LegacyAccessKey, Option<RenderGraphResourceAccessId>>,
    metadata: HashMap<RenderGraphResourceAccessId, RenderGraphResourceAccessMetadata>,
    produced_versions: HashMap<RenderGraphResourceAccessId, RenderGraphResourceVersion>,
    input_versions: HashMap<RenderGraphResourceAccessId, RenderGraphResourceVersion>,
    versioned_access_keys: HashMap<RenderGraphResourceAccessId, RenderGraphVersionedAccessKey>,
    ordered_versioned_access_keys: Vec<RenderGraphVersionedAccessKey>,
}

impl CompiledRenderGraphAccessIndex {
    pub(super) fn new(
        passes: &[CompiledRenderPass],
        resource_declarations: &[RenderGraphResourceDeclaration],
        resource_declaration_indices_by_name: &HashMap<String, usize>,
        pass_resource_versions: &[Vec<RenderGraphResourceVersion>],
        pass_resource_input_versions: &[Vec<Option<RenderGraphResourceVersion>>],
        pass_resource_access_metadata: &[Vec<RenderGraphResourceAccessMetadata>],
    ) -> Result<Self, RenderGraphError> {
        validate_table_shape("produced versions", passes, pass_resource_versions)?;
        validate_table_shape("input versions", passes, pass_resource_input_versions)?;
        validate_table_shape("access metadata", passes, pass_resource_access_metadata)?;

        let mut index = Self::default();

        for (pass_index, pass) in passes.iter().enumerate() {
            for (access_index, access) in pass.resources.iter().enumerate() {
                let declaration_index = *resource_declaration_indices_by_name
                    .get(access.name.as_str())
                    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                        resource: access.name.clone(),
                    })?;
                let declaration =
                    resource_declarations
                        .get(declaration_index)
                        .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                            resource: access.name.clone(),
                        })?;
                if declaration.kind != access.kind {
                    return Err(RenderGraphError::CompiledAccessResourceKindMismatch {
                        pass: pass.name.clone(),
                        resource: access.name.clone(),
                        access_kind: access.kind,
                        declaration_kind: declaration.kind,
                    });
                }

                let access_id = RenderGraphResourceAccessId::new(pass.id, access_index);
                index
                    .positions
                    .insert(access_id, (pass_index, access_index));
                index.record_legacy_access(access_id, declaration.resource, access.access);

                let metadata = pass_resource_access_metadata[pass_index][access_index];
                index.metadata.insert(access_id, metadata);
                let produced_version = pass_resource_versions[pass_index][access_index];
                index.produced_versions.insert(access_id, produced_version);
                if let Some(input_version) = pass_resource_input_versions[pass_index][access_index]
                {
                    index.input_versions.insert(access_id, input_version);
                }
                let binding_version = match access.access {
                    RenderGraphResourceAccessKind::Read => pass_resource_input_versions[pass_index]
                        [access_index]
                        .unwrap_or(produced_version),
                    // An attachment Load consumes the input version but this access also
                    // produces a successor. The physical key for a write must identify
                    // that successor; input_versions retains the load dependency.
                    RenderGraphResourceAccessKind::Write => produced_version,
                };
                let versioned_access_key = RenderGraphVersionedAccessKey::new(
                    access_id,
                    declaration.resource,
                    access.access,
                    binding_version,
                    metadata,
                );
                index
                    .versioned_access_keys
                    .insert(access_id, versioned_access_key);
                if !pass.culled {
                    index
                        .ordered_versioned_access_keys
                        .push(versioned_access_key);
                }
            }
        }

        Ok(index)
    }

    pub(super) fn access_id_at(
        &self,
        pass: RenderPassId,
        access_index: usize,
    ) -> Option<RenderGraphResourceAccessId> {
        let access_id = RenderGraphResourceAccessId::new(pass, access_index);
        self.positions.contains_key(&access_id).then_some(access_id)
    }

    pub(super) fn access_id_for(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceAccessId> {
        self.legacy_access_ids
            .get(&(pass, resource, access.into()))
            .copied()
            .flatten()
    }

    pub(super) fn pass_resource_access<'a>(
        &self,
        passes: &'a [CompiledRenderPass],
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<&'a RenderGraphPassResourceAccess> {
        let access_id = self.access_id_for(pass, resource, access)?;
        let (pass_index, access_index) = self.positions.get(&access_id)?;
        passes
            .get(*pass_index)
            .and_then(|compiled_pass| compiled_pass.resources.get(*access_index))
    }

    pub(super) fn metadata(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResourceAccessMetadata> {
        self.metadata.get(&access).copied()
    }

    pub(super) fn produced_version(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResourceVersion> {
        self.produced_versions.get(&access).copied()
    }

    pub(super) fn input_version(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResourceVersion> {
        self.input_versions.get(&access).copied()
    }

    pub(super) fn versioned_access_key(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphVersionedAccessKey> {
        self.versioned_access_keys.get(&access).copied()
    }

    pub(super) fn versioned_access_keys(&self) -> &[RenderGraphVersionedAccessKey] {
        &self.ordered_versioned_access_keys
    }

    fn record_legacy_access(
        &mut self,
        access_id: RenderGraphResourceAccessId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) {
        let key = (access_id.pass(), resource, access.into());
        if let Some(existing) = self.legacy_access_ids.get_mut(&key) {
            *existing = None;
        } else {
            self.legacy_access_ids.insert(key, Some(access_id));
        }
    }
}

fn validate_table_shape<T>(
    table: &'static str,
    passes: &[CompiledRenderPass],
    rows: &[Vec<T>],
) -> Result<(), RenderGraphError> {
    if rows.len() != passes.len() {
        return Err(RenderGraphError::CompiledAccessTablePassCountMismatch {
            table,
            expected: passes.len(),
            actual: rows.len(),
        });
    }
    for (pass, row) in passes.iter().zip(rows) {
        if row.len() != pass.resources.len() {
            return Err(RenderGraphError::CompiledAccessTableAccessCountMismatch {
                table,
                pass: pass.name.clone(),
                expected: pass.resources.len(),
                actual: row.len(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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

        let error =
            CompiledRenderGraphAccessIndex::new(&[pass], &[], &HashMap::new(), &[], &[], &[])
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
}

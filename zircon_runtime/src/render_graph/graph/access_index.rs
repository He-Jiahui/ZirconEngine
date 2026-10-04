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
                // 读绑定消费输入版本，写绑定指向新产生的 successor；两者都保留同一个 access id。
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

    pub(super) fn resource_for_access(
        &self,
        access: RenderGraphResourceAccessId,
    ) -> Option<RenderGraphResource> {
        self.versioned_access_keys
            .get(&access)
            .map(|key| key.resource)
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
#[path = "tests/access_index.rs"]
mod tests;

use super::access_index::CompiledRenderGraphAccessIndex;
use super::CompiledRenderPass;
use crate::render_graph::{
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessId, RenderGraphResourceDesc,
    RenderGraphResourceKind, RenderGraphResourceLifetime, RenderGraphVersionedAccessKey,
};

/// Compiler-owned identity for one live imported-resource access.
///
/// The packet deliberately contains no WGPU object. It records the exact
/// access identity and the producer-declared physical contract that a frame
/// lease must satisfy before an executor can encode commands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphExternalAccess {
    pub access_id: RenderGraphResourceAccessId,
    pub key: RenderGraphVersionedAccessKey,
    pub binding: RenderGraphExternalResourceBinding,
    pub desc: RenderGraphResourceDesc,
}

/// Immutable compiler-to-executor packet for live external accesses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphExternalAccessPacket {
    accesses: Vec<CompiledRenderGraphExternalAccess>,
}

impl CompiledRenderGraphExternalAccessPacket {
    pub fn accesses(&self) -> &[CompiledRenderGraphExternalAccess] {
        &self.accesses
    }

    pub fn access(
        &self,
        access_id: RenderGraphResourceAccessId,
    ) -> Option<&CompiledRenderGraphExternalAccess> {
        self.accesses
            .iter()
            .find(|access| access.access_id == access_id)
    }
}

pub(super) fn build_external_access_packet(
    passes: &[CompiledRenderPass],
    access_index: &CompiledRenderGraphAccessIndex,
    resource_lifetimes: &[RenderGraphResourceLifetime],
) -> Result<CompiledRenderGraphExternalAccessPacket, String> {
    let mut accesses = Vec::new();
    for pass in passes {
        if pass.culled {
            continue;
        }
        for (access_index_in_pass, access) in pass.resources.iter().enumerate() {
            if access.kind != RenderGraphResourceKind::External {
                continue;
            }
            let access_id = RenderGraphResourceAccessId::new(pass.id, access_index_in_pass);
            let key = access_index.versioned_access_key(access_id).ok_or_else(|| {
                format!(
                    "compiled external access packet is missing versioned key for pass `{}` at access ordinal {access_index_in_pass}",
                    pass.name
                )
            })?;
            // Resource lifetimes are sorted by unique declaration name during
            // compilation. Keep resource identity authoritative after the lookup.
            let lifetime = resource_lifetimes
                .binary_search_by(|lifetime| lifetime.name.as_str().cmp(access.name.as_str()))
                .ok()
                .and_then(|index| resource_lifetimes.get(index))
                .filter(|lifetime| lifetime.resource == key.resource)
                .ok_or_else(|| {
                    format!(
                        "compiled external access packet cannot find lifetime for pass `{}` access `{}`",
                        pass.name, access.name
                    )
                })?;
            accesses.push(CompiledRenderGraphExternalAccess {
                access_id,
                key,
                binding: lifetime.external_binding,
                desc: match (
                    &lifetime.external_texture_desc,
                    &lifetime.external_buffer_desc,
                ) {
                    (Some(desc), _) => RenderGraphResourceDesc::Texture(desc.clone()),
                    (_, Some(desc)) => RenderGraphResourceDesc::Buffer(desc.clone()),
                    _ => RenderGraphResourceDesc::External,
                },
            });
        }
    }
    Ok(CompiledRenderGraphExternalAccessPacket { accesses })
}

#[cfg(test)]
#[path = "tests/external_access_packet.rs"]
mod tests;

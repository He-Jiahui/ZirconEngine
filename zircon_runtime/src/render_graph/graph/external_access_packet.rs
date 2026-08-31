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
            let lifetime = resource_lifetimes
                .iter()
                .find(|lifetime| lifetime.resource == key.resource)
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
mod tests {
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
}

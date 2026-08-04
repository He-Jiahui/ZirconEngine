use crate::render_graph::{
    ExternalResource, PassFlags, QueueLane, RenderGraphBuilder, RenderGraphError, RenderPassId,
};

pub(crate) const RUNTIME_MIP_GEN_EXECUTOR_ID: &str = "texture.mip-gen";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeMipGenGraphInsertion {
    pass: RenderPassId,
    source_writer: RenderPassId,
}

impl RuntimeMipGenGraphInsertion {
    pub(crate) const fn pass(&self) -> RenderPassId {
        self.pass
    }

    pub(crate) const fn source_writer(&self) -> RenderPassId {
        self.source_writer
    }
}

/// Inserts a mip writer after the final producer of a texture resource.
pub(crate) fn insert_runtime_mipgen_after_last_writer(
    graph: &mut RenderGraphBuilder,
    texture: ExternalResource,
    texture_name: &str,
    last_writer: RenderPassId,
) -> Result<RuntimeMipGenGraphInsertion, RenderGraphError> {
    let pass = graph.add_pass_with_executor(
        format!("mip-gen:{texture_name}"),
        QueueLane::AsyncCompute,
        Some(RUNTIME_MIP_GEN_EXECUTOR_ID),
    );
    graph.set_pass_flags(
        pass,
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )?;
    graph.add_dependency(last_writer, pass)?;
    // The graph currently tracks texture-wide resources, while this executor reads one mip view
    // and writes non-overlapping higher views of the same texture. Record the conservative write.
    graph.write_storage_external(pass, texture)?;

    Ok(RuntimeMipGenGraphInsertion {
        pass,
        source_writer: last_writer,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_mipgen_graph_node_follows_the_last_writer() {
        let mut graph = RenderGraphBuilder::new("runtime-mipgen");
        let texture = graph.import_external_resource("runtime-texture");
        let writer = graph.add_pass("texture-producer", QueueLane::Graphics);
        graph
            .write_external(writer, texture)
            .expect("producer writes the imported texture");
        let insertion =
            insert_runtime_mipgen_after_last_writer(&mut graph, texture, "runtime-texture", writer)
                .expect("mip node is inserted after producer");
        let compiled = graph.compile().expect("runtime mip graph compiles");
        let mip_pass = compiled
            .passes()
            .iter()
            .find(|pass| pass.id == insertion.pass())
            .expect("compiled graph retains mip pass");

        assert_eq!(insertion.source_writer(), writer);
        assert!(mip_pass.dependencies.contains(&writer));
        assert_eq!(mip_pass.queue, QueueLane::AsyncCompute);
        assert_eq!(
            mip_pass.executor_id.as_deref(),
            Some(RUNTIME_MIP_GEN_EXECUTOR_ID)
        );
        assert_eq!(mip_pass.resources.len(), 1);
    }
}

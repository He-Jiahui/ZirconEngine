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
// TODO: [CR-GRAPHICS-AUX-B-0002] 资源上传现直接录制 mip 命令，尚未找到此 helper 的生产调用；
// 默认注册表不含该 ID，但支持外部注入；需确认启用入口及 executor/资源绑定，再验证图集成。
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
#[path = "tests/graph_insertion.rs"]
mod tests;

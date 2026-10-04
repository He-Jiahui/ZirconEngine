use crate::core::framework::render::RenderViewportPickPolicy;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::super::{
    hit_proxy_command_spec, MeshBatchRef, MeshDrawCommandList, MeshPassBuildContext,
    MeshPassProcessor,
};

pub(crate) struct HitProxyPassProcessor {
    policy: RenderViewportPickPolicy,
}

impl HitProxyPassProcessor {
    pub(crate) const fn new(policy: RenderViewportPickPolicy) -> Self {
        Self { policy }
    }
}

impl MeshPassProcessor for HitProxyPassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized,
    {
        let Some(spec) = hit_proxy_command_spec(batch, self.policy) else {
            return;
        };
        // 拾取策略只改写本次请求副本的 pipeline key，不改变共享批次的材质身份。
        let mut request_batch = batch.clone();
        if self.policy.includes_backfaces() {
            request_batch.pipeline_key.double_sided = true;
        }
        let pipeline_variant_id = context.pipeline_variant_id(spec.pipeline_kind, &request_batch);
        out.push(request_batch.command(spec.phase, spec.pipeline_kind, pipeline_variant_id));
    }
}

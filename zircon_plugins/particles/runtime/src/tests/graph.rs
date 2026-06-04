use zircon_runtime::graphics::{
    RenderFeatureResourceAccess, RenderPassExecutionContext, RenderPassExecutorId,
};
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
};

use crate::{particle_render_pass_executor_registrations, render_feature_descriptor};

use super::support::graph_resource_kind;

#[test]
fn particle_graph_indirect_executor_accepts_declared_resource_contract() {
    let registration = particle_render_pass_executor_registrations()
        .into_iter()
        .find(|registration| registration.executor_id().as_str() == "particle.gpu.indirect-args")
        .expect("particle indirect args executor should be registered");
    let pass = &render_feature_descriptor().stage_passes[2];
    let mut context = RenderPassExecutionContext::with_declared_graph_metadata_and_resources(
        "particle-gpu-build-indirect-args",
        RenderPassExecutorId::new("particle.gpu.indirect-args"),
        QueueLane::Graphics,
        QueueLane::AsyncCompute,
        PassFlags {
            allow_culling: true,
            has_side_effects: true,
        },
        pass.resources
            .iter()
            .map(|resource| RenderGraphPassResourceAccess {
                name: resource.name.clone(),
                kind: graph_resource_kind(resource.kind),
                access: match resource.access {
                    RenderFeatureResourceAccess::Read => RenderGraphResourceAccessKind::Read,
                    RenderFeatureResourceAccess::Write => RenderGraphResourceAccessKind::Write,
                },
                attachment_ops: None,
            })
            .collect(),
    );

    registration.execute(&mut context).unwrap();
}

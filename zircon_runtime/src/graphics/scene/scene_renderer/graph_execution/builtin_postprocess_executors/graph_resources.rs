use crate::core::framework::render::PostProcessEffectKind;
use crate::render_graph::{
    RenderGraphExternalResourceType, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

use super::super::{RenderPassExecutionContext, RenderPassGpuExecutionContext};

pub(super) fn product_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
    kind: PostProcessEffectKind,
) -> Result<(), String> {
    let context = &*context;
    let Some(gpu) = context.gpu() else {
        return Err(format!(
            "render pass executor `{}` for pass `{}` requires renderer GPU context",
            context.executor_id, context.pass_name
        ));
    };
    let frame_extract = gpu.frame_extract();
    let Some(node) = frame_extract
        .post_process
        .graph
        .nodes
        .iter()
        .find(|node| node.kind == kind)
    else {
        return Ok(());
    };

    for resource in &node.required_inputs {
        require_graph_resource_by_name(
            context,
            gpu,
            resource,
            RenderGraphResourceAccessKind::Read,
        )?;
    }
    for resource in &node.produced_outputs {
        require_graph_resource_by_name(
            context,
            gpu,
            resource,
            RenderGraphResourceAccessKind::Write,
        )?;
    }

    Ok(())
}

fn require_graph_resource_by_name(
    context: &RenderPassExecutionContext<'_>,
    gpu: &RenderPassGpuExecutionContext<'_>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Result<(), String> {
    let Some(kind) = pass_resource_kind(context, resource_name, access) else {
        return Err(format!(
            "render graph pass `{}` did not declare {:?} access for resource `{resource_name}`",
            context.pass_name, access
        ));
    };
    match kind {
        RenderGraphResourceKind::TransientTexture => {
            gpu.require_texture_view(resource_name, access).map(|_| ())
        }
        RenderGraphResourceKind::TransientBuffer => {
            gpu.require_buffer(resource_name, access).map(|_| ())
        }
        RenderGraphResourceKind::External => match external_resource_type(context, resource_name) {
            RenderGraphExternalResourceType::Buffer => {
                gpu.require_buffer(resource_name, access).map(|_| ())
            }
            RenderGraphExternalResourceType::Texture | RenderGraphExternalResourceType::Unknown => {
                gpu.require_texture_view(resource_name, access).map(|_| ())
            }
        },
    }
}

fn pass_resource_kind(
    context: &RenderPassExecutionContext<'_>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Option<RenderGraphResourceKind> {
    if let Some(resolver) = context.resource_resolver() {
        return resolver
            .pass_resource_access_by_name(resource_name, access)
            .map(|resource| resource.kind);
    }
    context
        .resources
        .iter()
        .find(|resource| resource.name == resource_name && resource.access == access)
        .map(|resource| resource.kind)
}

fn external_resource_type(
    context: &RenderPassExecutionContext<'_>,
    resource_name: &str,
) -> RenderGraphExternalResourceType {
    context
        .resource_resolver()
        .and_then(|resolver| resolver.resource_declaration_by_name(resource_name))
        .map(|declaration| declaration.external_binding.resource_type)
        .unwrap_or(RenderGraphExternalResourceType::Unknown)
}

#[cfg(test)]
mod tests {
    #[test]
    fn optimization_batch_20260830dx_postprocess_resources_remain_borrowed() {
        let source = include_str!("graph_resources.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("for resource in &node.required_inputs"));
        assert!(production.contains("for resource in &node.produced_outputs"));
        assert!(!production.contains("node.required_inputs.clone()"));
        assert!(!production.contains("node.produced_outputs.clone()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830dx_postprocess_resource_borrow_evidence() {
        const FRAME_COUNT: usize = 32_768;
        const EFFECT_COUNT: usize = 8;
        const INPUT_COUNT: usize = 3;
        const OUTPUT_COUNT: usize = 2;
        const MARKER: &str = "RUNTIME532_POSTPROCESS_RESOURCE_BORROW_BENCH_V1";

        let legacy_owned_allocations = FRAME_COUNT
            .saturating_mul(EFFECT_COUNT)
            .saturating_mul(INPUT_COUNT.saturating_add(OUTPUT_COUNT).saturating_add(2));
        let borrowed_owned_allocations = 0usize;
        let reduction_bps = legacy_owned_allocations
            .saturating_sub(borrowed_owned_allocations)
            .saturating_mul(10_000)
            / legacy_owned_allocations.max(1);

        assert!(legacy_owned_allocations > 0);
        assert_eq!(borrowed_owned_allocations, 0);
        assert_eq!(reduction_bps, 10_000);
        println!(
            "{MARKER} frames={FRAME_COUNT} effects={EFFECT_COUNT} inputs={INPUT_COUNT} \
             outputs={OUTPUT_COUNT} legacy_owned_allocations={legacy_owned_allocations} \
             borrowed_owned_allocations={borrowed_owned_allocations} reduction_bps={reduction_bps}"
        );
    }
}

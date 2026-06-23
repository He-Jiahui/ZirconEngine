use crate::core::framework::render::PostProcessEffectKind;
use crate::render_graph::{
    RenderGraphExternalResourceType, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

use super::super::RenderPassExecutionContext;

pub(super) fn product_postprocess_executor(
    context: &mut RenderPassExecutionContext<'_>,
    kind: PostProcessEffectKind,
) -> Result<(), String> {
    let (required_inputs, produced_outputs) = {
        let gpu = context.require_gpu()?;
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
        (node.required_inputs.clone(), node.produced_outputs.clone())
    };

    for resource in required_inputs {
        require_graph_resource_by_name(context, &resource, RenderGraphResourceAccessKind::Read)?;
    }
    for resource in produced_outputs {
        require_graph_resource_by_name(context, &resource, RenderGraphResourceAccessKind::Write)?;
    }

    Ok(())
}

fn require_graph_resource_by_name(
    context: &mut RenderPassExecutionContext<'_>,
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
        RenderGraphResourceKind::TransientTexture => context
            .require_texture_view_by_name(resource_name, access)
            .map(|_| ()),
        RenderGraphResourceKind::TransientBuffer => context
            .require_buffer_by_name(resource_name, access)
            .map(|_| ()),
        RenderGraphResourceKind::External => match external_resource_type(context, resource_name) {
            RenderGraphExternalResourceType::Buffer => context
                .require_buffer_by_name(resource_name, access)
                .map(|_| ()),
            RenderGraphExternalResourceType::Texture | RenderGraphExternalResourceType::Unknown => {
                context
                    .require_texture_view_by_name(resource_name, access)
                    .map(|_| ())
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

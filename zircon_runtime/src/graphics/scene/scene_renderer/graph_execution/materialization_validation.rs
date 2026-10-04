use crate::core::framework::render::RenderGraphMaterializationReport;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceDesc,
    RenderGraphResourceLifetime,
};

use super::render_graph_execution_resources::RenderGraphExecutionResources;

pub(super) fn validate_materialized_graph_resources(
    resources: &RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<RenderGraphMaterializationReport, String> {
    let mut report = RenderGraphMaterializationReport::default();
    let mut missing_materialized = Vec::new();
    let mut missing_required_external = Vec::new();
    let mut external_contract_violations = Vec::new();

    for lifetime in graph.resource_lifetimes() {
        match &lifetime.desc {
            RenderGraphResourceDesc::Texture(desc) if desc.is_sparse_reserved() => {
                report.sparse_texture_reservation_count += 1;
            }
            RenderGraphResourceDesc::Texture(_) => {
                report.required_texture_count += 1;
                if resources.has_texture_view(&lifetime.name) {
                    report.bound_texture_count += 1;
                } else {
                    report.missing_texture_count += 1;
                    missing_materialized.push(format!("texture `{}`", lifetime.name));
                }
            }
            RenderGraphResourceDesc::Buffer(_) => {
                report.required_buffer_count += 1;
                if resources.has_buffer(&lifetime.name) {
                    report.bound_buffer_count += 1;
                } else {
                    report.missing_buffer_count += 1;
                    missing_materialized.push(format!("buffer `{}`", lifetime.name));
                }
            }
            RenderGraphResourceDesc::External => {
                let is_bound = has_bound_external_lifetime(resources, lifetime);
                if is_bound {
                    if let Some(expected) = lifetime.external_texture_desc.as_ref() {
                        match resources.physical_texture_desc(&lifetime.name) {
                            Some(actual) => {
                                if let Some(error) = external_texture_contract_error(
                                    &lifetime.name,
                                    expected,
                                    actual,
                                ) {
                                    external_contract_violations.push(error);
                                }
                            }
                            None => external_contract_violations.push(format!(
                                "external texture `{}` is missing its physical descriptor",
                                lifetime.name
                            )),
                        }
                    }
                    if let Some(expected) = lifetime.external_buffer_desc.as_ref() {
                        match resources.physical_buffer_desc(&lifetime.name) {
                            Some(actual) => {
                                if let Some(error) =
                                    external_buffer_contract_error(&lifetime.name, expected, actual)
                                {
                                    external_contract_violations.push(error);
                                }
                                if let Some(error) = external_buffer_backing_size_error(
                                    &lifetime.name,
                                    actual,
                                    resources.physical_buffer_size(&lifetime.name),
                                ) {
                                    external_contract_violations.push(error);
                                }
                            }
                            None => external_contract_violations.push(format!(
                                "external buffer `{}` is missing its physical descriptor",
                                lifetime.name
                            )),
                        }
                    }
                }
                if lifetime.external_binding.is_required() {
                    report.required_external_count += 1;
                    if is_bound {
                        report.bound_required_external_count += 1;
                    } else {
                        report.missing_required_external_count += 1;
                        missing_required_external.push(format!(
                            "{} `{}`",
                            lifetime.external_binding.label(),
                            lifetime.name
                        ));
                    }
                } else {
                    report.report_only_external_count += 1;
                    if is_bound {
                        report.bound_report_only_external_count += 1;
                    } else {
                        report.missing_report_only_external_count += 1;
                    }
                }
            }
        }
    }

    let stale_texture_bindings = resources
        .bound_texture_view_names()
        .filter(|name| graph.resource_lifetime_by_name(name).is_none())
        .collect::<Vec<_>>();
    let stale_buffer_bindings = resources
        .bound_buffer_names()
        .filter(|name| graph.resource_lifetime_by_name(name).is_none())
        .collect::<Vec<_>>();
    report.stale_texture_binding_count = stale_texture_bindings.len();
    report.stale_buffer_binding_count = stale_buffer_bindings.len();

    if report.stale_binding_count() > 0 {
        let mut stale_bindings = stale_texture_bindings
            .iter()
            .map(|name| format!("texture `{name}`"))
            .chain(
                stale_buffer_bindings
                    .iter()
                    .map(|name| format!("buffer `{name}`")),
            )
            .collect::<Vec<_>>();
        stale_bindings.sort();
        return Err(format!(
            "render graph materialization has {} stale resource bindings outside live compiled lifetimes: {}",
            report.stale_binding_count(),
            stale_bindings.join(", ")
        ));
    }

    if !external_contract_violations.is_empty() {
        return Err(format!(
            "render graph materialization has {} external resource contract violation(s): {}",
            external_contract_violations.len(),
            external_contract_violations.join(", ")
        ));
    }

    if !missing_required_external.is_empty() {
        return Err(format!(
            "render graph materialization missing {} required external resource bindings: {}",
            missing_required_external.len(),
            missing_required_external.join(", ")
        ));
    }

    if report.materialized_resources_complete() {
        return Ok(report);
    }

    Err(format!(
        "render graph materialization missing {} typed resource bindings: {}",
        report.missing_materialized_resource_count(),
        missing_materialized.join(", ")
    ))
}

fn external_texture_contract_error(
    name: &str,
    expected: &crate::rhi::TextureDesc,
    actual: &crate::rhi::TextureDesc,
) -> Option<String> {
    if expected.format != actual.format {
        return Some(format!(
            "external texture `{name}` physical descriptor does not satisfy its compiled contract: format expected {:?}, got {:?}",
            expected.format, actual.format
        ));
    }
    if expected.width != actual.width
        || expected.height != actual.height
        || expected.depth != actual.depth
        || expected.array_layers != actual.array_layers
    {
        return Some(format!(
            "external texture `{name}` physical descriptor does not satisfy its compiled contract: extent expected {}x{}x{} layers {}, got {}x{}x{} layers {}",
            expected.width,
            expected.height,
            expected.depth,
            expected.array_layers,
            actual.width,
            actual.height,
            actual.depth,
            actual.array_layers
        ));
    }
    if expected.dimension != actual.dimension {
        return Some(format!(
            "external texture `{name}` physical descriptor does not satisfy its compiled contract: dimension expected {:?}, got {:?}",
            expected.dimension, actual.dimension
        ));
    }
    if expected.mip_levels != actual.mip_levels {
        return Some(format!(
            "external texture `{name}` physical descriptor does not satisfy its compiled contract: mip levels expected {}, got {}",
            expected.mip_levels, actual.mip_levels
        ));
    }
    if expected.sample_count != actual.sample_count {
        return Some(format!(
            "external texture `{name}` physical descriptor does not satisfy its compiled contract: sample count expected {}, got {}",
            expected.sample_count, actual.sample_count
        ));
    }
    if !actual.usage.contains(expected.usage) {
        return Some(format!(
            "external texture `{name}` physical descriptor does not satisfy its compiled contract: usage {:?} does not include {:?}",
            actual.usage, expected.usage
        ));
    }
    None
}

fn external_buffer_contract_error(
    name: &str,
    expected: &crate::rhi::BufferDesc,
    actual: &crate::rhi::BufferDesc,
) -> Option<String> {
    if actual.size_bytes < expected.size_bytes {
        return Some(format!(
            "external buffer `{name}` physical descriptor does not satisfy its compiled contract: size expected at least {}, got {}",
            expected.size_bytes, actual.size_bytes
        ));
    }
    if !actual.usage.contains(expected.usage) {
        return Some(format!(
            "external buffer `{name}` physical descriptor does not satisfy its compiled contract: usage {:?} does not include {:?}",
            actual.usage, expected.usage
        ));
    }
    None
}

fn external_buffer_backing_size_error(
    name: &str,
    physical_desc: &crate::rhi::BufferDesc,
    physical_buffer_size: Option<wgpu::BufferAddress>,
) -> Option<String> {
    let physical_buffer_size = physical_buffer_size?;
    (physical_buffer_size < physical_desc.size_bytes).then(|| {
        format!(
            "external buffer `{name}` physical WGPU buffer size {physical_buffer_size} is smaller than its supplied physical descriptor size {}",
            physical_desc.size_bytes
        )
    })
}

fn has_bound_external_lifetime(
    resources: &RenderGraphExecutionResources,
    lifetime: &RenderGraphResourceLifetime,
) -> bool {
    match lifetime.external_binding.resource_type {
        RenderGraphExternalResourceType::Unknown => resources.has_bound_resource(&lifetime.name),
        RenderGraphExternalResourceType::Texture => resources.has_texture_view(&lifetime.name),
        RenderGraphExternalResourceType::Buffer => resources.has_buffer(&lifetime.name),
    }
}

#[cfg(test)]
#[path = "tests/materialization_validation.rs"]
mod tests;

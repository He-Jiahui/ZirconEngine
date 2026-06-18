use std::collections::BTreeSet;

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
    let live_lifetime_names = graph
        .resource_lifetimes()
        .iter()
        .map(|lifetime| lifetime.name.as_str())
        .collect::<BTreeSet<_>>();

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
        .filter(|name| !live_lifetime_names.contains(*name))
        .collect::<Vec<_>>();
    let stale_buffer_bindings = resources
        .bound_buffer_names()
        .filter(|name| !live_lifetime_names.contains(*name))
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
mod tests {
    use super::RenderGraphExecutionResources;
    use crate::graphics::backend::RenderBackend;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    };
    use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

    #[test]
    fn materialization_validation_reports_unbound_compiled_lifetimes() {
        let mut builder = RenderGraphBuilder::new("unbound-materialization");
        let texture = builder.create_texture(TextureDesc::new(
            "scene-color",
            16,
            16,
            TextureFormat::Rgba8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        ));
        let buffer = builder.create_buffer(BufferDesc::new(
            "light-list",
            64,
            BufferUsage::STORAGE | BufferUsage::COPY_DST,
        ));
        let output = builder.import_external_resource("viewport-output");
        let pass = builder.add_pass("write", QueueLane::Graphics);
        builder.write_texture(pass, texture).unwrap();
        builder.write_buffer(pass, buffer).unwrap();
        builder.write_external(pass, output).unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();

        let error = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap_err();

        assert!(error.contains("buffer `light-list`"));
        assert!(error.contains("texture `scene-color`"));
    }

    #[test]
    fn materialization_validation_reports_unbound_external_lifetimes_without_failing() {
        let mut builder = RenderGraphBuilder::new("external-materialization-report");
        let output = builder.import_external_resource("viewport-output");
        let pass = builder.add_pass("write", QueueLane::Graphics);
        builder.write_external(pass, output).unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap();

        assert_eq!(report.required_external_count, 0);
        assert_eq!(report.bound_required_external_count, 0);
        assert_eq!(report.missing_required_external_count, 0);
        assert_eq!(report.report_only_external_count, 1);
        assert_eq!(report.bound_report_only_external_count, 0);
        assert_eq!(report.missing_report_only_external_count, 1);
        assert_eq!(report.external_count(), 1);
        assert_eq!(report.bound_external_count(), 0);
        assert_eq!(report.missing_external_count(), 1);
        assert!(report.materialized_resources_complete());
        assert!(!report.is_complete());
    }

    #[test]
    fn materialization_validation_reports_unbound_typed_optional_external_without_failing() {
        let mut builder = RenderGraphBuilder::new("typed-optional-external-report");
        let previous_color = builder.import_external_resource_with_binding(
            "history.previous-color",
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let previous_exposure = builder.import_external_resource_with_binding(
            "history.previous-exposure",
            RenderGraphExternalResourceBinding::report_only_buffer(),
        );
        let pass = builder.add_pass("read", QueueLane::Graphics);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.read_external(pass, previous_color).unwrap();
        builder.read_external(pass, previous_exposure).unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap();

        assert_eq!(report.required_external_count, 0);
        assert_eq!(report.bound_required_external_count, 0);
        assert_eq!(report.missing_required_external_count, 0);
        assert_eq!(report.report_only_external_count, 2);
        assert_eq!(report.bound_report_only_external_count, 0);
        assert_eq!(report.missing_report_only_external_count, 2);
        assert_eq!(report.external_count(), 2);
        assert_eq!(report.bound_external_count(), 0);
        assert_eq!(report.missing_external_count(), 2);
        assert!(report.materialized_resources_complete());
        assert!(!report.is_complete());
    }

    #[test]
    fn materialization_validation_reports_required_and_report_only_external_coverage_separately() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut builder = RenderGraphBuilder::new("mixed-external-coverage-report");
        let required_indirect = builder.import_external_resource_with_binding(
            "mesh.indirect-args",
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let optional_history = builder.import_external_resource_with_binding(
            "history.previous-color",
            RenderGraphExternalResourceBinding::report_only_texture(),
        );
        let pass = builder.add_pass("read", QueueLane::Graphics);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.read_external(pass, required_indirect).unwrap();
        builder.read_external(pass, optional_history).unwrap();
        let graph = builder.compile().unwrap();
        let mut resources = RenderGraphExecutionResources::new();
        let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-required-external-bound-buffer"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        resources.insert_buffer("mesh.indirect-args", buffer);

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap();

        assert_eq!(report.required_external_count, 1);
        assert_eq!(report.bound_required_external_count, 1);
        assert_eq!(report.missing_required_external_count, 0);
        assert_eq!(report.report_only_external_count, 1);
        assert_eq!(report.bound_report_only_external_count, 0);
        assert_eq!(report.missing_report_only_external_count, 1);
        assert_eq!(report.external_count(), 2);
        assert_eq!(report.bound_external_count(), 1);
        assert_eq!(report.missing_external_count(), 1);
        assert!(report.materialized_resources_complete());
        assert!(!report.is_complete());
    }

    #[test]
    fn materialization_validation_fails_unbound_required_external_buffer() {
        let mut builder = RenderGraphBuilder::new("required-external-buffer-materialization");
        let indirect_args = builder.import_external_resource_with_binding(
            "mesh.indirect-args",
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = builder.add_pass("hzb-occlusion-cull", QueueLane::AsyncCompute);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.read_external(pass, indirect_args).unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();

        let error = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap_err();

        assert!(error.contains("required external resource bindings"));
        assert!(error.contains("external buffer `mesh.indirect-args`"));
    }

    #[test]
    fn materialization_validation_fails_unbound_required_external_texture() {
        let mut builder = RenderGraphBuilder::new("required-external-texture-materialization");
        let previous_hzb = builder.import_external_resource_with_binding(
            "history.previous-hzb",
            RenderGraphExternalResourceBinding::required_texture(),
        );
        let pass = builder.add_pass("hzb-occlusion-cull", QueueLane::AsyncCompute);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        builder.read_external(pass, previous_hzb).unwrap();
        let graph = builder.compile().unwrap();
        let resources = RenderGraphExecutionResources::new();

        let error = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap_err();

        assert!(error.contains("required external resource bindings"));
        assert!(error.contains("external texture `history.previous-hzb`"));
    }

    #[test]
    fn materialization_validation_rejects_stale_texture_binding_outside_live_lifetimes() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut builder = RenderGraphBuilder::new("stale-texture-binding-validation");
        let live_output = builder.import_external_resource("viewport-output");
        let pass = builder.add_pass("write", QueueLane::Graphics);
        builder.write_external(pass, live_output).unwrap();
        let graph = builder.compile().unwrap();
        let stale_texture = backend.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-stale-graph-texture"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mut resources = RenderGraphExecutionResources::new();
        resources.import_texture_view(
            "culled-scene-color",
            stale_texture.create_view(&wgpu::TextureViewDescriptor::default()),
        );

        let error = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap_err();

        assert!(error.contains("stale resource bindings outside live compiled lifetimes"));
        assert!(error.contains("texture `culled-scene-color`"));
    }

    #[test]
    fn materialization_validation_rejects_stale_buffer_binding_outside_live_lifetimes() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let mut builder = RenderGraphBuilder::new("stale-buffer-binding-validation");
        let live_output = builder.import_external_resource("viewport-output");
        let pass = builder.add_pass("write", QueueLane::Graphics);
        builder.write_external(pass, live_output).unwrap();
        let graph = builder.compile().unwrap();
        let stale_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-stale-graph-buffer"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        resources.insert_buffer("culled-light-list", stale_buffer);

        let error = resources
            .validate_materialized_graph_resources(&graph)
            .unwrap_err();

        assert!(error.contains("stale resource bindings outside live compiled lifetimes"));
        assert!(error.contains("buffer `culled-light-list`"));
    }
}

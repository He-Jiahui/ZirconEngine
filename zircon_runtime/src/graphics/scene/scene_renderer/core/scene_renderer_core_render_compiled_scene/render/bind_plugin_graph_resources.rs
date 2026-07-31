use std::collections::BTreeSet;

use crate::graphics::RuntimePrepareExternalBufferBinding;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceDesc,
};

const PLUGIN_FALLBACK_BUFFER_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE
    .union(wgpu::BufferUsages::COPY_DST)
    .union(wgpu::BufferUsages::COPY_SRC)
    .union(wgpu::BufferUsages::INDIRECT);
const MIN_PLUGIN_EXTERNAL_BUFFER_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<u32>() as wgpu::BufferAddress;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_plugin_graph_resources(
    device: &wgpu::Device,
    graph: &CompiledRenderGraph,
    external_buffer_bindings: &[RuntimePrepareExternalBufferBinding],
    resources: &mut RenderGraphExecutionResources,
) {
    let mut bound_logical_names = BTreeSet::new();
    for binding in external_buffer_bindings {
        let logical_name = binding.logical_name();
        if !graph_declares_typed_external_buffer(graph, logical_name) {
            continue;
        }

        resources.bind_execution_owned_buffer(
            logical_name,
            binding.backing_name(),
            binding.buffer(),
        );
        bound_logical_names.insert(logical_name.to_string());
    }

    for logical_name in FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS {
        if bound_logical_names.contains(*logical_name) {
            continue;
        }
        if !graph_declares_typed_external_buffer(graph, logical_name) {
            continue;
        }

        let fallback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(plugin_external_fallback_label(logical_name)),
            size: plugin_external_fallback_size(logical_name),
            usage: PLUGIN_FALLBACK_BUFFER_USAGE,
            mapped_at_creation: false,
        });
        resources.bind_execution_owned_buffer(
            *logical_name,
            plugin_external_fallback_backing_name(logical_name),
            &fallback,
        );
    }
}

fn graph_declares_typed_external_buffer(graph: &CompiledRenderGraph, logical_name: &str) -> bool {
    graph
        .resource_lifetime_by_name(logical_name)
        .is_some_and(|lifetime| {
            matches!(&lifetime.desc, RenderGraphResourceDesc::External)
                && lifetime.external_binding.resource_type
                    == RenderGraphExternalResourceType::Buffer
        })
}

fn plugin_external_fallback_size(logical_name: &str) -> wgpu::BufferAddress {
    plugin_external_buffer_min_size(logical_name).max(MIN_PLUGIN_EXTERNAL_BUFFER_SIZE)
}

fn plugin_external_fallback_label(logical_name: &str) -> &'static str {
    match logical_name {
        "particles.gpu.indirect-draw-args" => {
            "zircon-plugin-external-particles-indirect-draw-args-fallback"
        }
        "particles.gpu.debug-readback" => {
            "zircon-plugin-external-particles-debug-readback-fallback"
        }
        "particles.gpu.emitter-params" => {
            "zircon-plugin-external-particles-emitter-params-fallback"
        }
        "particles.gpu.alive-indices" => "zircon-plugin-external-particles-alive-indices-fallback",
        "particles.gpu.particles-a" => "zircon-plugin-external-particles-a-fallback",
        "particles.gpu.particles-b" => "zircon-plugin-external-particles-b-fallback",
        "particles.gpu.counters" => "zircon-plugin-external-particles-counters-fallback",
        "virtual-geometry-feedback" => "zircon-plugin-external-virtual-geometry-feedback-fallback",
        _ => "zircon-plugin-external-buffer-fallback",
    }
}

fn plugin_external_fallback_backing_name(logical_name: &str) -> String {
    format!("{logical_name}:plugin-external-fallback")
}

fn plugin_external_buffer_min_size(logical_name: &str) -> wgpu::BufferAddress {
    match logical_name {
        "particles.gpu.indirect-draw-args" => {
            (4 * std::mem::size_of::<u32>()) as wgpu::BufferAddress
        }
        "particles.gpu.debug-readback" => (8 * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
        "particles.gpu.emitter-params" => 256,
        _ => MIN_PLUGIN_EXTERNAL_BUFFER_SIZE,
    }
}

const FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS: &[&str] = &[
    "particles.gpu.particles-a",
    "particles.gpu.emitter-params",
    "particles.gpu.particles-b",
    "particles.gpu.counters",
    "particles.gpu.alive-indices",
    "particles.gpu.indirect-draw-args",
    "particles.gpu.debug-readback",
    "virtual-geometry-feedback",
];

#[cfg(test)]
mod tests {
    use crate::graphics::backend::RenderBackend;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    };

    use super::*;

    #[test]
    fn plugin_external_fallback_buffers_satisfy_materialization_report() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = plugin_external_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_plugin_graph_resources(&backend.device, &graph, &[], &mut resources);

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("known plugin external buffers should bind before validation");
        assert_eq!(report.required_external_count, 0);
        assert_eq!(
            report.report_only_external_count,
            FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert_eq!(
            report.bound_report_only_external_count,
            FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert_eq!(
            report.bound_external_count(),
            FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert_eq!(report.missing_external_count(), 0);
        assert!(report.is_complete());
        let aliases = resources.resource_alias_report();
        assert_eq!(
            aliases.buffer_aliases.len(),
            FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert!(aliases.buffer_aliases.iter().any(|alias| {
            alias.logical_name == "virtual-geometry-feedback"
                && alias.backing_name.ends_with(":plugin-external-fallback")
        }));
    }

    #[test]
    fn plugin_external_binder_skips_unknown_and_untyped_externals() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = mixed_external_graph();
        let mut resources = RenderGraphExecutionResources::new();

        bind_plugin_graph_resources(&backend.device, &graph, &[], &mut resources);

        assert!(resources.has_buffer("particles.gpu.counters"));
        assert!(!resources.has_buffer("third-party.plugin.buffer"));
        assert!(!resources.has_bound_resource("particles.gpu.untyped"));
        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("report-only unknown externals should not fail validation");
        assert_eq!(report.required_external_count, 0);
        assert_eq!(report.report_only_external_count, 3);
        assert_eq!(report.bound_report_only_external_count, 1);
        assert_eq!(report.missing_report_only_external_count, 2);
        assert_eq!(report.bound_external_count(), 1);
        assert_eq!(report.missing_external_count(), 2);
        assert!(report.materialized_resources_complete());
        assert!(!report.is_complete());
    }

    #[test]
    fn plugin_external_binder_prefers_runtime_prepare_buffers_over_fallback() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = plugin_external_graph();
        let actual = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-particle-counters-actual"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        let bindings = [RuntimePrepareExternalBufferBinding::new(
            "particles.gpu.counters",
            "particles.gpu.counters:runtime-prepare-test",
            &actual,
        )];

        bind_plugin_graph_resources(&backend.device, &graph, &bindings, &mut resources);

        let aliases = resources.resource_alias_report();
        assert!(aliases.buffer_aliases.iter().any(|alias| {
            alias.logical_name == "particles.gpu.counters"
                && alias.backing_name == "particles.gpu.counters:runtime-prepare-test"
        }));
        assert!(!aliases.buffer_aliases.iter().any(|alias| {
            alias.logical_name == "particles.gpu.counters"
                && alias.backing_name.ends_with(":plugin-external-fallback")
        }));
        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("runtime prepare buffer plus fallbacks should satisfy plugin graph resources");
        assert!(report.is_complete());
    }

    #[test]
    fn plugin_external_binder_accepts_registered_non_fallback_plugin_names() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = registered_third_party_external_graph();
        let actual = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-third-party-plugin-actual-buffer"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        let bindings = [RuntimePrepareExternalBufferBinding::new(
            "third-party.plugin.buffer",
            "third-party.plugin.buffer:runtime-prepare-test",
            &actual,
        )];

        bind_plugin_graph_resources(&backend.device, &graph, &bindings, &mut resources);

        assert!(resources.has_buffer("third-party.plugin.buffer"));
        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("registered non-fallback typed external should bind");
        assert_eq!(report.required_external_count, 0);
        assert_eq!(report.report_only_external_count, 1);
        assert_eq!(report.bound_report_only_external_count, 1);
        assert_eq!(report.bound_external_count(), 1);
        assert!(report.is_complete());
    }

    fn plugin_external_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("plugin-external-binding");
        let externals = FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS
            .iter()
            .map(|name| report_only_buffer(&mut builder, name))
            .collect::<Vec<_>>();
        let pass = side_effect_pass(&mut builder, "plugin-use");
        for external in externals {
            builder.write_storage_external(pass, external).unwrap();
        }
        builder.compile().unwrap()
    }

    fn mixed_external_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("mixed-plugin-external-binding");
        let known = report_only_buffer(&mut builder, "particles.gpu.counters");
        let unknown = report_only_buffer(&mut builder, "third-party.plugin.buffer");
        let untyped = builder.import_external_resource("particles.gpu.untyped");
        let pass = side_effect_pass(&mut builder, "mixed-plugin-use");
        builder.write_storage_external(pass, known).unwrap();
        builder.write_storage_external(pass, unknown).unwrap();
        builder.write_external(pass, untyped).unwrap();
        builder.compile().unwrap()
    }

    fn registered_third_party_external_graph() -> CompiledRenderGraph {
        let mut builder = RenderGraphBuilder::new("registered-third-party-plugin-external-binding");
        let registered = report_only_buffer(&mut builder, "third-party.plugin.buffer");
        let pass = side_effect_pass(&mut builder, "registered-third-party-use");
        builder.write_storage_external(pass, registered).unwrap();
        builder.compile().unwrap()
    }

    fn side_effect_pass(
        builder: &mut RenderGraphBuilder,
        name: &'static str,
    ) -> crate::render_graph::RenderPassId {
        let pass = builder.add_pass(name, QueueLane::AsyncCompute);
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    allow_culling: true,
                    has_side_effects: true,
                },
            )
            .unwrap();
        pass
    }

    fn report_only_buffer(
        builder: &mut RenderGraphBuilder,
        name: &str,
    ) -> crate::render_graph::ExternalResource {
        builder.import_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }
}

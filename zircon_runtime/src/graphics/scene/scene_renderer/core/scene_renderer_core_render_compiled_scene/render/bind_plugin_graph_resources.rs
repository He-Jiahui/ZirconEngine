use crate::graphics::RuntimePrepareExternalBufferBindingPacket;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererNeutralGraphBuffers;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceDesc,
};

const MIN_PLUGIN_EXTERNAL_BUFFER_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<u32>() as wgpu::BufferAddress;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_plugin_graph_resources(
    device: &wgpu::Device,
    neutral_buffers: &mut SceneRendererNeutralGraphBuffers,
    graph: &CompiledRenderGraph,
    external_buffer_binding_packet: Option<&RuntimePrepareExternalBufferBindingPacket>,
    resources: &mut RenderGraphExecutionResources,
) -> Result<(), String> {
    if let Some(binding_packet) = external_buffer_binding_packet {
        binding_packet.ensure_device_epoch(resources.device_epoch())?;
        for binding in binding_packet.bindings() {
            let logical_name = binding.logical_name();
            if !graph_declares_typed_external_buffer(graph, logical_name) {
                continue;
            }

            if let Some(physical_desc) = binding.physical_desc() {
                resources.bind_borrowed_buffer_with_physical_desc(
                    logical_name,
                    binding.backing_name(),
                    binding.buffer(),
                    physical_desc.clone(),
                )?;
            } else {
                resources.bind_execution_owned_buffer(
                    logical_name,
                    binding.backing_name(),
                    binding.buffer(),
                );
            }
        }
    }

    for logical_name in FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS {
        if PARTICLE_PLUGIN_EXTERNAL_BUFFERS.contains(logical_name) {
            continue;
        }
        if resources.has_buffer(logical_name) {
            continue;
        }
        if !graph_declares_typed_external_buffer(graph, logical_name) {
            continue;
        }

        if let Some((buffer, backing_name)) = neutral_buffers.plugin_buffer(
            device,
            logical_name,
            plugin_external_fallback_size(logical_name),
        ) {
            resources.bind_execution_owned_buffer(*logical_name, backing_name, buffer);
        }
    }
    Ok(())
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
const PARTICLE_PLUGIN_EXTERNAL_BUFFERS: &[&str] = &[
    "particles.gpu.particles-a",
    "particles.gpu.emitter-params",
    "particles.gpu.particles-b",
    "particles.gpu.counters",
    "particles.gpu.alive-indices",
    "particles.gpu.indirect-draw-args",
    "particles.gpu.debug-readback",
];

#[cfg(test)]
mod tests {
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::TransientResourcePool;
    use crate::graphics::{
        RuntimePrepareExternalBufferBinding, RuntimePrepareExternalBufferBindingPacket,
    };
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    };
    use crate::rhi::{BufferDesc, BufferUsage};

    use super::*;

    #[test]
    fn plugin_external_fallback_buffers_satisfy_materialization_report() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = plugin_external_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            None,
            &mut resources,
        )
        .expect("fallback plugin bindings should not conflict");

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("known plugin external buffers should bind before validation");
        assert_eq!(report.required_external_count, 0);
        assert_eq!(
            report.report_only_external_count,
            FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert_eq!(report.bound_report_only_external_count, 1);
        assert_eq!(report.bound_external_count(), 1);
        assert_eq!(
            report.missing_report_only_external_count,
            PARTICLE_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert_eq!(
            report.missing_external_count(),
            PARTICLE_PLUGIN_EXTERNAL_BUFFERS.len()
        );
        assert!(report.materialized_resources_complete());
        assert!(!report.is_complete());
        let aliases = resources.resource_alias_report();
        assert_eq!(aliases.buffer_aliases.len(), 1);
        assert!(aliases.buffer_aliases.iter().any(|alias| {
            alias.logical_name == "virtual-geometry-feedback"
                && alias.backing_name.ends_with(":plugin-neutral")
        }));
        assert!(
            !aliases
                .buffer_aliases
                .iter()
                .any(|alias| alias.logical_name.starts_with("particles.gpu."))
        );
    }

    #[test]
    fn plugin_external_binder_skips_unknown_and_untyped_externals() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = mixed_external_graph();
        let mut resources = RenderGraphExecutionResources::new();
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            None,
            &mut resources,
        )
        .expect("untyped plugin graphs should not create binding conflicts");

        assert!(!resources.has_buffer("particles.gpu.counters"));
        assert!(!resources.has_buffer("third-party.plugin.buffer"));
        assert!(!resources.has_bound_resource("particles.gpu.untyped"));
        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("report-only unknown externals should not fail validation");
        assert_eq!(report.required_external_count, 0);
        assert_eq!(report.report_only_external_count, 3);
        assert_eq!(report.bound_report_only_external_count, 0);
        assert_eq!(report.missing_report_only_external_count, 3);
        assert_eq!(report.bound_external_count(), 0);
        assert_eq!(report.missing_external_count(), 3);
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
        let mut resources = device_qualified_resources(&backend, &graph);
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();
        let binding_packet = RuntimePrepareExternalBufferBindingPacket::new(
            backend.device_profile(),
            vec![RuntimePrepareExternalBufferBinding::new(
                "particles.gpu.counters",
                "particles.gpu.counters:runtime-prepare-test",
                &actual,
            )],
        )
        .expect("non-empty runtime prepare bindings must produce a packet");

        bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            Some(&binding_packet),
            &mut resources,
        )
        .expect("runtime-prepare binding should not conflict");

        let aliases = resources.resource_alias_report();
        assert!(aliases.buffer_aliases.iter().any(|alias| {
            alias.logical_name == "particles.gpu.counters"
                && alias.backing_name == "particles.gpu.counters:runtime-prepare-test"
        }));
        assert_eq!(
            aliases
                .buffer_aliases
                .iter()
                .filter(|alias| alias.logical_name.starts_with("particles.gpu."))
                .count(),
            1
        );
        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("runtime prepare buffer plus fallbacks should satisfy plugin graph resources");
        assert!(report.materialized_resources_complete());
        assert_eq!(report.missing_report_only_external_count, 6);
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
        let mut resources = device_qualified_resources(&backend, &graph);
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();
        let binding_packet = RuntimePrepareExternalBufferBindingPacket::new(
            backend.device_profile(),
            vec![RuntimePrepareExternalBufferBinding::new(
                "third-party.plugin.buffer",
                "third-party.plugin.buffer:runtime-prepare-test",
                &actual,
            )],
        )
        .expect("non-empty runtime prepare bindings must produce a packet");

        bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            Some(&binding_packet),
            &mut resources,
        )
        .expect("registered plugin binding should not conflict");

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

    #[test]
    fn plugin_external_binder_retains_schema_backed_physical_buffer_contracts() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let expected = BufferDesc::new(
            "third-party.plugin.typed-buffer",
            16,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        );
        let mut builder = RenderGraphBuilder::new("schema-backed-plugin-external-buffer");
        let external = builder.import_present_external_buffer_with_binding(
            "third-party.plugin.typed-buffer",
            expected,
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = side_effect_pass(&mut builder, "typed-plugin-use");
        builder.read_external(pass, external).unwrap();
        let graph = builder.compile().unwrap();
        let actual = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-schema-backed-plugin-external-buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let binding_packet = RuntimePrepareExternalBufferBindingPacket::new(
            backend.device_profile(),
            vec![RuntimePrepareExternalBufferBinding::new_with_physical_desc(
                "third-party.plugin.typed-buffer",
                "third-party.plugin.typed-buffer:runtime-prepare-test",
                &actual,
                BufferDesc::new(
                    "third-party.plugin.typed-buffer",
                    32,
                    BufferUsage::UNIFORM | BufferUsage::COPY_DST,
                ),
            )],
        )
        .expect("non-empty runtime prepare bindings must produce a packet");
        let mut resources = device_qualified_resources(&backend, &graph);
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            Some(&binding_packet),
            &mut resources,
        )
        .expect("schema-backed runtime-prepare lease should not conflict");

        let report = resources
            .validate_materialized_graph_resources(&graph)
            .expect("schema-backed runtime-prepare lease should satisfy the graph contract");
        assert_eq!(report.bound_required_external_count, 1);
        assert!(report.is_complete());
        assert!(
            resources
                .resource_alias_report()
                .buffer_aliases
                .iter()
                .any(|alias| {
                    alias.logical_name == "third-party.plugin.typed-buffer"
                        && alias.backing_name
                            == "third-party.plugin.typed-buffer:runtime-prepare-test"
                })
        );
    }

    #[test]
    fn plugin_external_binder_rejects_duplicate_schema_backed_physical_backings() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let expected_usage = BufferUsage::UNIFORM | BufferUsage::COPY_DST;
        let mut builder = RenderGraphBuilder::new("duplicate-schema-backed-plugin-backing");
        let first = builder.import_present_external_buffer_with_binding(
            "third-party.plugin.first",
            BufferDesc::new("third-party.plugin.first", 64, expected_usage),
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let second = builder.import_present_external_buffer_with_binding(
            "third-party.plugin.second",
            BufferDesc::new("third-party.plugin.second", 16, expected_usage),
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = side_effect_pass(&mut builder, "typed-plugin-use");
        builder.read_external(pass, first).unwrap();
        builder.read_external(pass, second).unwrap();
        let graph = builder.compile().unwrap();
        let first_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-first-schema-backed-plugin-buffer"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let second_buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-second-schema-backed-plugin-buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let binding_packet = RuntimePrepareExternalBufferBindingPacket::new(
            backend.device_profile(),
            vec![
                RuntimePrepareExternalBufferBinding::new_with_physical_desc(
                    "third-party.plugin.first",
                    "third-party.plugin.shared-backing",
                    &first_buffer,
                    BufferDesc::new("third-party.plugin.first", 64, expected_usage),
                ),
                RuntimePrepareExternalBufferBinding::new_with_physical_desc(
                    "third-party.plugin.second",
                    "third-party.plugin.shared-backing",
                    &second_buffer,
                    BufferDesc::new("third-party.plugin.second", 16, expected_usage),
                ),
            ],
        )
        .expect("non-empty runtime prepare bindings must produce a packet");
        let mut resources = device_qualified_resources(&backend, &graph);
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        let error = bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            Some(&binding_packet),
            &mut resources,
        )
        .unwrap_err();

        assert!(error.contains("third-party.plugin.second"));
        assert!(error.contains("reuses backing `third-party.plugin.shared-backing`"));
    }

    #[test]
    fn plugin_external_binder_rejects_a_foreign_device_epoch_before_binding() {
        let Ok(source_backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let Ok(destination_backend) = RenderBackend::new_offscreen() else {
            return;
        };
        assert_ne!(
            source_backend.device_profile().device_id(),
            destination_backend.device_profile().device_id()
        );
        let graph = registered_third_party_external_graph();
        let source_buffer = source_backend
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("zircon-test-foreign-device-plugin-buffer"),
                size: 64,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            });
        let binding_packet = RuntimePrepareExternalBufferBindingPacket::new(
            source_backend.device_profile(),
            vec![RuntimePrepareExternalBufferBinding::new(
                "third-party.plugin.buffer",
                "third-party.plugin.buffer:foreign-device-test",
                &source_buffer,
            )],
        )
        .expect("non-empty runtime prepare bindings must produce a packet");
        let mut resources = device_qualified_resources(&destination_backend, &graph);
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        let error = bind_plugin_graph_resources(
            &destination_backend.device,
            &mut neutral_buffers,
            &graph,
            Some(&binding_packet),
            &mut resources,
        )
        .expect_err("a foreign device packet must fail before native buffer binding");

        assert!(error.contains("runtime prepare external buffer packet belongs to device"));
        assert!(error.contains("expected device"));
        assert!(!resources.has_buffer("third-party.plugin.buffer"));
    }

    #[test]
    fn plugin_external_binder_rejects_a_packet_before_graph_epoch_establishment() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let graph = registered_third_party_external_graph();
        let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-unqualified-graph-plugin-buffer"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let binding_packet = RuntimePrepareExternalBufferBindingPacket::new(
            backend.device_profile(),
            vec![RuntimePrepareExternalBufferBinding::new(
                "third-party.plugin.buffer",
                "third-party.plugin.buffer:unqualified-graph-test",
                &buffer,
            )],
        )
        .expect("non-empty runtime prepare bindings must produce a packet");
        let mut resources = RenderGraphExecutionResources::new();
        let mut neutral_buffers = SceneRendererNeutralGraphBuffers::default();

        let error = bind_plugin_graph_resources(
            &backend.device,
            &mut neutral_buffers,
            &graph,
            Some(&binding_packet),
            &mut resources,
        )
        .expect_err("plugin binding must not precede graph device epoch establishment");

        assert!(error.contains("before the execution device epoch was established"));
        assert!(!resources.has_buffer("third-party.plugin.buffer"));
    }

    #[test]
    fn product_plugin_binder_does_not_create_frame_local_buffers_or_names() {
        let source = include_str!("bind_plugin_graph_resources.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert!(!source.contains("create_buffer"));
        assert!(!source.contains("format!("));
        assert!(!source.contains("BTreeSet"));
        assert!(source.contains("PARTICLE_PLUGIN_EXTERNAL_BUFFERS.contains(logical_name)"));
        assert!(source.contains("neutral_buffers.plugin_buffer("));
    }

    #[test]
    fn product_plugin_external_binding_packet_is_device_epoch_qualified() {
        let binder = include_str!("bind_plugin_graph_resources.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let packet = include_str!("../../../../../runtime_prepare_collector.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let readbacks = include_str!(
            "../../scene_renderer_core/advanced_plugin_readbacks/scene_renderer_advanced_plugin_readbacks.rs"
        )
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
        let dispatcher =
            include_str!("../../scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs")
                .split("\n#[cfg(test)]")
                .next()
                .unwrap_or_default();
        let compiled = include_str!("prepare_compiled_scene_graph_frame.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert!(packet.contains("struct RuntimePrepareExternalBufferBindingPacket"));
        assert!(packet.contains("fn ensure_device_epoch("));
        assert!(binder.contains("binding_packet.ensure_device_epoch(resources.device_epoch())?;"));
        assert!(binder.contains("for binding in binding_packet.bindings()"));
        assert!(readbacks.contains("external_buffer_binding_packet"));
        assert!(dispatcher.contains("device_profile: &RenderDeviceProfile"));
        assert!(compiled.contains("advanced_plugin_readbacks.external_buffer_binding_packet()"));
        assert!(!compiled.contains("advanced_plugin_readbacks.external_buffer_bindings()"));
    }

    fn device_qualified_resources(
        backend: &RenderBackend,
        graph: &CompiledRenderGraph,
    ) -> RenderGraphExecutionResources {
        let mut pool = TransientResourcePool::default();
        pool.begin_frame(backend.device_profile());
        let mut resources = RenderGraphExecutionResources::new();
        resources
            .materialize_transient_resources_with_pool(
                &backend.device,
                backend.device_profile(),
                graph,
                &mut pool,
            )
            .expect("plugin packet fixture must establish the graph device epoch");
        resources
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
        let untyped = builder.import_present_external_resource("particles.gpu.untyped");
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
        builder.import_present_external_resource_with_binding(
            name,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }
}

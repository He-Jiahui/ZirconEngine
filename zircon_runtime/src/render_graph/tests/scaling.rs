use std::time::{Duration, Instant};

use crate::render_graph::{CompiledRenderGraphStats, QueueLane, RenderGraphBuilder};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

const SCALE_PASS_COUNTS: &[usize] = &[16, 64, 256, 1024];
const SCALE_BENCHMARK_WARMUP_SAMPLES: usize = 3;
const SCALE_BENCHMARK_MEASURED_SAMPLES: usize = 31;

#[derive(Clone, Copy, Debug)]
enum ScaleGraphShape {
    Chain,
    FanOut,
    MultiWriter,
    PluginLabeledChain,
}

impl ScaleGraphShape {
    const ALL: [Self; 4] = [
        Self::Chain,
        Self::FanOut,
        Self::MultiWriter,
        Self::PluginLabeledChain,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Chain => "chain",
            Self::FanOut => "fanout",
            Self::MultiWriter => "multi_writer",
            Self::PluginLabeledChain => "plugin_labeled_chain",
        }
    }

    const fn resource_count(self, pass_count: usize) -> usize {
        match self {
            Self::MultiWriter => 1,
            Self::Chain | Self::FanOut | Self::PluginLabeledChain => pass_count,
        }
    }
}

#[test]
fn render_graph_compile_work_scales_linearly_for_supported_topologies() {
    for shape in ScaleGraphShape::ALL {
        for pass_count in SCALE_PASS_COUNTS {
            let graph = build_scale_graph(shape, *pass_count).compile().unwrap();
            assert_compile_work(shape, *pass_count, graph.stats());
        }
    }
}

#[test]
#[ignore = "run through the managed Render01 scale-validation lane to record p50/p95"]
fn render_graph_compile_scale_reports_p50_and_p95() {
    for shape in ScaleGraphShape::ALL {
        for pass_count in SCALE_PASS_COUNTS {
            let expected = build_scale_graph(shape, *pass_count)
                .compile()
                .unwrap()
                .stats();
            assert_compile_work(shape, *pass_count, expected);

            for _ in 0..SCALE_BENCHMARK_WARMUP_SAMPLES {
                let graph = build_scale_graph(shape, *pass_count).compile().unwrap();
                assert_eq!(graph.stats(), expected);
            }

            let mut samples = Vec::with_capacity(SCALE_BENCHMARK_MEASURED_SAMPLES);
            for _ in 0..SCALE_BENCHMARK_MEASURED_SAMPLES {
                let started = Instant::now();
                let graph = build_scale_graph(shape, *pass_count).compile().unwrap();
                samples.push(started.elapsed());
                assert_eq!(graph.stats(), expected);
            }
            samples.sort_unstable();

            println!(
                "render_graph_compile_scale shape={} passes={} resources={} p50_us={} p95_us={} access_visits={} execution_edges={} provenance_edges={} cull_edge_visits={}",
                shape.label(),
                pass_count,
                shape.resource_count(*pass_count),
                percentile(&samples, 50).as_micros(),
                percentile(&samples, 95).as_micros(),
                expected.compile_resource_access_visit_count,
                expected.compile_execution_dependency_count,
                expected.compile_provenance_dependency_count,
                expected.compile_cull_dependency_visit_count,
            );
        }
    }
}

fn build_scale_graph(shape: ScaleGraphShape, pass_count: usize) -> RenderGraphBuilder {
    assert!(pass_count >= 2);
    match shape {
        ScaleGraphShape::Chain => build_chain_graph(pass_count, false),
        ScaleGraphShape::FanOut => build_fan_out_graph(pass_count),
        ScaleGraphShape::MultiWriter => build_multi_writer_graph(pass_count),
        ScaleGraphShape::PluginLabeledChain => build_chain_graph(pass_count, true),
    }
}

fn build_chain_graph(pass_count: usize, plugin_labeled: bool) -> RenderGraphBuilder {
    let mut builder = RenderGraphBuilder::new("scale-chain");
    let output = builder.import_external_resource("scale-output");
    let textures = (0..pass_count)
        .map(|index| builder.create_texture(scale_texture_desc(index)))
        .collect::<Vec<_>>();

    for index in 0..pass_count {
        let pass = if plugin_labeled {
            builder.add_pass_with_executor(
                format!("plugin-pass-{index}"),
                QueueLane::Graphics,
                Some(format!("plugin.scale.{}", index % 8)),
            )
        } else {
            builder.add_pass(format!("chain-pass-{index}"), QueueLane::Graphics)
        };
        if index > 0 {
            builder.read_texture(pass, textures[index - 1]).unwrap();
        }
        builder.write_texture(pass, textures[index]).unwrap();
        if index + 1 == pass_count {
            builder.write_external(pass, output).unwrap();
        }
    }

    builder
}

fn build_fan_out_graph(pass_count: usize) -> RenderGraphBuilder {
    let mut builder = RenderGraphBuilder::new("scale-fanout");
    let source = builder.create_texture(scale_texture_desc(0));
    let consumer_textures = (1..pass_count)
        .map(|index| builder.create_texture(scale_texture_desc(index)))
        .collect::<Vec<_>>();
    let output = builder.import_external_resource("scale-output");
    let seed = builder.add_pass("fanout-seed", QueueLane::Graphics);
    builder.write_texture(seed, source).unwrap();

    for (index, texture) in consumer_textures.into_iter().enumerate() {
        let pass = builder.add_pass(format!("fanout-consumer-{index}"), QueueLane::Graphics);
        builder.read_texture(pass, source).unwrap();
        builder.write_texture(pass, texture).unwrap();
        if index + 1 == pass_count - 1 {
            builder.write_external(pass, output).unwrap();
        }
    }

    builder
}

fn build_multi_writer_graph(pass_count: usize) -> RenderGraphBuilder {
    let mut builder = RenderGraphBuilder::new("scale-multi-writer");
    let color = builder.create_texture(scale_texture_desc(0));
    let output = builder.import_external_resource("scale-output");

    for index in 0..pass_count {
        let pass = builder.add_pass(format!("writer-{index}"), QueueLane::Graphics);
        builder.write_texture(pass, color).unwrap();
        if index + 1 == pass_count {
            builder.read_texture(pass, color).unwrap();
            builder.write_external(pass, output).unwrap();
        }
    }

    builder
}

fn scale_texture_desc(index: usize) -> TextureDesc {
    TextureDesc::new(
        format!("scale-color-{index}"),
        4,
        4,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    )
}

fn assert_compile_work(shape: ScaleGraphShape, pass_count: usize, stats: CompiledRenderGraphStats) {
    let expected_access_visits = match shape {
        ScaleGraphShape::MultiWriter => pass_count + 2,
        ScaleGraphShape::Chain | ScaleGraphShape::FanOut | ScaleGraphShape::PluginLabeledChain => {
            pass_count * 2
        }
    };
    let expected_execution_edges = pass_count - 1;
    let expected_provenance_edges = match shape {
        ScaleGraphShape::MultiWriter => 0,
        ScaleGraphShape::Chain | ScaleGraphShape::FanOut | ScaleGraphShape::PluginLabeledChain => {
            pass_count - 1
        }
    };

    assert_eq!(
        stats.compile_resource_access_visit_count,
        expected_access_visits
    );
    assert_eq!(
        stats.compile_execution_dependency_count,
        expected_execution_edges
    );
    assert_eq!(
        stats.compile_provenance_dependency_count,
        expected_provenance_edges
    );
    assert_eq!(stats.compile_cull_root_count, 1);
    assert!(
        stats.compile_cull_dependency_visit_count <= expected_provenance_edges,
        "shape={} pass_count={} cull edge visits must stay bounded by provenance edges",
        shape.label(),
        pass_count
    );
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    assert!(!samples.is_empty());
    assert!(percentile <= 100);
    let index = (samples.len() * percentile).div_ceil(100).saturating_sub(1);
    samples[index]
}

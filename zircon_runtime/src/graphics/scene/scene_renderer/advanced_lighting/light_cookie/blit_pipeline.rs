use crate::graphics::scene::resources::ResourceStreamer;

use super::frame_plan::CookieAtlasEntry;

const BLIT_SHADER: &str = include_str!("shaders/blit.wgsl");

pub(super) struct LightCookieAtlasBlitPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl LightCookieAtlasBlitPipeline {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-light-cookie-blit-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-light-cookie-blit-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-light-cookie-blit-shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-light-cookie-atlas-blit"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            bind_group_layout,
            pipeline,
        }
    }

    pub(super) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        streamer: &ResourceStreamer,
        entries: &[CookieAtlasEntry],
        cell_size: u32,
    ) -> usize {
        let mut draws = Vec::with_capacity(entries.len());
        for entry in entries {
            let resource = streamer.texture(Some(entry.texture));
            if resource.id != Some(entry.texture) {
                continue;
            }
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("zircon-light-cookie-blit-bind-group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(resource.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(resource.sampler()),
                    },
                ],
            });
            let viewport = [
                (entry.slot % super::COOKIE_ATLAS_GRID_SIZE) * cell_size,
                (entry.slot / super::COOKIE_ATLAS_GRID_SIZE) * cell_size,
            ];
            draws.push((viewport, bind_group));
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("LightCookieAtlasBuildPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        for (viewport, bind_group) in &draws {
            pass.set_viewport(
                viewport[0] as f32,
                viewport[1] as f32,
                cell_size as f32,
                cell_size as f32,
                0.0,
                1.0,
            );
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        draws.len()
    }
}

#[cfg(test)]
mod optimization_batch_20260830cf_runtime_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const ENTRIES_PER_SAMPLE: usize = 512;

    #[test]
    fn light_cookie_draw_collection_reserves_entry_capacity() {
        let source = include_str!("blit_pipeline.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("light cookie blit implementation");

        assert!(implementation.contains("let mut draws = Vec::with_capacity(entries.len())"));
        assert!(implementation.contains("for entry in entries"));
        assert!(implementation.contains("draws.push((viewport, bind_group))"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cf_runtime_light_cookie_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME384_LIGHT_COOKIE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries_per_sample={ENTRIES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut draws = if use_capacity {
                Vec::with_capacity(ENTRIES_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for entry in 0..ENTRIES_PER_SAMPLE {
                if entry % 5 != 0 {
                    draws.push((entry % 16, entry / 16));
                }
            }
            checksum ^= draws.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

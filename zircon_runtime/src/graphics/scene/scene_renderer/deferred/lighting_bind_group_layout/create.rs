use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;

pub(in crate::graphics::scene::scene_renderer::deferred) fn create_lighting_bind_group_layout(
    device: &wgpu::Device,
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
) -> wgpu::BindGroupLayout {
    const ENVIRONMENT_ENTRY_COUNT: usize = 10;
    const FULL_LIGHTING_ENTRY_COUNT: usize = 29;
    let full_lighting = deferred_lighting_profile.uses_full_lighting_bind_group();
    let mut entries = Vec::with_capacity(if full_lighting {
        FULL_LIGHTING_ENTRY_COUNT
    } else {
        ENVIRONMENT_ENTRY_COUNT
    });
    entries.extend([
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Depth,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
    ]);
    entries.extend(
        crate::graphics::scene::scene_renderer::environment::reflection_probe_bind_group_layout_entries(),
    );
    if full_lighting {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        });
        entries.extend(
            crate::graphics::scene::scene_renderer::shadow::atlas::shadow_atlas_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::environment::lightmap_bind_group_layout_entries(
            ),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_apply_bind_group_layout_entries(
                wgpu::ShaderStages::FRAGMENT,
            ),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::light_cookie_bind_group_layout_entries(),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::irradiance_volume_bind_group_layout_entries(),
        );
        entries.extend([
            wgpu::BindGroupLayoutEntry {
                binding: 20,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 21,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 22,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]);
    }
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-deferred-lighting-bind-group-layout"),
        entries: &entries,
    })
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    #[test]
    fn environment_only_deferred_layout_retains_local_provider_abi_for_upgrades() {
        let implementation = include_str!("create.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting layout implementation");

        assert!(implementation.contains(
            "crate::graphics::scene::scene_renderer::environment::reflection_probe_bind_group_layout_entries()"
        ));
        assert!(
            !implementation.contains("defers_local_reflection_provider_resources"),
            "environment-only deferred layout must not drop ABI needed after a provider upgrade"
        );
    }

    #[test]
    fn optimization_batch_dr_deferred_layout_reserves_profile_capacity() {
        let source = include_str!("create.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("deferred lighting layout production source");
        assert!(production.contains("Vec::with_capacity(if full_lighting"));
        assert!(production.contains("ENVIRONMENT_ENTRY_COUNT: usize = 10"));
        assert!(production.contains("FULL_LIGHTING_ENTRY_COUNT: usize = 29"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dr_deferred_layout_entry_capacity_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const BUILDS_PER_SAMPLE: usize = 32_768;
        const ENTRY_COUNT: usize = 29;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_entry_builds(BUILDS_PER_SAMPLE, ENTRY_COUNT, false));
                optimized_samples.push(measure_entry_builds(BUILDS_PER_SAMPLE, ENTRY_COUNT, true));
            } else {
                optimized_samples.push(measure_entry_builds(BUILDS_PER_SAMPLE, ENTRY_COUNT, true));
                legacy_samples.push(measure_entry_builds(BUILDS_PER_SAMPLE, ENTRY_COUNT, false));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME426_DEFERRED_LAYOUT_ENTRY_CAPACITY_BENCH_V1 builds_per_sample={BUILDS_PER_SAMPLE} full_lighting_entries={ENTRY_COUNT} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "deferred layout entry capacity p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_entry_builds(build_count: usize, entry_count: usize, reserve: bool) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for build_index in 0..build_count {
                let mut entries = Vec::new();
                if reserve {
                    entries.reserve(entry_count);
                }
                for entry in 0..entry_count {
                    entries.push(entry ^ build_index);
                }
                checksum = checksum.wrapping_add(entries.len() ^ entries.capacity());
                black_box(&entries);
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}

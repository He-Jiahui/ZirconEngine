use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};
use crate::graphics::shader::{ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_mesh_pipeline;
use super::shader_source::{mesh_pipeline_shader_source, MeshPipelineShaderSource};
use super::MeshPipelineCache;

const MESH_SHADER_NAGA_VERSION: &str = "naga-29.0.1";
const MESH_SHADER_WGPU_VERSION: &str = "wgpu-29.0.1";

impl MeshPipelineCache {
    pub(crate) fn pipeline_uses_builtin_fallback_shader(
        &self,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
    ) -> bool {
        key.uses_fallback_shader() || streamer.shader_source(&key.shader_id).is_none()
    }

    pub(crate) fn ensure_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        if kind != MeshPassPipelineKind::Base {
            return None;
        }
        let shader_source = match mesh_pipeline_shader_source(
            streamer,
            &pipeline_key,
            shader_variant_key.geometry_source,
        ) {
            Ok(source) => source,
            Err(_) => {
                self.record_shader_variant_disk_error(&shader_variant_key);
                return None;
            }
        };
        let shader_key = mesh_shader_module_cache_key(
            &pipeline_key,
            &shader_variant_key,
            &shader_source.source_hash,
        );
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, &shader_variant_key);
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self.mesh_variant_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let pipeline = create_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                self.target_format,
                &pipeline_key,
            );
            self.mesh_variant_pipelines.insert(variant_id, pipeline);
        }
        self.mesh_variant_pipelines.get(&variant_id)
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn mesh_pipeline_shader_source_with_cache(
        &mut self,
        source: MeshPipelineShaderSource,
        variant_key: &ShaderVariantKey,
    ) -> String {
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            variant_key,
            source.cache_content_hashes.iter().map(String::as_str),
        );
        match self.shader_variant_disk_cache.lookup(&disk_key) {
            ShaderVariantCacheDiskLookup::Hit(entry) => {
                self.record_shader_variant_disk_hit(variant_key);
                entry.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Miss => {
                self.record_shader_variant_compile_miss(variant_key);
                match self.shader_variant_disk_cache.write(
                    &disk_key,
                    &source.wgsl_source,
                    &source.template_revision,
                    MESH_SHADER_NAGA_VERSION,
                    MESH_SHADER_WGPU_VERSION,
                ) {
                    Ok(_) => self.record_shader_variant_disk_write(variant_key),
                    Err(_) => self.record_shader_variant_disk_error(variant_key),
                }
                source.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Error(_) => {
                self.record_shader_variant_disk_error(variant_key);
                source.wgsl_source
            }
        }
    }
}

fn mesh_shader_module_cache_key(
    key: &PipelineKey,
    variant_key: &ShaderVariantKey,
    source_hash: &str,
) -> String {
    format!(
        "{}@{}#{}#{}",
        key.shader_id,
        key.shader_revision,
        variant_key.canonical_string(),
        source_hash
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use crate::asset::pipeline::manager::ProjectAssetManager;
    use crate::core::framework::render::{ShaderPassType, ShaderQualityTier};
    use crate::dynamic_api::builtin_fallback_shader_prewarm_manifest;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::gpu_scene::GpuScene;
    use crate::graphics::scene::resources::{
        default_pipeline_key, ResourceStreamer, GPU_MATERIAL_UNIFORM_MIN_SIZE,
    };
    use crate::graphics::shader::{prewarm_shader_variants_to_disk, ShaderVariantCacheDisk};

    use super::super::super::mesh_pass::MeshPassPipelineKind;
    use super::super::mesh_pipeline_standard_material_template_source;
    use super::super::MeshPipelineCache;
    use super::mesh_shader_module_cache_key;

    #[test]
    fn mesh_pipeline_template_source_hashes_feed_disk_and_module_keys() {
        let key = default_pipeline_key();
        let variant_key = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

        let source = match mesh_pipeline_standard_material_template_source(&key) {
            Ok(source) => source,
            Err(error) => panic!("standard material template assembly failed: {error:?}"),
        };
        let module_key = mesh_shader_module_cache_key(&key, &variant_key, &source.source_hash);

        assert!(source.cache_content_hashes.len() > 1);
        assert!(source.cache_content_hashes.contains(&source.source_hash));
        assert!(module_key.contains(&source.source_hash));
        assert_eq!(source.template_revision, "zr-material-template-v1");
    }

    #[test]
    fn runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss() {
        let root = std::env::temp_dir().join(format!(
            "zircon_runtime_base_mesh_staged_cache_hit_test_{}",
            std::process::id()
        ));
        let runtime_root = root.join("runtime");
        let staged_root = root.join("staged");
        let _ = fs::remove_dir_all(&root);

        let manifest = builtin_fallback_shader_prewarm_manifest();
        let prewarm_report = prewarm_shader_variants_to_disk(&manifest, &staged_root);
        assert_eq!(prewarm_report.requested_count, manifest.variants.len());
        assert_eq!(prewarm_report.written_count, manifest.variants.len());
        assert_eq!(prewarm_report.failed_count, 0);

        let Ok(backend) = RenderBackend::new_offscreen() else {
            let _ = fs::remove_dir_all(root);
            return;
        };
        let RenderBackend { device, queue, .. } = backend;
        let texture_layout = test_texture_bind_group_layout(&device);
        let streamer = ResourceStreamer::new_for_test(
            Arc::new(ProjectAssetManager::default()),
            &device,
            &queue,
            &texture_layout,
        );
        let scene_layout = test_scene_bind_group_layout(&device);
        let material_layout = test_standard_material_bind_group_layout(&device);
        let gpu_scene = test_gpu_scene(&device);
        let mut cache = MeshPipelineCache::new(
            &device,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            &scene_layout,
            &material_layout,
            gpu_scene.scene_bind_group_layout(),
        );
        cache.shader_variant_disk_cache =
            ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root]);

        cache.reset_shader_variant_miss_report();
        let pipeline_key = default_pipeline_key();
        let variant_id = cache.resolve_variant(
            MeshPassPipelineKind::Base,
            &pipeline_key,
            ShaderQualityTier::Medium,
        );
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

        assert!(
            cache
                .ensure_pipeline_for_variant(&device, &streamer, variant_id)
                .is_some(),
            "runtime Base mesh pipeline should be created from staged prewarm cache hit"
        );

        let error = pollster::block_on(error_scope.pop());
        assert!(
            error.is_none(),
            "staged prewarm cache hit source should create the runtime Base mesh WGPU pipeline: {error:?}"
        );
        let miss_report = cache.shader_variant_miss_report();
        assert_eq!(miss_report.request_count, 1);
        assert_eq!(miss_report.disk_hit_count, 1);
        assert_eq!(miss_report.compile_miss_count, 0);
        assert_eq!(miss_report.disk_write_count, 0);
        assert_eq!(miss_report.disk_error_count, 0);

        let _ = fs::remove_dir_all(root);
    }

    fn test_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-runtime-staged-cache-texture-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
        })
    }

    fn test_scene_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-runtime-staged-cache-scene-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn test_standard_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-runtime-staged-cache-material-layout"),
            entries: &[
                material_texture_entry(0),
                material_sampler_entry(1),
                material_texture_entry(2),
                material_sampler_entry(3),
                material_texture_entry(4),
                material_sampler_entry(5),
                material_texture_entry(6),
                material_sampler_entry(7),
                material_texture_entry(8),
                material_sampler_entry(9),
                material_uniform_entry(10),
            ],
        })
    }

    fn material_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        }
    }

    fn material_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    fn material_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
            },
            count: None,
        }
    }

    fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("zircon-test-runtime-staged-cache-joint-palette"),
                size: 256 * 64 + 16,
                usage: wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: false,
            })),
            wgpu::BufferSize::new(256 * 64 + 16).expect("test joint palette size"),
        )
    }
}

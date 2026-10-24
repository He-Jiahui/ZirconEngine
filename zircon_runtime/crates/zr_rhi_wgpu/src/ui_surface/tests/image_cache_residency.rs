use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::{
    committed_external_image_prepare_generation, committed_image_prepare_generation,
    encode_linear_premultiplied_srgba8, reusable_image_prepare_generation,
};
use zr_rhi::{UiSurfaceDrawList, UiSurfaceImageResource, UiSurfaceImageResourceTable};

struct G42Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    shared: super::WgpuUiSharedImageRegistry,
}

impl G42Gpu {
    fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("G-42 regression requires an offscreen WGPU adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("G-42 regression requires an offscreen WGPU device");
        let layout = super::super::pipeline::create_image_bind_group_layout(&device);
        let sampler = super::super::pipeline::create_image_sampler(&device);
        Self {
            device,
            queue,
            layout,
            sampler,
            shared: super::WgpuUiSharedImageRegistry::default(),
        }
    }

    fn present(
        &self,
        cache: &mut super::WgpuUiImageCache,
        present: u64,
        key: &str,
        generation: u64,
        size: (u32, u32),
    ) -> super::WgpuUiImageResourceStats {
        self.present_with_active(cache, present, key, generation, size, &[])
    }

    fn present_with_active(
        &self,
        cache: &mut super::WgpuUiImageCache,
        present: u64,
        key: &str,
        generation: u64,
        (width, height): (u32, u32),
        active: &[(&str, u64)],
    ) -> super::WgpuUiImageResourceStats {
        let bytes = u64::from(width) * u64::from(height) * 4;
        let command = zr_rhi::UiSurfaceCommand {
            z_index: 0,
            frame: zr_rhi::UiSurfaceRect {
                x: 0.0,
                y: 0.0,
                width: width as f32,
                height: height as f32,
            },
            clip: None,
            kind: zr_rhi::UiSurfaceCommandKind::Image {
                payload: zr_rhi::UiSurfaceImagePayload {
                    resource_key: key.to_owned(),
                    resource_generation: generation,
                    width,
                    height,
                    upload_bytes: bytes,
                    rgba: None,
                    atlas_uv: None,
                },
            },
        };
        let mut staged = UiSurfaceImageResourceTable::default();
        staged.insert(
            key.to_owned(),
            UiSurfaceImageResource {
                generation,
                width,
                height,
                upload_bytes: bytes,
                rgba: vec![0x40; bytes as usize].into(),
            },
        );
        let draw_list =
            UiSurfaceDrawList::with_generation((width, height), None, vec![command], present);
        let mut sources = vec![super::ImageUploadSource {
            resource_key: key.to_owned(),
            resource_generation: generation,
            command_indices: vec![0],
        }];
        sources.extend(
            active
                .iter()
                .map(|(key, generation)| super::ImageUploadSource {
                    resource_key: (*key).to_owned(),
                    resource_generation: *generation,
                    command_indices: Vec::new(),
                }),
        );
        sources.sort_unstable_by(|left, right| {
            (&left.resource_key, left.resource_generation)
                .cmp(&(&right.resource_key, right.resource_generation))
        });
        cache.prepare(
            &self.device,
            &self.queue,
            &self.layout,
            &self.sampler,
            present,
            &draw_list,
            &sources,
            None,
            &self.shared,
            &mut staged,
        )
    }
}

fn g42_stream(per_frame_keys: bool, resize: bool) {
    let gpu = G42Gpu::new();
    let mut cache = super::WgpuUiImageCache::default();
    let mut submissions = std::collections::VecDeque::new();
    for present in 1..=30 {
        let key = if per_frame_keys {
            format!("play-viewport:test:{present}")
        } else {
            "play-viewport:stream".to_owned()
        };
        let generation = if per_frame_keys { 0 } else { present };
        let size = if resize && present >= 12 {
            (1437, 1004)
        } else {
            (1251, 769)
        };
        let stats = gpu.present(&mut cache, present, &key, generation, size);
        assert!(
            cache.is_resident(&key, generation),
            "present {present} lost the image: {stats:?}"
        );
        assert_eq!(stats.cache_admission_reject_count, 0, "present {present}");
        assert_eq!(
            stats.shared_upload_write_count, 1,
            "present {present} must upload new pixels"
        );
        assert!(stats.device_allocation_bytes <= super::MAX_UI_IMAGE_CACHE_BYTES);
        assert!(gpu.shared.resident_bytes() <= super::MAX_UI_IMAGE_CACHE_BYTES);
        if !per_frame_keys {
            assert_eq!(cache.entry_count(), 1, "present {present}");
            assert_eq!(stats.cache_superseded_release_count, u64::from(present > 1));
        }
        // Keep two submissions alive, matching the present-retirement lifetime contract.
        submissions.push_back(
            cache
                .pin_prepared_allocations_for_submission()
                .expect("uploaded allocation"),
        );
        if submissions.len() > 2 {
            submissions.pop_front();
        }
    }
    if !per_frame_keys {
        assert_eq!(
            cache.entry_count(),
            1,
            "only the current stream generation stays window-resident"
        );
    }
}

#[test]
fn g42_per_frame_keys_stay_resident_past_registry_budget() {
    g42_stream(true, false);
}

#[test]
fn g42_stream_key_replaces_superseded_generations() {
    g42_stream(false, false);
}

#[test]
fn g42_resize_mid_stream_stays_resident() {
    g42_stream(false, true);
}

#[test]
fn g42_superseded_release_preserves_an_active_older_generation() {
    let gpu = G42Gpu::new();
    let mut cache = super::WgpuUiImageCache::default();
    let key = "play-viewport:stream";
    gpu.present(&mut cache, 1, key, 1, (1, 1));
    let stats = gpu.present_with_active(&mut cache, 2, key, 2, (1, 1), &[(key, 1)]);
    assert_eq!(stats.cache_superseded_release_count, 0);
    assert!(cache.is_resident(key, 1));
    assert!(cache.is_resident(key, 2));
    assert_eq!(cache.entry_count(), 2);

    let stats = gpu.present(&mut cache, 3, key, 3, (1, 1));
    assert_eq!(stats.cache_superseded_release_count, 2);
    assert!(cache.is_resident(key, 3));
    assert_eq!(cache.entry_count(), 1);
}

#[test]
fn g42_registry_recovery_preserves_active_window_entries() {
    let gpu = G42Gpu::new();
    let mut cache = super::WgpuUiImageCache::default();
    let active_key = "play-viewport:active";
    gpu.present(&mut cache, 1, active_key, 0, (1251, 769));
    for present in 2..=17 {
        let key = format!("play-viewport:test:{present}");
        assert_eq!(
            gpu.present(&mut cache, present, &key, 0, (1251, 769))
                .cache_admission_reject_count,
            0
        );
    }
    let stats = gpu.present_with_active(
        &mut cache,
        18,
        "play-viewport:new",
        0,
        (1251, 769),
        &[(active_key, 0)],
    );
    assert_eq!(stats.cache_registry_reject_recovery_count, 1);
    assert_eq!(stats.cache_admission_reject_count, 0);
    assert_eq!(stats.shared_upload_write_count, 1);
    assert!(cache.is_resident(active_key, 0));
    assert!(cache.is_resident("play-viewport:new", 0));
    assert_eq!(cache.entry_count(), 2);
    assert!(stats.device_allocation_bytes <= super::MAX_UI_IMAGE_CACHE_BYTES);
}

#[test]
fn external_provider_cache_preserves_confirmations_until_revision_changes() {
    struct Provider {
        revision: AtomicU64,
        resolves: AtomicU64,
        image: super::WgpuUiExternalImage,
    }
    impl super::WgpuUiSurfaceExternalImageProvider for Provider {
        fn resolve(&self, _: &str, _: u64) -> Option<super::WgpuUiExternalImage> {
            self.resolves.fetch_add(1, Ordering::Relaxed);
            Some(self.image.clone())
        }
        fn cache_revision(&self) -> Option<u64> {
            Some(self.revision.load(Ordering::Relaxed))
        }
    }

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("external image cache regression requires an offscreen WGPU adapter");
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("external image cache regression requires an offscreen WGPU device");
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("external-image-cache-confirmation"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let provider = Provider {
        revision: AtomicU64::new(3),
        resolves: AtomicU64::new(0),
        image: super::WgpuUiExternalImage::new_opaque(texture, 1, 1, 17),
    };
    let layout = super::super::pipeline::create_image_bind_group_layout(&device);
    let sampler = super::super::pipeline::create_image_sampler(&device);
    let shared = super::WgpuUiSharedImageRegistry::default();
    let sources = [super::ImageUploadSource {
        resource_key: "viewport://test".to_string(),
        resource_generation: 17,
        command_indices: Vec::new(),
    }];
    let draw_list = UiSurfaceDrawList::with_generation((8, 8), None, Vec::new(), 17);
    let mut cache = super::WgpuUiImageCache::default();
    let mut staged = UiSurfaceImageResourceTable::default();
    for present in 0..2 {
        cache.prepare(
            &device,
            &queue,
            &layout,
            &sampler,
            present,
            &draw_list,
            &sources,
            Some(&provider),
            &shared,
            &mut staged,
        );
        assert_eq!(
            cache.resolved_external_source_indices(),
            &[0],
            "a prepared product must still be confirmable after a failed submission retries"
        );
    }
    assert_eq!(provider.resolves.load(Ordering::Relaxed), 1);

    provider.revision.store(4, Ordering::Relaxed);
    cache.prepare(
        &device,
        &queue,
        &layout,
        &sampler,
        2,
        &draw_list,
        &sources,
        Some(&provider),
        &shared,
        &mut staged,
    );
    assert_eq!(provider.resolves.load(Ordering::Relaxed), 2);
    assert_eq!(cache.resolved_external_source_indices(), &[0]);

    cache.prepare(
        &device,
        &queue,
        &layout,
        &sampler,
        3,
        &draw_list,
        &[],
        None,
        &shared,
        &mut staged,
    );
    assert!(cache.resolved_external_source_indices().is_empty());
}

#[test]
fn image_prepare_generation_cache_requires_empty_staged_resources() {
    let draw_list = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 17);
    let mut staged = UiSurfaceImageResourceTable::default();

    assert_eq!(
        reusable_image_prepare_generation(&draw_list, &staged, false, None),
        Some(17)
    );
    assert_eq!(
        reusable_image_prepare_generation(&draw_list, &staged, true, None),
        None,
        "an unversioned external provider may publish a newer GPU product within the same UI generation"
    );
    assert_eq!(
        reusable_image_prepare_generation(&draw_list, &staged, true, Some(3)),
        Some(17),
        "a stable external provider revision makes the retained GPU product reusable"
    );

    staged.insert(
        "icon://changed".to_string(),
        UiSurfaceImageResource {
            generation: 17,
            width: 1,
            height: 1,
            upload_bytes: 4,
            rgba: Arc::from([255, 255, 255, 255]),
        },
    );
    assert_eq!(
        reusable_image_prepare_generation(&draw_list, &staged, false, None),
        None
    );
}

#[test]
fn external_provider_generation_cache_requires_resident_products_and_a_revision() {
    assert_eq!(
        committed_external_image_prepare_generation(Some(17), true, Some(3), false),
        Some(17)
    );
    assert_eq!(
        committed_external_image_prepare_generation(Some(17), false, Some(3), false),
        None
    );
    assert_eq!(
        committed_external_image_prepare_generation(Some(17), true, None, false),
        None
    );
    assert_eq!(
        committed_external_image_prepare_generation(Some(17), true, Some(3), true),
        None
    );
}

#[test]
fn image_prepare_generation_cache_commits_only_complete_residency() {
    assert_eq!(
        committed_image_prepare_generation(Some(17), true, false, false),
        Some(17)
    );
    assert_eq!(
        committed_image_prepare_generation(Some(17), false, false, false),
        None
    );
    assert_eq!(
        committed_image_prepare_generation(Some(17), true, true, false),
        None,
        "external provider readiness is not versioned by the draw-list generation"
    );
    assert_eq!(
        committed_image_prepare_generation(Some(17), true, false, true),
        None,
        "a prepare that consumed staged resources must not publish a reusable generation"
    );
    assert_eq!(
        committed_image_prepare_generation(None, true, false, false),
        None
    );
}

#[test]
fn straight_srgba_is_linear_premultiplied_before_srgb_gpu_sampling() {
    let source = Arc::<[u8]>::from([
        255, 0, 0, 0, // transparent red must contribute no filtered color
        0, 0, 255, 128, // half-transparent blue
        40, 80, 120, 255, // opaque pixels remain byte-identical
    ]);

    let premultiplied = encode_linear_premultiplied_srgba8(source);

    assert_eq!(
        premultiplied.as_ref(),
        &[0, 0, 0, 0, 0, 0, 188, 128, 40, 80, 120, 255]
    );
}

#[test]
fn opaque_srgb_byte_lattice_remains_byte_exact() {
    let source = Arc::<[u8]>::from(
        (0_u8..=u8::MAX)
            .flat_map(|value| [value, value, value, u8::MAX])
            .collect::<Vec<_>>(),
    );

    assert_eq!(
        encode_linear_premultiplied_srgba8(Arc::clone(&source)),
        source,
        "linear-light texture admission must not shift opaque UI palette bytes"
    );
}

use std::collections::HashMap;

use crate::rhi::{UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImageResource};

use super::batching::ImageUploadSource;

// Editor image bytes are byte-space UI colors; avoid sRGB decode on the direct swapchain path.
const UI_IMAGE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MAX_UI_IMAGE_CACHE_ENTRIES: usize = 256;
const MAX_UI_IMAGE_CACHE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WgpuUiImageResourceStats {
    pub(super) upload_bytes: u64,
    pub(super) upload_write_count: u64,
    pub(super) cache_key_allocation_count: u64,
    pub(super) cache_prune_visit_count: u64,
    pub(super) cache_admission_reject_count: u64,
    pub(super) invalid_payload_count: u64,
    pub(super) cache_resident_bytes: u64,
    pub(super) cpu_resident_bytes: u64,
    pub(super) prepare_command_visit_count: u64,
    pub(super) prepare_cache_hit_count: u64,
}

#[derive(Default)]
pub(super) struct WgpuUiImageCache {
    resources: HashMap<String, WgpuUiImageResource>,
    resident_bytes: u64,
}

impl WgpuUiImageCache {
    pub(super) fn resources(&self) -> &HashMap<String, WgpuUiImageResource> {
        &self.resources
    }

    pub(super) fn is_resident(&self, resource_key: &str, generation: u64) -> bool {
        self.resources
            .get(resource_key)
            .is_some_and(|resource| resource.last_uploaded_generation == Some(generation))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_bind_group_layout: &wgpu::BindGroupLayout,
        image_sampler: &wgpu::Sampler,
        present_index: u64,
        draw_list: &UiSurfaceDrawList,
        image_upload_sources: &[ImageUploadSource],
        image_resources: &mut HashMap<String, UiSurfaceImageResource>,
    ) -> WgpuUiImageResourceStats {
        let mut stats = WgpuUiImageResourceStats::default();
        let max_texture_dimension_2d = device.limits().max_texture_dimension_2d;
        let mut entry_saturated_at_count = None;
        for image_upload_source in image_upload_sources {
            let cache_key = image_upload_source.resource_key.as_str();
            if cache_key.is_empty() {
                continue;
            }
            let mut staged_image_resource = image_resources.remove(cache_key);
            let image_metadata = staged_image_resource
                .as_ref()
                .map(|resource| {
                    (
                        resource.generation,
                        resource.width,
                        resource.height,
                        resource.rgba.len(),
                    )
                })
                .or_else(|| {
                    draw_list.image_resource(cache_key).map(|resource| {
                        (
                            resource.generation,
                            resource.width,
                            resource.height,
                            resource.rgba.len(),
                        )
                    })
                });
            let image_generation = image_metadata
                .map(|(generation, _, _, _)| generation)
                .or_else(|| image_payload_generation(draw_list, image_upload_source))
                .or_else(|| draw_list.generation());
            if let Some(generation) = image_generation {
                if let Some(resource) = self.resources.get_mut(cache_key) {
                    let resource_size_matches = image_metadata
                        .map_or(true, |(_, width, height, _)| {
                            resource.size == (width, height)
                        });
                    if resource_size_matches
                        && !image_upload_needs_write(
                            Some(generation),
                            resource.last_uploaded_generation,
                        )
                    {
                        resource.last_touched_present = present_index;
                        stats.prepare_cache_hit_count =
                            stats.prepare_cache_hit_count.saturating_add(1);
                        continue;
                    }
                }
            }
            let command_indices = if image_metadata.is_some() {
                image_upload_source
                    .command_indices
                    .first()
                    .map(std::slice::from_ref)
                    .unwrap_or_default()
            } else {
                image_upload_source.command_indices.as_slice()
            };
            for command_index in command_indices {
                stats.prepare_command_visit_count =
                    stats.prepare_command_visit_count.saturating_add(1);
                let Some(command) = draw_list.commands.get(*command_index) else {
                    continue;
                };
                let UiSurfaceCommandKind::Image { payload } = &command.kind else {
                    continue;
                };
                if payload.resource_key != cache_key {
                    continue;
                }
                let (width, height, rgba_len) =
                    if let Some((_, width, height, rgba_len)) = image_metadata {
                        (width, height, rgba_len)
                    } else {
                        let Some(rgba) = payload.rgba.as_deref() else {
                            if let Some(resource) = self.resources.get_mut(cache_key) {
                                resource.last_touched_present = present_index;
                            }
                            continue;
                        };
                        (payload.width, payload.height, rgba.len())
                    };
                let Some(layout) = image_payload_layout(width, height, max_texture_dimension_2d)
                else {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate(cache_key);
                    continue;
                };
                if rgba_len < layout.expected_len {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate(cache_key);
                    continue;
                }
                let cached_size = self.resources.get(cache_key).map(|resource| resource.size);
                let replace = cached_size != Some((width, height));
                if replace {
                    if !self.admit(
                        cache_key,
                        layout.expected_len as u64,
                        image_upload_sources,
                        cached_size.is_none(),
                        &mut entry_saturated_at_count,
                        &mut stats,
                    ) {
                        self.invalidate(cache_key);
                        continue;
                    }
                    let resource = WgpuUiImageResource::new(
                        device,
                        image_bind_group_layout,
                        image_sampler,
                        (width, height),
                        layout.expected_len as u64,
                        take_image_source_pixels(
                            &mut staged_image_resource,
                            draw_list,
                            cache_key,
                            payload.rgba.as_deref(),
                            layout.expected_len,
                        )
                        .expect("validated UI image source is available"),
                        present_index,
                    );
                    if let Some(cached) = self.resources.get_mut(cache_key) {
                        self.resident_bytes = self
                            .resident_bytes
                            .saturating_sub(cached.byte_size)
                            .saturating_add(resource.byte_size);
                        *cached = resource;
                    } else {
                        self.resident_bytes =
                            self.resident_bytes.saturating_add(resource.byte_size);
                        self.resources.insert(cache_key.to_owned(), resource);
                        stats.cache_key_allocation_count =
                            stats.cache_key_allocation_count.saturating_add(1);
                    }
                }
                if let Some(resource) = self.resources.get_mut(cache_key) {
                    resource.last_touched_present = present_index;
                    if image_upload_needs_write(image_generation, resource.last_uploaded_generation)
                    {
                        if !replace {
                            resource.cpu_rgba = take_image_source_pixels(
                                &mut staged_image_resource,
                                draw_list,
                                cache_key,
                                payload.rgba.as_deref(),
                                layout.expected_len,
                            )
                            .expect("validated UI image source is available");
                        }
                        queue.write_texture(
                            resource.texture.as_image_copy(),
                            &resource.cpu_rgba,
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(layout.bytes_per_row),
                                rows_per_image: Some(height),
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );
                        resource.last_uploaded_generation = image_generation;
                        stats.upload_write_count = stats.upload_write_count.saturating_add(1);
                        stats.upload_bytes = stats
                            .upload_bytes
                            .saturating_add(layout.expected_len as u64);
                    }
                    break;
                }
            }
        }
        image_resources.clear();
        stats.cache_resident_bytes = self.resident_bytes;
        stats.cpu_resident_bytes = self.resident_bytes;
        stats
    }

    fn admit(
        &mut self,
        cache_key: &str,
        required_bytes: u64,
        active_sources: &[ImageUploadSource],
        new_key: bool,
        entry_saturated_at_count: &mut Option<usize>,
        stats: &mut WgpuUiImageResourceStats,
    ) -> bool {
        if new_key && *entry_saturated_at_count == Some(self.resources.len()) {
            stats.cache_admission_reject_count =
                stats.cache_admission_reject_count.saturating_add(1);
            return false;
        }
        let replaced_bytes = self
            .resources
            .get(cache_key)
            .map_or(0, |resource| resource.byte_size);
        let entry_count_after = self.resources.len().saturating_add(usize::from(new_key));
        let cache_bytes_after = self
            .resident_bytes
            .saturating_sub(replaced_bytes)
            .saturating_add(required_bytes);
        let (action, visit_count) = image_cache_admission_plan(
            self.resources.iter().map(|(key, resource)| {
                let key_str = key.as_str();
                (
                    key_str,
                    resource.last_touched_present,
                    resource.byte_size,
                    active_sources
                        .binary_search_by(|source| source.resource_key.as_str().cmp(key_str))
                        .is_ok(),
                    key_str == cache_key,
                )
            }),
            entry_count_after,
            cache_bytes_after,
            MAX_UI_IMAGE_CACHE_ENTRIES,
            MAX_UI_IMAGE_CACHE_BYTES,
            required_bytes,
        );
        stats.cache_prune_visit_count = stats.cache_prune_visit_count.saturating_add(visit_count);
        match action {
            ImageCacheAdmissionAction::Admit { evict_keys } => {
                stats.cache_key_allocation_count = stats
                    .cache_key_allocation_count
                    .saturating_add(evict_keys.len() as u64);
                for key in evict_keys {
                    self.invalidate(&key);
                }
                true
            }
            ImageCacheAdmissionAction::Reject { entry_saturated } => {
                if new_key && entry_saturated {
                    *entry_saturated_at_count = Some(self.resources.len());
                }
                stats.cache_admission_reject_count =
                    stats.cache_admission_reject_count.saturating_add(1);
                false
            }
        }
    }

    fn invalidate(&mut self, cache_key: &str) -> bool {
        let Some(resource) = remove_cached_image(&mut self.resources, cache_key) else {
            return false;
        };
        self.resident_bytes = self.resident_bytes.saturating_sub(resource.byte_size);
        true
    }
}

fn image_payload_generation(
    draw_list: &UiSurfaceDrawList,
    image_upload_source: &ImageUploadSource,
) -> Option<u64> {
    image_upload_source
        .command_indices
        .iter()
        .find_map(|command_index| {
            let command = draw_list.commands.get(*command_index)?;
            let UiSurfaceCommandKind::Image { payload } = &command.kind else {
                return None;
            };
            (payload.resource_key == image_upload_source.resource_key)
                .then_some(payload.resource_generation)
        })
}

pub(super) fn take_image_source_pixels(
    staged_image_resource: &mut Option<UiSurfaceImageResource>,
    draw_list: &UiSurfaceDrawList,
    cache_key: &str,
    payload_rgba: Option<&[u8]>,
    expected_len: usize,
) -> Option<Vec<u8>> {
    if let Some(mut resource) = staged_image_resource.take() {
        resource.rgba.truncate(expected_len);
        return Some(resource.rgba);
    }
    let rgba = draw_list
        .image_resource(cache_key)
        .map(|resource| resource.rgba.as_slice())
        .or(payload_rgba)?;
    Some(rgba[..expected_len].to_vec())
}

pub(super) struct WgpuUiImageResource {
    texture: wgpu::Texture,
    pub(super) bind_group: wgpu::BindGroup,
    size: (u32, u32),
    byte_size: u64,
    // The bounded cache owns the canonical CPU source for this GPU texture.
    cpu_rgba: Vec<u8>,
    last_touched_present: u64,
    last_uploaded_generation: Option<u64>,
}

impl WgpuUiImageResource {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        size: (u32, u32),
        byte_size: u64,
        cpu_rgba: Vec<u8>,
        last_touched_present: u64,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-ui-image"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: UI_IMAGE_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-ui-image-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        Self {
            texture,
            bind_group,
            size,
            byte_size,
            cpu_rgba,
            last_touched_present,
            last_uploaded_generation: None,
        }
    }
}

pub(super) fn image_upload_needs_write(
    image_generation: Option<u64>,
    last_uploaded_generation: Option<u64>,
) -> bool {
    image_generation != last_uploaded_generation || image_generation.is_none()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ImagePayloadLayout {
    pub(super) expected_len: usize,
    pub(super) bytes_per_row: u32,
}

pub(super) fn image_payload_layout(
    width: u32,
    height: u32,
    max_texture_dimension_2d: u32,
) -> Option<ImagePayloadLayout> {
    if width == 0
        || height == 0
        || width > max_texture_dimension_2d
        || height > max_texture_dimension_2d
    {
        return None;
    }
    let bytes_per_row = width.checked_mul(4)?;
    let expected_len = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixel_count| pixel_count.checked_mul(4))
        .and_then(|byte_count| usize::try_from(byte_count).ok())?;
    Some(ImagePayloadLayout {
        expected_len,
        bytes_per_row,
    })
}

pub(super) fn remove_cached_image<T>(cache: &mut HashMap<String, T>, cache_key: &str) -> Option<T> {
    cache.remove(cache_key)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ImageCacheAdmissionAction {
    Admit { evict_keys: Vec<String> },
    Reject { entry_saturated: bool },
}

pub(super) fn image_cache_admission_plan<'a>(
    entries: impl Iterator<Item = (&'a str, u64, u64, bool, bool)>,
    cache_entry_count_after: usize,
    cache_bytes_after: u64,
    max_entries: usize,
    max_bytes: u64,
    required_bytes: u64,
) -> (ImageCacheAdmissionAction, u64) {
    if cache_entry_count_after <= max_entries && cache_bytes_after <= max_bytes {
        return (
            ImageCacheAdmissionAction::Admit {
                evict_keys: Vec::new(),
            },
            0,
        );
    }
    if max_entries == 0 || required_bytes > max_bytes {
        return (
            ImageCacheAdmissionAction::Reject {
                entry_saturated: false,
            },
            0,
        );
    }
    let mut visit_count = 0_u64;
    let mut candidates = Vec::new();
    for (key, last_touched_present, byte_size, active, target) in entries {
        visit_count = visit_count.saturating_add(1);
        if active || target {
            continue;
        }
        candidates.push((last_touched_present, key, byte_size));
    }
    candidates.sort_unstable_by_key(|(last_touched_present, key, _)| (*last_touched_present, *key));
    let mut retained_entries = cache_entry_count_after;
    let mut retained_bytes = cache_bytes_after;
    let mut evict_keys = Vec::new();
    for (_, key, byte_size) in candidates {
        if retained_entries <= max_entries && retained_bytes <= max_bytes {
            break;
        }
        retained_entries = retained_entries.saturating_sub(1);
        retained_bytes = retained_bytes.saturating_sub(byte_size);
        evict_keys.push(key.to_owned());
    }
    let action = if retained_entries <= max_entries && retained_bytes <= max_bytes {
        ImageCacheAdmissionAction::Admit { evict_keys }
    } else {
        ImageCacheAdmissionAction::Reject {
            entry_saturated: retained_entries > max_entries,
        }
    };
    (action, visit_count)
}

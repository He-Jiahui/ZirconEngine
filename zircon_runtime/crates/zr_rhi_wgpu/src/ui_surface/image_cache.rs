use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use zr_rhi::{
    UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImageResource, UiSurfaceImageResourceTable,
};

use super::batching::ImageUploadSource;
use super::color_space::encode_linear_premultiplied_srgba8;
use super::shared_image_registry::{
    SharedImagePrepareResult, WgpuUiImageAllocationSet, WgpuUiImageInFlightPins,
    WgpuUiSharedImageRegistry,
};
use super::{WgpuUiExternalImage, WgpuUiSurfaceExternalImageProvider};

mod resource;

pub(super) use resource::WgpuUiImageResource;

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
    pub(super) shared_resolve_count: u64,
    pub(super) shared_upload_write_count: u64,
    pub(super) shared_upload_bytes: u64,
    pub(super) shared_resident_bytes: u64,
    pub(super) device_allocation_count: u64,
    pub(super) device_allocation_bytes: u64,
    pub(super) registry_evicted_pinned_bytes: u64,
    pub(super) surface_pin_count: u64,
    pub(super) in_flight_present_pin_count: u64,
    pub(super) eviction_completion_count: u64,
}

#[derive(Default)]
pub(super) struct WgpuUiImageCache {
    resources: HashMap<String, BTreeMap<u64, WgpuUiImageResource>>,
    entry_count: usize,
    resident_bytes: u64,
    prepared_generation: Option<u64>,
    prepared_source_count: u64,
    prepared_allocation_set: WgpuUiImageAllocationSet,
    resolved_external_source_indices: Vec<usize>,
}

impl WgpuUiImageCache {
    pub(super) fn get(&self, resource_key: &str, generation: u64) -> Option<&WgpuUiImageResource> {
        self.resources
            .get(resource_key)
            .and_then(|generations| generations.get(&generation))
    }

    fn get_mut(&mut self, resource_key: &str, generation: u64) -> Option<&mut WgpuUiImageResource> {
        self.resources
            .get_mut(resource_key)
            .and_then(|generations| generations.get_mut(&generation))
    }

    fn insert(&mut self, resource_key: String, generation: u64, resource: WgpuUiImageResource) {
        let replaced = self
            .resources
            .entry(resource_key)
            .or_default()
            .insert(generation, resource);
        if let Some(replaced) = replaced {
            self.resident_bytes = self.resident_bytes.saturating_sub(replaced.byte_size);
        } else {
            self.entry_count = self.entry_count.saturating_add(1);
        }
    }

    fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub(super) fn residency_stats(
        &self,
        shared_images: &WgpuUiSharedImageRegistry,
    ) -> WgpuUiImageResourceStats {
        let allocation_stats = shared_images.allocation_stats();
        WgpuUiImageResourceStats {
            cache_resident_bytes: self.resident_bytes,
            shared_resident_bytes: shared_images.resident_bytes(),
            device_allocation_count: allocation_stats.allocation_count,
            device_allocation_bytes: allocation_stats.unique_allocation_bytes,
            registry_evicted_pinned_bytes: allocation_stats.registry_evicted_pinned_bytes,
            surface_pin_count: allocation_stats.surface_pin_count,
            in_flight_present_pin_count: allocation_stats.in_flight_present_pin_count,
            eviction_completion_count: allocation_stats.eviction_completion_count,
            ..WgpuUiImageResourceStats::default()
        }
    }

    pub(super) fn pin_prepared_allocations_for_submission(
        &self,
    ) -> Option<WgpuUiImageInFlightPins> {
        self.prepared_allocation_set.begin_in_flight()
    }

    pub(super) fn is_resident(&self, resource_key: &str, generation: u64) -> bool {
        self.get(resource_key, generation)
            .is_some_and(|resource| resource.last_uploaded_generation == Some(generation))
    }

    pub(super) fn resolved_external_source_indices(&self) -> &[usize] {
        &self.resolved_external_source_indices
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
        external_images: Option<&dyn WgpuUiSurfaceExternalImageProvider>,
        shared_images: &WgpuUiSharedImageRegistry,
        image_resources: &mut UiSurfaceImageResourceTable,
    ) -> WgpuUiImageResourceStats {
        // Provider readiness can change without a new draw-list generation.
        self.resolved_external_source_indices.clear();
        let external_images_present = external_images.is_some();
        let had_staged_resources = !image_resources.is_empty();
        if let Some(generation) =
            reusable_image_prepare_generation(draw_list, image_resources, external_images_present)
        {
            if self.prepared_generation == Some(generation) {
                let mut stats = self.residency_stats(shared_images);
                stats.prepare_cache_hit_count = self.prepared_source_count;
                return stats;
            }
        }
        self.prepared_generation = None;
        self.prepared_source_count = 0;
        self.prepared_allocation_set = WgpuUiImageAllocationSet::default();
        let mut stats = WgpuUiImageResourceStats::default();
        let max_texture_dimension_2d = device.limits().max_texture_dimension_2d;
        let mut entry_saturated_at_count = None;
        'source: for (source_index, image_upload_source) in image_upload_sources.iter().enumerate()
        {
            let cache_key = image_upload_source.resource_key.as_str();
            let image_generation = image_upload_source.resource_generation;
            if cache_key.is_empty() {
                continue;
            }
            if let Some(external_image) = external_images
                .and_then(|provider| provider.resolve(cache_key, image_generation))
                .filter(|image| image.matches_generation(image_generation))
            {
                if self.prepare_external_image(
                    device,
                    image_bind_group_layout,
                    image_sampler,
                    present_index,
                    cache_key,
                    image_generation,
                    external_image,
                    image_upload_sources,
                    &mut entry_saturated_at_count,
                    &mut stats,
                ) {
                    self.resolved_external_source_indices.push(source_index);
                }
                continue;
            }
            let mut staged_image_resource = image_resources.remove(cache_key, image_generation);
            let image_metadata = staged_image_resource
                .as_ref()
                .map(|resource| (resource.width, resource.height, resource.rgba.len()))
                .or_else(|| {
                    draw_list
                        .image_resource(cache_key, image_generation)
                        .map(|resource| (resource.width, resource.height, resource.rgba.len()))
                });
            let mut invalidate_size_mismatch = false;
            if let Some(resource) = self.get_mut(cache_key, image_generation) {
                let resource_size_matches = image_metadata
                    .map_or(true, |(width, height, _)| resource.size == (width, height));
                if resource_size_matches
                    && !image_upload_needs_write(
                        Some(image_generation),
                        resource.last_uploaded_generation,
                    )
                {
                    resource.last_touched_present = present_index;
                    stats.prepare_cache_hit_count = stats.prepare_cache_hit_count.saturating_add(1);
                    continue;
                }
                invalidate_size_mismatch = !resource_size_matches;
            }
            if invalidate_size_mismatch {
                self.invalidate(cache_key, image_generation);
            }
            if let Some(shared_image) =
                shared_images
                    .resolve(cache_key, image_generation)
                    .filter(|image| {
                        image_metadata.map_or(true, |(width, height, _)| {
                            image.width == width && image.height == height
                        })
                    })
            {
                stats.shared_resolve_count = stats.shared_resolve_count.saturating_add(1);
                self.prepare_external_image(
                    device,
                    image_bind_group_layout,
                    image_sampler,
                    present_index,
                    cache_key,
                    image_generation,
                    shared_image,
                    image_upload_sources,
                    &mut entry_saturated_at_count,
                    &mut stats,
                );
                continue;
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
                if payload.resource_key != cache_key
                    || payload.resource_generation != image_generation
                {
                    continue;
                }
                let (width, height, rgba_len) =
                    if let Some((width, height, rgba_len)) = image_metadata {
                        (width, height, rgba_len)
                    } else {
                        let Some(rgba) = payload.rgba.as_deref() else {
                            if let Some(resource) = self.get_mut(cache_key, image_generation) {
                                resource.last_touched_present = present_index;
                            }
                            continue;
                        };
                        (payload.width, payload.height, rgba.len())
                    };
                let Some(layout) = image_payload_layout(width, height, max_texture_dimension_2d)
                else {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate(cache_key, image_generation);
                    continue;
                };
                if rgba_len < layout.expected_len {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate(cache_key, image_generation);
                    continue;
                }
                let Some(source_pixels) = take_image_source_pixels(
                    &mut staged_image_resource,
                    draw_list,
                    cache_key,
                    image_generation,
                    payload.rgba.as_deref(),
                    layout.expected_len,
                ) else {
                    stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
                    self.invalidate(cache_key, image_generation);
                    continue;
                };
                let source_pixels = encode_linear_premultiplied_srgba8(source_pixels);
                match shared_images.prepare_pixels(
                    device,
                    queue,
                    cache_key,
                    image_generation,
                    width,
                    height,
                    &source_pixels,
                ) {
                    SharedImagePrepareResult::Cached(image) => {
                        stats.shared_resolve_count = stats.shared_resolve_count.saturating_add(1);
                        self.prepare_external_image(
                            device,
                            image_bind_group_layout,
                            image_sampler,
                            present_index,
                            cache_key,
                            image_generation,
                            image,
                            image_upload_sources,
                            &mut entry_saturated_at_count,
                            &mut stats,
                        );
                        continue 'source;
                    }
                    SharedImagePrepareResult::Uploaded {
                        image,
                        upload_bytes,
                    } => {
                        stats.upload_write_count = stats.upload_write_count.saturating_add(1);
                        stats.upload_bytes = stats.upload_bytes.saturating_add(upload_bytes);
                        stats.shared_upload_write_count =
                            stats.shared_upload_write_count.saturating_add(1);
                        stats.shared_upload_bytes =
                            stats.shared_upload_bytes.saturating_add(upload_bytes);
                        self.prepare_external_image(
                            device,
                            image_bind_group_layout,
                            image_sampler,
                            present_index,
                            cache_key,
                            image_generation,
                            image,
                            image_upload_sources,
                            &mut entry_saturated_at_count,
                            &mut stats,
                        );
                        continue 'source;
                    }
                    SharedImagePrepareResult::Rejected => {
                        stats.cache_admission_reject_count =
                            stats.cache_admission_reject_count.saturating_add(1);
                        self.invalidate(cache_key, image_generation);
                        continue 'source;
                    }
                }
            }
        }
        *image_resources = UiSurfaceImageResourceTable::default();
        self.prepared_allocation_set = WgpuUiImageAllocationSet::from_surface_pins(
            image_upload_sources
                .iter()
                .filter_map(|source| {
                    self.get(&source.resource_key, source.resource_generation)
                        .and_then(|resource| resource.shared_allocation_pin.as_ref())
                        .cloned()
                })
                .collect(),
        );
        let all_sources_resident = !external_images_present
            && image_upload_sources
                .iter()
                .all(|source| self.is_resident(&source.resource_key, source.resource_generation));
        self.prepared_generation = committed_image_prepare_generation(
            draw_list.generation(),
            all_sources_resident,
            external_images_present,
            had_staged_resources,
        );
        if self.prepared_generation.is_some() {
            self.prepared_source_count = image_upload_sources.len() as u64;
        }
        let residency = self.residency_stats(shared_images);
        stats.cache_resident_bytes = residency.cache_resident_bytes;
        stats.cpu_resident_bytes = residency.cpu_resident_bytes;
        stats.shared_resident_bytes = residency.shared_resident_bytes;
        stats.device_allocation_count = residency.device_allocation_count;
        stats.device_allocation_bytes = residency.device_allocation_bytes;
        stats.registry_evicted_pinned_bytes = residency.registry_evicted_pinned_bytes;
        stats.surface_pin_count = residency.surface_pin_count;
        stats.in_flight_present_pin_count = residency.in_flight_present_pin_count;
        stats.eviction_completion_count = residency.eviction_completion_count;
        stats
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_external_image(
        &mut self,
        device: &wgpu::Device,
        image_bind_group_layout: &wgpu::BindGroupLayout,
        image_sampler: &wgpu::Sampler,
        present_index: u64,
        cache_key: &str,
        generation: u64,
        image: WgpuUiExternalImage,
        active_sources: &[ImageUploadSource],
        entry_saturated_at_count: &mut Option<usize>,
        stats: &mut WgpuUiImageResourceStats,
    ) -> bool {
        let Some(layout) = image_payload_layout(
            image.width,
            image.height,
            device.limits().max_texture_dimension_2d,
        ) else {
            stats.invalid_payload_count = stats.invalid_payload_count.saturating_add(1);
            self.invalidate(cache_key, generation);
            return false;
        };
        let cached = self
            .get(cache_key, generation)
            .map(|resource| (resource.size, resource.last_uploaded_generation));
        if cached.is_some_and(|(size, uploaded_generation)| {
            size == (image.width, image.height) && uploaded_generation == Some(generation)
        }) {
            if let Some(resource) = self.get_mut(cache_key, generation) {
                resource.last_touched_present = present_index;
            }
            stats.prepare_cache_hit_count = stats.prepare_cache_hit_count.saturating_add(1);
            return true;
        }
        let cached_size = cached.map(|(size, _)| size);
        if !self.admit(
            cache_key,
            generation,
            layout.expected_len as u64,
            active_sources,
            cached_size.is_none(),
            entry_saturated_at_count,
            stats,
        ) {
            self.invalidate(cache_key, generation);
            return false;
        }
        let resource = WgpuUiImageResource::from_external(
            device,
            image_bind_group_layout,
            image_sampler,
            image,
            layout.expected_len as u64,
            present_index,
        );
        if cached_size.is_some() {
            let replaced_bytes = {
                let cached = self
                    .get_mut(cache_key, generation)
                    .expect("admission retains the replacement target");
                let replaced_bytes = cached.byte_size;
                *cached = resource;
                replaced_bytes
            };
            self.resident_bytes = self
                .resident_bytes
                .saturating_sub(replaced_bytes)
                .saturating_add(layout.expected_len as u64);
        } else {
            self.resident_bytes = self
                .resident_bytes
                .saturating_add(layout.expected_len as u64);
            self.insert(cache_key.to_owned(), generation, resource);
            stats.cache_key_allocation_count = stats.cache_key_allocation_count.saturating_add(1);
        }
        true
    }

    fn admit(
        &mut self,
        cache_key: &str,
        generation: u64,
        required_bytes: u64,
        active_sources: &[ImageUploadSource],
        new_key: bool,
        entry_saturated_at_count: &mut Option<usize>,
        stats: &mut WgpuUiImageResourceStats,
    ) -> bool {
        if new_key && *entry_saturated_at_count == Some(self.entry_count()) {
            stats.cache_admission_reject_count =
                stats.cache_admission_reject_count.saturating_add(1);
            return false;
        }
        let replaced_bytes = self
            .get(cache_key, generation)
            .map_or(0, |resource| resource.byte_size);
        let entry_count_after = self.entry_count().saturating_add(usize::from(new_key));
        let cache_bytes_after = self
            .resident_bytes
            .saturating_sub(replaced_bytes)
            .saturating_add(required_bytes);
        let (action, visit_count) = image_cache_admission_plan(
            self.resources.iter().flat_map(|(key, generations)| {
                generations
                    .iter()
                    .map(move |(cached_generation, resource)| {
                        let key_str = key.as_str();
                        (
                            key_str,
                            *cached_generation,
                            resource.last_touched_present,
                            resource.byte_size,
                            active_sources
                                .binary_search_by(|source| {
                                    (source.resource_key.as_str(), source.resource_generation)
                                        .cmp(&(key_str, *cached_generation))
                                })
                                .is_ok(),
                            key_str == cache_key && *cached_generation == generation,
                        )
                    })
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
                for (key, generation) in evict_keys {
                    self.invalidate(&key, generation);
                }
                true
            }
            ImageCacheAdmissionAction::Reject { entry_saturated } => {
                if new_key && entry_saturated {
                    *entry_saturated_at_count = Some(self.entry_count());
                }
                stats.cache_admission_reject_count =
                    stats.cache_admission_reject_count.saturating_add(1);
                false
            }
        }
    }

    fn invalidate(&mut self, cache_key: &str, generation: u64) -> bool {
        let Some(resource) = remove_cached_image(&mut self.resources, cache_key, generation) else {
            return false;
        };
        self.entry_count = self.entry_count.saturating_sub(1);
        self.resident_bytes = self.resident_bytes.saturating_sub(resource.byte_size);
        true
    }
}

fn reusable_image_prepare_generation(
    draw_list: &UiSurfaceDrawList,
    image_resources: &UiSurfaceImageResourceTable,
    external_images_present: bool,
) -> Option<u64> {
    (!external_images_present && image_resources.is_empty())
        .then(|| draw_list.generation())
        .flatten()
}

fn committed_image_prepare_generation(
    generation: Option<u64>,
    all_sources_resident: bool,
    external_images_present: bool,
    had_staged_resources: bool,
) -> Option<u64> {
    (!external_images_present && !had_staged_resources && all_sources_resident)
        .then_some(generation)
        .flatten()
}

#[cfg(test)]
mod residency_tests {
    use std::sync::Arc;

    use super::{
        committed_image_prepare_generation, encode_linear_premultiplied_srgba8,
        reusable_image_prepare_generation,
    };
    use zr_rhi::{UiSurfaceDrawList, UiSurfaceImageResource, UiSurfaceImageResourceTable};

    #[test]
    fn image_prepare_generation_cache_requires_empty_staged_resources() {
        let draw_list = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 17);
        let mut staged = UiSurfaceImageResourceTable::default();

        assert_eq!(
            reusable_image_prepare_generation(&draw_list, &staged, false),
            Some(17)
        );
        assert_eq!(
            reusable_image_prepare_generation(&draw_list, &staged, true),
            None,
            "external providers may publish a newer GPU product within the same UI generation"
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
            reusable_image_prepare_generation(&draw_list, &staged, false),
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
}

pub(super) fn take_image_source_pixels(
    staged_image_resource: &mut Option<UiSurfaceImageResource>,
    draw_list: &UiSurfaceDrawList,
    cache_key: &str,
    generation: u64,
    payload_rgba: Option<&[u8]>,
    expected_len: usize,
) -> Option<Arc<[u8]>> {
    if let Some(resource) = staged_image_resource.take() {
        if resource.rgba.len() < expected_len {
            return None;
        }
        if resource.rgba.len() == expected_len {
            return Some(resource.rgba);
        }
        return Some(Arc::from(&resource.rgba[..expected_len]));
    }
    if let Some(resource) = draw_list.image_resource(cache_key, generation) {
        if resource.rgba.len() < expected_len {
            return None;
        }
        if resource.rgba.len() == expected_len {
            return Some(Arc::clone(&resource.rgba));
        }
        return Some(Arc::from(&resource.rgba[..expected_len]));
    }
    let rgba = payload_rgba?;
    rgba.get(..expected_len).map(Arc::from)
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

pub(super) fn remove_cached_image<T>(
    cache: &mut HashMap<String, BTreeMap<u64, T>>,
    cache_key: &str,
    generation: u64,
) -> Option<T> {
    let (resource, remove_key) = {
        let generations = cache.get_mut(cache_key)?;
        let resource = generations.remove(&generation);
        (resource, generations.is_empty())
    };
    if remove_key {
        cache.remove(cache_key);
    }
    resource
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ImageCacheAdmissionAction {
    Admit { evict_keys: Vec<(String, u64)> },
    Reject { entry_saturated: bool },
}

pub(super) fn image_cache_admission_plan<'a>(
    entries: impl Iterator<Item = (&'a str, u64, u64, u64, bool, bool)>,
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
    for (key, generation, last_touched_present, byte_size, active, target) in entries {
        visit_count = visit_count.saturating_add(1);
        if active || target {
            continue;
        }
        candidates.push((last_touched_present, key, generation, byte_size));
    }
    candidates.sort_unstable_by_key(|(last_touched_present, key, generation, _)| {
        (*last_touched_present, *key, *generation)
    });
    let mut retained_entries = cache_entry_count_after;
    let mut retained_bytes = cache_bytes_after;
    let mut evict_keys = Vec::new();
    for (_, key, generation, byte_size) in candidates {
        if retained_entries <= max_entries && retained_bytes <= max_bytes {
            break;
        }
        retained_entries = retained_entries.saturating_sub(1);
        retained_bytes = retained_bytes.saturating_sub(byte_size);
        evict_keys.push((key.to_owned(), generation));
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

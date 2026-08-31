use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::color_space::UI_IMAGE_TEXTURE_FORMAT;
use super::image_cache::{image_payload_layout, remove_cached_image, ImageCacheAdmissionAction};
use super::WgpuUiExternalImage;

mod allocation_ledger;

pub(crate) use allocation_ledger::WgpuUiImageInFlightPins;
use allocation_ledger::{WgpuUiImageAllocationLedger, WgpuUiImageRegistryPin};
pub(super) use allocation_ledger::{
    WgpuUiImageAllocationSet, WgpuUiImageAllocationStats, WgpuUiImageSurfacePin,
};

const MAX_SHARED_UI_IMAGE_ENTRIES: usize = 256;
const MAX_SHARED_UI_IMAGE_BYTES: u64 = 64 * 1024 * 1024;

/// Device-scoped static UI texture storage shared by every native presenter.
///
/// The registry owns texture allocations, while each presenter owns only its view/bind-group
/// state. Resource generations are independent entries so an in-flight frame can keep sampling an
/// older texture after a newer revision has been published.
#[derive(Default)]
pub struct WgpuUiSharedImageRegistry {
    state: Mutex<SharedImageRegistryState>,
    resident_bytes_snapshot: AtomicU64,
    allocation_ledger: Arc<WgpuUiImageAllocationLedger>,
}

#[derive(Default)]
struct SharedImageRegistryState {
    resources: HashMap<String, BTreeMap<u64, SharedImageEntry>>,
    entry_count: usize,
    resident_bytes: u64,
    touch_epoch: u64,
}

struct SharedImageEntry {
    image: WgpuUiExternalImage,
    allocation: WgpuUiImageRegistryPin,
    byte_size: u64,
    last_touched_epoch: u64,
}

pub(super) enum SharedImagePrepareResult {
    Cached(WgpuUiExternalImage),
    Uploaded {
        image: WgpuUiExternalImage,
        upload_bytes: u64,
    },
    Rejected,
}

impl WgpuUiSharedImageRegistry {
    pub(super) fn resident_bytes(&self) -> u64 {
        self.resident_bytes_snapshot.load(Ordering::Relaxed)
    }

    pub(super) fn allocation_stats(&self) -> WgpuUiImageAllocationStats {
        self.allocation_ledger.stats()
    }

    pub(super) fn resolve(
        &self,
        resource_key: &str,
        generation: u64,
    ) -> Option<WgpuUiExternalImage> {
        let mut state = self.lock_state();
        state.touch_epoch = state.touch_epoch.saturating_add(1);
        let touch_epoch = state.touch_epoch;
        let entry = state
            .resources
            .get_mut(resource_key)?
            .get_mut(&generation)?;
        if !entry.image.matches_generation(generation) {
            return None;
        }
        entry.last_touched_epoch = touch_epoch;
        Some(
            entry
                .image
                .with_shared_allocation(entry.allocation.surface_pin()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_pixels(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resource_key: &str,
        generation: u64,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> SharedImagePrepareResult {
        // The caller canonicalizes straight producer bytes before this device-shared upload.
        let Some(layout) =
            image_payload_layout(width, height, device.limits().max_texture_dimension_2d)
        else {
            return SharedImagePrepareResult::Rejected;
        };
        let Some(rgba) = rgba.get(..layout.expected_len) else {
            return SharedImagePrepareResult::Rejected;
        };

        let mut state = self.lock_state();
        state.touch_epoch = state.touch_epoch.saturating_add(1);
        let touch_epoch = state.touch_epoch;
        if let Some(entry) = state
            .resources
            .get_mut(resource_key)
            .and_then(|generations| generations.get_mut(&generation))
            .filter(|entry| {
                entry.image.matches_generation(generation)
                    && entry.image.width == width
                    && entry.image.height == height
            })
        {
            entry.last_touched_epoch = touch_epoch;
            return SharedImagePrepareResult::Cached(
                entry
                    .image
                    .with_shared_allocation(entry.allocation.surface_pin()),
            );
        }

        let required_bytes = layout.expected_len as u64;
        let replaced = state
            .resources
            .get(resource_key)
            .and_then(|generations| generations.get(&generation))
            .map(|entry| {
                (
                    entry.byte_size,
                    entry.allocation.is_exclusively_registry_owned(),
                )
            });
        let replaced_bytes = replaced.map_or(0, |(byte_size, _)| byte_size);
        let is_new_entry = replaced_bytes == 0;
        let entry_count_after = state.entry_count.saturating_add(usize::from(is_new_entry));
        let unique_allocation_bytes_after = self
            .allocation_ledger
            .unique_allocation_bytes()
            .saturating_sub(
                replaced
                    .filter(|(_, exclusively_registry_owned)| *exclusively_registry_owned)
                    .map_or(0, |(byte_size, _)| byte_size),
            )
            .saturating_add(required_bytes);
        let action = shared_image_admission_plan(
            state.resources.iter().flat_map(|(key, generations)| {
                generations.iter().map(move |(cached_generation, entry)| {
                    (
                        key.as_str(),
                        *cached_generation,
                        entry.last_touched_epoch,
                        entry.byte_size,
                        entry.allocation.is_exclusively_registry_owned(),
                        key == resource_key && *cached_generation == generation,
                    )
                })
            }),
            entry_count_after,
            unique_allocation_bytes_after,
            required_bytes,
        );
        let ImageCacheAdmissionAction::Admit { evict_keys } = action else {
            return SharedImagePrepareResult::Rejected;
        };
        for (key, cached_generation) in evict_keys {
            remove_shared_image(&mut state, &key, cached_generation);
        }
        remove_shared_image(&mut state, resource_key, generation);

        let Some(allocation) =
            self.allocation_ledger
                .try_allocate(required_bytes, MAX_SHARED_UI_IMAGE_BYTES, || {
                    device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("zircon-ui-shared-image"),
                        size: wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: UI_IMAGE_TEXTURE_FORMAT,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    })
                })
        else {
            self.resident_bytes_snapshot
                .store(state.resident_bytes, Ordering::Relaxed);
            return SharedImagePrepareResult::Rejected;
        };
        queue.write_texture(
            allocation.texture().as_image_copy(),
            rgba,
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
        let image = WgpuUiExternalImage::new_premultiplied(
            allocation.texture().clone(),
            width,
            height,
            generation,
        );
        let surface_pin = allocation.surface_pin();
        let replaced_entry = state
            .resources
            .entry(resource_key.to_owned())
            .or_default()
            .insert(
                generation,
                SharedImageEntry {
                    image: image.clone(),
                    allocation,
                    byte_size: required_bytes,
                    last_touched_epoch: touch_epoch,
                },
            );
        if let Some(replaced_entry) = replaced_entry {
            state.resident_bytes = state
                .resident_bytes
                .saturating_sub(replaced_entry.byte_size);
        } else {
            state.entry_count = state.entry_count.saturating_add(1);
        }
        state.resident_bytes = state.resident_bytes.saturating_add(required_bytes);
        self.resident_bytes_snapshot
            .store(state.resident_bytes, Ordering::Relaxed);
        SharedImagePrepareResult::Uploaded {
            image: image.with_shared_allocation(surface_pin),
            upload_bytes: required_bytes,
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, SharedImageRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn remove_shared_image(state: &mut SharedImageRegistryState, resource_key: &str, generation: u64) {
    if let Some(entry) = remove_cached_image(&mut state.resources, resource_key, generation) {
        state.entry_count = state.entry_count.saturating_sub(1);
        state.resident_bytes = state.resident_bytes.saturating_sub(entry.byte_size);
    }
}

fn shared_image_admission_plan<'a>(
    entries: impl Iterator<Item = (&'a str, u64, u64, u64, bool, bool)>,
    entry_count_after: usize,
    unique_allocation_bytes_after: u64,
    required_bytes: u64,
) -> ImageCacheAdmissionAction {
    if entry_count_after <= MAX_SHARED_UI_IMAGE_ENTRIES
        && unique_allocation_bytes_after <= MAX_SHARED_UI_IMAGE_BYTES
    {
        return ImageCacheAdmissionAction::Admit {
            evict_keys: Vec::new(),
        };
    }
    if MAX_SHARED_UI_IMAGE_ENTRIES == 0 || required_bytes > MAX_SHARED_UI_IMAGE_BYTES {
        return ImageCacheAdmissionAction::Reject {
            entry_saturated: false,
        };
    }
    let mut candidates = entries
        .filter_map(
            |(key, generation, touched, byte_size, immediately_releasable, target)| {
                (!target).then_some((touched, key, generation, byte_size, immediately_releasable))
            },
        )
        .collect::<Vec<_>>();
    candidates
        .sort_unstable_by_key(|(touched, key, generation, _, _)| (*touched, *key, *generation));
    let mut retained_entries = entry_count_after;
    let mut retained_allocation_bytes = unique_allocation_bytes_after;
    let mut evict_keys = Vec::new();
    for (_, key, generation, byte_size, immediately_releasable) in candidates {
        if retained_entries <= MAX_SHARED_UI_IMAGE_ENTRIES
            && retained_allocation_bytes <= MAX_SHARED_UI_IMAGE_BYTES
        {
            break;
        }
        retained_entries = retained_entries.saturating_sub(1);
        if immediately_releasable {
            retained_allocation_bytes = retained_allocation_bytes.saturating_sub(byte_size);
        }
        evict_keys.push((key.to_owned(), generation));
    }
    if retained_entries <= MAX_SHARED_UI_IMAGE_ENTRIES
        && retained_allocation_bytes <= MAX_SHARED_UI_IMAGE_BYTES
    {
        ImageCacheAdmissionAction::Admit { evict_keys }
    } else {
        ImageCacheAdmissionAction::Reject {
            entry_saturated: retained_entries > MAX_SHARED_UI_IMAGE_ENTRIES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{shared_image_admission_plan, ImageCacheAdmissionAction};

    #[test]
    fn shared_registry_evicts_the_least_recent_cross_window_texture() {
        let entries = [
            ("older", 1, 3, 32 * 1024 * 1024, true, false),
            ("newer", 1, 8, 32 * 1024 * 1024, true, false),
        ];

        let action =
            shared_image_admission_plan(entries.into_iter(), 3, 80 * 1024 * 1024, 16 * 1024 * 1024);

        assert_eq!(
            action,
            ImageCacheAdmissionAction::Admit {
                evict_keys: vec![("older".to_owned(), 1)],
            }
        );
    }

    #[test]
    fn shared_registry_never_evicts_the_resource_being_replaced() {
        let entries = [("target", 7, 1, 60 * 1024 * 1024, true, true)];

        let action =
            shared_image_admission_plan(entries.into_iter(), 1, 65 * 1024 * 1024, 65 * 1024 * 1024);

        assert_eq!(
            action,
            ImageCacheAdmissionAction::Reject {
                entry_saturated: false,
            }
        );
    }

    #[test]
    fn shared_registry_counts_pinned_evictions_as_live_device_bytes() {
        let entries = [
            ("pinned", 1, 1, 32 * 1024 * 1024, false, false),
            ("releasable", 1, 2, 32 * 1024 * 1024, true, false),
        ];

        let action =
            shared_image_admission_plan(entries.into_iter(), 3, 80 * 1024 * 1024, 16 * 1024 * 1024);

        assert_eq!(
            action,
            ImageCacheAdmissionAction::Admit {
                evict_keys: vec![("pinned".to_owned(), 1), ("releasable".to_owned(), 1),],
            }
        );
    }

    #[test]
    fn shared_registry_rejects_when_only_surface_pinned_bytes_remain() {
        let entries = [("pinned", 1, 1, 64 * 1024 * 1024, false, false)];

        let action =
            shared_image_admission_plan(entries.into_iter(), 2, 65 * 1024 * 1024, 1024 * 1024);

        assert_eq!(
            action,
            ImageCacheAdmissionAction::Reject {
                entry_saturated: false,
            }
        );
    }
}

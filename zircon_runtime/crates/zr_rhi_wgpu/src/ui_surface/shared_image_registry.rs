use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use super::image_cache::{
    image_cache_admission_plan, image_payload_layout, remove_cached_image,
    ImageCacheAdmissionAction, UI_IMAGE_TEXTURE_FORMAT,
};
use super::WgpuUiExternalImage;

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
}

#[derive(Default)]
struct SharedImageRegistryState {
    resources: HashMap<String, BTreeMap<u64, SharedImageEntry>>,
    resident_bytes: u64,
    touch_epoch: u64,
}

struct SharedImageEntry {
    image: WgpuUiExternalImage,
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
        Some(entry.image.clone())
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
            return SharedImagePrepareResult::Cached(entry.image.clone());
        }

        let required_bytes = layout.expected_len as u64;
        let replaced_bytes = state
            .resources
            .get(resource_key)
            .and_then(|generations| generations.get(&generation))
            .map_or(0, |entry| entry.byte_size);
        let is_new_entry = replaced_bytes == 0;
        let entry_count_after =
            shared_entry_count(&state.resources).saturating_add(usize::from(is_new_entry));
        let resident_bytes_after = state
            .resident_bytes
            .saturating_sub(replaced_bytes)
            .saturating_add(required_bytes);
        let action = shared_image_admission_plan(
            state.resources.iter().flat_map(|(key, generations)| {
                generations.iter().map(move |(cached_generation, entry)| {
                    (
                        key.as_str(),
                        *cached_generation,
                        entry.last_touched_epoch,
                        entry.byte_size,
                        key == resource_key && *cached_generation == generation,
                    )
                })
            }),
            entry_count_after,
            resident_bytes_after,
            required_bytes,
        );
        let ImageCacheAdmissionAction::Admit { evict_keys } = action else {
            return SharedImagePrepareResult::Rejected;
        };
        for (key, cached_generation) in evict_keys {
            remove_shared_image(&mut state, &key, cached_generation);
        }
        remove_shared_image(&mut state, resource_key, generation);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
        });
        queue.write_texture(
            texture.as_image_copy(),
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
        let image = WgpuUiExternalImage::new_premultiplied(texture, width, height, generation);
        state
            .resources
            .entry(resource_key.to_owned())
            .or_default()
            .insert(
                generation,
                SharedImageEntry {
                    image: image.clone(),
                    byte_size: required_bytes,
                    last_touched_epoch: touch_epoch,
                },
            );
        state.resident_bytes = state.resident_bytes.saturating_add(required_bytes);
        self.resident_bytes_snapshot
            .store(state.resident_bytes, Ordering::Relaxed);
        SharedImagePrepareResult::Uploaded {
            image,
            upload_bytes: required_bytes,
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, SharedImageRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn shared_entry_count(resources: &HashMap<String, BTreeMap<u64, SharedImageEntry>>) -> usize {
    resources.values().map(BTreeMap::len).sum()
}

fn remove_shared_image(state: &mut SharedImageRegistryState, resource_key: &str, generation: u64) {
    if let Some(entry) = remove_cached_image(&mut state.resources, resource_key, generation) {
        state.resident_bytes = state.resident_bytes.saturating_sub(entry.byte_size);
    }
}

fn shared_image_admission_plan<'a>(
    entries: impl Iterator<Item = (&'a str, u64, u64, u64, bool)>,
    entry_count_after: usize,
    resident_bytes_after: u64,
    required_bytes: u64,
) -> ImageCacheAdmissionAction {
    image_cache_admission_plan(
        entries.map(|(key, generation, touched, bytes, target)| {
            (key, generation, touched, bytes, false, target)
        }),
        entry_count_after,
        resident_bytes_after,
        MAX_SHARED_UI_IMAGE_ENTRIES,
        MAX_SHARED_UI_IMAGE_BYTES,
        required_bytes,
    )
    .0
}

#[cfg(test)]
mod tests {
    use super::{shared_image_admission_plan, ImageCacheAdmissionAction};

    #[test]
    fn shared_registry_evicts_the_least_recent_cross_window_texture() {
        let entries = [
            ("older", 1, 3, 32 * 1024 * 1024, false),
            ("newer", 1, 8, 32 * 1024 * 1024, false),
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
        let entries = [("target", 7, 1, 60 * 1024 * 1024, true)];

        let action =
            shared_image_admission_plan(entries.into_iter(), 1, 65 * 1024 * 1024, 65 * 1024 * 1024);

        assert_eq!(
            action,
            ImageCacheAdmissionAction::Reject {
                entry_saturated: false,
            }
        );
    }
}

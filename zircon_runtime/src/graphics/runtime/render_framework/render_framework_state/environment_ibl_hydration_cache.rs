use std::collections::{HashSet, VecDeque};
use std::mem::size_of_val;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    EnvironmentIblHydrationReport, IblBakeArtifactRequest, SourceCubemapEnvironment,
    ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY,
};
use crate::graphics::EnvironmentIblBakeReservation;

const ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY: usize = ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY;

struct EnvironmentIblHydrationCacheEntry {
    request: IblBakeArtifactRequest,
    environment: SourceCubemapEnvironment,
}

/// Keeps decoded cache payloads and their prepared upload rows out of the frame hot path.
#[derive(Default)]
pub(in crate::graphics::runtime::render_framework) struct EnvironmentIblHydrationCache {
    entries: VecDeque<EnvironmentIblHydrationCacheEntry>,
    pending_runtime_bakes: VecDeque<IblBakeArtifactRequest>,
    observation_epoch: u64,
    resident_decoded_texel_bytes: u64,
    resident_prepared_upload_bytes: u64,
    hit_count: u64,
    miss_count: u64,
    insert_count: u64,
    eviction_count: u64,
    reservation_count: u64,
    reservation_suppression_count: u64,
    reservation_release_count: u64,
}

impl EnvironmentIblHydrationCache {
    pub(in crate::graphics::runtime::render_framework) fn get(
        &mut self,
        request: &IblBakeArtifactRequest,
        source: &SourceCubemapEnvironment,
    ) -> Option<SourceCubemapEnvironment> {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.request == *request)
        else {
            self.miss_count = self.miss_count.saturating_add(1);
            self.advance_observation();
            return None;
        };
        self.hit_count = self.hit_count.saturating_add(1);
        if index == 0 {
            let mut environment = self.entries.front()?.environment.clone();
            environment.intensity = source.intensity;
            environment.rotation_radians = source.rotation_radians;
            self.advance_observation();
            return Some(environment);
        }
        let entry = self.entries.remove(index)?;
        let mut environment = entry.environment.clone();
        environment.intensity = source.intensity;
        environment.rotation_radians = source.rotation_radians;
        self.entries.push_front(entry);
        self.advance_observation();
        Some(environment)
    }

    pub(in crate::graphics::runtime::render_framework) fn insert(
        &mut self,
        request: IblBakeArtifactRequest,
        environment: SourceCubemapEnvironment,
    ) {
        if self.remove_pending_runtime_bake(&request) {
            self.reservation_release_count = self.reservation_release_count.saturating_add(1);
        }
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.request == request)
        {
            self.entries.remove(index);
        }
        self.entries.push_front(EnvironmentIblHydrationCacheEntry {
            request,
            environment,
        });
        self.insert_count = self.insert_count.saturating_add(1);
        if self.entries.len() > ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY {
            self.entries.pop_back();
            self.eviction_count = self.eviction_count.saturating_add(1);
        }
        self.refresh_resident_bytes();
        self.advance_observation();
    }

    /// Reserves one runtime bake request before graph compilation.
    ///
    /// The matching GPU readback owns persistence. Until it publishes a cache
    /// payload, later frames suppress duplicate bake graphs for this request.
    pub(in crate::graphics::runtime::render_framework) fn begin_runtime_bake(
        &mut self,
        request: IblBakeArtifactRequest,
    ) -> bool {
        if self
            .pending_runtime_bakes
            .iter()
            .any(|pending| *pending == request)
            || self.pending_runtime_bakes.len() >= ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY
        {
            self.reservation_suppression_count =
                self.reservation_suppression_count.saturating_add(1);
            self.advance_observation();
            return false;
        }
        self.pending_runtime_bakes.push_back(request);
        self.reservation_count = self.reservation_count.saturating_add(1);
        self.advance_observation();
        true
    }

    pub(in crate::graphics::runtime::render_framework) fn reserve_runtime_bake(
        cache: &Arc<Mutex<Self>>,
        request: IblBakeArtifactRequest,
    ) -> Option<EnvironmentIblBakeReservation> {
        let reserved = cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .begin_runtime_bake(request);
        reserved.then(|| {
            let cache = Arc::clone(cache);
            EnvironmentIblBakeReservation::new(move || {
                cache
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .clear_pending_runtime_bake(&request);
            })
        })
    }

    pub(in crate::graphics::runtime::render_framework) fn clear_pending_runtime_bake(
        &mut self,
        request: &IblBakeArtifactRequest,
    ) {
        if self.remove_pending_runtime_bake(request) {
            self.reservation_release_count = self.reservation_release_count.saturating_add(1);
            self.advance_observation();
        }
    }

    fn remove_pending_runtime_bake(&mut self, request: &IblBakeArtifactRequest) -> bool {
        if let Some(index) = self
            .pending_runtime_bakes
            .iter()
            .position(|pending| pending == request)
        {
            self.pending_runtime_bakes.remove(index);
            return true;
        }
        false
    }

    fn advance_observation(&mut self) {
        self.observation_epoch = self.observation_epoch.saturating_add(1);
    }

    fn refresh_resident_bytes(&mut self) {
        let mut decoded_allocations = HashSet::new();
        let mut prepared_upload_allocations = HashSet::new();
        let mut decoded_texel_bytes = 0_u64;
        let mut prepared_upload_bytes = 0_u64;

        for entry in &self.entries {
            let environment = &entry.environment;
            accumulate_unique_slice_bytes(
                &mut decoded_allocations,
                environment.mip_chain.source_texels(),
                &mut decoded_texel_bytes,
            );
            accumulate_unique_slice_bytes(
                &mut decoded_allocations,
                environment.mip_chain.pmrem_texels(),
                &mut decoded_texel_bytes,
            );
            if let Some(irradiance) = environment.irradiance_cube() {
                accumulate_unique_slice_bytes(
                    &mut decoded_allocations,
                    irradiance.texels(),
                    &mut decoded_texel_bytes,
                );
            }
            let Some(upload) = environment.prepared_upload_artifact() else {
                continue;
            };
            for mip in upload.source_mips().iter().chain(upload.pmrem_mips()) {
                accumulate_unique_slice_bytes(
                    &mut prepared_upload_allocations,
                    mip.bytes(),
                    &mut prepared_upload_bytes,
                );
            }
            accumulate_unique_slice_bytes(
                &mut prepared_upload_allocations,
                upload.irradiance_mip().bytes(),
                &mut prepared_upload_bytes,
            );
        }

        self.resident_decoded_texel_bytes = decoded_texel_bytes;
        self.resident_prepared_upload_bytes = prepared_upload_bytes;
    }

    pub(in crate::graphics::runtime::render_framework) fn report(
        &self,
    ) -> EnvironmentIblHydrationReport {
        let mut resident_requests = [None; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY];
        for (slot, entry) in self.entries.iter().enumerate() {
            resident_requests[slot] = Some(entry.request);
        }
        let mut pending_requests = [None; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY];
        for (slot, request) in self.pending_runtime_bakes.iter().enumerate() {
            pending_requests[slot] = Some(*request);
        }

        EnvironmentIblHydrationReport {
            observation_epoch: self.observation_epoch,
            resident_count: self.entries.len() as u32,
            pending_count: self.pending_runtime_bakes.len() as u32,
            resident_decoded_texel_bytes: self.resident_decoded_texel_bytes,
            resident_prepared_upload_bytes: self.resident_prepared_upload_bytes,
            resident_payload_bytes: self
                .resident_decoded_texel_bytes
                .saturating_add(self.resident_prepared_upload_bytes),
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            insert_count: self.insert_count,
            eviction_count: self.eviction_count,
            reservation_count: self.reservation_count,
            reservation_suppression_count: self.reservation_suppression_count,
            reservation_release_count: self.reservation_release_count,
            resident_requests,
            pending_requests,
        }
    }
}

fn accumulate_unique_slice_bytes<T>(
    allocations: &mut HashSet<(usize, usize)>,
    slice: &[T],
    total_bytes: &mut u64,
) {
    if slice.is_empty() {
        return;
    }
    let byte_len = size_of_val(slice);
    let identity = (slice.as_ptr().cast::<()>() as usize, byte_len);
    if allocations.insert(identity) {
        *total_bytes = total_bytes.saturating_add(u64::try_from(byte_len).unwrap_or(u64::MAX));
    }
}

#[cfg(test)]
#[path = "tests/environment_ibl_hydration_cache.rs"]
mod tests;

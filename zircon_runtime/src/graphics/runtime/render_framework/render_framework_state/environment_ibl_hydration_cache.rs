use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{IblBakeArtifactRequest, SourceCubemapEnvironment};
use crate::graphics::EnvironmentIblBakeReservation;

const ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY: usize = 4;

struct EnvironmentIblHydrationCacheEntry {
    request: IblBakeArtifactRequest,
    environment: SourceCubemapEnvironment,
}

/// Keeps decoded cache payloads and their prepared upload rows out of the frame hot path.
#[derive(Default)]
pub(in crate::graphics::runtime::render_framework) struct EnvironmentIblHydrationCache {
    entries: VecDeque<EnvironmentIblHydrationCacheEntry>,
    pending_runtime_bakes: VecDeque<IblBakeArtifactRequest>,
}

impl EnvironmentIblHydrationCache {
    pub(in crate::graphics::runtime::render_framework) fn get(
        &mut self,
        request: &IblBakeArtifactRequest,
        source: &SourceCubemapEnvironment,
    ) -> Option<SourceCubemapEnvironment> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.request == *request)?;
        let entry = self.entries.remove(index)?;
        let mut environment = entry.environment.clone();
        environment.intensity = source.intensity;
        environment.rotation_radians = source.rotation_radians;
        self.entries.push_front(entry);
        Some(environment)
    }

    pub(in crate::graphics::runtime::render_framework) fn insert(
        &mut self,
        request: IblBakeArtifactRequest,
        environment: SourceCubemapEnvironment,
    ) {
        self.clear_pending_runtime_bake(&request);
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
        self.entries
            .truncate(ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY);
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
            return false;
        }
        self.pending_runtime_bakes.push_back(request);
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
        if let Some(index) = self
            .pending_runtime_bakes
            .iter()
            .position(|pending| pending == request)
        {
            self.pending_runtime_bakes.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::core::framework::render::{
        IblBakeArtifactRequest, IblBakeKey, SourceCubemapEnvironment, SourceCubemapMipChain,
    };

    use super::{EnvironmentIblHydrationCache, ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY};

    #[test]
    fn hydration_cache_reuses_prepared_payload_and_applies_current_runtime_controls() {
        let request = request(7);
        let cached = environment(7, 1.0, 0.0).with_prepared_upload_artifact();
        let cached_pmrem = cached.mip_chain.pmrem_texels().as_ptr();
        let mut cache = EnvironmentIblHydrationCache::default();
        cache.insert(request, cached);

        let current = environment(7, 2.5, 0.75);
        for _ in 0..60 {
            let reused = cache
                .get(&request, &current)
                .expect("an unchanged request should reuse its hydration");
            assert_eq!(reused.intensity, 2.5);
            assert_eq!(reused.rotation_radians, 0.75);
            assert_eq!(reused.mip_chain.pmrem_texels().as_ptr(), cached_pmrem);
            assert!(reused.prepared_upload_artifact().is_some());
        }
    }

    #[test]
    fn hydration_cache_is_bounded_and_evicts_the_least_recent_request() {
        let mut cache = EnvironmentIblHydrationCache::default();
        for identity in 0..=ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY as u64 {
            cache.insert(request(identity), environment(identity, 1.0, 0.0));
        }

        assert!(cache.get(&request(0), &environment(0, 1.0, 0.0)).is_none());
        assert!(cache
            .get(
                &request(ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY as u64),
                &environment(ENVIRONMENT_IBL_HYDRATION_CACHE_CAPACITY as u64, 1.0, 0.0,),
            )
            .is_some());
    }

    #[test]
    fn pending_runtime_bake_reserves_one_graph_until_cache_hydration() {
        let request = request(7);
        let mut cache = EnvironmentIblHydrationCache::default();

        assert!(cache.begin_runtime_bake(request));
        assert!(!cache.begin_runtime_bake(request));

        cache.insert(request, environment(7, 1.0, 0.0));
        assert!(cache.begin_runtime_bake(request));
    }

    #[test]
    fn dropped_runtime_bake_reservation_releases_the_request_for_retry() {
        let request = request(7);
        let cache = Arc::new(Mutex::<EnvironmentIblHydrationCache>::default());
        let reservation = EnvironmentIblHydrationCache::reserve_runtime_bake(&cache, request)
            .expect("first runtime bake should reserve the request");
        assert!(EnvironmentIblHydrationCache::reserve_runtime_bake(&cache, request).is_none());

        drop(reservation);

        assert!(EnvironmentIblHydrationCache::reserve_runtime_bake(&cache, request).is_some());
    }

    fn request(identity: u64) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            IblBakeKey::source_cubemap(identity, [identity as u32; 4]),
            1,
            1,
        )
        .with_pmrem_layout(1, 1)
    }

    fn environment(
        identity: u64,
        intensity: f32,
        rotation_radians: f32,
    ) -> SourceCubemapEnvironment {
        let mip_chain = SourceCubemapMipChain::new(
            1,
            1,
            vec![[identity as f32, 0.0, 0.0, 1.0]; 6],
            1,
            1,
            vec![[0.0, identity as f32, 0.0, 1.0]; 6],
        );
        let mut environment =
            SourceCubemapEnvironment::new(mip_chain, identity, [identity as u32; 4]);
        environment.intensity = intensity;
        environment.rotation_radians = rotation_radians;
        environment
    }
}

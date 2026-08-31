use super::super::random::RandomService;
use super::CoreHandle;

impl CoreHandle {
    /// Returns this runtime instance's seed authority and unique stream registry.
    ///
    /// A seed is selected while the runtime is constructed. Changing it during
    /// simulation would make stream selection depend on call timing, so replay
    /// restoration creates a new runtime from `RandomServiceCheckpoint`. A
    /// `RandomServiceState` restores only the authority for future unseen keys.
    pub fn random_service(&self) -> &RandomService {
        self.inner.random_service()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::CoreRuntime;
    use zr_contracts::random::{
        RandomPurposeKey, RandomState, RandomStreamKey, RandomSystemKey, RandomWorldKey,
    };

    fn stream_key() -> RandomStreamKey {
        RandomStreamKey::for_world(
            RandomWorldKey::new(3, 1),
            RandomSystemKey::new(8),
            RandomPurposeKey::new(13),
            0x5eed,
        )
    }

    fn stream_state(runtime: &CoreRuntime) -> RandomState {
        runtime
            .random_service()
            .acquire_stream(stream_key())
            .expect("runtime stream admission")
            .release()
    }

    #[test]
    fn runtime_owns_a_stable_seed_authority_without_using_wall_time() {
        let default_runtime = CoreRuntime::new();
        let same_default_runtime = CoreRuntime::new();
        let configured_runtime = CoreRuntime::with_random_seed(0x7788);
        let restored_runtime =
            CoreRuntime::with_random_service_state(configured_runtime.random_service().snapshot());

        assert_eq!(
            stream_state(&default_runtime),
            stream_state(&same_default_runtime)
        );
        assert_ne!(
            stream_state(&default_runtime),
            stream_state(&configured_runtime)
        );
        assert_eq!(
            stream_state(&configured_runtime),
            stream_state(&restored_runtime)
        );

        let configured_stream = stream_state(&configured_runtime);
        let mut detached_service = crate::core::runtime::random::RandomService::from_state(
            configured_runtime.random_service().snapshot(),
        );
        detached_service
            .reseed(0x9900)
            .expect("detached generation zero can be advanced");
        assert_eq!(stream_state(&configured_runtime), configured_stream);
    }

    #[test]
    fn runtime_checkpoint_restores_registered_stream_progress() {
        let runtime = CoreRuntime::with_random_seed(0x7788);
        let mut lease = runtime
            .random_service()
            .acquire_stream(stream_key())
            .expect("runtime stream admission");
        lease.try_next_u32().expect("first draw");
        lease.release();
        let checkpoint = runtime
            .random_service()
            .checkpoint()
            .expect("idle registry checkpoint");
        let restored = CoreRuntime::with_random_service_checkpoint(checkpoint)
            .expect("checkpoint within the Runtime stream capacity");

        let mut original_next = runtime
            .random_service()
            .acquire_stream(stream_key())
            .expect("original stream resume");
        let mut restored_next = restored
            .random_service()
            .acquire_stream(stream_key())
            .expect("restored stream resume");
        assert_eq!(original_next.try_next_u32(), restored_next.try_next_u32());
        assert_eq!(original_next.draw_index(), restored_next.draw_index());
    }
}

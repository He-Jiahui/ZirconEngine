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
#[path = "tests/random.rs"]
mod tests;

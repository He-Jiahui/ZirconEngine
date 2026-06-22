/// Shared per-frame context used by provider-specific prepare input wrappers.
pub(crate) struct RuntimeProviderPrepareInput<'a, E> {
    extract: Option<&'a E>,
    generation: u64,
}

impl<'a, E> RuntimeProviderPrepareInput<'a, E> {
    pub(crate) fn new(extract: Option<&'a E>, generation: u64) -> Self {
        Self {
            extract,
            generation,
        }
    }

    pub(crate) fn extract(&self) -> Option<&'a E> {
        self.extract
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }
}

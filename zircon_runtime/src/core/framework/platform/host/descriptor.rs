use super::{PlatformHostBackendKind, PlatformHostThreadAffinity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformHostDescriptor {
    backend: PlatformHostBackendKind,
    thread_affinity: PlatformHostThreadAffinity,
}

impl PlatformHostDescriptor {
    pub const fn new(
        backend: PlatformHostBackendKind,
        thread_affinity: PlatformHostThreadAffinity,
    ) -> Self {
        Self {
            backend,
            thread_affinity,
        }
    }

    pub const fn backend(self) -> PlatformHostBackendKind {
        self.backend
    }

    pub const fn thread_affinity(self) -> PlatformHostThreadAffinity {
        self.thread_affinity
    }
}

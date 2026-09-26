use super::{PlatformHostBackendKind, PlatformHostThreadAffinity};

/// 宿主安装时声明的后端身份与线程约束；它不是能力已经可用的观测证据。
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

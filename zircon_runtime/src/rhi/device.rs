use super::capabilities::{RenderBackendCaps, RenderQueueClass};

pub trait CommandList: Send {
    fn queue_class(&self) -> RenderQueueClass;
    fn label(&self) -> Option<&str>;
}

pub trait RenderDevice: Send + Sync {
    fn caps(&self) -> &RenderBackendCaps;

    fn backend_name(&self) -> &str {
        &self.caps().backend_name
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FenceValue(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransientAllocatorStats {
    pub bytes_reserved: u64,
    pub allocations: u32,
}

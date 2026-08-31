use serde::{Deserialize, Serialize};

const REFERENCE_TRANSIENT_TEXTURE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;
const REFERENCE_TRANSIENT_BUFFER_BUDGET_BYTES: u64 = 256 * 1024 * 1024;
const REFERENCE_STAGING_BUDGET_BYTES: u64 = 64 * 1024 * 1024;
const REFERENCE_PERSISTENT_TEXTURE_BUDGET_BYTES: u64 = 1024 * 1024 * 1024;
const REFERENCE_MAX_PENDING_UPLOADS: usize = 16;

/// RHI-owned memory classes that can reject admission under a hard budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuMemoryClass {
    Buffer,
    Texture,
    UploadStaging,
}

/// Explicit device-generation memory policy.
///
/// The reference policy is the existing 1080p mid-tier render framework
/// configuration, now owned by RHI so allocators, upload admission, and frame
/// diagnostics cannot drift apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuMemoryBudget {
    transient_texture_bytes: u64,
    transient_buffer_bytes: u64,
    staging_bytes: u64,
    persistent_texture_bytes: u64,
    max_pending_uploads: usize,
}

impl GpuMemoryBudget {
    pub const fn new(
        transient_texture_bytes: u64,
        transient_buffer_bytes: u64,
        staging_bytes: u64,
    ) -> Self {
        Self {
            transient_texture_bytes,
            transient_buffer_bytes,
            staging_bytes,
            persistent_texture_bytes: REFERENCE_PERSISTENT_TEXTURE_BUDGET_BYTES,
            max_pending_uploads: REFERENCE_MAX_PENDING_UPLOADS,
        }
    }

    pub const fn with_persistent_texture_bytes(mut self, persistent_texture_bytes: u64) -> Self {
        self.persistent_texture_bytes = persistent_texture_bytes;
        self
    }

    pub const fn with_max_pending_uploads(mut self, max_pending_uploads: usize) -> Self {
        self.max_pending_uploads = max_pending_uploads;
        self
    }

    pub const fn reference_1080p_mid() -> Self {
        Self::new(
            REFERENCE_TRANSIENT_TEXTURE_BUDGET_BYTES,
            REFERENCE_TRANSIENT_BUFFER_BUDGET_BYTES,
            REFERENCE_STAGING_BUDGET_BYTES,
        )
        .with_persistent_texture_bytes(REFERENCE_PERSISTENT_TEXTURE_BUDGET_BYTES)
        .with_max_pending_uploads(REFERENCE_MAX_PENDING_UPLOADS)
    }

    pub const fn transient_texture_bytes(self) -> u64 {
        self.transient_texture_bytes
    }

    pub const fn transient_buffer_bytes(self) -> u64 {
        self.transient_buffer_bytes
    }

    pub const fn staging_bytes(self) -> u64 {
        self.staging_bytes
    }

    pub const fn persistent_texture_bytes(self) -> u64 {
        self.persistent_texture_bytes
    }

    pub const fn max_pending_uploads(self) -> usize {
        self.max_pending_uploads
    }
}

impl Default for GpuMemoryBudget {
    fn default() -> Self {
        Self::reference_1080p_mid()
    }
}

/// A device-generation-local snapshot of RHI-owned memory classes.
///
/// This reports only physical buffer/texture backing and CPU upload payloads
/// owned by the RHI. Views, descriptor objects, and higher-level caches must
/// be accounted for by their own owners rather than duplicating these bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuMemorySnapshot {
    pub active_buffer_bytes: u64,
    pub active_texture_bytes: u64,
    pub retired_buffer_bytes: u64,
    pub retired_texture_bytes: u64,
    pub pending_upload_bytes: u64,
    pub active_allocations: u32,
    pub retired_allocations: u32,
}

impl GpuMemorySnapshot {
    pub const fn active_resource_bytes(self) -> u64 {
        self.active_buffer_bytes
            .saturating_add(self.active_texture_bytes)
    }

    pub const fn retired_resource_bytes(self) -> u64 {
        self.retired_buffer_bytes
            .saturating_add(self.retired_texture_bytes)
    }

    pub const fn reserved_resource_bytes(self) -> u64 {
        self.active_resource_bytes()
            .saturating_add(self.retired_resource_bytes())
    }

    pub const fn reserved_resource_allocations(self) -> u32 {
        self.active_allocations
            .saturating_add(self.retired_allocations)
    }
}

use std::collections::BTreeMap;

use thiserror::Error;

use crate::core::framework::render::RenderGraphTransientPoolReport;
use crate::graphics::resource_identity::SampledTextureIdentity;
use crate::rhi::{
    BufferDesc, RenderDeviceProfile, RhiError, SubmissionStatus, SubmissionTicket, TextureDesc,
    TextureDimension, TextureFormat, TextureResidency,
};

use super::RenderPassDeviceEpoch;
use super::materialization::{create_wgpu_buffer, create_wgpu_texture};

mod allocation;

pub(in crate::graphics::scene::scene_renderer) use allocation::{
    TransientBufferAllocation, TransientTextureAllocation,
};

pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_KEEP_FRAMES: u64 = 8;
const TRANSIENT_RESOURCE_POOL_MIB: u64 = 1024 * 1024;
pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES:
    u64 = 256 * TRANSIENT_RESOURCE_POOL_MIB;
pub(in crate::graphics::scene::scene_renderer) const TRANSIENT_RESOURCE_POOL_BUFFER_BUDGET_BYTES:
    u64 = 64 * TRANSIENT_RESOURCE_POOL_MIB;

#[derive(Debug, Error)]
pub(in crate::graphics::scene::scene_renderer) enum TransientResourcePoolError {
    #[error("transient resource pool has no active device epoch; begin_frame is required")]
    MissingActiveDeviceEpoch,
    #[error("transient resource allocation belongs to {actual:?}, expected {expected:?}")]
    WrongDeviceEpoch {
        expected: RenderPassDeviceEpoch,
        actual: RenderPassDeviceEpoch,
    },
    #[error(
        "transient texture `{label}` has a storage size that exceeds the supported u64 pool allocation range"
    )]
    TextureStorageSizeOverflow { label: String },
    #[error("failed to create transient texture `{label}`: {message}")]
    TextureCreation { label: String, message: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransientTextureAcquireClass {
    Aliasable,
    PersistentExtraction,
}

pub(in crate::graphics::scene::scene_renderer) struct TransientResourcePool {
    frame_index: u64,
    active_device_epoch: Option<RenderPassDeviceEpoch>,
    texture_budget_bytes: u64,
    buffer_budget_bytes: u64,
    frame_report: RenderGraphTransientPoolReport,
    last_frame_report: RenderGraphTransientPoolReport,
    textures: BTreeMap<TransientTextureKey, Vec<TransientTextureAllocation>>,
    buffers: BTreeMap<TransientBufferKey, Vec<TransientBufferAllocation>>,
    pending_textures: Vec<TransientTextureAllocation>,
    pending_buffers: Vec<TransientBufferAllocation>,
}

impl Default for TransientResourcePool {
    fn default() -> Self {
        Self {
            frame_index: 0,
            active_device_epoch: None,
            texture_budget_bytes: TRANSIENT_RESOURCE_POOL_TEXTURE_BUDGET_BYTES,
            buffer_budget_bytes: TRANSIENT_RESOURCE_POOL_BUFFER_BUDGET_BYTES,
            frame_report: RenderGraphTransientPoolReport::default(),
            last_frame_report: RenderGraphTransientPoolReport::default(),
            textures: BTreeMap::new(),
            buffers: BTreeMap::new(),
            pending_textures: Vec::new(),
            pending_buffers: Vec::new(),
        }
    }
}

impl TransientResourcePool {
    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn with_budgets(
        texture_budget_bytes: u64,
        buffer_budget_bytes: u64,
    ) -> Self {
        Self {
            texture_budget_bytes,
            buffer_budget_bytes,
            ..Self::default()
        }
    }

    pub fn begin_frame(&mut self, device_profile: &RenderDeviceProfile) {
        self.frame_report = RenderGraphTransientPoolReport {
            frame_index: self.frame_index,
            texture_pool_budget_bytes: self.texture_budget_bytes,
            buffer_pool_budget_bytes: self.buffer_budget_bytes,
            ..Default::default()
        };
        self.activate_device_epoch(device_profile);
    }

    pub fn ensure_active_device_profile(
        &self,
        device_profile: &RenderDeviceProfile,
    ) -> Result<(), String> {
        let expected = RenderPassDeviceEpoch::from_profile(device_profile);
        match self.active_device_epoch {
            Some(active) if active == expected => Ok(()),
            Some(active) => Err(format!(
                "transient resource pool is active for retired device epoch {active:?}, not {expected:?}"
            )),
            None => Err(format!(
                "transient resource pool has no active device epoch; begin_frame({expected:?}) is required before materialization"
            )),
        }
    }

    pub fn acquire_texture(
        &mut self,
        device: &wgpu::Device,
        desc: &TextureDesc,
    ) -> Result<TransientTextureAllocation, TransientResourcePoolError> {
        self.acquire_texture_with_class(device, desc, TransientTextureAcquireClass::Aliasable)
    }

    /// Acquires an extraction source that cannot alias another logical graph
    /// resource before the frame-end history copy has consumed it.
    pub fn acquire_persistent_texture(
        &mut self,
        device: &wgpu::Device,
        desc: &TextureDesc,
    ) -> Result<TransientTextureAllocation, TransientResourcePoolError> {
        self.acquire_texture_with_class(
            device,
            desc,
            TransientTextureAcquireClass::PersistentExtraction,
        )
    }

    fn acquire_texture_with_class(
        &mut self,
        device: &wgpu::Device,
        desc: &TextureDesc,
        acquire_class: TransientTextureAcquireClass,
    ) -> Result<TransientTextureAllocation, TransientResourcePoolError> {
        let epoch = self
            .active_device_epoch
            .ok_or(TransientResourcePoolError::MissingActiveDeviceEpoch)?;
        let requested_bytes = texture_desc_pool_size_bytes(desc).ok_or_else(|| {
            TransientResourcePoolError::TextureStorageSizeOverflow {
                label: texture_desc_label(desc),
            }
        })?;
        if acquire_class == TransientTextureAcquireClass::PersistentExtraction {
            self.frame_report.persistent_texture_request_count = self
                .frame_report
                .persistent_texture_request_count
                .saturating_add(1);
            self.frame_report.persistent_texture_requested_bytes = self
                .frame_report
                .persistent_texture_requested_bytes
                .saturating_add(requested_bytes);
        }
        let key = TransientTextureKey::from(desc);
        if let Some(mut allocation) = self
            .textures
            .get_mut(&key)
            .and_then(|entries| entries.pop())
        {
            if allocation.epoch() != epoch {
                return Err(TransientResourcePoolError::WrongDeviceEpoch {
                    expected: epoch,
                    actual: allocation.epoch(),
                });
            }
            allocation.rebind(desc.clone(), self.frame_index);
            self.frame_report.texture_reused_count += 1;
            if acquire_class == TransientTextureAcquireClass::PersistentExtraction {
                self.frame_report.persistent_texture_reused_count = self
                    .frame_report
                    .persistent_texture_reused_count
                    .saturating_add(1);
            }
            return Ok(allocation);
        }

        let texture = create_wgpu_texture(device, desc).map_err(|message| {
            TransientResourcePoolError::TextureCreation {
                label: texture_desc_label(desc),
                message,
            }
        })?;
        self.frame_report.texture_created_count += 1;
        if acquire_class == TransientTextureAcquireClass::PersistentExtraction {
            self.frame_report.persistent_texture_created_count = self
                .frame_report
                .persistent_texture_created_count
                .saturating_add(1);
        }
        Ok(TransientTextureAllocation::new(
            epoch,
            key,
            desc.clone(),
            texture,
            SampledTextureIdentity::new(),
            self.frame_index,
            requested_bytes,
        ))
    }

    pub fn release_texture(&mut self, allocation: TransientTextureAllocation) {
        if !self.accepts_epoch(allocation.epoch()) {
            self.frame_report.device_epoch_discarded_texture_count = self
                .frame_report
                .device_epoch_discarded_texture_count
                .saturating_add(1);
            return;
        }
        debug_assert!(allocation.last_use_ticket().is_none());
        self.textures
            .entry(allocation.key())
            .or_default()
            .push(allocation);
    }

    /// Defers reuse until the queue confirms the submitted frame completed.
    pub fn release_texture_after_submission(
        &mut self,
        mut allocation: TransientTextureAllocation,
        ticket: SubmissionTicket,
    ) {
        if !self.accepts_epoch(allocation.epoch()) || !self.accepts_ticket(ticket) {
            self.frame_report.device_epoch_discarded_texture_count = self
                .frame_report
                .device_epoch_discarded_texture_count
                .saturating_add(1);
            return;
        }
        allocation.retire_after(ticket);
        self.pending_textures.push(allocation);
    }

    pub fn acquire_buffer(
        &mut self,
        device: &wgpu::Device,
        desc: &BufferDesc,
    ) -> Result<TransientBufferAllocation, TransientResourcePoolError> {
        let epoch = self
            .active_device_epoch
            .ok_or(TransientResourcePoolError::MissingActiveDeviceEpoch)?;
        let key = TransientBufferKey::from(desc);
        if let Some(mut allocation) = self.buffers.get_mut(&key).and_then(|entries| entries.pop()) {
            if allocation.epoch() != epoch {
                return Err(TransientResourcePoolError::WrongDeviceEpoch {
                    expected: epoch,
                    actual: allocation.epoch(),
                });
            }
            allocation.rebind(desc.clone(), self.frame_index);
            self.frame_report.buffer_reused_count += 1;
            return Ok(allocation);
        }

        self.frame_report.buffer_created_count += 1;
        Ok(TransientBufferAllocation::new(
            epoch,
            key,
            desc.clone(),
            create_wgpu_buffer(device, desc),
            self.frame_index,
        ))
    }

    pub fn release_buffer(&mut self, allocation: TransientBufferAllocation) {
        if !self.accepts_epoch(allocation.epoch()) {
            self.frame_report.device_epoch_discarded_buffer_count = self
                .frame_report
                .device_epoch_discarded_buffer_count
                .saturating_add(1);
            return;
        }
        debug_assert!(allocation.last_use_ticket().is_none());
        self.buffers
            .entry(allocation.key())
            .or_default()
            .push(allocation);
    }

    /// Defers reuse until the queue confirms the submitted frame completed.
    pub fn release_buffer_after_submission(
        &mut self,
        mut allocation: TransientBufferAllocation,
        ticket: SubmissionTicket,
    ) {
        if !self.accepts_epoch(allocation.epoch()) || !self.accepts_ticket(ticket) {
            self.frame_report.device_epoch_discarded_buffer_count = self
                .frame_report
                .device_epoch_discarded_buffer_count
                .saturating_add(1);
            return;
        }
        allocation.retire_after(ticket);
        self.pending_buffers.push(allocation);
    }

    /// Returns completed submission backings to the free pool and drops every
    /// abnormal terminal result. A status-query error is fail-closed because
    /// the resource may belong to a faulted or retired device generation.
    pub fn collect_completed_submissions<F>(&mut self, mut status_for: F)
    where
        F: FnMut(SubmissionTicket) -> Result<SubmissionStatus, RhiError>,
    {
        crate::profile_scope!(
            "render",
            "render_graph.transient_pool",
            "collect_completed_submissions"
        );
        for mut allocation in std::mem::take(&mut self.pending_textures) {
            let Some(ticket) = allocation.last_use_ticket() else {
                self.frame_report.completion_discarded_texture_count = self
                    .frame_report
                    .completion_discarded_texture_count
                    .saturating_add(1);
                continue;
            };
            self.frame_report.completion_texture_status_query_count = self
                .frame_report
                .completion_texture_status_query_count
                .saturating_add(1);
            match status_for(ticket) {
                Ok(SubmissionStatus::Completed) => {
                    allocation.make_reusable();
                    self.textures
                        .entry(allocation.key())
                        .or_default()
                        .push(allocation);
                    self.frame_report.completion_reclaimed_texture_count = self
                        .frame_report
                        .completion_reclaimed_texture_count
                        .saturating_add(1);
                }
                Ok(status) if !status.is_terminal() => {
                    self.pending_textures.push(allocation);
                }
                Ok(_) | Err(_) => {
                    self.frame_report.completion_discarded_texture_count = self
                        .frame_report
                        .completion_discarded_texture_count
                        .saturating_add(1);
                }
            }
        }

        for mut allocation in std::mem::take(&mut self.pending_buffers) {
            let Some(ticket) = allocation.last_use_ticket() else {
                self.frame_report.completion_discarded_buffer_count = self
                    .frame_report
                    .completion_discarded_buffer_count
                    .saturating_add(1);
                continue;
            };
            self.frame_report.completion_buffer_status_query_count = self
                .frame_report
                .completion_buffer_status_query_count
                .saturating_add(1);
            match status_for(ticket) {
                Ok(SubmissionStatus::Completed) => {
                    allocation.make_reusable();
                    self.buffers
                        .entry(allocation.key())
                        .or_default()
                        .push(allocation);
                    self.frame_report.completion_reclaimed_buffer_count = self
                        .frame_report
                        .completion_reclaimed_buffer_count
                        .saturating_add(1);
                }
                Ok(status) if !status.is_terminal() => {
                    self.pending_buffers.push(allocation);
                }
                Ok(_) | Err(_) => {
                    self.frame_report.completion_discarded_buffer_count = self
                        .frame_report
                        .completion_discarded_buffer_count
                        .saturating_add(1);
                }
            }
        }
    }

    pub fn end_frame(&mut self) {
        crate::profile_scope!(
            "render",
            "render_graph.transient_pool",
            "end_frame_maintenance"
        );
        self.frame_index = self.frame_index.saturating_add(1);
        let (evicted_texture_count, stale_texture_scan_count) = evict_stale_textures(
            &mut self.textures,
            self.frame_index,
            TRANSIENT_RESOURCE_POOL_KEEP_FRAMES,
        );
        let (evicted_buffer_count, stale_buffer_scan_count) = evict_stale_buffers(
            &mut self.buffers,
            self.frame_index,
            TRANSIENT_RESOURCE_POOL_KEEP_FRAMES,
        );
        let (
            budget_evicted_texture_count,
            texture_pool_entry_count,
            texture_pool_retained_bytes,
            budget_texture_accounted_count,
            budget_texture_sort_candidate_count,
        ) = evict_textures_to_budget(&mut self.textures, self.texture_budget_bytes);
        let (
            budget_evicted_buffer_count,
            buffer_pool_entry_count,
            buffer_pool_retained_bytes,
            budget_buffer_accounted_count,
            budget_buffer_sort_candidate_count,
        ) = evict_buffers_to_budget(&mut self.buffers, self.buffer_budget_bytes);
        self.frame_report.evicted_texture_count = evicted_texture_count;
        self.frame_report.evicted_buffer_count = evicted_buffer_count;
        self.frame_report.budget_evicted_texture_count = budget_evicted_texture_count;
        self.frame_report.budget_evicted_buffer_count = budget_evicted_buffer_count;
        self.frame_report.stale_texture_scan_count = stale_texture_scan_count;
        self.frame_report.stale_buffer_scan_count = stale_buffer_scan_count;
        self.frame_report.budget_texture_accounted_count = budget_texture_accounted_count;
        self.frame_report.budget_buffer_accounted_count = budget_buffer_accounted_count;
        self.frame_report.budget_texture_sort_candidate_count = budget_texture_sort_candidate_count;
        self.frame_report.budget_buffer_sort_candidate_count = budget_buffer_sort_candidate_count;
        self.frame_report.texture_pool_entry_count = texture_pool_entry_count;
        self.frame_report.buffer_pool_entry_count = buffer_pool_entry_count;
        self.frame_report.texture_pool_retained_bytes = texture_pool_retained_bytes;
        self.frame_report.buffer_pool_retained_bytes = buffer_pool_retained_bytes;
        self.frame_report.pending_retire_texture_count = self.pending_textures.len();
        self.frame_report.pending_retire_texture_bytes = self
            .pending_textures
            .iter()
            .fold(0_u64, |total, allocation| {
                total.saturating_add(allocation.byte_size())
            });
        self.frame_report.pending_retire_buffer_count = self.pending_buffers.len();
        self.frame_report.pending_retire_buffer_bytes = self
            .pending_buffers
            .iter()
            .fold(0_u64, |total, allocation| {
                total.saturating_add(allocation.byte_size())
            });
        self.frame_report.texture_pool_budget_bytes = self.texture_budget_bytes;
        self.frame_report.buffer_pool_budget_bytes = self.buffer_budget_bytes;
        self.last_frame_report = self.frame_report;
    }

    pub fn last_frame_report(&self) -> RenderGraphTransientPoolReport {
        self.last_frame_report
    }

    fn accepts_epoch(&self, epoch: RenderPassDeviceEpoch) -> bool {
        self.active_device_epoch == Some(epoch)
    }

    fn accepts_ticket(&self, ticket: SubmissionTicket) -> bool {
        self.active_device_epoch.is_some_and(|epoch| {
            let (device_id, generation) = epoch.raw_parts();
            device_id == ticket.device_id().raw() && generation == ticket.generation().raw()
        })
    }

    fn activate_device_epoch(&mut self, device_profile: &RenderDeviceProfile) {
        let next_epoch = RenderPassDeviceEpoch::from_profile(device_profile);
        if self
            .active_device_epoch
            .is_some_and(|active| active != next_epoch)
        {
            self.discard_retired_device_epoch_backings();
        }
        self.active_device_epoch = Some(next_epoch);
    }

    fn discard_retired_device_epoch_backings(&mut self) {
        let discarded_texture_count = self
            .textures
            .values()
            .map(Vec::len)
            .sum::<usize>()
            .saturating_add(self.pending_textures.len());
        let discarded_buffer_count = self
            .buffers
            .values()
            .map(Vec::len)
            .sum::<usize>()
            .saturating_add(self.pending_buffers.len());
        self.textures.clear();
        self.buffers.clear();
        self.pending_textures.clear();
        self.pending_buffers.clear();
        self.frame_report.device_epoch_discarded_texture_count = self
            .frame_report
            .device_epoch_discarded_texture_count
            .saturating_add(discarded_texture_count);
        self.frame_report.device_epoch_discarded_buffer_count = self
            .frame_report
            .device_epoch_discarded_buffer_count
            .saturating_add(discarded_buffer_count);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransientTextureKey {
    width: u32,
    height: u32,
    depth: u32,
    mip_levels: u32,
    sample_count: u32,
    format: u8,
    view_format_bits: u16,
    usage_bits: u32,
    dimension: u8,
    residency: u8,
}

impl From<&TextureDesc> for TransientTextureKey {
    fn from(desc: &TextureDesc) -> Self {
        Self {
            width: desc.width,
            height: desc.height,
            depth: desc.depth,
            mip_levels: desc.mip_levels,
            sample_count: desc.sample_count,
            format: texture_format_tag(desc.format),
            view_format_bits: texture_view_format_bits(desc),
            usage_bits: desc.usage.bits(),
            dimension: texture_dimension_tag(desc.dimension),
            residency: texture_residency_tag(desc.residency),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TransientBufferKey {
    size_bytes: u64,
    usage_bits: u32,
}

impl From<&BufferDesc> for TransientBufferKey {
    fn from(desc: &BufferDesc) -> Self {
        Self {
            size_bytes: desc.size_bytes,
            usage_bits: desc.usage.bits(),
        }
    }
}

fn evict_stale_textures(
    textures: &mut BTreeMap<TransientTextureKey, Vec<TransientTextureAllocation>>,
    frame_index: u64,
    keep_frames: u64,
) -> (usize, usize) {
    let mut evicted = 0;
    let mut scanned = 0_usize;
    textures.retain(|_, entries| {
        let before = entries.len();
        scanned = scanned.saturating_add(before);
        entries.retain(|entry| frame_index.saturating_sub(entry.last_used_frame()) <= keep_frames);
        evicted += before.saturating_sub(entries.len());
        !entries.is_empty()
    });
    (evicted, scanned)
}

fn evict_textures_to_budget(
    textures: &mut BTreeMap<TransientTextureKey, Vec<TransientTextureAllocation>>,
    budget_bytes: u64,
) -> (usize, usize, u64, usize, usize) {
    evict_pool_to_budget(textures, budget_bytes, |entry| {
        (entry.last_used_frame(), entry.byte_size())
    })
}

fn evict_buffers_to_budget(
    buffers: &mut BTreeMap<TransientBufferKey, Vec<TransientBufferAllocation>>,
    budget_bytes: u64,
) -> (usize, usize, u64, usize, usize) {
    evict_pool_to_budget(buffers, budget_bytes, |entry| {
        (entry.last_used_frame(), entry.byte_size())
    })
}

fn evict_pool_to_budget<K, V, F>(
    pool: &mut BTreeMap<K, Vec<V>>,
    budget_bytes: u64,
    entry_metadata: F,
) -> (usize, usize, u64, usize, usize)
where
    K: Copy + Ord,
    F: Copy + Fn(&V) -> (u64, u64),
{
    let (mut retained_count, mut retained_bytes) = pool
        .values()
        .flat_map(|entries| entries.iter())
        .fold((0_usize, 0_u128), |(count, bytes), entry| {
            (
                count.saturating_add(1),
                bytes + u128::from(entry_metadata(entry).1),
            )
        });
    let budget_bytes = u128::from(budget_bytes);
    let accounted_count = retained_count;
    if retained_bytes <= budget_bytes {
        return (
            0,
            retained_count,
            saturating_pool_byte_count(retained_bytes),
            accounted_count,
            0,
        );
    }

    let mut candidates = pool
        .iter()
        .flat_map(|(key, entries)| {
            entries.iter().enumerate().map(move |(index, entry)| {
                let (last_used_frame, byte_size) = entry_metadata(entry);
                (last_used_frame, *key, index, byte_size)
            })
        })
        .collect::<Vec<_>>();
    let sort_candidate_count = candidates.len();
    candidates
        .sort_unstable_by(|left, right| (left.0, left.1, left.2).cmp(&(right.0, right.1, right.2)));

    let mut evicted = 0;
    let mut selected_indices = BTreeMap::<K, Vec<usize>>::new();
    let mut candidates = candidates.into_iter();
    while retained_bytes > budget_bytes {
        let Some((_, key, index, byte_size)) = candidates.next() else {
            break;
        };
        retained_bytes -= u128::from(byte_size);
        retained_count = retained_count.saturating_sub(1);
        selected_indices.entry(key).or_default().push(index);
        evicted += 1;
    }

    for (key, mut indices) in selected_indices {
        indices.sort_unstable_by(|left, right| right.cmp(left));
        let remove_bucket = {
            let Some(entries) = pool.get_mut(&key) else {
                continue;
            };
            for index in indices {
                debug_assert!(index < entries.len());
                entries.swap_remove(index);
            }
            entries.is_empty()
        };
        if remove_bucket {
            pool.remove(&key);
        }
    }

    (
        evicted,
        retained_count,
        saturating_pool_byte_count(retained_bytes),
        accounted_count,
        sort_candidate_count,
    )
}

fn saturating_pool_byte_count(byte_count: u128) -> u64 {
    byte_count.min(u128::from(u64::MAX)) as u64
}

fn texture_desc_pool_size_bytes(desc: &TextureDesc) -> Option<u64> {
    desc.checked_storage_size_bytes()
}

fn texture_desc_label(desc: &TextureDesc) -> String {
    desc.label
        .clone()
        .unwrap_or_else(|| "unnamed transient texture".to_owned())
}

fn evict_stale_buffers(
    buffers: &mut BTreeMap<TransientBufferKey, Vec<TransientBufferAllocation>>,
    frame_index: u64,
    keep_frames: u64,
) -> (usize, usize) {
    let mut evicted = 0;
    let mut scanned = 0_usize;
    buffers.retain(|_, entries| {
        let before = entries.len();
        scanned = scanned.saturating_add(before);
        entries.retain(|entry| frame_index.saturating_sub(entry.last_used_frame()) <= keep_frames);
        evicted += before.saturating_sub(entries.len());
        !entries.is_empty()
    });
    (evicted, scanned)
}

fn texture_format_tag(format: TextureFormat) -> u8 {
    match format {
        TextureFormat::R8Unorm => 0,
        TextureFormat::R16Float => 1,
        TextureFormat::R32Float => 2,
        TextureFormat::Rg16Float => 3,
        TextureFormat::Rg11b10Ufloat => 4,
        TextureFormat::Rgba8Unorm => 5,
        TextureFormat::Rgba8UnormSrgb => 6,
        TextureFormat::Bgra8Unorm => 7,
        TextureFormat::Bgra8UnormSrgb => 8,
        TextureFormat::Rgba16Float => 9,
        TextureFormat::Rgba32Float => 10,
        TextureFormat::Depth24Plus => 11,
        TextureFormat::Depth24PlusStencil8 => 12,
        TextureFormat::Depth32Float => 13,
    }
}

pub(super) fn texture_view_format_bits(desc: &TextureDesc) -> u16 {
    desc.view_formats.iter().fold(0_u16, |bits, format| {
        bits | (1_u16 << texture_format_tag(*format))
    })
}

fn texture_dimension_tag(dimension: TextureDimension) -> u8 {
    match dimension {
        TextureDimension::D1 => 0,
        TextureDimension::D2 => 1,
        TextureDimension::D2Array => 2,
        TextureDimension::D3 => 3,
        TextureDimension::Cube => 4,
    }
}

fn texture_residency_tag(residency: TextureResidency) -> u8 {
    match residency {
        TextureResidency::Dense => 0,
        TextureResidency::SparseReserved => 1,
    }
}

#[cfg(test)]
mod tests;

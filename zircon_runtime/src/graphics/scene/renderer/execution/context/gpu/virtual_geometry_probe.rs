//! Frame-owned native sources; no global registry or background polling service.
use super::super::super::transient_resource_pool::TransientBufferProbeLease;
use super::*;
use crate::render_graph::RenderGraphResourceAccessKind;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex, OnceLock};
type Gather = dyn for<'f, 'b> Fn(
        &mut dyn RenderPassGpuRecordingContext<
            ResourceFactory = (dyn RenderPassGpuResourceFactory + 'f),
        >,
        &[Option<(wgpu::BufferBinding<'b>, u32, u32)>],
        u32,
    ) -> Result<(wgpu::Buffer, std::ops::Range<u64>), String>
    + Send
    + Sync
    + 'static;
// Imported is only the B producer's frame-fresh indexed args allocation.
// Role/name narrow admission; freshness is proven by the audited B create/import
// call chain, not by a name. No persistent, pooled or unknown import is eligible.
#[derive(Clone)]
enum SourceAllocation {
    Pooled(TransientBufferProbeLease),
    Imported {
        native: wgpu::Buffer,
        _physical_desc: crate::rhi::BufferDesc,
        epoch: RenderPassDeviceEpoch,
    },
}
impl SourceAllocation {
    fn native(&self) -> &wgpu::Buffer {
        match self {
            Self::Pooled(lease) => lease.native(),
            Self::Imported { native, .. } => native,
        }
    }
    fn epoch(&self) -> RenderPassDeviceEpoch {
        match self {
            Self::Pooled(lease) => lease.epoch(),
            Self::Imported { epoch, .. } => *epoch,
        }
    }
}
#[derive(Clone)]
struct Source {
    allocation: SourceAllocation,
    offset: u64,
    size: NonZeroU64,
    capacity: u32,
    stride_words: u32,
}
pub(super) struct NativeProbe {
    scope: VirtualGeometryReadbackScope,
    epoch: RenderPassDeviceEpoch,
    origin: OnceLock<crate::rhi::SubmissionTicket>,
    sources: Mutex<Vec<Option<Source>>>,
    source_credit: Arc<VirtualGeometryReadbackCredit>,
    staging_bytes: u64,
    gather: Box<Gather>,
}
struct Recording<'f, 'e> {
    factory: &'f (dyn RenderPassGpuResourceFactory + 'f),
    encoder: &'e mut wgpu::CommandEncoder,
}
impl<'f> RenderPassGpuRecordingContext for Recording<'f, '_> {
    type ResourceFactory = dyn RenderPassGpuResourceFactory + 'f;
    fn resource_factory(&self) -> &Self::ResourceFactory {
        self.factory
    }
    fn command_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder
    }
}
impl RenderPassGpuExecutionContext<'_> {
    /// Retains actual pooled or trusted frame-fresh graph buffers for a diagnostic gather.
    ///
    /// Source positions are plugin-local roles. Present tuples contain graph name,
    /// proven access, record capacity and record stride in u32 words. None denotes
    /// actual absence, not an empty GPU header; unresolved present sources are errors.
    /// Callback tuples contain exact binding, capacity and stride in the same positions.
    /// Its final u32 is the plugin-local read kind. The callback encodes GPU work and
    /// returns its MAP_READ staging buffer and exact staging_bytes range. Capacity
    /// never substitutes for a GPU live count. The factory cannot escape the callback.
    /// Imported role 10 must be the frame-fresh `virtual-geometry-indexed-args`
    /// allocation created once by build_indexed_plan, shared by this frame's draw
    /// commands and imported with its physical descriptor. That producer must never
    /// cache, pool, or rewrite it in later frames. This trusted internal call-chain
    /// requirement is not inferred from the graph name. Every other source requires
    /// a real pool lease; an invalid pooled source never falls back to an import.
    /// False means no feedback scope or insufficient shared 64 MiB / three-slot credit.
    pub fn retain_virtual_geometry_native_probe<F>(
        &mut self,
        sources: &[Option<(&str, RenderGraphResourceAccessKind, u32, u32)>],
        staging_bytes: u64,
        gather: F,
    ) -> Result<bool, String>
    where
        F: for<'f, 'b> Fn(
                &mut dyn RenderPassGpuRecordingContext<
                    ResourceFactory = (dyn RenderPassGpuResourceFactory + 'f),
                >,
                &[Option<(wgpu::BufferBinding<'b>, u32, u32)>],
                u32,
            ) -> Result<(wgpu::Buffer, std::ops::Range<u64>), String>
            + Send
            + Sync
            + 'static,
    {
        let Some(scope) = self.frame.virtual_geometry_readback_scope.as_ref() else {
            return Ok(false);
        };
        let epoch = self
            .device_epoch()
            .ok_or("native probe has no device epoch")?;
        if sources.is_empty()
            || sources.len() > 16
            || staging_bytes == 0
            || staging_bytes > VirtualGeometryReadbackBudget::LIMIT_BYTES
            || staging_bytes % wgpu::COPY_BUFFER_ALIGNMENT != 0
        {
            return Err("native probe source count or staging range is invalid".into());
        }
        let packet = self
            .virtual_geometry_readbacks
            .last()
            .ok_or("native probe requires the actual producer feedback packet")?;
        if packet.scope.scope_id != scope.scope_id
            || packet.epoch != epoch
            || packet.native_probe.is_some()
        {
            return Err("native probe does not match the producer packet".into());
        }
        let mut owned = Vec::with_capacity(sources.len());
        let mut physical_bytes = 0u64;
        for (role, source) in sources.iter().enumerate() {
            let Some((name, access, capacity, stride_words)) = *source else {
                owned.push(None);
                continue;
            };
            let binding = self.require_buffer_binding(name, access)?;
            let allocation = if let Some(lease) = self.resources.probe_owned_buffer(name) {
                SourceAllocation::Pooled(lease)
            } else {
                if role != 10 || name != "virtual-geometry-indexed-args" || stride_words != 5 {
                    return Err(format!(
                        "native probe source {name} has no physical owned lease"
                    ));
                }
                let imported_epoch = self
                    .resources
                    .device_epoch()
                    .ok_or("native probe imported source has no graph device epoch")?;
                let native = self
                    .resources
                    .buffer(name)
                    .ok_or("native probe imported indexed args binding is missing")?;
                let physical_desc = self
                    .resources
                    .physical_buffer_desc(name)
                    .ok_or("native probe imported indexed args has no physical descriptor")?;
                let usage = crate::rhi::BufferUsage::STORAGE
                    | crate::rhi::BufferUsage::INDIRECT
                    | crate::rhi::BufferUsage::COPY_SRC
                    | crate::rhi::BufferUsage::COPY_DST;
                let native_usage = wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST;
                if imported_epoch != epoch
                    || physical_desc.size_bytes != native.size()
                    || physical_desc.usage != usage
                    || native.usage() != native_usage
                {
                    return Err(
                        "native probe imported indexed args has incompatible physical facts".into(),
                    );
                }
                // Both this native and require_buffer_binding resolve the same
                // immutable per-frame backing. Clone retains that real allocation.
                SourceAllocation::Imported {
                    native: native.clone(),
                    _physical_desc: physical_desc.clone(),
                    epoch: imported_epoch,
                }
            };
            let size = binding
                .size
                .map(NonZeroU64::get)
                .unwrap_or_else(|| binding.buffer.size().saturating_sub(binding.offset));
            let end = binding
                .offset
                .checked_add(size)
                .ok_or("native probe binding overflow")?;
            let required = u64::from(capacity)
                .checked_mul(u64::from(stride_words))
                .and_then(|n| n.checked_mul(4))
                .ok_or("native probe capacity overflow")?;
            if allocation.epoch() != epoch
                || stride_words == 0
                || size == 0
                || binding.offset % wgpu::COPY_BUFFER_ALIGNMENT != 0
                || size % wgpu::COPY_BUFFER_ALIGNMENT != 0
                || end > allocation.native().size()
                || required > size
            {
                return Err(format!(
                    "native probe source {name} has an invalid binding ABI"
                ));
            }
            physical_bytes = physical_bytes
                .checked_add(allocation.native().size())
                .ok_or("native probe physical byte count overflow")?;
            owned.push(Some(Source {
                allocation,
                offset: binding.offset,
                size: NonZeroU64::new(size).ok_or("native probe binding is empty")?,
                capacity,
                stride_words,
            }));
        }
        // Charge complete physical allocations, not just active binding windows.
        let Some(credit) = scope.try_acquire_credit(physical_bytes.max(4)) else {
            return Ok(false);
        };
        let probe = Arc::new(NativeProbe {
            scope: scope.clone(),
            epoch,
            origin: OnceLock::new(),
            sources: Mutex::new(owned),
            source_credit: Arc::new(credit),
            staging_bytes,
            gather: Box::new(gather),
        });
        self.virtual_geometry_readbacks
            .last_mut()
            .ok_or("native probe producer packet disappeared")?
            .native_probe = Some(probe);
        Ok(true)
    }
}
impl NativeProbe {
    pub(super) fn bind_origin(&self, ticket: crate::rhi::SubmissionTicket) {
        self.origin.get_or_init(|| ticket);
    }
    fn read(
        &self,
        backend: &crate::graphics::backend::RenderBackend,
        kind: u32,
    ) -> Result<Vec<u8>, String> {
        if kind > 4 {
            return Err("unknown native probe read kind".into());
        }
        if self.epoch != RenderPassDeviceEpoch::from_profile(backend.device_profile()) {
            return Err("native probe device epoch retired".into());
        }
        let origin = *self
            .origin
            .get()
            .ok_or("native probe producer was never submitted")?;
        if backend
            .submission_status(origin)
            .map_err(|e| e.to_string())?
            != crate::rhi::SubmissionStatus::Completed
        {
            return Err("native probe producer has not completed successfully".into());
        }
        // Independent snapshots retain real allocations even after source/viewport drop.
        let snapshot = self
            .sources
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let source_credit = Arc::clone(&self.source_credit);
        let credit = self
            .scope
            .try_acquire_credit(self.staging_bytes)
            .ok_or("native gather exceeds shared readback budget")?;
        let views = snapshot
            .iter()
            .map(|slot| {
                slot.as_ref().map(|s| {
                    (
                        wgpu::BufferBinding {
                            buffer: s.allocation.native(),
                            offset: s.offset,
                            size: Some(s.size),
                        },
                        s.capacity,
                        s.stride_words,
                    )
                })
            })
            .collect::<Vec<_>>();
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-vg-native-probe-gather"),
            });
        let (staging, range) = (self.gather)(
            &mut Recording {
                factory: &backend.device,
                encoder: &mut encoder,
            },
            &views,
            kind,
        )?;
        drop(views);
        if range.start != 0
            || range.end != self.staging_bytes
            || staging.size() != self.staging_bytes
            || range.end.checked_sub(range.start) != Some(self.staging_bytes)
            || range.end > staging.size()
            || !staging
                .usage()
                .contains(wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST)
        {
            return Err("native gather returned an invalid staging allocation".into());
        }
        let keepalive = (snapshot, source_credit);
        let packet = NativeReadbackPacket {
            scope: self.scope.clone(),
            epoch: self.epoch,
            staging,
            range,
            credit,
            native_probe: None,
            inspection_decoder: None,
            decoder: Box::new(move |bytes| {
                let _ = &keepalive;
                Ok(bytes.to_vec())
            }),
        };
        // This ticket belongs to the new encoder, never to the original producer.
        let ticket = backend
            .submit_graphics_command_buffers(vec![encoder.finish()])
            .map_err(|e| e.to_string())?;
        let mut pending = packet.bind_ticket(ticket);
        let deadline = web_time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            backend
                .poll_submission_completions()
                .map_err(|e| e.to_string())?;
            pending.observe_status(backend.submission_status(ticket).ok());
            if let Some(ready) = pending.take_ready() {
                return ready.map(|ready| ready.outputs);
            }
            if web_time::Instant::now() >= deadline {
                return Err("native gather deadline exceeded".into());
            }
            std::thread::yield_now();
        }
    }
}
impl VirtualGeometryReadyReadback {
    pub(in crate::graphics) fn into_native_probe_handle(mut self) -> Option<Self> {
        drop(std::mem::take(&mut self.outputs));
        drop(self.inspection.take());
        drop(self._credit.take());
        self.native_probe.as_ref()?;
        Some(self)
    }
    pub(in crate::graphics) fn clone_native_probe_handle(&self) -> Option<Self> {
        Some(Self {
            outputs: Default::default(),
            inspection: None,
            _credit: None,
            native_probe: Some(Arc::clone(self.native_probe.as_ref()?)),
        })
    }
    pub(in crate::graphics) fn native_probe_scope(&self) -> Option<&VirtualGeometryReadbackScope> {
        Some(&self.native_probe.as_ref()?.scope)
    }
    pub(in crate::graphics) fn native_probe_matches(
        &self,
        generation: u64,
        epoch: RenderPassDeviceEpoch,
    ) -> bool {
        self.native_probe
            .as_ref()
            .is_some_and(|p| p.scope.viewport_generation == generation && p.epoch == epoch)
    }
    pub(in crate::graphics) fn native_probe_source(
        &self,
        role: u32,
        drop_source: bool,
    ) -> Result<bool, String> {
        let probe = self
            .native_probe
            .as_ref()
            .ok_or("native probe unavailable")?;
        let mut sources = probe.sources.lock().unwrap_or_else(|e| e.into_inner());
        let slot = sources
            .get_mut(role as usize)
            .ok_or("native role not registered")?;
        let present = slot.is_some();
        if drop_source {
            drop(slot.take());
        }
        Ok(present)
    }
    pub(in crate::graphics) fn read_native_probe(
        &self,
        backend: &crate::graphics::backend::RenderBackend,
        kind: u32,
    ) -> Result<Vec<u8>, String> {
        self.native_probe
            .as_ref()
            .ok_or("native probe unavailable")?
            .read(backend, kind)
    }
}

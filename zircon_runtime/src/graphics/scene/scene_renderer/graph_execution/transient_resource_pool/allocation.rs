use crate::graphics::resource_identity::SampledTextureIdentity;
use crate::rhi::{BufferDesc, SubmissionTicket, TextureDesc};

use super::super::RenderPassDeviceEpoch;
use super::{TransientBufferKey, TransientTextureKey};

/// Move-only physical texture lease owned by the RDG materializer or pool.
#[derive(Debug)]
pub(in crate::graphics::scene::scene_renderer) struct TransientTextureAllocation {
    epoch: RenderPassDeviceEpoch,
    key: TransientTextureKey,
    desc: TextureDesc,
    native: wgpu::Texture,
    identity: SampledTextureIdentity,
    last_used_frame: u64,
    byte_size: u64,
    last_use_ticket: Option<SubmissionTicket>,
}

impl TransientTextureAllocation {
    pub(super) fn new(
        epoch: RenderPassDeviceEpoch,
        key: TransientTextureKey,
        desc: TextureDesc,
        native: wgpu::Texture,
        identity: SampledTextureIdentity,
        last_used_frame: u64,
        byte_size: u64,
    ) -> Self {
        Self {
            epoch,
            key,
            desc,
            native,
            identity,
            last_used_frame,
            byte_size,
            last_use_ticket: None,
        }
    }

    pub(super) fn rebind(&mut self, desc: TextureDesc, frame_index: u64) {
        debug_assert_eq!(self.key, TransientTextureKey::from(&desc));
        debug_assert!(self.last_use_ticket.is_none());
        self.desc = desc;
        self.last_used_frame = frame_index;
    }

    pub(super) const fn epoch(&self) -> RenderPassDeviceEpoch {
        self.epoch
    }

    pub(super) const fn key(&self) -> TransientTextureKey {
        self.key
    }

    pub(in crate::graphics::scene::scene_renderer) const fn identity(
        &self,
    ) -> SampledTextureIdentity {
        self.identity
    }

    pub(super) const fn last_used_frame(&self) -> u64 {
        self.last_used_frame
    }

    pub(super) const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    pub(super) const fn last_use_ticket(&self) -> Option<SubmissionTicket> {
        self.last_use_ticket
    }

    pub(super) fn retire_after(&mut self, ticket: SubmissionTicket) {
        debug_assert!(self.last_use_ticket.is_none());
        self.last_use_ticket = Some(ticket);
    }

    pub(super) fn make_reusable(&mut self) {
        debug_assert!(self.last_use_ticket.is_some());
        self.last_use_ticket = None;
    }

    pub(in crate::graphics::scene::scene_renderer) const fn desc(&self) -> &TextureDesc {
        &self.desc
    }

    pub(in crate::graphics::scene::scene_renderer) const fn native(&self) -> &wgpu::Texture {
        &self.native
    }
}

/// Move-only physical buffer lease owned by the RDG materializer or pool.
#[derive(Debug)]
pub(in crate::graphics::scene::scene_renderer) struct TransientBufferAllocation {
    epoch: RenderPassDeviceEpoch,
    key: TransientBufferKey,
    desc: BufferDesc,
    native: wgpu::Buffer,
    last_used_frame: u64,
    byte_size: u64,
    last_use_ticket: Option<SubmissionTicket>,
}

impl TransientBufferAllocation {
    pub(super) fn new(
        epoch: RenderPassDeviceEpoch,
        key: TransientBufferKey,
        desc: BufferDesc,
        native: wgpu::Buffer,
        last_used_frame: u64,
    ) -> Self {
        let byte_size = desc.size_bytes;
        Self {
            epoch,
            key,
            desc,
            native,
            last_used_frame,
            byte_size,
            last_use_ticket: None,
        }
    }

    pub(super) fn rebind(&mut self, desc: BufferDesc, frame_index: u64) {
        debug_assert_eq!(self.key, TransientBufferKey::from(&desc));
        debug_assert!(self.last_use_ticket.is_none());
        self.desc = desc;
        self.last_used_frame = frame_index;
    }

    pub(super) const fn epoch(&self) -> RenderPassDeviceEpoch {
        self.epoch
    }

    pub(super) const fn key(&self) -> TransientBufferKey {
        self.key
    }

    pub(super) const fn last_used_frame(&self) -> u64 {
        self.last_used_frame
    }

    pub(super) const fn byte_size(&self) -> u64 {
        self.byte_size
    }

    pub(super) const fn last_use_ticket(&self) -> Option<SubmissionTicket> {
        self.last_use_ticket
    }

    pub(super) fn retire_after(&mut self, ticket: SubmissionTicket) {
        debug_assert!(self.last_use_ticket.is_none());
        self.last_use_ticket = Some(ticket);
    }

    pub(super) fn make_reusable(&mut self) {
        debug_assert!(self.last_use_ticket.is_some());
        self.last_use_ticket = None;
    }

    pub(in crate::graphics::scene::scene_renderer) const fn desc(&self) -> &BufferDesc {
        &self.desc
    }

    pub(in crate::graphics::scene::scene_renderer) const fn native(&self) -> &wgpu::Buffer {
        &self.native
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_contract_keeps_generation_and_last_use_with_the_native_owner() {
        let source = include_str!("allocation.rs");

        for owner in [
            "struct TransientTextureAllocation",
            "struct TransientBufferAllocation",
        ] {
            let owner = source
                .split(owner)
                .nth(1)
                .expect("allocation owner must remain declared");
            let fields = owner
                .split('}')
                .next()
                .expect("allocation owner must retain explicit fields");
            for field in [
                "epoch:",
                "key:",
                "desc:",
                "native:",
                "last_used_frame:",
                "byte_size:",
                "last_use_ticket:",
            ] {
                assert!(fields.contains(field), "allocation is missing `{field}`");
            }
        }
    }
}

use std::ops::Range;
use std::sync::Arc;

use super::super::{RenderArtifactBlockDescriptor, RenderArtifactManifest};

#[derive(Clone, Debug)]
pub struct RenderArtifactCookedBlock {
    descriptor: RenderArtifactBlockDescriptor,
    payload: Arc<Vec<u8>>,
    range: Range<usize>,
}

impl RenderArtifactCookedBlock {
    pub(super) fn new(
        descriptor: RenderArtifactBlockDescriptor,
        payload: Arc<Vec<u8>>,
        range: Range<usize>,
    ) -> Self {
        Self {
            descriptor,
            payload,
            range,
        }
    }

    pub const fn descriptor(&self) -> &RenderArtifactBlockDescriptor {
        &self.descriptor
    }

    pub fn bytes(&self) -> &[u8] {
        &self.payload[self.range.clone()]
    }
}

#[derive(Clone, Debug)]
pub struct RenderArtifactCookOutput {
    manifest: RenderArtifactManifest,
    blocks: Arc<[RenderArtifactCookedBlock]>,
}

impl RenderArtifactCookOutput {
    pub(super) fn new(
        manifest: RenderArtifactManifest,
        blocks: Vec<RenderArtifactCookedBlock>,
    ) -> Self {
        Self {
            manifest,
            blocks: blocks.into(),
        }
    }

    pub const fn manifest(&self) -> &RenderArtifactManifest {
        &self.manifest
    }

    pub fn blocks(&self) -> &[RenderArtifactCookedBlock] {
        self.blocks.as_ref()
    }
}

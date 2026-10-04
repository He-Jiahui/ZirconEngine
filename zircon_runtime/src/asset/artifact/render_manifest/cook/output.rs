use std::ops::Range;
use std::sync::Arc;

use super::super::{RenderArtifactBlockDescriptor, RenderArtifactManifest};

/// cook 输出中对共享载荷的一段只读引用；发布器以描述符核对它与清单的对应关系。
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

/// 同一 cook 的清单与编码块集合；应交给 publication 入口先发布块、后发布清单。
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

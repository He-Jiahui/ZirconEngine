use thiserror::Error;

use super::{
    RenderArtifactPublishStatus, RenderArtifactStore, RenderArtifactStoreError,
    RenderArtifactStoreLimits,
};
use crate::asset::artifact::render_manifest::{RenderArtifactCookOutput, RenderSubresourceId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactCookPublicationReport {
    published_blocks: usize,
    reused_blocks: usize,
    published_encoded_bytes: u64,
    reused_encoded_bytes: u64,
    manifest: RenderArtifactPublishStatus,
}

impl RenderArtifactCookPublicationReport {
    pub const fn published_blocks(self) -> usize {
        self.published_blocks
    }

    pub const fn reused_blocks(self) -> usize {
        self.reused_blocks
    }

    pub const fn published_encoded_bytes(self) -> u64 {
        self.published_encoded_bytes
    }

    pub const fn reused_encoded_bytes(self) -> u64 {
        self.reused_encoded_bytes
    }

    pub const fn manifest(self) -> RenderArtifactPublishStatus {
        self.manifest
    }
}

#[derive(Debug, Error)]
pub enum RenderArtifactCookPublicationError {
    #[error(
        "render artifact cook output has {actual} payload blocks but its manifest declares {expected}"
    )]
    BlockCountMismatch { expected: usize, actual: usize },
    #[error("render artifact cook output block {subresource:?} does not match its manifest")]
    BlockDescriptorMismatch { subresource: RenderSubresourceId },
    #[error("render artifact cook publication byte accounting overflowed")]
    ByteAccountingOverflow,
    #[error(transparent)]
    Store(#[from] RenderArtifactStoreError),
}

pub fn publish_render_artifact_cook_output(
    store: &RenderArtifactStore,
    output: &RenderArtifactCookOutput,
    limits: RenderArtifactStoreLimits,
) -> Result<RenderArtifactCookPublicationReport, RenderArtifactCookPublicationError> {
    let manifest_blocks = output.manifest().blocks();
    if output.blocks().len() != manifest_blocks.len() {
        return Err(RenderArtifactCookPublicationError::BlockCountMismatch {
            expected: manifest_blocks.len(),
            actual: output.blocks().len(),
        });
    }
    for (payload, descriptor) in output.blocks().iter().zip(manifest_blocks) {
        if payload.descriptor() != descriptor {
            return Err(
                RenderArtifactCookPublicationError::BlockDescriptorMismatch {
                    subresource: payload.descriptor().subresource(),
                },
            );
        }
    }

    let mut published_blocks = 0_usize;
    let mut reused_blocks = 0_usize;
    let mut published_encoded_bytes = 0_u64;
    let mut reused_encoded_bytes = 0_u64;
    for block in output.blocks() {
        let status = store.publish_block(block.descriptor(), block.bytes(), limits)?;
        match status {
            RenderArtifactPublishStatus::Published => {
                published_blocks = published_blocks.saturating_add(1);
                published_encoded_bytes = published_encoded_bytes
                    .checked_add(block.descriptor().encoded_bytes())
                    .ok_or(RenderArtifactCookPublicationError::ByteAccountingOverflow)?;
            }
            RenderArtifactPublishStatus::Reused => {
                reused_blocks = reused_blocks.saturating_add(1);
                reused_encoded_bytes = reused_encoded_bytes
                    .checked_add(block.descriptor().encoded_bytes())
                    .ok_or(RenderArtifactCookPublicationError::ByteAccountingOverflow)?;
            }
        }
    }
    let manifest = store.publish_verified_manifest(output.manifest(), limits)?;
    Ok(RenderArtifactCookPublicationReport {
        published_blocks,
        reused_blocks,
        published_encoded_bytes,
        reused_encoded_bytes,
        manifest,
    })
}

#[cfg(test)]
#[path = "cook_publication/tests.rs"]
mod tests;

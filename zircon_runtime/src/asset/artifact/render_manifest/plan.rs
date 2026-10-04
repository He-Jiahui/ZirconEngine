use std::collections::BTreeSet;
use std::sync::Arc;

use thiserror::Error;

use crate::core::resource::UntypedResourceHandle;

use super::{
    RenderArtifactBlockDescriptor, RenderArtifactManifest, RenderArtifactManifestError,
    RenderArtifactResidencyClass,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderArtifactLoadScope {
    Bootstrap,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderArtifactLoadBatch {
    blocks: Arc<[RenderArtifactBlockDescriptor]>,
}

impl RenderArtifactLoadBatch {
    pub fn blocks(&self) -> &[RenderArtifactBlockDescriptor] {
        self.blocks.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderArtifactLoadPlan {
    resource: UntypedResourceHandle,
    asset_revision: u64,
    scope: RenderArtifactLoadScope,
    block_count: usize,
    total_encoded_bytes: u64,
    total_decoded_bytes: u64,
    batches: Arc<[RenderArtifactLoadBatch]>,
}

impl RenderArtifactLoadPlan {
    pub const fn resource(&self) -> UntypedResourceHandle {
        self.resource
    }

    pub const fn asset_revision(&self) -> u64 {
        self.asset_revision
    }

    pub const fn scope(&self) -> RenderArtifactLoadScope {
        self.scope
    }

    pub const fn block_count(&self) -> usize {
        self.block_count
    }

    pub const fn total_encoded_bytes(&self) -> u64 {
        self.total_encoded_bytes
    }

    pub const fn total_decoded_bytes(&self) -> u64 {
        self.total_decoded_bytes
    }

    pub fn batches(&self) -> &[RenderArtifactLoadBatch] {
        self.batches.as_ref()
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactLoadPlanError {
    #[error(transparent)]
    Manifest(#[from] RenderArtifactManifestError),
    #[error("render artifact load-plan byte totals overflow")]
    ByteTotalOverflow,
    #[error("render artifact load-plan dependency frontier is incomplete")]
    IncompleteDependencyFrontier,
}

impl RenderArtifactManifest {
    pub fn load_plan(
        &self,
        scope: RenderArtifactLoadScope,
    ) -> Result<RenderArtifactLoadPlan, RenderArtifactLoadPlanError> {
        self.validate()?;
        build_load_plan(self, scope)
    }
}

fn build_load_plan(
    manifest: &RenderArtifactManifest,
    scope: RenderArtifactLoadScope,
) -> Result<RenderArtifactLoadPlan, RenderArtifactLoadPlanError> {
    let blocks = manifest.blocks();
    let mut selected = blocks
        .iter()
        .map(|block| {
            scope == RenderArtifactLoadScope::All
                || block.residency() == RenderArtifactResidencyClass::Bootstrap
        })
        .collect::<Vec<_>>();
    let mut pending = dependency_frontier_seed(scope, &selected);
    while let Some(index) = pending.pop() {
        for dependency in blocks[index].dependencies() {
            let dependency_index = blocks
                .binary_search_by_key(dependency, RenderArtifactBlockDescriptor::subresource)
                .map_err(|_| RenderArtifactLoadPlanError::IncompleteDependencyFrontier)?;
            if !selected[dependency_index] {
                selected[dependency_index] = true;
                pending.push(dependency_index);
            }
        }
    }

    let mut indegree = vec![0_usize; blocks.len()];
    let mut dependents = vec![Vec::<usize>::new(); blocks.len()];
    let mut block_count = 0_usize;
    let mut total_encoded_bytes = 0_u64;
    let mut total_decoded_bytes = 0_u64;
    for (block_index, block) in blocks.iter().enumerate() {
        if !selected[block_index] {
            continue;
        }
        block_count = block_count.saturating_add(1);
        total_encoded_bytes = total_encoded_bytes
            .checked_add(block.encoded_bytes())
            .ok_or(RenderArtifactLoadPlanError::ByteTotalOverflow)?;
        total_decoded_bytes = total_decoded_bytes
            .checked_add(block.decoded_bytes())
            .ok_or(RenderArtifactLoadPlanError::ByteTotalOverflow)?;
        for dependency in block.dependencies() {
            let dependency_index = blocks
                .binary_search_by_key(dependency, RenderArtifactBlockDescriptor::subresource)
                .map_err(|_| RenderArtifactLoadPlanError::IncompleteDependencyFrontier)?;
            if selected[dependency_index] {
                indegree[block_index] = indegree[block_index].saturating_add(1);
                dependents[dependency_index].push(block_index);
            }
        }
    }

    let mut ready = selected
        .iter()
        .enumerate()
        .filter_map(|(index, selected)| {
            (*selected && indegree[index] == 0).then_some((blocks[index].subresource(), index))
        })
        .collect::<BTreeSet<_>>();
    let mut visited = 0_usize;
    let mut batches = Vec::with_capacity(block_count);
    while !ready.is_empty() {
        let frontier = std::mem::take(&mut ready);
        let mut batch = Vec::with_capacity(frontier.len());
        for (_, index) in frontier {
            visited = visited.saturating_add(1);
            batch.push(blocks[index].clone());
            for dependent in &dependents[index] {
                indegree[*dependent] = indegree[*dependent].saturating_sub(1);
                if indegree[*dependent] == 0 {
                    ready.insert((blocks[*dependent].subresource(), *dependent));
                }
            }
        }
        batches.push(RenderArtifactLoadBatch {
            blocks: batch.into(),
        });
    }
    if visited != block_count {
        return Err(RenderArtifactLoadPlanError::IncompleteDependencyFrontier);
    }

    Ok(RenderArtifactLoadPlan {
        resource: manifest.resource(),
        asset_revision: manifest.asset_revision(),
        scope,
        block_count,
        total_encoded_bytes,
        total_decoded_bytes,
        batches: batches.into(),
    })
}

fn dependency_frontier_seed(scope: RenderArtifactLoadScope, selected: &[bool]) -> Vec<usize> {
    if scope == RenderArtifactLoadScope::All {
        return Vec::new();
    }
    selected
        .iter()
        .enumerate()
        .filter_map(|(index, selected)| (*selected).then_some(index))
        .collect()
}

#[cfg(test)]
#[path = "plan/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "plan/tests/optimization_batch_ja_runtime640_tests.rs"]
mod optimization_batch_ja_runtime640_tests;

#[cfg(test)]
#[path = "tests/plan_optimization_batch_ha_runtime582_tests.rs"]
mod optimization_batch_ha_runtime582_tests;

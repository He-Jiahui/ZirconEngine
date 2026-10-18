//! Complete importer-source publication protocol; candidate errors and active residents are separate.
use crate::{
    ResourceActiveGeneration, ResourceActiveVersionKey, ResourceData, ResourceId, ResourceRecord,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceSourceOutcome {
    CompleteSuccess,
    Failed,
    Unchanged,
    Removed,
}

#[derive(Clone, Debug)]
pub struct ResourceSourceMember {
    pub record: ResourceRecord,
    /// An owned importer result, shared across recapture attempts without copying its bytes.
    pub payload: Option<Arc<dyn ResourceData>>,
}

#[derive(Clone, Debug)]
pub struct ResourceSourcePublication {
    pub(crate) owner: ResourceId,
    pub(crate) previous_members: Vec<ResourceId>,
    pub(crate) members: Vec<ResourceSourceMember>,
    pub(crate) outcome: ResourceSourceOutcome,
    pub(crate) expected_active: Vec<(ResourceId, Option<ResourceActiveVersionKey>)>,
}

impl ResourceSourcePublication {
    /// Members must come from the complete importer/meta manifest, never a path prefix.
    pub fn capture(
        owner: ResourceId,
        mut previous_members: Vec<ResourceId>,
        members: Vec<ResourceSourceMember>,
        outcome: ResourceSourceOutcome,
        active: &ResourceActiveGeneration,
    ) -> Self {
        previous_members.extend_from_slice(active.source_members(owner));
        previous_members.sort_unstable();
        previous_members.dedup();
        let mut affected = previous_members.clone();
        affected.extend(members.iter().map(|member| member.record.id));
        affected.push(owner);
        affected.sort_unstable();
        affected.dedup();
        let expected_active = affected
            .into_iter()
            .map(|id| (id, active.key(id)))
            .collect();
        Self {
            owner,
            previous_members,
            members,
            outcome,
            expected_active,
        }
    }

    pub fn recapture(&self, active: &ResourceActiveGeneration) -> Self {
        Self::capture(
            self.owner,
            self.previous_members.clone(),
            self.members.clone(),
            self.outcome,
            active,
        )
    }

    pub fn owner(&self) -> ResourceId {
        self.owner
    }
    pub fn outcome(&self) -> ResourceSourceOutcome {
        self.outcome
    }
    pub fn members(&self) -> &[ResourceSourceMember] {
        &self.members
    }
    pub fn previous_members(&self) -> &[ResourceId] {
        &self.previous_members
    }
}

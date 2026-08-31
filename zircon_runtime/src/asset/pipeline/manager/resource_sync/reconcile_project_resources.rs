use crate::asset::project::ProjectManager;
use crate::core::resource::{ResourceId, ResourceLocator, ResourceMutationBatch};

#[derive(Clone, Debug)]
pub(in crate::asset::pipeline::manager) struct ProjectResourceIdentity {
    id: ResourceId,
    locator: ResourceLocator,
}

pub(in crate::asset::pipeline::manager) fn project_resource_identities(
    project: &ProjectManager,
) -> Vec<ProjectResourceIdentity> {
    project
        .registry()
        .values()
        .map(|record| ProjectResourceIdentity {
            id: record.id(),
            locator: record.primary_locator().clone(),
        })
        .collect()
}

/// Reconciles project-generation resources by stable ID before publication.
///
/// A locator change for the same resource ID is an explicit runtime rename, not a remove/add
/// pair. This keeps ResourceManager's identity contract aligned with sidecar GUID preservation.
pub(in crate::asset::pipeline::manager) fn reconcile_project_resources(
    mut batch: ResourceMutationBatch,
    previous_identities: &[ProjectResourceIdentity],
    candidate: &ProjectManager,
) -> ResourceMutationBatch {
    let mut previous_identities = previous_identities.iter().collect::<Vec<_>>();
    previous_identities.sort_by(|left, right| {
        left.locator
            .cmp(&right.locator)
            .then_with(|| left.id.to_string().cmp(&right.id.to_string()))
    });
    for previous in previous_identities {
        match candidate.registry().get(previous.id) {
            Some(current) if current.primary_locator() != &previous.locator => {
                batch = batch.rename(previous.locator.clone(), current.primary_locator().clone());
            }
            None => {
                batch = batch.remove(previous.locator.clone());
            }
            Some(_) => {}
        }
    }
    batch
}

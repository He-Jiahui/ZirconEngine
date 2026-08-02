use crate::{
    asset::{AssetEvent, SceneAsset, facade::AssetEventPoll},
    core::{
        JobScheduler,
        framework::asset::{
            ResourceManagementQuery, ResourceManagementRow, ResourceManagementScan,
        },
        resource::{
            ResourceEvent, ResourceEventKind, ResourceKind, ResourceState, approximate_event_bytes,
        },
    },
};

use super::{super::reports::DynamicSceneAssetReloadDrainReport, DynamicSceneAssetReloadQueue};

#[derive(Debug)]
pub(super) struct DynamicSceneAssetReloadReconciliation {
    scan: ResourceManagementScan,
}

impl DynamicSceneAssetReloadReconciliation {
    fn new(scan: ResourceManagementScan) -> Self {
        Self { scan }
    }
}

impl DynamicSceneAssetReloadQueue {
    pub(super) fn begin_reconciliation(&mut self) {
        let generation = self.resource_manager.management_generation();
        self.reconciliation = Some(DynamicSceneAssetReloadReconciliation::new(generation.scan(
            ResourceManagementQuery {
                kind: Some(ResourceKind::Scene),
                state: None,
            },
        )));
    }

    pub(super) fn drain_reconciliation(
        &mut self,
        scheduler: &JobScheduler,
        report: &mut DynamicSceneAssetReloadDrainReport,
        started: std::time::Instant,
    ) {
        loop {
            if !self.event_budget_available(report, started) {
                break;
            }
            let Some(row) = self
                .reconciliation
                .as_mut()
                .and_then(|reconciliation| reconciliation.scan.next_row())
            else {
                self.reconciliation = None;
                break;
            };
            let poll = reconciliation_poll(&row);
            let keep_draining = self.process_event_poll(scheduler, poll, report);
            let finished = self
                .reconciliation
                .as_ref()
                .is_some_and(|reconciliation| reconciliation.scan.is_complete());
            if finished {
                self.reconciliation = None;
            }
            if !keep_draining {
                break;
            }
        }
    }
}

fn reconciliation_poll(row: &ResourceManagementRow) -> AssetEventPoll<SceneAsset> {
    let resource_event = ResourceEvent {
        kind: match row.state {
            ResourceState::Pending => ResourceEventKind::Added,
            ResourceState::Ready | ResourceState::Reloading => ResourceEventKind::Updated,
            ResourceState::Error => ResourceEventKind::ReloadFailed,
        },
        resource_kind: ResourceKind::Scene,
        id: row.id,
        locator: crate::asset::AssetUri::parse(row.primary_locator.as_ref()).ok(),
        previous_locator: None,
        revision: row.revision,
    };
    let approximate_bytes = approximate_event_bytes(&resource_event);
    if row.state == ResourceState::Pending {
        AssetEventPoll::Filtered { approximate_bytes }
    } else {
        AssetEventPoll::Relevant {
            event: AssetEvent::from_resource_event(resource_event)
                .expect("scene reconciliation event must stay scene-typed"),
            approximate_bytes,
        }
    }
}

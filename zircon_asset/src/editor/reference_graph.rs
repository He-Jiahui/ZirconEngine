use std::collections::{HashMap, HashSet};

use crate::{AssetCatalogRecord, AssetUuid};

#[derive(Clone, Debug, Default)]
pub struct ReferenceGraph {
    outgoing: HashMap<AssetUuid, HashSet<AssetUuid>>,
    incoming: HashMap<AssetUuid, HashSet<AssetUuid>>,
}

impl ReferenceGraph {
    pub fn rebuild<'a>(records: impl Iterator<Item = &'a AssetCatalogRecord>) -> Self {
        let mut graph = Self::default();
        let records = records.cloned().collect::<Vec<_>>();
        let known_by_uuid = records
            .iter()
            .map(|record| (record.asset_uuid, record.asset_uuid))
            .collect::<HashMap<_, _>>();
        let known_by_locator = records
            .iter()
            .map(|record| (record.locator.clone(), record.asset_uuid))
            .collect::<HashMap<_, _>>();

        for record in &records {
            for reference in &record.direct_references {
                let target_uuid = known_by_uuid
                    .get(&reference.uuid)
                    .copied()
                    .or_else(|| known_by_locator.get(&reference.locator).copied());
                if let Some(target_uuid) = target_uuid {
                    graph
                        .outgoing
                        .entry(record.asset_uuid)
                        .or_default()
                        .insert(target_uuid);
                    graph
                        .incoming
                        .entry(target_uuid)
                        .or_default()
                        .insert(record.asset_uuid);
                }
            }
        }

        graph
    }

    pub fn outgoing(&self, uuid: AssetUuid) -> Vec<AssetUuid> {
        self.outgoing
            .get(&uuid)
            .map(|uuids| uuids.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn incoming(&self, uuid: AssetUuid) -> Vec<AssetUuid> {
        self.incoming
            .get(&uuid)
            .map(|uuids| uuids.iter().copied().collect())
            .unwrap_or_default()
    }
}

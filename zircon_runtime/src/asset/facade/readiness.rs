use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::{Asset, AssetLoadState, AssetLoadStates, Handle};
use crate::asset::{AssetId, AssetKind, AssetUri, ProjectAssetManager};
use crate::core::resource::{ResourceDiagnostic, ResourceMarker, ResourceRecord};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetReadinessReport {
    pub root: AssetReadinessNode,
    pub load_states: AssetLoadStates,
    pub dependencies: Vec<AssetDependencyReadiness>,
}

impl AssetReadinessReport {
    pub fn is_loaded(&self) -> bool {
        self.load_states.is_loaded()
    }

    pub fn is_loaded_with_direct_dependencies(&self) -> bool {
        self.load_states.is_loaded_with_direct_dependencies()
    }

    pub fn is_loaded_with_dependencies(&self) -> bool {
        self.load_states.is_loaded_with_dependencies()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetReadinessNode {
    pub id: AssetId,
    pub locator: Option<AssetUri>,
    pub kind: Option<AssetKind>,
    pub revision: Option<u64>,
    pub load_state: AssetLoadState,
    pub diagnostics: Vec<ResourceDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetDependencyReadiness {
    pub id: AssetId,
    pub locator: Option<AssetUri>,
    pub kind: Option<AssetKind>,
    pub revision: Option<u64>,
    pub depth: u32,
    pub direct: bool,
    pub load_state: AssetLoadState,
    pub diagnostics: Vec<ResourceDiagnostic>,
}

impl ProjectAssetManager {
    pub fn readiness_report<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetReadinessReport {
        let record = self.resource_manager().registry().get(handle.id()).cloned();
        let load_states = self.load_states(handle);
        let root = self.readiness_root_node::<TAsset>(handle, record.as_ref());
        let dependencies = record
            .as_ref()
            .filter(|record| record.kind == TAsset::Marker::KIND)
            .map(|record| self.collect_dependency_readiness(record.id, &record.dependency_ids))
            .unwrap_or_default();

        AssetReadinessReport {
            root,
            load_states,
            dependencies,
        }
    }
}

impl ProjectAssetManager {
    fn readiness_root_node<TAsset: Asset>(
        &self,
        handle: Handle<TAsset>,
        record: Option<&ResourceRecord>,
    ) -> AssetReadinessNode {
        let Some(record) = record else {
            return AssetReadinessNode {
                id: handle.id(),
                locator: None,
                kind: None,
                revision: None,
                load_state: AssetLoadState::NotLoaded,
                diagnostics: vec![ResourceDiagnostic::error(format!(
                    "missing asset record {}",
                    handle.id()
                ))],
            };
        };

        let mut diagnostics = record.diagnostics.clone();
        if record.kind != TAsset::Marker::KIND {
            diagnostics.push(ResourceDiagnostic::error(format!(
                "asset {} was {:?}, not {:?}",
                record.primary_locator,
                record.kind,
                TAsset::Marker::KIND
            )));
            return AssetReadinessNode {
                id: record.id,
                locator: Some(record.primary_locator.clone()),
                kind: Some(record.kind),
                revision: Some(record.revision),
                load_state: AssetLoadState::NotLoaded,
                diagnostics,
            };
        }

        let load_state = AssetLoadState::from_resource(
            Some(record),
            self.resource_manager().runtime_state(record.id),
            self.resource_manager()
                .get::<TAsset::Marker, TAsset>(handle.resource_handle())
                .is_some(),
        );

        AssetReadinessNode {
            id: record.id,
            locator: Some(record.primary_locator.clone()),
            kind: Some(record.kind),
            revision: Some(record.revision),
            load_state,
            diagnostics,
        }
    }
}

impl ProjectAssetManager {
    fn collect_dependency_readiness(
        &self,
        root_id: AssetId,
        dependency_ids: &[AssetId],
    ) -> Vec<AssetDependencyReadiness> {
        let mut rows = Vec::new();
        let mut row_by_id = HashMap::new();
        let mut expanded = HashSet::new();
        expanded.insert(root_id);

        let mut queue = VecDeque::new();
        for dependency_id in dependency_ids {
            queue.push_back((*dependency_id, 1_u32, true));
        }

        while let Some((dependency_id, depth, direct)) = queue.pop_front() {
            let record = self
                .resource_manager()
                .registry()
                .get(dependency_id)
                .cloned();
            let row = self.dependency_readiness_row(dependency_id, record.as_ref(), depth, direct);
            upsert_dependency_row(&mut rows, &mut row_by_id, row);

            let Some(record) = record else {
                continue;
            };
            if !expanded.insert(dependency_id) {
                continue;
            }
            for nested in &record.dependency_ids {
                queue.push_back((*nested, depth + 1, false));
            }
        }

        rows
    }

    fn dependency_readiness_row(
        &self,
        dependency_id: AssetId,
        record: Option<&ResourceRecord>,
        depth: u32,
        direct: bool,
    ) -> AssetDependencyReadiness {
        let Some(record) = record else {
            return AssetDependencyReadiness {
                id: dependency_id,
                locator: None,
                kind: None,
                revision: None,
                depth,
                direct,
                load_state: AssetLoadState::Failed,
                diagnostics: vec![ResourceDiagnostic::error(format!(
                    "missing asset dependency record {}",
                    dependency_id
                ))],
            };
        };

        AssetDependencyReadiness {
            id: record.id,
            locator: Some(record.primary_locator.clone()),
            kind: Some(record.kind),
            revision: Some(record.revision),
            depth,
            direct,
            load_state: AssetLoadState::from_resource(
                Some(record),
                self.resource_manager().runtime_state(record.id),
                self.resource_manager().get_untyped(record.id).is_some(),
            ),
            diagnostics: record.diagnostics.clone(),
        }
    }
}

fn upsert_dependency_row(
    rows: &mut Vec<AssetDependencyReadiness>,
    row_by_id: &mut HashMap<AssetId, usize>,
    row: AssetDependencyReadiness,
) {
    if let Some(index) = row_by_id.get(&row.id).copied() {
        let existing = &mut rows[index];
        if row.depth < existing.depth {
            existing.depth = row.depth;
        }
        existing.direct |= row.direct;
        return;
    }

    row_by_id.insert(row.id, rows.len());
    rows.push(row);
}

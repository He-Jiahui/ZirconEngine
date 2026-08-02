use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{Asset, AssetLoadState, AssetLoadStates, Handle};
use crate::asset::{AssetId, AssetKind, AssetUri, ProjectAssetManager};
use crate::core::resource::{
    ResourceDiagnostic, ResourceMarker, ResourceReadinessGeneration, ResourceReadinessRow,
};

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
        let generation = self.resource_manager().readiness_generation();
        let load_states = self.load_states_from_generation(handle, &generation);
        let root = readiness_root_node::<TAsset>(handle, &generation);
        let dependencies = generation
            .row(handle.id())
            .filter(|row| row.record.kind == TAsset::Marker::KIND)
            .map(|row| {
                collect_dependency_readiness(&generation, row.record.id, &row.record.dependency_ids)
            })
            .unwrap_or_default();

        AssetReadinessReport {
            root,
            load_states,
            dependencies,
        }
    }
}

fn readiness_root_node<TAsset: Asset>(
    handle: Handle<TAsset>,
    generation: &Arc<ResourceReadinessGeneration>,
) -> AssetReadinessNode {
    let Some(row) = generation.row(handle.id()) else {
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

    let record = &row.record;
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

    AssetReadinessNode {
        id: record.id,
        locator: Some(record.primary_locator.clone()),
        kind: Some(record.kind),
        revision: Some(record.revision),
        load_state: row.typed_load_state::<TAsset>().into(),
        diagnostics,
    }
}

fn collect_dependency_readiness(
    generation: &ResourceReadinessGeneration,
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
        if let Some(index) = row_by_id.get(&dependency_id).copied() {
            let existing: &mut AssetDependencyReadiness = &mut rows[index];
            existing.depth = existing.depth.min(depth);
            existing.direct |= direct;
            continue;
        }

        let readiness_row = generation.row(dependency_id);
        let row = dependency_readiness_row(dependency_id, readiness_row, depth, direct);
        row_by_id.insert(dependency_id, rows.len());
        rows.push(row);

        let Some(readiness_row) = readiness_row else {
            continue;
        };
        if !expanded.insert(dependency_id) {
            continue;
        }
        for nested in &readiness_row.record.dependency_ids {
            queue.push_back((*nested, depth + 1, false));
        }
    }

    rows
}

fn dependency_readiness_row(
    dependency_id: AssetId,
    row: Option<&Arc<ResourceReadinessRow>>,
    depth: u32,
    direct: bool,
) -> AssetDependencyReadiness {
    let Some(row) = row else {
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
        id: row.record.id,
        locator: Some(row.record.primary_locator.clone()),
        kind: Some(row.record.kind),
        revision: Some(row.record.revision),
        depth,
        direct,
        load_state: row.load_state.into(),
        diagnostics: row.record.diagnostics.clone(),
    }
}

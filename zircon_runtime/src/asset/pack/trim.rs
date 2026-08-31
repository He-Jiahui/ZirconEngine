use std::collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackTrimConfig {
    pub roots: Vec<String>,
    pub asset_filter: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackTrimInputAsset {
    pub path: String,
    pub dependencies: Vec<String>,
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackTrimReport {
    pub included_assets: Vec<String>,
    pub trimmed_assets: Vec<ZrPackTrimmedAsset>,
    pub missing_dependencies: Vec<ZrPackMissingDependency>,
    pub duplicate_assets: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackTrimmedAsset {
    pub path: String,
    pub reason: ZrPackTrimReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackMissingDependency {
    pub owner: String,
    pub dependency: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZrPackTrimReason {
    Unreferenced,
    AssetFilterMismatch(String),
    UnreferencedAndAssetFilterMismatch(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackTrimPlanner;

impl ZrPackTrimConfig {
    pub fn new(roots: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            roots: roots.into_iter().map(Into::into).collect(),
            asset_filter: None,
        }
    }

    pub fn with_asset_filter(mut self, asset_filter: impl Into<String>) -> Self {
        self.asset_filter = Some(asset_filter.into());
        self
    }
}

impl ZrPackTrimInputAsset {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            dependencies: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }
}

impl ZrPackTrimReport {
    pub fn included_asset_count(&self) -> usize {
        self.included_assets.len()
    }

    pub fn trimmed_asset_count(&self) -> usize {
        self.trimmed_assets.len()
    }

    pub fn has_missing_dependencies(&self) -> bool {
        !self.missing_dependencies.is_empty()
    }

    pub fn has_duplicate_assets(&self) -> bool {
        !self.duplicate_assets.is_empty()
    }
}

impl ZrPackTrimmedAsset {
    pub fn new(path: impl Into<String>, reason: ZrPackTrimReason) -> Self {
        Self {
            path: path.into(),
            reason,
        }
    }
}

impl ZrPackMissingDependency {
    pub fn new(owner: impl Into<String>, dependency: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            dependency: dependency.into(),
        }
    }
}

impl ZrPackTrimReason {
    pub fn diagnostic_label(&self) -> String {
        match self {
            Self::Unreferenced => "unreferenced".to_string(),
            Self::AssetFilterMismatch(filter) => {
                format!("asset_filter {filter} did not match")
            }
            Self::UnreferencedAndAssetFilterMismatch(filter) => {
                format!("unreferenced; asset_filter {filter} did not match")
            }
        }
    }
}

impl ZrPackTrimPlanner {
    pub fn trim(
        config: ZrPackTrimConfig,
        assets: impl IntoIterator<Item = ZrPackTrimInputAsset>,
    ) -> ZrPackTrimReport {
        let (asset_map, mut duplicate_assets, diagnostics) = collect_assets(assets);
        let (reachable_assets, mut missing_dependencies, mut diagnostics) =
            reachable_asset_closure(&config, &asset_map, diagnostics);
        let mut included_assets = Vec::new();
        let mut trimmed_assets = Vec::new();

        // Reachability is resolved before profile filtering so the report can
        // distinguish unreferenced assets from reachable assets cut by labels.
        for (path, asset) in &asset_map {
            let is_reachable = reachable_assets.contains(path);
            let matches_filter = matches_asset_filter(asset, config.asset_filter.as_deref());
            if is_reachable && matches_filter {
                included_assets.push(path.clone());
                continue;
            }

            let reason = trim_reason(is_reachable, matches_filter, config.asset_filter.as_deref());
            diagnostics.push(format!(
                "trimmed asset {path}: {}",
                reason.diagnostic_label()
            ));
            trimmed_assets.push(ZrPackTrimmedAsset::new(path.clone(), reason));
        }

        missing_dependencies.sort_unstable_by(|left, right| {
            left.owner
                .cmp(&right.owner)
                .then(left.dependency.cmp(&right.dependency))
        });
        duplicate_assets.sort_unstable();
        duplicate_assets.dedup();
        diagnostics.sort_unstable();

        ZrPackTrimReport {
            included_assets,
            trimmed_assets,
            missing_dependencies,
            duplicate_assets,
            diagnostics,
        }
    }
}

fn collect_assets(
    assets: impl IntoIterator<Item = ZrPackTrimInputAsset>,
) -> (
    BTreeMap<String, ZrPackTrimInputAsset>,
    Vec<String>,
    Vec<String>,
) {
    let mut asset_map = BTreeMap::new();
    let mut duplicate_assets = Vec::new();
    let mut diagnostics = Vec::new();
    for asset in assets {
        match asset_map.entry(asset.path.clone()) {
            Entry::Occupied(_) => {
                diagnostics.push(format!("asset {} is duplicated in trim input", asset.path));
                duplicate_assets.push(asset.path);
            }
            Entry::Vacant(entry) => {
                entry.insert(asset);
            }
        }
    }

    (asset_map, duplicate_assets, diagnostics)
}

fn reachable_asset_closure(
    config: &ZrPackTrimConfig,
    asset_map: &BTreeMap<String, ZrPackTrimInputAsset>,
    mut diagnostics: Vec<String>,
) -> (BTreeSet<String>, Vec<ZrPackMissingDependency>, Vec<String>) {
    let mut reachable_assets = BTreeSet::new();
    let mut queue = VecDeque::new();
    let mut missing_dependencies = Vec::new();

    for root in &config.roots {
        if asset_map.contains_key(root) {
            if reachable_assets.insert(root.clone()) {
                queue.push_back(root.clone());
            }
            continue;
        }

        diagnostics.push(format!("root asset {root} is missing"));
        missing_dependencies.push(ZrPackMissingDependency::new("<root>", root.clone()));
    }

    while let Some(path) = queue.pop_front() {
        let Some(asset) = asset_map.get(&path) else {
            continue;
        };
        for dependency in &asset.dependencies {
            if asset_map.contains_key(dependency) {
                if reachable_assets.insert(dependency.clone()) {
                    queue.push_back(dependency.clone());
                }
                continue;
            }

            diagnostics.push(format!(
                "asset {path} references missing dependency {dependency}"
            ));
            missing_dependencies.push(ZrPackMissingDependency::new(
                path.clone(),
                dependency.clone(),
            ));
        }
    }

    (reachable_assets, missing_dependencies, diagnostics)
}

fn matches_asset_filter(asset: &ZrPackTrimInputAsset, asset_filter: Option<&str>) -> bool {
    asset_filter
        .map(|filter| asset.labels.iter().any(|label| label == filter))
        .unwrap_or(true)
}

fn trim_reason(
    is_reachable: bool,
    matches_filter: bool,
    asset_filter: Option<&str>,
) -> ZrPackTrimReason {
    match (is_reachable, matches_filter, asset_filter) {
        (false, false, Some(filter)) => {
            ZrPackTrimReason::UnreferencedAndAssetFilterMismatch(filter.to_string())
        }
        (true, false, Some(filter)) => ZrPackTrimReason::AssetFilterMismatch(filter.to_string()),
        _ => ZrPackTrimReason::Unreferenced,
    }
}

#[cfg(test)]
#[path = "trim/optimization_tests.rs"]
mod optimization_tests;

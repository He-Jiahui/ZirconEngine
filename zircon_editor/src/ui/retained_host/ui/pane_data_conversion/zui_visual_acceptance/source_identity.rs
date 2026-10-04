use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{json, Value};
use zircon_runtime::ui::v2::{UiV2SourceFileReceipt, UiV2UnresolvedSourceImport, UiZuiAssetLoader};

use super::contract::{contained, hash_file, Entry, ReviewCase};
use crate::ui::template_runtime::RetainedUiHostNodeProjection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AuthoredNodeIdentity {
    pub(super) source_path: String,
    pub(super) source_node_id: String,
    pub(super) control_id: String,
    pub(super) component: String,
    pub(super) instance_path: String,
}

fn resource_uri_for_source_path(source_path: &str) -> Option<String> {
    if source_path.starts_with("res://") {
        return Some(source_path.to_owned());
    }
    let normalized = source_path.replace('\\', "/");
    let asset_relative = normalized
        .rsplit_once("/assets/")
        .map(|(_, relative)| relative)
        .or_else(|| normalized.strip_prefix("assets/"))?;
    (!asset_relative.is_empty()).then(|| format!("res://{asset_relative}"))
}

#[derive(Default)]
pub(super) struct SourceIdentityIndex {
    by_source_node: BTreeMap<(String, String), AuthoredNodeIdentity>,
    source_paths: BTreeSet<String>,
    resource_uri_sources: BTreeMap<String, BTreeSet<String>>,
    selected: Option<super::state::WorkbenchStateSelector>,
    source_fingerprints: Vec<[String; 2]>,
    source_map_fingerprint: Option<[String; 2]>,
}

impl SourceIdentityIndex {
    pub(super) fn load(repo: &Path, entry: &Entry, case: &ReviewCase) -> Result<Self, String> {
        let mut sources = BTreeSet::new();
        sources.insert(entry.source_path.clone());
        let selected = super::state::workbench_state_selector(&case.data)?;
        if let Some(selector) = &selected {
            sources.insert(selector.source_path.clone());
        }
        for path in entry.zui_dependencies(repo)? {
            let relative = path
                .strip_prefix(repo)
                .map_err(|_| "catalog dependency escaped repository root")?
                .to_string_lossy()
                .replace('\\', "/");
            sources.insert(relative);
        }

        let mut source_map_fingerprint = None;
        if case.host == "editor" {
            let layout_root = repo.join("docs/layout");
            let map_relative = format!(
                "{}/{}/evidence/editor-host-sources.json",
                entry.category, entry.name
            );
            let map_path = contained(&layout_root, &map_relative)?;
            if map_path.is_file() {
                let map_source_path = map_path
                    .strip_prefix(repo)
                    .map_err(|_| "editor host source map escaped repository root")?
                    .to_string_lossy()
                    .replace('\\', "/");
                source_map_fingerprint = Some([map_source_path, hash_file(&map_path)?]);
                let map: serde_json::Value = serde_json::from_slice(
                    &std::fs::read(&map_path).map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
                let consumers = map
                    .as_array()
                    .ok_or("editor host source map must be an array")?;
                for consumer in consumers {
                    let source = consumer
                        .get("sourcePath")
                        .and_then(serde_json::Value::as_str)
                        .ok_or("editor host source map contains a missing sourcePath")?;
                    if source.ends_with(".zui") {
                        sources.insert(source.to_owned());
                    }
                }
            }
        }

        let mut index = Self {
            selected,
            source_map_fingerprint,
            ..Self::default()
        };
        let mut source_fingerprints = Vec::new();
        for source_path in sources {
            let path = contained(repo, &source_path)?;
            let sha256 = hash_file(&path)?;
            let document = UiZuiAssetLoader::load_zui_file(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            source_fingerprints.push([source_path.clone(), sha256]);
            index.source_paths.insert(source_path.clone());
            if let Some(resource_uri) = resource_uri_for_source_path(&source_path) {
                index
                    .resource_uri_sources
                    .entry(resource_uri)
                    .or_default()
                    .insert(source_path.clone());
            }
            for (source_node_id, node) in document.nodes {
                let Some(control_id) = node.control_id else {
                    continue;
                };
                let identity = AuthoredNodeIdentity {
                    source_path: source_path.clone(),
                    source_node_id: source_node_id.clone(),
                    control_id,
                    component: node.component,
                    instance_path: String::new(),
                };
                if index
                    .by_source_node
                    .insert((source_path.clone(), source_node_id.clone()), identity)
                    .is_some()
                {
                    return Err(format!(
                        "duplicate authored source identity: {source_path}#{source_node_id}"
                    ));
                }
            }
        }
        index.source_fingerprints = source_fingerprints;
        Ok(index)
    }

    pub(super) fn runtime_loaded_sources_audit(
        &self,
        document_ids: &[String],
        receipts: &[UiV2SourceFileReceipt],
        unresolved_imports: &[UiV2UnresolvedSourceImport],
    ) -> (Value, Vec<String>) {
        let mut issues = Vec::new();
        if document_ids.is_empty() {
            issues.push("runtime source receipt has no requested document roots".to_owned());
        }
        for required_root in [
            "res://ui/editor/host/workbench_shell.zui",
            "res://ui/editor/windows/workbench_window.zui",
        ] {
            if !document_ids
                .iter()
                .any(|document_id| document_id == required_root)
            {
                issues.push(format!(
                    "runtime source receipt omitted required product root {required_root}"
                ));
            }
        }
        if receipts.is_empty() {
            issues.push("runtime source receipt has no loaded V2 source files".to_owned());
        }
        for document_id in document_ids {
            if !receipts.iter().any(|receipt| {
                receipt.resource_uri.as_deref() == Some(document_id)
                    || receipt.source_path.as_deref() == Some(document_id)
                    || receipt.asset_id == *document_id
            }) {
                issues.push(format!(
                    "requested V2 document root has no matching raw source receipt: {document_id}"
                ));
            }
        }

        let mut files = Vec::with_capacity(receipts.len());
        let mut observed_sources = BTreeSet::new();
        for receipt in receipts {
            let expected_source = self.source_path_for_receipt(receipt);
            let expected_hash = expected_source.as_deref().and_then(|source_path| {
                self.source_fingerprints
                    .iter()
                    .find(|[path, _]| path == source_path)
                    .map(|[_, hash]| hash.as_str())
            });
            let catalog_matches = expected_hash == Some(receipt.sha256.as_str());
            let current_hash = hash_file(&receipt.physical_path).ok();
            let current_file_matches = current_hash.as_deref() == Some(receipt.sha256.as_str());

            if expected_source.is_none() {
                issues.push(format!(
                    "loaded runtime source has no unique catalog identity: {}",
                    receipt
                        .resource_uri
                        .as_deref()
                        .or(receipt.source_path.as_deref())
                        .unwrap_or("<no source URI>")
                ));
            }
            if !catalog_matches {
                issues.push(format!(
                    "loaded runtime source does not match catalog bytes: {}",
                    expected_source
                        .as_deref()
                        .or(receipt.resource_uri.as_deref())
                        .unwrap_or("<no source URI>")
                ));
            }
            if !current_file_matches {
                issues.push(format!(
                    "loaded runtime source file changed or disappeared during capture: {}",
                    receipt.physical_path.display()
                ));
            }
            if !receipt.physical_path.is_absolute() {
                issues.push(format!(
                    "loaded runtime source path is not physical and absolute: {}",
                    receipt.physical_path.display()
                ));
            }
            if let Some(source_path) = &expected_source {
                if !observed_sources.insert(source_path.clone()) {
                    issues.push(format!(
                        "runtime source closure contains duplicate logical source {source_path}"
                    ));
                }
            }
            files.push(json!({
                "assetId": receipt.asset_id,
                "sourcePath": expected_source
                    .clone()
                    .or_else(|| receipt.source_path.clone()),
                "resourceUri": receipt.resource_uri,
                "physicalPath": receipt.physical_path.to_string_lossy(),
                "sha256": receipt.sha256,
                "catalogMatches": catalog_matches,
                "currentFileMatches": current_file_matches,
            }));
        }
        for unresolved in unresolved_imports {
            issues.push(format!(
                "unresolved V2 import {} from {}",
                unresolved.reference,
                unresolved
                    .source_path
                    .as_deref()
                    .or(unresolved.resource_uri.as_deref())
                    .unwrap_or(&unresolved.source_asset_id)
            ));
        }
        files.sort_by(|left, right| {
            left["sourcePath"]
                .as_str()
                .cmp(&right["sourcePath"].as_str())
                .then_with(|| {
                    left["physicalPath"]
                        .as_str()
                        .cmp(&right["physicalPath"].as_str())
                })
        });
        let audit = json!({
            "complete": issues.is_empty(),
            "documentIds": document_ids,
            "files": files,
            "unresolvedImports": unresolved_imports,
        });
        (audit, issues)
    }

    fn source_path_for_receipt(&self, receipt: &UiV2SourceFileReceipt) -> Option<String> {
        if let Some(source_path) = &receipt.source_path {
            if self.source_paths.contains(source_path) {
                return Some(source_path.clone());
            }
        }
        let resource_uri = receipt.resource_uri.as_deref().or_else(|| {
            receipt
                .source_path
                .as_deref()
                .filter(|source_path| source_path.starts_with("res://"))
        })?;
        let source_paths = self.resource_uri_sources.get(resource_uri)?;
        (source_paths.len() == 1)
            .then(|| source_paths.iter().next().cloned())
            .flatten()
    }

    pub(super) fn verify_current(&self, repo: &Path) -> Result<Vec<String>, String> {
        let mut issues = Vec::new();
        for [source_path, expected] in &self.source_fingerprints {
            let path = contained(repo, source_path)?;
            if hash_file(&path)? != *expected {
                issues.push(format!(
                    "source identity input changed during capture: {source_path}"
                ));
            }
        }
        if let Some([source_path, expected]) = &self.source_map_fingerprint {
            let path = contained(repo, source_path)?;
            if hash_file(&path)? != *expected {
                issues.push(format!(
                    "editor host source map changed during capture: {source_path}"
                ));
            }
        }
        Ok(issues)
    }

    pub(super) fn dependency_issues(&self, entry: &Entry) -> Vec<String> {
        let dependencies = entry
            .dependency_fingerprints
            .iter()
            .filter_map(|dependency| {
                Some((
                    dependency.get("sourcePath")?.as_str()?,
                    dependency.get("sha256")?.as_str()?,
                ))
            })
            .collect::<BTreeSet<_>>();
        self.source_fingerprints
            .iter()
            .filter(|[source_path, sha256]| {
                source_path != &entry.source_path
                    && !dependencies.contains(&(source_path.as_str(), sha256.as_str()))
            })
            .map(|[source_path, _]| {
                format!("authored semantic source is not in catalog dependencies: {source_path}")
            })
            .collect()
    }

    pub(super) fn provenance(&self) -> serde_json::Value {
        serde_json::json!({
            "sourceMapFingerprint": self.source_map_fingerprint.clone(),
            "sources": self.source_fingerprints.clone(),
        })
    }

    pub(super) fn source_map_fingerprint(&self) -> Option<[String; 2]> {
        self.source_map_fingerprint.clone()
    }

    pub(super) fn resolve(
        &self,
        node: &RetainedUiHostNodeProjection,
    ) -> Result<AuthoredNodeIdentity, String> {
        let control_id = node
            .control_id
            .as_deref()
            .ok_or_else(|| format!("retained node {} has no control id", node.node_id))?;
        let source_path = node
            .source_path
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("retained node {} has no authored source path", node.node_id))?;
        let source_path = self
            .unique_catalog_source_path(source_path)
            .ok_or_else(|| {
                format!(
                    "retained node {} source URI has no unique catalog identity: {source_path}",
                    node.node_id
                )
            })?;
        let source_node_id = node
            .source_node_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "retained node {} has no authored source node id",
                    node.node_id
                )
            })?;
        let instance_path = node.instance_path.as_ref().ok_or_else(|| {
            format!(
                "retained node {} has no authored instance path",
                node.node_id
            )
        })?;
        let identity = self
            .by_source_node
            .get(&(source_path.clone(), source_node_id.to_owned()))
            .ok_or_else(|| {
                format!("no loaded authored source node matches {source_path}#{source_node_id}")
            })?;
        if identity.component != node.component || identity.control_id != control_id {
            return Err(format!(
                "retained authored identity {source_path}#{source_node_id} differs in component or control id"
            ));
        }
        if let Some(selector) = self
            .selected
            .as_ref()
            .filter(|selector| selector.control_id == control_id)
        {
            let selected_source_node = selector
                .source_node_id
                .as_deref()
                .map_or(true, |selected| selected == source_node_id);
            let selected_instance = selector.instance_path.as_deref().map_or(true, |selected| {
                selected == serialize_instance_path(instance_path)
            });
            if selector.source_path != source_path || !selected_source_node || !selected_instance {
                return Err(format!(
                    "workbenchState identity does not match retained node {source_path}#{source_node_id}"
                ));
            }
        }
        let mut resolved = identity.clone();
        resolved.instance_path = serialize_instance_path(instance_path);
        Ok(resolved)
    }

    pub(super) fn resolve_template_row(
        &self,
        source_path: &str,
        source_node_id: &str,
        control_id: &str,
        instance_path: &str,
    ) -> Result<AuthoredNodeIdentity, String> {
        if source_path.is_empty()
            || source_node_id.is_empty()
            || control_id.is_empty()
            || instance_path.is_empty()
        {
            return Err("template row has incomplete authored identity".into());
        }
        let source_path = self
            .unique_catalog_source_path(source_path)
            .ok_or_else(|| {
                format!("template row source URI has no unique catalog identity: {source_path}")
            })?;
        let instance_steps: Vec<zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep> =
            serde_json::from_str(instance_path)
                .map_err(|error| format!("template row instance path is invalid: {error}"))?;
        if serialize_instance_path(&instance_steps) != instance_path {
            return Err("template row instance path is not canonical JSON".into());
        }
        let identity = self
            .by_source_node
            .get(&(source_path.clone(), source_node_id.to_owned()))
            .ok_or_else(|| {
                format!("no loaded authored source node matches {source_path}#{source_node_id}")
            })?;
        if identity.control_id != control_id {
            return Err(format!(
                "template row authored control id differs for {source_path}#{source_node_id}"
            ));
        }
        if let Some(selector) = self
            .selected
            .as_ref()
            .filter(|selector| selector.control_id == control_id)
        {
            let selected_source_node = selector
                .source_node_id
                .as_deref()
                .map_or(true, |selected| selected == source_node_id);
            let selected_instance = selector
                .instance_path
                .as_deref()
                .map_or(true, |selected| selected == instance_path);
            if selector.source_path != source_path || !selected_source_node || !selected_instance {
                return Err(format!(
                    "workbenchState identity does not match template row {source_path}#{source_node_id}"
                ));
            }
        }
        let mut resolved = identity.clone();
        resolved.instance_path = instance_path.to_owned();
        Ok(resolved)
    }

    fn unique_catalog_source_path(&self, candidate: &str) -> Option<String> {
        if self.source_paths.contains(candidate)
            || self
                .by_source_node
                .keys()
                .any(|(source_path, _)| source_path == candidate)
        {
            return Some(candidate.to_owned());
        }
        let resource_uri = resource_uri_for_source_path(candidate)?;
        let candidates = self.resource_uri_sources.get(&resource_uri)?;
        (candidates.len() == 1)
            .then(|| candidates.iter().next().cloned())
            .flatten()
    }
}

fn serialize_instance_path(
    path: &[zircon_runtime_interface::ui::v2::UiTemplateNodeInstancePathStep],
) -> String {
    serde_json::to_string(path).unwrap_or_default()
}

#[cfg(test)]
#[path = "tests/source_identity.rs"]
mod tests;

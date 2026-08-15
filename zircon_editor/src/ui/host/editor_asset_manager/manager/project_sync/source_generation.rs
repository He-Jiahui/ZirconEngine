use std::collections::HashMap;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::{PackageAssetRegistry, ProjectManager, ProjectManifest};
use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::host::editor_asset_manager::manager) struct EditorAssetProjectSourceGeneration {
    project_root: PathBuf,
    manifest: Option<ProjectManifest>,
    package_assets: Option<PackageAssetRegistry>,
    records: HashMap<ResourceId, ResourceRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::host::editor_asset_manager::manager) struct EditorAssetProjectSourceRename {
    pub previous: ResourceRecord,
    pub current: ResourceRecord,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::host::editor_asset_manager::manager) struct EditorAssetProjectSourceDelta {
    pub project_metadata_changed: bool,
    pub added: Vec<ResourceRecord>,
    pub modified: Vec<ResourceRecord>,
    pub removed: Vec<ResourceRecord>,
    pub renamed: Vec<EditorAssetProjectSourceRename>,
}

impl EditorAssetProjectSourceGeneration {
    pub(in crate::ui::host::editor_asset_manager::manager) fn capture(
        project: &ProjectManager,
    ) -> Self {
        Self::from_records(
            project.paths().root(),
            project.manifest().clone(),
            project.package_assets().clone(),
            project.registry().values().cloned(),
        )
    }

    pub(in crate::ui::host::editor_asset_manager::manager) fn delta_since(
        &self,
        previous: &Self,
    ) -> EditorAssetProjectSourceDelta {
        if self.project_root != previous.project_root {
            return EditorAssetProjectSourceDelta {
                project_metadata_changed: true,
                added: sorted_records(self.records.values().cloned()),
                removed: sorted_records(previous.records.values().cloned()),
                ..EditorAssetProjectSourceDelta::default()
            };
        }

        let mut delta = EditorAssetProjectSourceDelta {
            project_metadata_changed: self.manifest != previous.manifest
                || self.package_assets != previous.package_assets,
            ..EditorAssetProjectSourceDelta::default()
        };
        for (id, current) in &self.records {
            let Some(previous_record) = previous.records.get(id) else {
                delta.added.push(current.clone());
                continue;
            };
            if current.primary_locator != previous_record.primary_locator {
                delta.renamed.push(EditorAssetProjectSourceRename {
                    previous: previous_record.clone(),
                    current: current.clone(),
                });
            } else if current != previous_record {
                delta.modified.push(current.clone());
            }
        }
        for (id, previous_record) in &previous.records {
            if !self.records.contains_key(id) {
                delta.removed.push(previous_record.clone());
            }
        }
        sort_delta(&mut delta);
        delta
    }

    fn from_records(
        project_root: &Path,
        manifest: ProjectManifest,
        package_assets: PackageAssetRegistry,
        records: impl IntoIterator<Item = ResourceRecord>,
    ) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            manifest: Some(manifest),
            package_assets: Some(package_assets),
            records: records
                .into_iter()
                .map(|record| (record.id, record))
                .collect(),
        }
    }
}

impl EditorAssetProjectSourceDelta {
    pub(in crate::ui::host::editor_asset_manager::manager) fn is_unchanged(&self) -> bool {
        !self.project_metadata_changed
            && self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.renamed.is_empty()
    }

    pub(in crate::ui::host::editor_asset_manager::manager) fn touched_record_count(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len() + self.renamed.len()
    }

    pub(in crate::ui::host::editor_asset_manager::manager) fn affects_shader(&self) -> bool {
        self.project_metadata_changed
            || self
                .added
                .iter()
                .chain(&self.modified)
                .chain(&self.removed)
                .any(|record| record.kind == ResourceKind::Shader)
            || self.renamed.iter().any(|rename| {
                rename.previous.kind == ResourceKind::Shader
                    || rename.current.kind == ResourceKind::Shader
            })
    }
}

fn sort_delta(delta: &mut EditorAssetProjectSourceDelta) {
    delta.added.sort_by_key(locator_key);
    delta.modified.sort_by_key(locator_key);
    delta.removed.sort_by_key(locator_key);
    delta
        .renamed
        .sort_by_key(|rename| rename.current.primary_locator.to_string());
}

fn sorted_records(records: impl IntoIterator<Item = ResourceRecord>) -> Vec<ResourceRecord> {
    let mut records = records.into_iter().collect::<Vec<_>>();
    records.sort_by_key(locator_key);
    records
}

fn locator_key(record: &ResourceRecord) -> String {
    record.primary_locator.to_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use zircon_runtime::asset::project::{PackageAssetRegistry, ProjectManifest};
    use zircon_runtime::asset::AssetUri;
    use zircon_runtime::core::resource::{
        ResourceId, ResourceKind, ResourceLocator, ResourceRecord,
    };

    use super::EditorAssetProjectSourceGeneration;

    #[test]
    fn source_generation_classifies_added_modified_removed_and_renamed() {
        let removed = record("res://removed.asset");
        let rename_before = record("res://rename-before.asset");
        let mut rename_after = rename_before.clone();
        rename_after.primary_locator = locator("res://rename-after.asset");
        let modified_before = record("res://modified.asset");
        let mut modified_after = modified_before.clone();
        modified_after.source_hash = "changed".to_string();
        let added = record("res://added.asset");

        let previous = EditorAssetProjectSourceGeneration::from_records(
            Path::new("project"),
            manifest(),
            PackageAssetRegistry::default(),
            [removed, rename_before, modified_before],
        );
        let current = EditorAssetProjectSourceGeneration::from_records(
            Path::new("project"),
            manifest(),
            PackageAssetRegistry::default(),
            [added, rename_after, modified_after],
        );
        let delta = current.delta_since(&previous);

        assert_eq!(delta.added.len(), 1);
        assert_eq!(delta.modified.len(), 1);
        assert_eq!(delta.removed.len(), 1);
        assert_eq!(delta.renamed.len(), 1);
        assert!(!delta.is_unchanged());
        assert!(current.delta_since(&current).is_unchanged());
    }

    #[test]
    fn source_delta_reports_shader_impact_and_touched_count() {
        let mut shader = record("res://shader.asset");
        shader.kind = ResourceKind::Shader;
        let mut data = record("res://data.asset");
        data.source_hash = "changed".to_string();
        let delta = super::EditorAssetProjectSourceDelta {
            added: vec![shader],
            modified: vec![data],
            ..super::EditorAssetProjectSourceDelta::default()
        };

        assert_eq!(delta.touched_record_count(), 2);
        assert!(delta.affects_shader());
        assert!(!super::EditorAssetProjectSourceDelta {
            modified: vec![record("res://only-data.asset")],
            ..super::EditorAssetProjectSourceDelta::default()
        }
        .affects_shader());
    }

    fn record(locator_text: &str) -> ResourceRecord {
        let locator = locator(locator_text);
        ResourceRecord::new(
            ResourceId::from_locator(&locator),
            ResourceKind::Data,
            locator,
        )
    }

    fn manifest() -> ProjectManifest {
        ProjectManifest::new(
            "Project",
            AssetUri::parse("res://main.scene").expect("default scene"),
            1,
        )
    }

    fn locator(value: &str) -> ResourceLocator {
        ResourceLocator::parse(value).expect("valid locator")
    }
}

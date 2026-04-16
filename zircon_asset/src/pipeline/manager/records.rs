use zircon_manager::{
    AssetRecordKind, AssetStatusRecord, ProjectInfo, ResourceChangeKind, ResourceChangeRecord,
    ResourceStateRecord, ResourceStatusRecord,
};

use crate::{
    AssetKind, AssetMetadata, ProjectManager, ResourceEvent, ResourceEventKind, ResourceState,
};

pub(super) fn project_info(project: &ProjectManager) -> ProjectInfo {
    ProjectInfo {
        root_path: project.paths().root().to_string_lossy().into_owned(),
        name: project.manifest().name.clone(),
        default_scene_uri: project.manifest().default_scene.to_string(),
        library_version: project.manifest().library_version,
    }
}

pub(super) fn status_record(metadata: &AssetMetadata) -> AssetStatusRecord {
    AssetStatusRecord {
        id: metadata.id().to_string(),
        uri: metadata.primary_locator().to_string(),
        kind: match metadata.kind {
            AssetKind::Texture => AssetRecordKind::Texture,
            AssetKind::Shader => AssetRecordKind::Shader,
            AssetKind::Material => AssetRecordKind::Material,
            AssetKind::Scene => AssetRecordKind::Scene,
            AssetKind::Model => AssetRecordKind::Model,
            AssetKind::UiLayout => AssetRecordKind::UiLayout,
            AssetKind::UiWidget => AssetRecordKind::UiWidget,
            AssetKind::UiStyle => AssetRecordKind::UiStyle,
        },
        artifact_uri: metadata.artifact_locator().map(ToString::to_string),
        imported: metadata.artifact_locator().is_some(),
        source_hash: metadata.source_hash.clone(),
        importer_version: metadata.importer_version,
        config_hash: metadata.config_hash.clone(),
    }
}

pub(super) fn resource_status_record(metadata: &AssetMetadata) -> ResourceStatusRecord {
    ResourceStatusRecord {
        id: metadata.id().to_string(),
        locator: metadata.primary_locator().to_string(),
        kind: match metadata.kind {
            AssetKind::Texture => AssetRecordKind::Texture,
            AssetKind::Shader => AssetRecordKind::Shader,
            AssetKind::Material => AssetRecordKind::Material,
            AssetKind::Scene => AssetRecordKind::Scene,
            AssetKind::Model => AssetRecordKind::Model,
            AssetKind::UiLayout => AssetRecordKind::UiLayout,
            AssetKind::UiWidget => AssetRecordKind::UiWidget,
            AssetKind::UiStyle => AssetRecordKind::UiStyle,
        },
        artifact_locator: metadata.artifact_locator().map(ToString::to_string),
        revision: metadata.revision,
        state: match metadata.state {
            ResourceState::Pending => ResourceStateRecord::Pending,
            ResourceState::Ready => ResourceStateRecord::Ready,
            ResourceState::Error => ResourceStateRecord::Error,
            ResourceState::Reloading => ResourceStateRecord::Reloading,
        },
        dependency_ids: metadata
            .dependency_ids
            .iter()
            .map(ToString::to_string)
            .collect(),
        diagnostics: metadata
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect(),
    }
}

pub(super) fn resource_change_record(event: ResourceEvent) -> ResourceChangeRecord {
    ResourceChangeRecord {
        kind: match event.kind {
            ResourceEventKind::Added => ResourceChangeKind::Added,
            ResourceEventKind::Updated => ResourceChangeKind::Updated,
            ResourceEventKind::Removed => ResourceChangeKind::Removed,
            ResourceEventKind::Renamed => ResourceChangeKind::Renamed,
            ResourceEventKind::ReloadFailed => ResourceChangeKind::ReloadFailed,
        },
        id: event.id.to_string(),
        locator: event.locator.map(|locator| locator.to_string()),
        previous_locator: event.previous_locator.map(|locator| locator.to_string()),
        revision: event.revision,
    }
}

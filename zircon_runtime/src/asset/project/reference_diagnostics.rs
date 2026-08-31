use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::asset::{AssetUri, AssetUuid};
use crate::core::resource::{ResourceId, ResourceLocator};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectReferenceDiagnosticPhase {
    Load,
    Save,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectReferenceDiagnosticKind {
    DanglingAssetReference {
        uuid: AssetUuid,
        locator: ResourceLocator,
    },
    PersistedDanglingReference {
        uuid: AssetUuid,
        path_hint: Arc<str>,
        subasset: Option<Arc<str>>,
    },
    UnresolvedResourceHandle {
        resource_id: ResourceId,
        role: Arc<str>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectReferenceDiagnostic {
    document: AssetUri,
    phase: ProjectReferenceDiagnosticPhase,
    kind: ProjectReferenceDiagnosticKind,
}

impl ProjectReferenceDiagnostic {
    pub fn dangling(
        document: AssetUri,
        phase: ProjectReferenceDiagnosticPhase,
        uuid: AssetUuid,
        locator: ResourceLocator,
    ) -> Self {
        Self {
            document,
            phase,
            kind: ProjectReferenceDiagnosticKind::DanglingAssetReference { uuid, locator },
        }
    }

    pub fn unresolved_handle(
        document: AssetUri,
        phase: ProjectReferenceDiagnosticPhase,
        resource_id: ResourceId,
        role: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            document,
            phase,
            kind: ProjectReferenceDiagnosticKind::UnresolvedResourceHandle {
                resource_id,
                role: role.into(),
            },
        }
    }

    pub fn persisted_dangling(
        document: AssetUri,
        phase: ProjectReferenceDiagnosticPhase,
        uuid: AssetUuid,
        path_hint: impl Into<Arc<str>>,
        subasset: Option<impl Into<Arc<str>>>,
    ) -> Self {
        Self {
            document,
            phase,
            kind: ProjectReferenceDiagnosticKind::PersistedDanglingReference {
                uuid,
                path_hint: path_hint.into(),
                subasset: subasset.map(Into::into),
            },
        }
    }

    pub fn document(&self) -> &AssetUri {
        &self.document
    }

    pub const fn phase(&self) -> ProjectReferenceDiagnosticPhase {
        self.phase
    }

    pub const fn kind(&self) -> &ProjectReferenceDiagnosticKind {
        &self.kind
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectReferenceDiagnosticsSnapshot {
    sequence: u64,
    diagnostics: Arc<[ProjectReferenceDiagnostic]>,
}

impl ProjectReferenceDiagnosticsSnapshot {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn diagnostics(&self) -> &[ProjectReferenceDiagnostic] {
        &self.diagnostics
    }

    pub fn diagnostics_for_document<'a>(
        &'a self,
        document: &'a AssetUri,
    ) -> impl Iterator<Item = &'a ProjectReferenceDiagnostic> + 'a {
        self.diagnostics
            .iter()
            .filter(move |diagnostic| diagnostic.document() == document)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectReferenceDiagnosticsEvent {
    sequence: u64,
    document: AssetUri,
    phase: ProjectReferenceDiagnosticPhase,
    diagnostics: Arc<[ProjectReferenceDiagnostic]>,
}

impl ProjectReferenceDiagnosticsEvent {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn document(&self) -> &AssetUri {
        &self.document
    }

    pub const fn phase(&self) -> ProjectReferenceDiagnosticPhase {
        self.phase
    }

    pub fn diagnostics(&self) -> &[ProjectReferenceDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Default)]
pub(crate) struct ProjectReferenceDiagnosticsStore {
    state: Mutex<ProjectReferenceDiagnosticsState>,
}

#[derive(Debug, Default)]
struct ProjectReferenceDiagnosticsState {
    sequence: u64,
    by_document: BTreeMap<AssetUri, Arc<[ProjectReferenceDiagnostic]>>,
    latest_event: Option<ProjectReferenceDiagnosticsEvent>,
}

impl ProjectReferenceDiagnosticsStore {
    pub(crate) fn replace_document(
        &self,
        document: AssetUri,
        phase: ProjectReferenceDiagnosticPhase,
        diagnostics: Vec<ProjectReferenceDiagnostic>,
    ) -> ProjectReferenceDiagnosticsEvent {
        debug_assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.document() == &document && diagnostic.phase() == phase
        }));
        let diagnostics: Arc<[ProjectReferenceDiagnostic]> = diagnostics.into();
        let mut state = self.lock_state();
        state.sequence = state.sequence.saturating_add(1);
        if diagnostics.is_empty() {
            state.by_document.remove(&document);
        } else {
            state
                .by_document
                .insert(document.clone(), Arc::clone(&diagnostics));
        }
        let event = ProjectReferenceDiagnosticsEvent {
            sequence: state.sequence,
            document,
            phase,
            diagnostics,
        };
        state.latest_event = Some(event.clone());
        event
    }

    pub(crate) fn snapshot(&self) -> ProjectReferenceDiagnosticsSnapshot {
        let state = self.lock_state();
        let diagnostics = state
            .by_document
            .values()
            .flat_map(|diagnostics| diagnostics.iter().cloned())
            .collect::<Vec<_>>();
        ProjectReferenceDiagnosticsSnapshot {
            sequence: state.sequence,
            diagnostics: diagnostics.into(),
        }
    }

    pub(crate) fn latest_event(&self) -> Option<ProjectReferenceDiagnosticsEvent> {
        self.lock_state().latest_event.clone()
    }

    fn lock_state(&self) -> MutexGuard<'_, ProjectReferenceDiagnosticsState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uri(value: &str) -> AssetUri {
        AssetUri::parse(value).unwrap()
    }

    #[test]
    fn document_publication_replaces_stale_diagnostics_and_keeps_other_documents() {
        let store = ProjectReferenceDiagnosticsStore::default();
        let scene_a = uri("res://scenes/a.scene.toml");
        let scene_b = uri("res://scenes/b.scene.toml");
        let missing_a = AssetUuid::new();
        let missing_b = AssetUuid::new();
        store.replace_document(
            scene_a.clone(),
            ProjectReferenceDiagnosticPhase::Load,
            vec![ProjectReferenceDiagnostic::dangling(
                scene_a.clone(),
                ProjectReferenceDiagnosticPhase::Load,
                missing_a,
                uri("res://models/missing-a.glb"),
            )],
        );
        store.replace_document(
            scene_b.clone(),
            ProjectReferenceDiagnosticPhase::Load,
            vec![ProjectReferenceDiagnostic::dangling(
                scene_b.clone(),
                ProjectReferenceDiagnosticPhase::Load,
                missing_b,
                uri("res://models/missing-b.glb"),
            )],
        );

        let cleared = store.replace_document(
            scene_a.clone(),
            ProjectReferenceDiagnosticPhase::Save,
            Vec::new(),
        );
        let snapshot = store.snapshot();

        assert!(cleared.diagnostics().is_empty());
        assert_eq!(cleared.sequence(), 3);
        assert_eq!(snapshot.sequence(), 3);
        assert_eq!(snapshot.diagnostics_for_document(&scene_a).count(), 0);
        assert_eq!(snapshot.diagnostics_for_document(&scene_b).count(), 1);
    }

    #[test]
    fn latest_event_is_bounded_to_one_document_replacement() {
        let store = ProjectReferenceDiagnosticsStore::default();
        let scene = uri("res://scenes/main.scene.toml");
        let resource_id = ResourceId::new();

        store.replace_document(
            scene.clone(),
            ProjectReferenceDiagnosticPhase::Save,
            vec![ProjectReferenceDiagnostic::unresolved_handle(
                scene.clone(),
                ProjectReferenceDiagnosticPhase::Save,
                resource_id,
                "material",
            )],
        );

        let event = store.latest_event().unwrap();
        assert_eq!(event.document(), &scene);
        assert_eq!(event.phase(), ProjectReferenceDiagnosticPhase::Save);
        assert_eq!(event.diagnostics().len(), 1);
        assert!(matches!(
            event.diagnostics()[0].kind(),
            ProjectReferenceDiagnosticKind::UnresolvedResourceHandle {
                resource_id: observed,
                role,
            } if *observed == resource_id && role.as_ref() == "material"
        ));
    }
}

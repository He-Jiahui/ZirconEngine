use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Mutex, MutexGuard,
};

use serde::{Deserialize, Serialize};
use zircon_runtime::asset::project::ProjectPaths;

use crate::core::editor_message::{DocumentId, DocumentMessage};

mod retention_snapshot;

pub use retention_snapshot::DocumentLifecycleRetentionSnapshot;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DOCUMENT_ID_COLLISION_STEP: u64 = 0x9e37_79b9_7f4a_7c15;
const MAX_TRACKED_DOCUMENT_ROOTS: usize = 1_024;
static NEXT_PROJECT_SESSION: AtomicU64 = AtomicU64::new(1);

/// Identifies one active project generation for picker and document-route requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ProjectSessionId(u64);

impl ProjectSessionId {
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ProjectSessionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        if value == 0 {
            return Err(serde::de::Error::custom(
                "a project session identity must be non-zero",
            ));
        }
        Ok(Self(value))
    }
}

/// An opaque picker capability bound to the project session that initiated the request.
///
/// Picker results must return this ticket unchanged. A later project activation invalidates it
/// instead of reinterpreting its `res://` URI against the new project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePickerTicket {
    project_root: PathBuf,
    session: ProjectSessionId,
}

impl ScenePickerTicket {
    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(crate) fn session(&self) -> ProjectSessionId {
        self.session
    }
}

/// The project document transition and the session that owns subsequent scene requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectSessionActivation {
    pub session: ProjectSessionId,
    pub messages: Vec<DocumentMessage>,
}

/// The document transition produced by opening or creating a project-owned scene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneDocumentActivation {
    pub document: DocumentId,
    pub messages: Vec<DocumentMessage>,
    pub already_active: bool,
}

/// A lifecycle identity that has passed session validation but is not yet active.
///
/// Scene routes create this before replacing an authoring world. Holding the route gate prevents
/// another project/session transition until the route either installs the world and commits this
/// reservation or returns without publishing it.
#[derive(Debug)]
pub(super) struct SceneDocumentActivationReservation {
    session: ProjectSessionId,
    key: SceneDocumentKey,
    document: DocumentId,
    activation_revision: u64,
}

impl SceneDocumentActivationReservation {
    pub(super) const fn document(&self) -> DocumentId {
        self.document
    }
}

/// Immutable identity for the scene currently owned by a project session.
///
/// The lifecycle authority owns this value. Consumers use it to select an explicit scene
/// source for persistence instead of inferring one from a project manifest default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveSceneDocumentIdentity {
    document: DocumentId,
    project_root: PathBuf,
    scene_uri: String,
    activation_revision: u64,
}

impl ActiveSceneDocumentIdentity {
    pub const fn document(&self) -> DocumentId {
        self.document
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn scene_uri(&self) -> &str {
        &self.scene_uri
    }

    pub const fn activation_revision(&self) -> u64 {
        self.activation_revision
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneDocumentLifecycleError {
    NoActiveProjectSession {
        project_root: PathBuf,
    },
    StaleProjectSession {
        project_root: PathBuf,
        received: ProjectSessionId,
        active: Option<ProjectSessionId>,
    },
    SceneActivationRevisionExhausted,
}

impl fmt::Display for SceneDocumentLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveProjectSession { project_root } => write!(
                formatter,
                "scene document request requires an active project session for {}",
                ProjectPaths::display_path(project_root).display()
            ),
            Self::StaleProjectSession {
                project_root,
                received,
                active,
            } => write!(
                formatter,
                "scene document request for project {} used stale session {:?}; active session is {:?}",
                ProjectPaths::display_path(project_root).display(),
                received,
                active
            ),
            Self::SceneActivationRevisionExhausted => {
                formatter.write_str("scene document activation revision is exhausted")
            }
        }
    }
}

impl std::error::Error for SceneDocumentLifecycleError {}

/// Identifies whether a scene activation failed while reserving lifecycle state or while the
/// caller bound a reservation-owned service before publication.
#[derive(Debug)]
pub enum SceneDocumentActivationBindingError<BindingError> {
    Lifecycle(SceneDocumentLifecycleError),
    Binding(BindingError),
}

impl<BindingError: fmt::Display> fmt::Display
    for SceneDocumentActivationBindingError<BindingError>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifecycle(error) => error.fmt(formatter),
            Self::Binding(error) => write!(formatter, "scene document binding failed: {error}"),
        }
    }
}

impl<BindingError: std::error::Error + 'static> std::error::Error
    for SceneDocumentActivationBindingError<BindingError>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lifecycle(error) => Some(error),
            Self::Binding(error) => Some(error),
        }
    }
}

/// Owns document identity and structural lifecycle transitions for one editor manager.
///
/// Callers receive facts only after this authority releases its state lock, so bus observers
/// cannot re-enter while a document transition is still being committed.
#[derive(Default)]
pub struct DocumentLifecycleAuthority {
    state: Mutex<DocumentLifecycleState>,
    scene_route_gate: Mutex<()>,
}

#[derive(Default)]
struct DocumentLifecycleState {
    active_document: Option<DocumentId>,
    ids_by_root: BTreeMap<PathBuf, DocumentId>,
    active_scene_key: Option<SceneDocumentKey>,
    active_scene_activation_revision: Option<u64>,
    next_scene_activation_revision: u64,
    ids_by_scene_key: BTreeMap<SceneDocumentKey, DocumentId>,
    active_project_session: Option<ActiveProjectSession>,
    probe_counters: retention_snapshot::DocumentLifecycleProbeCounters,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SceneDocumentKey {
    project_root: PathBuf,
    scene_uri: String,
}

#[derive(Clone, Debug)]
struct ActiveProjectSession {
    id: ProjectSessionId,
    root: PathBuf,
}

impl DocumentLifecycleAuthority {
    pub fn activate(&self, root: &Path) -> Vec<DocumentMessage> {
        let _route_guard = self.lock_scene_route_gate();
        let mut state = self.lock_state();
        state.active_project_session = None;
        activate_project_document(&mut state, root)
    }

    /// Starts a new project session before any scene picker result may be accepted.
    pub fn begin_project_session(&self, root: &Path) -> ProjectSessionActivation {
        let _route_guard = self.lock_scene_route_gate();
        let mut state = self.lock_state();
        let session = next_project_session_id();
        state.active_project_session = Some(ActiveProjectSession {
            id: session,
            root: root.to_path_buf(),
        });
        ProjectSessionActivation {
            session,
            messages: activate_project_document(&mut state, root),
        }
    }

    /// Test-only shorthand for lifecycle tests that do not install a project-bound service.
    #[cfg(test)]
    pub(crate) fn activate_scene(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
    ) -> Result<SceneDocumentActivation, SceneDocumentLifecycleError> {
        let _route_guard = self.lock_scene_route_gate();
        let reservation =
            self.prepare_scene_activation_while_routed(session, project_root, scene_uri)?;
        Ok(self.commit_scene_activation_while_routed(reservation))
    }

    /// Runs a side-effect-free document binding after identity reservation and before lifecycle
    /// publication. A failed binding leaves the active scene unchanged.
    pub fn activate_scene_with_binding<BindingError>(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
        bind: impl FnOnce(DocumentId) -> Result<(), BindingError>,
    ) -> Result<SceneDocumentActivation, SceneDocumentActivationBindingError<BindingError>> {
        let _route_guard = self.lock_scene_route_gate();
        let reservation = self
            .prepare_scene_activation_while_routed(session, project_root, scene_uri)
            .map_err(SceneDocumentActivationBindingError::Lifecycle)?;
        bind(reservation.document()).map_err(SceneDocumentActivationBindingError::Binding)?;
        Ok(self.commit_scene_activation_while_routed(reservation))
    }

    /// Reserves the target scene identity after validating its project session.
    ///
    /// This intentionally leaves `active_document` and `active_scene_key` unchanged. The route
    /// commits only after its installer has replaced the authoring world successfully.
    pub(super) fn prepare_scene_activation_while_routed(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
    ) -> Result<SceneDocumentActivationReservation, SceneDocumentLifecycleError> {
        let mut state = self.lock_state();
        validate_scene_session(&state, session, project_root)?;
        let key = SceneDocumentKey {
            project_root: project_root.to_path_buf(),
            scene_uri: scene_uri.to_string(),
        };
        let document = state
            .ids_by_scene_key
            .get(&key)
            .copied()
            .unwrap_or_else(|| scene_document_id_for(&mut state, &key));
        let activation_revision = state
            .next_scene_activation_revision
            .checked_add(1)
            .ok_or(SceneDocumentLifecycleError::SceneActivationRevisionExhausted)?;
        state.next_scene_activation_revision = activation_revision;
        Ok(SceneDocumentActivationReservation {
            session,
            key,
            document,
            activation_revision,
        })
    }

    /// Publishes a prevalidated scene identity while the scene-route gate remains held.
    ///
    /// `SceneDocumentActivationReservation` can only be created by the matching prepare method,
    /// and route serialization forbids another session transition between preparation and commit.
    /// This keeps the post-install step infallible: callers never receive an activation error
    /// after the authoring world has already changed.
    pub(super) fn commit_scene_activation_while_routed(
        &self,
        reservation: SceneDocumentActivationReservation,
    ) -> SceneDocumentActivation {
        let mut state = self.lock_state();
        debug_assert!(state
            .active_project_session
            .as_ref()
            .is_some_and(|session| {
                session.id == reservation.session && session.root == reservation.key.project_root
            }));
        if state.active_document == Some(reservation.document)
            && state.active_scene_key.as_ref() == Some(&reservation.key)
        {
            return SceneDocumentActivation {
                document: reservation.document,
                messages: Vec::new(),
                already_active: true,
            };
        }

        let previous_document = state.active_document.replace(reservation.document);
        state.active_scene_key = Some(reservation.key);
        state.active_scene_activation_revision = Some(reservation.activation_revision);
        state.trim_closed_roots();
        state.trim_closed_scene_documents();
        let mut messages = Vec::with_capacity(2);
        if let Some(previous_document) = previous_document {
            messages.push(DocumentMessage::Closed {
                doc: previous_document,
            });
        }
        messages.push(DocumentMessage::Opened {
            doc: reservation.document,
        });
        SceneDocumentActivation {
            document: reservation.document,
            messages,
            already_active: false,
        }
    }

    pub fn close(&self, root: &Path) -> Option<DocumentMessage> {
        let _route_guard = self.lock_scene_route_gate();
        let mut state = self.lock_state();
        let document_id = state.ids_by_root.get(root).copied()?;
        if state.active_document != Some(document_id) {
            return None;
        }

        state.active_document = None;
        state.active_scene_key = None;
        state.active_scene_activation_revision = None;
        if state
            .active_project_session
            .as_ref()
            .is_some_and(|session| session.root == root)
        {
            state.active_project_session = None;
        }
        Some(DocumentMessage::Closed { doc: document_id })
    }

    /// Closes whichever project or scene document is active for the current project session.
    pub fn end_project_session(&self, root: &Path) -> Vec<DocumentMessage> {
        let _route_guard = self.lock_scene_route_gate();
        let mut state = self.lock_state();
        let Some(session) = state.active_project_session.as_ref() else {
            return Vec::new();
        };
        if session.root != root {
            return Vec::new();
        }

        state.active_project_session = None;
        state.active_scene_key = None;
        state.active_scene_activation_revision = None;
        state
            .active_document
            .take()
            .map(|document| vec![DocumentMessage::Closed { doc: document }])
            .unwrap_or_default()
    }

    pub fn project_session(&self, root: &Path) -> Option<ProjectSessionId> {
        self.lock_state()
            .active_project_session
            .as_ref()
            .filter(|session| session.root == root)
            .map(|session| session.id)
    }

    /// Issues the capability that a scene picker must return with its selected asset URI.
    pub fn issue_scene_picker_ticket(
        &self,
        root: &Path,
    ) -> Result<ScenePickerTicket, SceneDocumentLifecycleError> {
        let _route_guard = self.lock_scene_route_gate();
        let state = self.lock_state();
        let session = state
            .active_project_session
            .as_ref()
            .filter(|session| session.root == root)
            .map(|session| session.id)
            .ok_or_else(|| SceneDocumentLifecycleError::NoActiveProjectSession {
                project_root: root.to_path_buf(),
            })?;
        Ok(ScenePickerTicket {
            project_root: root.to_path_buf(),
            session,
        })
    }

    /// Returns the selected source identity for the active scene in `root`.
    ///
    /// A project session without a scene has no save target. Callers must surface that condition
    /// instead of falling back to `ProjectManifest::default_scene`.
    pub fn active_scene_identity(&self, root: &Path) -> Option<ActiveSceneDocumentIdentity> {
        let _route_guard = self.lock_scene_route_gate();
        self.active_scene_identity_while_routed(root)
    }

    /// Returns the scene selected by the single active project session.
    pub fn active_scene_identity_for_session(&self) -> Option<ActiveSceneDocumentIdentity> {
        let _route_guard = self.lock_scene_route_gate();
        let state = self.lock_state();
        active_scene_identity_from_state(&state, None)
    }

    pub(super) fn active_scene_identity_while_routed(
        &self,
        root: &Path,
    ) -> Option<ActiveSceneDocumentIdentity> {
        let state = self.lock_state();
        active_scene_identity_from_state(&state, Some(root))
    }

    /// Confirms a save only when the captured scene identity is still the active one.
    ///
    /// Persistence runs outside the lifecycle lock. This guards the post-write document event
    /// against a later scene-route commit incorrectly marking a different document as saved.
    pub fn save_scene_identity_if_active(
        &self,
        identity: &ActiveSceneDocumentIdentity,
    ) -> Option<DocumentMessage> {
        let state = self.lock_state();
        let session = state.active_project_session.as_ref()?;
        let active_scene = state.active_scene_key.as_ref()?;
        (session.root == identity.project_root
            && state.active_document == Some(identity.document)
            && active_scene.project_root == identity.project_root
            && active_scene.scene_uri == identity.scene_uri
            && state.active_scene_activation_revision == Some(identity.activation_revision))
        .then_some(DocumentMessage::Saved {
            doc: identity.document,
        })
    }

    pub fn validate_project_session(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
    ) -> Result<(), SceneDocumentLifecycleError> {
        let _route_guard = self.lock_scene_route_gate();
        self.validate_project_session_while_routed(session, project_root)
    }

    pub(super) fn validate_project_session_while_routed(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
    ) -> Result<(), SceneDocumentLifecycleError> {
        let state = self.lock_state();
        validate_scene_session(&state, session, project_root)
    }

    pub(super) fn validate_scene_picker_ticket_while_routed(
        &self,
        ticket: &ScenePickerTicket,
        project_root: &Path,
    ) -> Result<(), SceneDocumentLifecycleError> {
        if ticket.project_root() != project_root {
            let state = self.lock_state();
            return Err(stale_scene_session_error(
                &state,
                ticket.session(),
                ticket.project_root(),
            ));
        }
        self.validate_project_session_while_routed(ticket.session(), ticket.project_root())
    }

    pub fn active_scene_document(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
    ) -> Result<Option<DocumentId>, SceneDocumentLifecycleError> {
        let _route_guard = self.lock_scene_route_gate();
        self.active_scene_document_while_routed(session, project_root, scene_uri)
    }

    pub(super) fn active_scene_document_while_routed(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
    ) -> Result<Option<DocumentId>, SceneDocumentLifecycleError> {
        let state = self.lock_state();
        validate_scene_session(&state, session, project_root)?;
        let key = SceneDocumentKey {
            project_root: project_root.to_path_buf(),
            scene_uri: scene_uri.to_string(),
        };
        Ok((state.active_scene_key.as_ref() == Some(&key))
            .then(|| state.ids_by_scene_key.get(&key).copied())
            .flatten())
    }

    pub fn save(&self, root: &Path) -> Option<DocumentMessage> {
        let state = self.lock_state();
        let document_id = state.ids_by_root.get(root).copied()?;
        (state.active_document == Some(document_id))
            .then_some(DocumentMessage::Saved { doc: document_id })
    }

    pub fn document_id(&self, root: &Path) -> Option<DocumentId> {
        self.lock_state().ids_by_root.get(root).copied()
    }

    /// Captures lifecycle retention and probe totals without creating another path owner.
    ///
    /// This diagnostic snapshot is intended for controlled performance captures. It does not
    /// reset counters or participate in lifecycle routing decisions.
    pub fn retention_snapshot(&self) -> DocumentLifecycleRetentionSnapshot {
        let state = self.lock_state();
        DocumentLifecycleRetentionSnapshot::from_state(&state)
    }

    fn lock_state(&self) -> MutexGuard<'_, DocumentLifecycleState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn with_scene_route<R>(&self, route: impl FnOnce() -> R) -> R {
        let _route_guard = self.lock_scene_route_gate();
        route()
    }

    fn lock_scene_route_gate(&self) -> MutexGuard<'_, ()> {
        self.scene_route_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn active_scene_identity_from_state(
    state: &DocumentLifecycleState,
    expected_root: Option<&Path>,
) -> Option<ActiveSceneDocumentIdentity> {
    let session = state.active_project_session.as_ref()?;
    if expected_root.is_some_and(|root| session.root != root) {
        return None;
    }
    let scene = state.active_scene_key.as_ref()?;
    if scene.project_root != session.root {
        return None;
    }
    Some(ActiveSceneDocumentIdentity {
        document: state.active_document?,
        project_root: scene.project_root.clone(),
        scene_uri: scene.scene_uri.clone(),
        activation_revision: state.active_scene_activation_revision?,
    })
}

fn next_project_session_id() -> ProjectSessionId {
    loop {
        let candidate = NEXT_PROJECT_SESSION.fetch_add(1, Ordering::Relaxed);
        if candidate != 0 {
            return ProjectSessionId(candidate);
        }
    }
}

impl DocumentLifecycleState {
    fn trim_closed_roots(&mut self) {
        while self.ids_by_root.len() > MAX_TRACKED_DOCUMENT_ROOTS {
            let mut scanned_entries = 0;
            let root = self.ids_by_root.iter().find_map(|(root, document_id)| {
                scanned_entries += 1;
                (Some(*document_id) != self.active_document).then(|| root.clone())
            });
            self.probe_counters
                .record_root_eviction_scan(scanned_entries);
            let Some(root) = root else {
                break;
            };
            self.ids_by_root.remove(&root);
            self.probe_counters.record_root_eviction();
        }
    }

    fn trim_closed_scene_documents(&mut self) {
        while self.ids_by_scene_key.len() > MAX_TRACKED_DOCUMENT_ROOTS {
            let mut scanned_entries = 0;
            let key = self.ids_by_scene_key.iter().find_map(|(key, document_id)| {
                scanned_entries += 1;
                (Some(*document_id) != self.active_document).then(|| key.clone())
            });
            self.probe_counters
                .record_scene_eviction_scan(scanned_entries);
            let Some(key) = key else {
                break;
            };
            self.ids_by_scene_key.remove(&key);
            self.probe_counters.record_scene_eviction();
        }
    }
}

fn activate_project_document(
    state: &mut DocumentLifecycleState,
    root: &Path,
) -> Vec<DocumentMessage> {
    let document_id = state
        .ids_by_root
        .get(root)
        .copied()
        .unwrap_or_else(|| document_id_for(state, root));
    if state.active_document == Some(document_id) && state.active_scene_key.is_none() {
        return Vec::new();
    }

    let previous_document = state.active_document.replace(document_id);
    state.active_scene_key = None;
    state.active_scene_activation_revision = None;
    state.trim_closed_roots();
    state.trim_closed_scene_documents();
    let mut messages = Vec::with_capacity(2);
    if let Some(previous_document) = previous_document {
        messages.push(DocumentMessage::Closed {
            doc: previous_document,
        });
    }
    messages.push(DocumentMessage::Opened { doc: document_id });
    messages
}

fn validate_scene_session(
    state: &DocumentLifecycleState,
    session: ProjectSessionId,
    project_root: &Path,
) -> Result<(), SceneDocumentLifecycleError> {
    let active = state.active_project_session.as_ref();
    if active.is_some_and(|active| active.id == session && active.root == project_root) {
        return Ok(());
    }
    Err(stale_scene_session_error(state, session, project_root))
}

fn stale_scene_session_error(
    state: &DocumentLifecycleState,
    received: ProjectSessionId,
    project_root: &Path,
) -> SceneDocumentLifecycleError {
    SceneDocumentLifecycleError::StaleProjectSession {
        project_root: project_root.to_path_buf(),
        received,
        active: state
            .active_project_session
            .as_ref()
            .map(|session| session.id),
    }
}

fn document_id_for(state: &mut DocumentLifecycleState, root: &Path) -> DocumentId {
    if let Some(document_id) = state.ids_by_root.get(root) {
        return *document_id;
    }

    let mut document_id = stable_document_id(root);
    loop {
        state.probe_counters.record_document_id_occupancy_probe();
        if !document_id_is_occupied(state, document_id) {
            break;
        }
        document_id = DocumentId::new(document_id.value().wrapping_add(DOCUMENT_ID_COLLISION_STEP));
    }
    state.ids_by_root.insert(root.to_path_buf(), document_id);
    document_id
}

fn scene_document_id_for(state: &mut DocumentLifecycleState, key: &SceneDocumentKey) -> DocumentId {
    if let Some(document_id) = state.ids_by_scene_key.get(key) {
        return *document_id;
    }

    let identity = format!("scene:{}:{}", key.project_root.display(), key.scene_uri);
    let mut document_id = stable_document_id(Path::new(&identity));
    loop {
        state.probe_counters.record_document_id_occupancy_probe();
        if !document_id_is_occupied(state, document_id) {
            break;
        }
        document_id = DocumentId::new(document_id.value().wrapping_add(DOCUMENT_ID_COLLISION_STEP));
    }
    state.ids_by_scene_key.insert(key.clone(), document_id);
    document_id
}

fn document_id_is_occupied(state: &DocumentLifecycleState, candidate: DocumentId) -> bool {
    state
        .ids_by_root
        .values()
        .any(|document| *document == candidate)
        || state
            .ids_by_scene_key
            .values()
            .any(|document| *document == candidate)
}

fn stable_document_id(root: &Path) -> DocumentId {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in root.to_string_lossy().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    DocumentId::new(hash)
}

#[cfg(test)]
mod tests;

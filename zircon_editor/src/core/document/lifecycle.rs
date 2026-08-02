use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicU64, Ordering},
};

use crate::core::editor_message::{DocumentId, DocumentMessage};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DOCUMENT_ID_COLLISION_STEP: u64 = 0x9e37_79b9_7f4a_7c15;
const MAX_TRACKED_DOCUMENT_ROOTS: usize = 1_024;
static NEXT_PROJECT_SESSION: AtomicU64 = AtomicU64::new(1);

/// Identifies one active project generation for picker and document-route requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectSessionId(u64);

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
}

impl fmt::Display for SceneDocumentLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveProjectSession { project_root } => write!(
                formatter,
                "scene document request requires an active project session for {}",
                project_root.display()
            ),
            Self::StaleProjectSession {
                project_root,
                received,
                active,
            } => write!(
                formatter,
                "scene document request for project {} used stale session {:?}; active session is {:?}",
                project_root.display(),
                received,
                active
            ),
        }
    }
}

impl std::error::Error for SceneDocumentLifecycleError {}

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
    ids_by_scene_key: BTreeMap<SceneDocumentKey, DocumentId>,
    active_project_session: Option<ActiveProjectSession>,
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

    /// Activates a distinct scene document only when the project session still owns the project.
    pub fn activate_scene(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
    ) -> Result<SceneDocumentActivation, SceneDocumentLifecycleError> {
        let _route_guard = self.lock_scene_route_gate();
        self.activate_scene_while_routed(session, project_root, scene_uri)
    }

    pub(super) fn activate_scene_while_routed(
        &self,
        session: ProjectSessionId,
        project_root: &Path,
        scene_uri: &str,
    ) -> Result<SceneDocumentActivation, SceneDocumentLifecycleError> {
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
        if state.active_document == Some(document) && state.active_scene_key.as_ref() == Some(&key)
        {
            return Ok(SceneDocumentActivation {
                document,
                messages: Vec::new(),
                already_active: true,
            });
        }

        let previous_document = state.active_document.replace(document);
        state.active_scene_key = Some(key);
        state.trim_closed_roots();
        state.trim_closed_scene_documents();
        let mut messages = Vec::with_capacity(2);
        if let Some(previous_document) = previous_document {
            messages.push(DocumentMessage::Closed {
                doc: previous_document,
            });
        }
        messages.push(DocumentMessage::Opened { doc: document });
        Ok(SceneDocumentActivation {
            document,
            messages,
            already_active: false,
        })
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

    pub fn save_active_project_session(&self, root: &Path) -> Option<DocumentMessage> {
        let state = self.lock_state();
        let session = state.active_project_session.as_ref()?;
        if session.root != root {
            return None;
        }
        state
            .active_document
            .map(|document| DocumentMessage::Saved { doc: document })
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

    #[cfg(test)]
    fn retention_metrics(&self) -> DocumentLifecycleRetentionMetrics {
        let state = self.lock_state();
        DocumentLifecycleRetentionMetrics {
            tracked_root_count: state.ids_by_root.len(),
            root_body_owner_count: state.ids_by_root.len(),
            active_document_count: usize::from(state.active_document.is_some()),
            active_document_id: state.active_document,
        }
    }
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
            let Some(root) = self
                .ids_by_root
                .iter()
                .find(|(_, document_id)| Some(**document_id) != self.active_document)
                .map(|(root, _)| root.clone())
            else {
                break;
            };
            self.ids_by_root.remove(&root);
        }
    }

    fn trim_closed_scene_documents(&mut self) {
        while self.ids_by_scene_key.len() > MAX_TRACKED_DOCUMENT_ROOTS {
            let Some(key) = self
                .ids_by_scene_key
                .iter()
                .find(|(_, document_id)| Some(**document_id) != self.active_document)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.ids_by_scene_key.remove(&key);
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

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DocumentLifecycleRetentionMetrics {
    tracked_root_count: usize,
    root_body_owner_count: usize,
    active_document_count: usize,
    active_document_id: Option<DocumentId>,
}

#[cfg(test)]
impl DocumentLifecycleRetentionMetrics {
    fn without_active_document(mut self) -> Self {
        self.active_document_count = 0;
        self.active_document_id = None;
        self
    }
}

fn document_id_for(state: &mut DocumentLifecycleState, root: &Path) -> DocumentId {
    if let Some(document_id) = state.ids_by_root.get(root) {
        return *document_id;
    }

    let mut document_id = stable_document_id(root);
    while document_id_is_occupied(state, document_id) {
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
    while document_id_is_occupied(state, document_id) {
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
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    use crate::core::editor_message::{DocumentId, DocumentMessage};

    use super::{
        DOCUMENT_ID_COLLISION_STEP, DocumentLifecycleAuthority, DocumentLifecycleState,
        SceneDocumentLifecycleError, stable_document_id,
    };

    #[test]
    fn activation_emits_open_once_and_keeps_the_document_id_for_a_noop() {
        let authority = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/forest");

        let opened = authority.activate(root);
        let document_id = match opened.as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected one opened message, got {actual:?}"),
        };

        assert_eq!(authority.activate(root), Vec::new());
        assert_eq!(authority.document_id(root), Some(document_id));
    }

    #[test]
    fn switching_documents_closes_the_previous_before_opening_the_next() {
        let authority = DocumentLifecycleAuthority::default();
        let forest = Path::new("C:/projects/forest");
        let desert = Path::new("C:/projects/desert");

        let forest_id = match authority.activate(forest).as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected forest to open, got {actual:?}"),
        };
        let transition = authority.activate(desert);
        let desert_id = match transition.as_slice() {
            [
                DocumentMessage::Closed { doc: closed },
                DocumentMessage::Opened { doc: opened },
            ] => {
                assert_eq!(*closed, forest_id);
                *opened
            }
            actual => panic!("expected close then open, got {actual:?}"),
        };

        assert_eq!(
            authority.close(desert),
            Some(DocumentMessage::Closed { doc: desert_id })
        );
        assert_eq!(
            authority.activate(forest),
            vec![DocumentMessage::Opened { doc: forest_id }]
        );
    }

    #[test]
    fn save_emits_only_for_the_active_document() {
        let authority = DocumentLifecycleAuthority::default();
        let forest = Path::new("C:/projects/forest");
        let desert = Path::new("C:/projects/desert");

        assert_eq!(authority.save(forest), None);
        let document_id = match authority.activate(forest).as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected forest to open, got {actual:?}"),
        };

        assert_eq!(authority.save(desert), None);
        assert_eq!(
            authority.save(forest),
            Some(DocumentMessage::Saved { doc: document_id })
        );
    }

    #[test]
    fn close_requires_the_active_root_and_reopening_reuses_the_document_id() {
        let authority = DocumentLifecycleAuthority::default();
        let forest = Path::new("C:/projects/forest");
        let desert = Path::new("C:/projects/desert");

        let document_id = match authority.activate(forest).as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected forest to open, got {actual:?}"),
        };

        assert_eq!(authority.close(desert), None);
        assert_eq!(
            authority.close(forest),
            Some(DocumentMessage::Closed { doc: document_id })
        );
        assert_eq!(authority.close(forest), None);
        assert_eq!(
            authority.activate(forest),
            vec![DocumentMessage::Opened { doc: document_id }]
        );
    }

    #[test]
    fn lifecycle_bounds_closed_root_owners_without_changing_stable_document_ids() {
        let authority = DocumentLifecycleAuthority::default();
        let original_root = Path::new("C:/projects/original");
        let original_id = match authority.activate(original_root).as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected original root to open, got {actual:?}"),
        };
        assert_eq!(
            authority.close(original_root),
            Some(DocumentMessage::Closed { doc: original_id })
        );

        for index in 0..(super::MAX_TRACKED_DOCUMENT_ROOTS + 2) {
            let root = PathBuf::from(format!("C:/projects/bounded/{index:04}"));
            let _ = authority.activate(&root);
        }

        let metrics = authority.retention_metrics();
        assert!(metrics.tracked_root_count <= super::MAX_TRACKED_DOCUMENT_ROOTS);
        assert_eq!(metrics.root_body_owner_count, metrics.tracked_root_count);
        assert_eq!(metrics.active_document_count, 1);
        assert_eq!(
            authority.activate(original_root),
            vec![
                DocumentMessage::Closed {
                    doc: metrics.active_document_id.unwrap()
                },
                DocumentMessage::Opened { doc: original_id }
            ]
        );
    }

    #[test]
    fn lifecycle_rederives_an_evicted_root_identity_after_100k_closed_roots() {
        let authority = DocumentLifecycleAuthority::default();
        let evicted_root = Path::new("C:/projects/0000-evicted");
        let document_id = match authority.activate(evicted_root).as_slice() {
            [DocumentMessage::Opened { doc }] => *doc,
            actual => panic!("expected evicted root to open, got {actual:?}"),
        };
        assert_eq!(
            authority.close(evicted_root),
            Some(DocumentMessage::Closed { doc: document_id })
        );

        for index in 0..100_000 {
            let root = PathBuf::from(format!("C:/projects/1000-churn/{index:06}"));
            let _ = authority.activate(&root);
        }

        assert_eq!(authority.document_id(evicted_root), None);
        let before_reopen = authority.retention_metrics();
        assert!(before_reopen.tracked_root_count <= super::MAX_TRACKED_DOCUMENT_ROOTS);
        let previous_document = before_reopen.active_document_id.unwrap();
        assert_eq!(
            authority.activate(evicted_root),
            vec![
                DocumentMessage::Closed {
                    doc: previous_document
                },
                DocumentMessage::Opened { doc: document_id }
            ]
        );
        assert_eq!(
            authority.retention_metrics().active_document_id,
            Some(document_id)
        );
    }

    #[test]
    fn known_root_queries_do_not_create_another_root_owner() {
        let authority = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/borrowed-query");
        let _ = authority.activate(root);
        let before = authority.retention_metrics();

        assert_eq!(authority.activate(root), Vec::new());
        assert!(authority.save(root).is_some());
        assert!(authority.close(root).is_some());
        assert_eq!(authority.close(root), None);

        assert_eq!(
            authority.retention_metrics(),
            before.without_active_document()
        );
    }

    #[test]
    fn collision_stepped_document_id_reopens_with_the_same_identity_and_order() {
        let candidate = Path::new("C:/projects/collision-candidate");
        let occupied = PathBuf::from("C:/projects/occupied-collision-id");
        let base_id = stable_document_id(candidate);
        let collision_id =
            DocumentId::new(base_id.value().wrapping_add(DOCUMENT_ID_COLLISION_STEP));
        let authority = DocumentLifecycleAuthority {
            state: Mutex::new(DocumentLifecycleState {
                active_document: None,
                ids_by_root: BTreeMap::from([(occupied, base_id)]),
                ..Default::default()
            }),
            scene_route_gate: Mutex::new(()),
        };

        assert_eq!(
            authority.activate(candidate),
            vec![DocumentMessage::Opened { doc: collision_id }]
        );
        let other = Path::new("C:/projects/collision-other");
        let other_id = match authority.activate(other).as_slice() {
            [
                DocumentMessage::Closed { doc },
                DocumentMessage::Opened { doc: opened },
            ] => {
                assert_eq!(*doc, collision_id);
                *opened
            }
            actual => {
                panic!("expected collision candidate to close before other opens, got {actual:?}")
            }
        };
        assert_eq!(
            authority.activate(candidate),
            vec![
                DocumentMessage::Closed { doc: other_id },
                DocumentMessage::Opened { doc: collision_id }
            ]
        );
        assert_eq!(
            authority.close(candidate),
            Some(DocumentMessage::Closed { doc: collision_id })
        );
        assert_eq!(
            authority.activate(candidate),
            vec![DocumentMessage::Opened { doc: collision_id }]
        );
    }

    #[test]
    fn scene_sessions_distinguish_scene_documents_and_reject_stale_picker_results() {
        let authority = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/scene-route");
        let first_session = authority.begin_project_session(root).session;

        let first = authority
            .activate_scene(first_session, root, "res://scenes/first.scene.toml")
            .unwrap();
        assert!(!first.already_active);
        assert_eq!(first.messages.len(), 2);

        let repeated = authority
            .activate_scene(first_session, root, "res://scenes/first.scene.toml")
            .unwrap();
        assert!(repeated.already_active);
        assert_eq!(repeated.document, first.document);
        assert!(repeated.messages.is_empty());

        let second = authority
            .activate_scene(first_session, root, "res://scenes/second.scene.toml")
            .unwrap();
        assert!(!second.already_active);
        assert_ne!(second.document, first.document);
        assert_eq!(
            second.messages,
            vec![
                DocumentMessage::Closed {
                    doc: first.document
                },
                DocumentMessage::Opened {
                    doc: second.document
                },
            ]
        );

        let second_session = authority.begin_project_session(root).session;
        assert_ne!(second_session, first_session);
        assert!(matches!(
            authority.activate_scene(first_session, root, "res://scenes/first.scene.toml"),
            Err(SceneDocumentLifecycleError::StaleProjectSession { .. })
        ));
    }

    #[test]
    fn scene_picker_ticket_cannot_cross_into_a_new_project_session() {
        let authority = DocumentLifecycleAuthority::default();
        let first_root = Path::new("C:/projects/first-project");
        let second_root = Path::new("C:/projects/second-project");
        let first_session = authority.begin_project_session(first_root).session;
        let ticket = authority.issue_scene_picker_ticket(first_root).unwrap();
        let second_session = authority.begin_project_session(second_root).session;

        let error = authority.with_scene_route(|| {
            authority.validate_scene_picker_ticket_while_routed(&ticket, second_root)
        });

        assert!(matches!(
            error,
            Err(SceneDocumentLifecycleError::StaleProjectSession {
                project_root,
                received,
                active: Some(active),
            }) if project_root == first_root && received == first_session && active == second_session
        ));
    }

    #[test]
    fn scene_picker_ticket_cannot_cross_lifecycle_authorities_for_the_same_project() {
        let first_authority = DocumentLifecycleAuthority::default();
        let second_authority = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/shared-project");
        let first_session = first_authority.begin_project_session(root).session;
        let ticket = first_authority.issue_scene_picker_ticket(root).unwrap();
        let second_session = second_authority.begin_project_session(root).session;

        assert_ne!(first_session, second_session);
        let error = second_authority.with_scene_route(|| {
            second_authority.validate_scene_picker_ticket_while_routed(&ticket, root)
        });

        assert!(matches!(
            error,
            Err(SceneDocumentLifecycleError::StaleProjectSession {
                project_root,
                received,
                active: Some(active),
            }) if project_root == root && received == first_session && active == second_session
        ));
    }

    #[test]
    fn project_session_saves_and_closes_the_active_scene_document() {
        let authority = DocumentLifecycleAuthority::default();
        let root = Path::new("C:/projects/scene-session-close");
        let session = authority.begin_project_session(root).session;
        let scene = authority
            .activate_scene(session, root, "res://scenes/main.scene.toml")
            .unwrap();

        assert_eq!(
            authority.save_active_project_session(root),
            Some(DocumentMessage::Saved {
                doc: scene.document
            })
        );
        assert_eq!(
            authority.end_project_session(root),
            vec![DocumentMessage::Closed {
                doc: scene.document
            }]
        );
        assert_eq!(authority.project_session(root), None);
    }
}

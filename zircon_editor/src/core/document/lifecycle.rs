use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::core::editor_message::{DocumentId, DocumentMessage};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DOCUMENT_ID_COLLISION_STEP: u64 = 0x9e37_79b9_7f4a_7c15;
const MAX_TRACKED_DOCUMENT_ROOTS: usize = 1_024;

/// Owns document identity and structural lifecycle transitions for one editor manager.
///
/// Callers receive facts only after this authority releases its state lock, so bus observers
/// cannot re-enter while a document transition is still being committed.
#[derive(Default)]
pub struct DocumentLifecycleAuthority {
    state: Mutex<DocumentLifecycleState>,
}

#[derive(Default)]
struct DocumentLifecycleState {
    active_document: Option<DocumentId>,
    ids_by_root: BTreeMap<PathBuf, DocumentId>,
}

impl DocumentLifecycleAuthority {
    pub fn activate(&self, root: &Path) -> Vec<DocumentMessage> {
        let mut state = self.lock_state();
        let document_id = state
            .ids_by_root
            .get(root)
            .copied()
            .unwrap_or_else(|| document_id_for(&mut state, root));
        if state.active_document == Some(document_id) {
            return Vec::new();
        }

        let previous_document = state.active_document.replace(document_id);
        state.trim_closed_roots();
        let mut messages = Vec::with_capacity(2);
        if let Some(previous_document) = previous_document {
            messages.push(DocumentMessage::Closed {
                doc: previous_document,
            });
        }
        messages.push(DocumentMessage::Opened { doc: document_id });
        messages
    }

    pub fn close(&self, root: &Path) -> Option<DocumentMessage> {
        let mut state = self.lock_state();
        let document_id = state.ids_by_root.get(root).copied()?;
        if state.active_document != Some(document_id) {
            return None;
        }

        state.active_document = None;
        Some(DocumentMessage::Closed { doc: document_id })
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
    while state
        .ids_by_root
        .iter()
        .any(|(occupied_root, occupied_id)| {
            *occupied_id == document_id && occupied_root.as_path() != root
        })
    {
        document_id = DocumentId::new(document_id.value().wrapping_add(DOCUMENT_ID_COLLISION_STEP));
    }
    state.ids_by_root.insert(root.to_path_buf(), document_id);
    document_id
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
        stable_document_id,
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
            }),
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
}

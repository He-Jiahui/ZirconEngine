use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::core::editor_message::{DocumentId, DocumentMessage};

use super::{
    stable_document_id, DocumentLifecycleAuthority, DocumentLifecycleState,
    SceneDocumentActivationBindingError, SceneDocumentLifecycleError, DOCUMENT_ID_COLLISION_STEP,
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
        [DocumentMessage::Closed { doc: closed }, DocumentMessage::Opened { doc: opened }] => {
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

    let snapshot = authority.retention_snapshot();
    assert!(snapshot.root_identity_count <= super::MAX_TRACKED_DOCUMENT_ROOTS);
    assert_eq!(snapshot.active_document_count, 1);
    assert!(snapshot.root_eviction_count > 0);
    assert!(snapshot.root_eviction_scan_entry_count >= snapshot.root_eviction_count);
    assert_eq!(
        authority.activate(original_root),
        vec![
            DocumentMessage::Closed {
                doc: snapshot.active_document_id.unwrap()
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
    let before_reopen = authority.retention_snapshot();
    assert!(before_reopen.root_identity_count <= super::MAX_TRACKED_DOCUMENT_ROOTS);
    assert!(before_reopen.root_eviction_count > 0);
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
        authority.retention_snapshot().active_document_id,
        Some(document_id)
    );
}

#[test]
fn known_root_queries_do_not_create_another_root_owner() {
    let authority = DocumentLifecycleAuthority::default();
    let root = Path::new("C:/projects/borrowed-query");
    let _ = authority.activate(root);
    let before = authority.retention_snapshot();

    assert_eq!(authority.activate(root), Vec::new());
    assert!(authority.save(root).is_some());
    assert!(authority.close(root).is_some());
    assert_eq!(authority.close(root), None);

    let after = authority.retention_snapshot();
    assert_eq!(after.root_identity_count, before.root_identity_count);
    assert_eq!(after.scene_identity_count, before.scene_identity_count);
    assert_eq!(after.root_path_bytes, before.root_path_bytes);
    assert_eq!(
        after.scene_project_root_path_bytes,
        before.scene_project_root_path_bytes
    );
    assert_eq!(after.scene_uri_bytes, before.scene_uri_bytes);
    assert_eq!(
        after.document_id_occupancy_probe_count,
        before.document_id_occupancy_probe_count
    );
    assert_eq!(after.root_eviction_count, before.root_eviction_count);
    assert_eq!(
        after.root_eviction_scan_entry_count,
        before.root_eviction_scan_entry_count
    );
    assert_eq!(after.active_document_count, 0);
    assert_eq!(after.active_document_id, None);
}

#[test]
fn collision_stepped_document_id_reopens_with_the_same_identity_and_order() {
    let candidate = Path::new("C:/projects/collision-candidate");
    let occupied = PathBuf::from("C:/projects/occupied-collision-id");
    let base_id = stable_document_id(candidate);
    let collision_id = DocumentId::new(base_id.value().wrapping_add(DOCUMENT_ID_COLLISION_STEP));
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
    assert_eq!(
        authority
            .retention_snapshot()
            .document_id_occupancy_probe_count,
        2
    );
    let other = Path::new("C:/projects/collision-other");
    let other_id = match authority.activate(other).as_slice() {
        [DocumentMessage::Closed { doc }, DocumentMessage::Opened { doc: opened }] => {
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
fn prepared_scene_activation_defers_lifecycle_state_change_until_commit() {
    let authority = DocumentLifecycleAuthority::default();
    let root = Path::new("E:/projects/prepared-scene-activation");
    let session = authority.begin_project_session(root).session;

    let activation = authority.with_scene_route(|| {
        let reservation = authority
            .prepare_scene_activation_while_routed(session, root, "res://scenes/next.scene.toml")
            .unwrap();
        assert!(
            authority
                .active_scene_document_while_routed(session, root, "res://scenes/next.scene.toml",)
                .unwrap()
                .is_none(),
            "reservation must not publish a replacement before world installation succeeds"
        );

        authority.commit_scene_activation_while_routed(reservation)
    });

    assert!(!activation.already_active);
    assert_eq!(
        authority
            .active_scene_document(session, root, "res://scenes/next.scene.toml")
            .unwrap(),
        Some(activation.document)
    );
}

#[test]
fn active_scene_identity_follows_the_latest_scene_document_not_the_project_default() {
    let authority = DocumentLifecycleAuthority::default();
    let root = Path::new("E:/projects/active-scene-identity");
    let session = authority.begin_project_session(root).session;
    let first = authority
        .activate_scene(session, root, "res://scenes/default.scene.toml")
        .unwrap();
    let first_identity = authority
        .active_scene_identity(root)
        .expect("the first scene activation must publish an identity");
    let second = authority
        .activate_scene(session, root, "res://scenes/level_b.scene.toml")
        .unwrap();

    let active = authority
        .active_scene_identity(root)
        .expect("the active project session must retain its selected scene identity");
    assert_eq!(active.document(), second.document);
    assert_eq!(active.project_root(), root);
    assert_eq!(active.scene_uri(), "res://scenes/level_b.scene.toml");
    assert_ne!(active.document(), first.document);
    assert_eq!(
        authority.save_scene_identity_if_active(&active),
        Some(DocumentMessage::Saved {
            doc: second.document,
        })
    );
    assert_eq!(
        authority.save_scene_identity_if_active(&first_identity),
        None
    );
    assert!(authority
        .active_scene_identity(Path::new("E:/projects/other-project"))
        .is_none());
}

#[test]
fn retention_snapshot_reports_scene_and_session_path_owners_without_cloning_them() {
    let authority = DocumentLifecycleAuthority::default();
    let root = Path::new("C:/projects/profile-snapshot");
    let scene_uri = "res://scenes/profile.scene.toml";
    let session = authority.begin_project_session(root).session;
    let _ = authority.activate_scene(session, root, scene_uri).unwrap();

    let snapshot = authority.retention_snapshot();
    assert_eq!(snapshot.root_identity_count, 1);
    assert_eq!(snapshot.scene_identity_count, 1);
    assert_eq!(snapshot.root_path_bytes, root.as_os_str().len());
    assert_eq!(
        snapshot.scene_project_root_path_bytes,
        root.as_os_str().len()
    );
    assert_eq!(snapshot.scene_uri_bytes, scene_uri.len());
    assert_eq!(snapshot.active_project_session_count, 1);
    assert_eq!(
        snapshot.active_project_session_root_path_bytes,
        root.as_os_str().len()
    );
    assert_eq!(snapshot.active_scene_identity_count, 1);
    assert_eq!(snapshot.active_document_count, 1);
}

#[cfg(windows)]
#[test]
fn lifecycle_errors_display_windows_operation_roots_without_verbatim_prefixes() {
    let error = SceneDocumentLifecycleError::NoActiveProjectSession {
        project_root: PathBuf::from(r"\\?\C:\projects\scene-route"),
    };

    assert_eq!(
        error.to_string(),
        r"scene document request requires an active project session for C:\projects\scene-route"
    );
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
    let identity = authority.active_scene_identity(root).unwrap();

    assert_eq!(
        authority.save_scene_identity_if_active(&identity),
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

#[test]
fn binding_failure_does_not_publish_or_activate_the_reserved_scene_document() {
    let authority = DocumentLifecycleAuthority::default();
    let root = Path::new("C:/projects/binding-failure");
    let session = authority.begin_project_session(root).session;

    let error = authority
        .activate_scene_with_binding(session, root, "res://scenes/main.scene.toml", |_| {
            Err("journal source is invalid")
        })
        .unwrap_err();

    assert!(matches!(
        error,
        SceneDocumentActivationBindingError::Binding("journal source is invalid")
    ));
    assert!(authority.active_scene_identity(root).is_none());
}

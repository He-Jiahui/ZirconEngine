use std::error::Error;
use std::path::PathBuf;

use super::{
    AnimationEditorDocumentLoadDiagnostic, AnimationEditorTargetDiagnostic,
    AnimationEditorTargetKind, AnimationEditorTargetUnavailableReason, EditorError,
};
use zircon_runtime::asset::AssetImportError;
use zircon_runtime::core::resource::{ResourceLocator, ResourceLocatorError};

#[test]
fn typed_asset_and_uri_errors_remain_in_the_editor_error_source_chain() {
    let asset_error: EditorError = AssetImportError::MissingProjectAssetRoot.into();
    assert!(asset_error
        .source()
        .is_some_and(|source| source.downcast_ref::<AssetImportError>().is_some()));

    let uri_source = ResourceLocator::parse("not-a-resource-uri").unwrap_err();
    let uri_error: EditorError = uri_source.into();
    assert!(uri_error
        .source()
        .is_some_and(|source| source.downcast_ref::<ResourceLocatorError>().is_some()));
}

#[test]
fn hub_focus_forwarding_exposes_the_existing_editor_process_id() {
    let error = EditorError::HubFocusForwarded { process_id: 913 };
    let created = EditorError::HubFocusForwardedAfterCreate {
        process_id: 914,
        project_root: PathBuf::from("created-project"),
    };

    assert_eq!(error.hub_focus_forwarded_process_id(), Some(913));
    assert_eq!(created.hub_focus_forwarded_process_id(), Some(914));
    assert!(created.to_string().contains("created-project"));
    assert_eq!(
        EditorError::Project("other".to_string()).hub_focus_forwarded_process_id(),
        None
    );
}

#[test]
fn animation_target_diagnostics_expose_stable_codes_without_message_matching() {
    let diagnostic = AnimationEditorTargetDiagnostic::new(
        AnimationEditorTargetKind::Graph,
        AnimationEditorTargetUnavailableReason::WrongFocusedViewKind,
    );
    let error = EditorError::AnimationTargetUnavailable { diagnostic };

    assert_eq!(diagnostic.code(), "ZR-ANIM-TARGET-006");
    assert_eq!(
        diagnostic.to_string(),
        "[ZR-ANIM-TARGET-006] focused view is not an animation graph editor"
    );
    assert_eq!(error.animation_target_diagnostic(), Some(diagnostic));
}

#[test]
fn animation_document_load_diagnostic_exposes_binary_kind_mismatch_contract() {
    let diagnostic =
        AnimationEditorDocumentLoadDiagnostic::binary_kind_mismatch("sequence", "graph");
    let error = EditorError::AnimationDocumentLoad { diagnostic };

    assert_eq!(diagnostic.code(), "ZR-ANIM-LOAD-001");
    assert_eq!(diagnostic.expected(), "sequence");
    assert_eq!(diagnostic.actual(), "graph");
    assert_eq!(
        diagnostic.to_string(),
        "[ZR-ANIM-LOAD-001] animation binary kind mismatch: expected sequence, found graph"
    );
    assert_eq!(error.animation_document_load_diagnostic(), Some(diagnostic));
}

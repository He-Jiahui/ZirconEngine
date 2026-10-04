use crate::ui::host::{
    AnimationEditorTargetDiagnostic, AnimationEditorTargetKind,
    AnimationEditorTargetUnavailableReason,
};

use super::{should_tolerate_missing_animation_target, EditorError};

#[test]
fn only_typed_animation_target_errors_are_tolerated() {
    let typed = EditorError::AnimationTargetUnavailable {
        diagnostic: AnimationEditorTargetDiagnostic::new(
            AnimationEditorTargetKind::Sequence,
            AnimationEditorTargetUnavailableReason::NoFocusedView,
        ),
    };

    assert!(should_tolerate_missing_animation_target(&typed));
    assert!(!should_tolerate_missing_animation_target(
        &EditorError::UiAsset("no focused animation sequence editor".to_string(),)
    ));
}

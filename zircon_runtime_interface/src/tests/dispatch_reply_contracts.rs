use crate::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchPhase, UiDispatchReply,
        UiDispatchReplyStep, UiDispatchReplyStepTrace, UiFocusEffectReason,
        UiRedrawRequestReason,
    },
    event_ui::UiNodeId,
    tree::UiDirtyFlags,
};

#[test]
fn dispatch_reply_merge_drops_unhandled_step_effects_and_keeps_passthrough_effects() {
    let root = UiNodeId::new(1);
    let field = UiNodeId::new(2);
    let ignored = UiNodeId::new(3);

    let report = UiDispatchReply::merge_route([
        UiDispatchReplyStep::new(
            UiDispatchPhase::Preprocess,
            Some(ignored),
            UiDispatchReply::unhandled().with_effect(UiDispatchEffect::SetFocus {
                target: ignored,
                reason: UiFocusEffectReason::Input,
            }),
        ),
        UiDispatchReplyStep::new(
            UiDispatchPhase::PreviewTunnel,
            Some(root),
            UiDispatchReply::passthrough().with_effect(UiDispatchEffect::DirtyRedraw {
                target: root,
                dirty: UiDirtyFlags {
                    input: true,
                    ..UiDirtyFlags::default()
                },
                reason: UiRedrawRequestReason::Input,
            }),
        ),
        UiDispatchReplyStep::new(
            UiDispatchPhase::Target,
            Some(field),
            UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
                target: field,
                reason: UiFocusEffectReason::Input,
            }),
        ),
    ]);

    assert_eq!(report.step_count, 3);
    assert!(report.stopped);
    assert_eq!(report.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(report.reply.handler, Some(field));
    assert_eq!(report.reply.phase, Some(UiDispatchPhase::Target));
    assert_eq!(report.reply.effects.len(), 2);
    assert!(matches!(
        report.reply.effects[0],
        UiDispatchEffect::DirtyRedraw { target, .. } if target == root
    ));
    assert!(matches!(
        report.reply.effects[1],
        UiDispatchEffect::SetFocus { target, .. } if target == field
    ));

    assert_eq!(report.steps.len(), 3);
    assert_eq!(report.steps[0].handler, Some(ignored));
    assert_eq!(report.steps[0].effect_start, 0);
    assert_eq!(report.steps[0].effect_count, 0);
    assert_eq!(report.steps[0].ignored_effect_count, 1);
    assert_eq!(
        report.steps[0].disposition,
        UiDispatchDisposition::Unhandled
    );
    assert!(!report.steps[0].stopped);

    assert_eq!(report.steps[1].handler, Some(root));
    assert_eq!(report.steps[1].effect_start, 0);
    assert_eq!(report.steps[1].effect_count, 1);
    assert_eq!(report.steps[1].ignored_effect_count, 0);
    assert_eq!(
        report.steps[1].disposition,
        UiDispatchDisposition::Passthrough
    );
    assert!(!report.steps[1].stopped);

    assert_eq!(report.steps[2].handler, Some(field));
    assert_eq!(report.steps[2].effect_start, 1);
    assert_eq!(report.steps[2].effect_count, 1);
    assert_eq!(report.steps[2].ignored_effect_count, 0);
    assert_eq!(report.steps[2].disposition, UiDispatchDisposition::Handled);
    assert!(report.steps[2].stopped);

    let mut legacy_step = serde_json::to_value(&report.steps[1]).unwrap();
    legacy_step
        .as_object_mut()
        .unwrap()
        .remove("ignored_effect_count");
    let legacy_step: UiDispatchReplyStepTrace = serde_json::from_value(legacy_step).unwrap();
    assert_eq!(legacy_step.ignored_effect_count, 0);
}

//! 核对窗口事件投影出的瞬态关闭命令，以及批次迭代的顺序与借用边界。

use crate::ui::{
    dispatch::{
        UiDispatchEffect, UiInputSequence, UiInputTimestamp, UiTransientDismissalReason,
        UiTransientDismissalTarget, UiWindowId,
    },
    window::{UiWindowAction, UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata},
    window::{UiWindowInputPumpBatch, UiWindowInputPumpEvent},
};

fn sample_window_metadata() -> UiWindowEventMetadata {
    UiWindowEventMetadata::for_window(
        UiWindowId::new("editor.main"),
        UiInputTimestamp::from_micros(123),
        UiInputSequence::new(7),
    )
}

/// 区分应用停用、窗口失焦的全部关闭与非客户区点击的弹窗栈关闭，并核对无效果分支。
#[test]
fn ui_window_events_preserve_application_activation_and_transient_dismissal_intent() {
    let metadata = sample_window_metadata();
    let app_active = UiWindowEvent::application_activation_changed(metadata.clone(), true);
    let app_inactive = UiWindowEvent::application_activation_changed(metadata.clone(), false);
    let non_client_action =
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::ClickedNonClientArea);
    let window_menu_action =
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::WindowMenu);
    let window_focus_lost = UiWindowEvent::window_focused(metadata, false);

    assert!(matches!(
        app_active.kind,
        UiWindowEventKind::ApplicationActivation { is_active: true }
    ));
    assert!(matches!(
        app_inactive.kind,
        UiWindowEventKind::ApplicationActivation { is_active: false }
    ));
    assert_eq!(app_active.transient_dismissal_effect(), None);
    assert_eq!(
        app_inactive.transient_dismissal_effect(),
        Some(UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::ApplicationDeactivated,
        })
    );
    assert_eq!(
        non_client_action.transient_dismissal_effect(),
        Some(UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::PopupStack,
            reason: UiTransientDismissalReason::WindowAction,
        })
    );
    assert_eq!(window_menu_action.transient_dismissal_effect(), None);
    assert_eq!(
        window_focus_lost.transient_dismissal_effect(),
        Some(UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::All,
            reason: UiTransientDismissalReason::FocusLost,
        })
    );
    assert_eq!(
        serde_json::from_value::<UiWindowEvent>(serde_json::to_value(&app_inactive).unwrap())
            .unwrap(),
        app_inactive
    );
}

/// 按追加顺序投影三项瞬态关闭效果，忽略没有对应效果的窗口菜单动作。
/// 收集结果拥有独立命令值，遍历期间批次由迭代器共享借用。
#[test]
fn ui_window_input_pump_projects_transient_dismissal_effects_without_mutating_events() {
    let metadata = sample_window_metadata();
    let mut batch = UiWindowInputPumpBatch::default();
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::application_activation_changed(metadata.clone(), false),
    ));
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::window_action(metadata.clone(), UiWindowAction::ClickedNonClientArea),
    ));
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::window_focused(metadata.clone(), false),
    ));
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::window_action(metadata, UiWindowAction::WindowMenu),
    ));

    let effects = batch.transient_dismissal_effects().collect::<Vec<_>>();

    assert_eq!(
        effects,
        vec![
            UiDispatchEffect::DismissTransientUi {
                target: UiTransientDismissalTarget::All,
                reason: UiTransientDismissalReason::ApplicationDeactivated,
            },
            UiDispatchEffect::DismissTransientUi {
                target: UiTransientDismissalTarget::PopupStack,
                reason: UiTransientDismissalReason::WindowAction,
            },
            UiDispatchEffect::DismissTransientUi {
                target: UiTransientDismissalTarget::All,
                reason: UiTransientDismissalReason::FocusLost,
            },
        ]
    );
    assert_eq!(batch.events.len(), 4);
}

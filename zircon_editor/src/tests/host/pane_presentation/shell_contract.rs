use super::support::{chrome_fixture, pane_body_spec};

use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, PaneActionPresentation, PaneEmptyStatePresentation, PanePayload,
    PanePayloadBuildContext, PanePresentation, PaneShellPresentation,
};

#[test]
fn pane_presentation_keeps_shell_and_body_split_without_erasing_payload_type() {
    let empty_state = PaneEmptyStatePresentation {
        title: "No Console".to_string(),
        body: "Nothing has been written yet.".to_string(),
        primary_action: Some(PaneActionPresentation {
            label: "Open".to_string(),
            action_id: "workbench.console.open".to_string(),
        }),
        secondary_action: Some(PaneActionPresentation {
            label: "Dismiss".to_string(),
            action_id: "workbench.console.dismiss".to_string(),
        }),
        secondary_hint: "Wait for editor output".to_string(),
    };
    let shell = PaneShellPresentation::new(
        "Console",
        "console",
        "Task Output",
        "Console ready",
        Some(empty_state),
        false,
        blank_viewport_chrome(),
    );
    let chrome = chrome_fixture();
    let context = PanePayloadBuildContext::new(&chrome);
    let body = build_pane_body_presentation(&pane_body_spec("editor.console"), &context);
    let presentation = PanePresentation::new(shell.clone(), body.clone());

    assert_eq!(presentation.shell.title, "Console");
    assert_eq!(presentation.shell.icon_key, "console");
    assert_eq!(presentation.shell.subtitle, "Task Output");
    assert_eq!(presentation.shell.info, "Console ready");
    assert!(!presentation.shell.show_toolbar);
    assert_eq!(presentation.shell.viewport.mode, "");
    assert_eq!(
        presentation
            .shell
            .empty_state
            .as_ref()
            .and_then(|state| state.primary_action.as_ref())
            .map(|action| action.label.as_str()),
        Some("Open")
    );
    assert_eq!(
        presentation
            .shell
            .empty_state
            .as_ref()
            .map(|state| state.secondary_hint.as_str()),
        Some("Wait for editor output")
    );
    assert_eq!(
        presentation.body.document_id,
        "res://ui/editor/host/console_body.zui"
    );
    match presentation.body.payload {
        PanePayload::ConsoleV1(payload) => {
            assert_eq!(payload.output.as_ref(), "Console ready")
        }
        unexpected => panic!("expected console payload, found {unexpected:?}"),
    }
}

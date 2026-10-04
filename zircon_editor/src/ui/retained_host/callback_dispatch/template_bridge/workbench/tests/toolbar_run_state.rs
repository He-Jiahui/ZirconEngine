use super::*;

#[test]
fn run_controls_project_visibility_and_checked_state_from_the_session_owner() {
    assert_eq!(
        toolbar_run_projection(false),
        ToolbarRunProjection {
            play: RunControlProjection {
                visible: true,
                checked: false,
            },
            stop: RunControlProjection {
                visible: false,
                checked: false,
            },
        }
    );
    assert_eq!(
        toolbar_run_projection(true),
        ToolbarRunProjection {
            play: RunControlProjection {
                visible: false,
                checked: false,
            },
            stop: RunControlProjection {
                visible: true,
                checked: true,
            },
        }
    );
}

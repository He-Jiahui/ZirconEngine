use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const PLAY_CONTROL_ID: &str = "WorkbenchRunPlay";
const STOP_CONTROL_ID: &str = "WorkbenchRunStop";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RunControlProjection {
    visible: bool,
    checked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ToolbarRunProjection {
    play: RunControlProjection,
    stop: RunControlProjection,
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_toolbar_run_state(
        &mut self,
        model: &WorkbenchViewModel,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let projection = toolbar_run_projection(model.is_playing);
        self.set_visible(PLAY_CONTROL_ID, projection.play.visible)?;
        self.mutate_control_property(
            PLAY_CONTROL_ID,
            "checked",
            UiValue::Bool(projection.play.checked),
        )?;
        self.set_visible(STOP_CONTROL_ID, projection.stop.visible)?;
        self.mutate_control_property(
            STOP_CONTROL_ID,
            "checked",
            UiValue::Bool(projection.stop.checked),
        )?;
        Ok(())
    }
}

fn toolbar_run_projection(is_playing: bool) -> ToolbarRunProjection {
    ToolbarRunProjection {
        play: RunControlProjection {
            visible: !is_playing,
            checked: false,
        },
        stop: RunControlProjection {
            visible: is_playing,
            checked: is_playing,
        },
    }
}

#[cfg(test)]
mod tests {
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
}

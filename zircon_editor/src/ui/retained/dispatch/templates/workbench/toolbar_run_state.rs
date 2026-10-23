use crate::core::editor_message::PlayStateKind;
use crate::ui::workbench::model::{RunControlsModel, WorkbenchViewModel};
use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const PLAY_CONTROL_ID: &str = "WorkbenchRunPlay";
const STOP_CONTROL_ID: &str = "WorkbenchRunStop";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RunControlProjection {
    visible: bool,
    checked: bool,
    enabled: bool,
    label: &'static str,
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
        self.apply_run_controls(model.run_controls)
    }

    pub(crate) fn apply_run_controls(
        &mut self,
        controls: RunControlsModel,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let projection = toolbar_run_projection(controls);
        // Native Play has a serialized transform contract; generic quaternion
        // reflection remains read-only. Keep the new numeric Edit controls honest.
        for control in [
            "WorkbenchTransformRotationX",
            "WorkbenchTransformRotationY",
            "WorkbenchTransformRotationZ",
        ] {
            if self.has_control(control) {
                self.mutate_control_property(
                    control,
                    "disabled",
                    UiValue::Bool(controls.state != PlayStateKind::Edit),
                )?;
            }
        }
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
        for (control, state) in [
            (PLAY_CONTROL_ID, projection.play),
            (STOP_CONTROL_ID, projection.stop),
        ] {
            self.mutate_control_property(control, "disabled", UiValue::Bool(!state.enabled))?;
            self.mutate_control_property(
                control,
                "label",
                UiValue::String(state.label.to_owned()),
            )?;
            self.mutate_control_property(
                control,
                "tooltip",
                UiValue::String(state.label.to_owned()),
            )?;
        }
        Ok(())
    }
}

fn toolbar_run_projection(model: RunControlsModel) -> ToolbarRunProjection {
    let show_stop = matches!(
        model.state,
        PlayStateKind::Playing | PlayStateKind::Stopping | PlayStateKind::CleanupFailed
    );
    ToolbarRunProjection {
        play: RunControlProjection {
            visible: !show_stop,
            checked: false,
            enabled: model.enter_enabled,
            label: match model.state {
                PlayStateKind::Building => "Building…",
                PlayStateKind::Starting => "Starting…",
                _ => "Play",
            },
        },
        stop: RunControlProjection {
            visible: show_stop,
            checked: model.state == PlayStateKind::Playing,
            enabled: model.stop_enabled,
            label: match model.state {
                PlayStateKind::CleanupFailed => "Retry Stop",
                PlayStateKind::Stopping => "Stopping…",
                _ => "Stop",
            },
        },
    }
}

#[cfg(test)]
#[path = "../../workbench/tests/toolbar_run_state.rs"]
mod tests;

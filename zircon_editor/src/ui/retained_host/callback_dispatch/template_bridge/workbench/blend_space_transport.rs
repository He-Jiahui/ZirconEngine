use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const RECORD_CONTROL: &str = "WorkbenchTransportRecord";
const PLAY_CONTROL: &str = "WorkbenchTransportPlay";
const PAUSE_CONTROL: &str = "WorkbenchTransportPause";
const LOOP_CONTROL: &str = "WorkbenchTransportLoop";
const PREVIEW_ASSET_CONTROL: &str = "WorkbenchExtensionBlendSpacePreviewAsset";
const PREVIEW_STATUS_CONTROL: &str = "WorkbenchExtensionBlendSpacePreviewStatus";
const PREVIEW_TIMELINE_CONTROL: &str = "WorkbenchExtensionBlendSpacePreviewTimeline";

const TRANSPORT_ACTIONS: &[(&str, &str)] = &[
    (
        "workbench.extension.animation_transport.record.toggle",
        RECORD_CONTROL,
    ),
    (
        "workbench.extension.animation_transport.play.invoke",
        PLAY_CONTROL,
    ),
    (
        "workbench.extension.animation_transport.pause.invoke",
        PAUSE_CONTROL,
    ),
    (
        "workbench.extension.animation_transport.previous.invoke",
        "WorkbenchTransportPrevious",
    ),
    (
        "workbench.extension.animation_transport.next.invoke",
        "WorkbenchTransportNext",
    ),
    (
        "workbench.extension.animation_transport.loop.toggle",
        LOOP_CONTROL,
    ),
];

pub(super) fn is_animation_transport_action(action_id: &str) -> bool {
    animation_transport_control_id(action_id).is_some()
}

fn animation_transport_control_id(action_id: &str) -> Option<&'static str> {
    TRANSPORT_ACTIONS
        .iter()
        .find_map(|(action, control)| (*action == action_id).then_some(*control))
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_blend_space_transport_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_control_active(RECORD_CONTROL, false)?;
        self.set_control_active(PLAY_CONTROL, true)?;
        self.set_control_active(PAUSE_CONTROL, false)?;
        self.set_control_active(LOOP_CONTROL, true)?;
        Ok(())
    }

    pub(super) fn apply_blend_space_transport_action(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let feedback = match action_id {
            "workbench.extension.animation_transport.record.toggle" => {
                let recording = !self.control_bool(RECORD_CONTROL, "checked");
                self.set_control_active(RECORD_CONTROL, recording)?;
                if recording {
                    TransportFeedback::new("Recording armed", "Recording armed")
                } else {
                    TransportFeedback::new("Recording disarmed", "Recording disarmed")
                }
            }
            "workbench.extension.animation_transport.play.invoke" => {
                self.set_control_active(PLAY_CONTROL, true)?;
                self.set_control_active(PAUSE_CONTROL, false)?;
                TransportFeedback::new("Blend preview playing", "Previewing")
            }
            "workbench.extension.animation_transport.pause.invoke" => {
                self.set_control_active(PLAY_CONTROL, false)?;
                self.set_control_active(PAUSE_CONTROL, true)?;
                TransportFeedback::new("Blend preview paused", "Paused")
            }
            "workbench.extension.animation_transport.previous.invoke" => {
                self.set_timeline_time(0.0)?;
                TransportFeedback::new("Blend preview moved to start", "Start")
            }
            "workbench.extension.animation_transport.next.invoke" => {
                self.set_timeline_time(3.0)?;
                TransportFeedback::new("Blend preview moved to end", "End")
            }
            "workbench.extension.animation_transport.loop.toggle" => {
                let looping = !self.control_bool(LOOP_CONTROL, "checked");
                self.set_control_active(LOOP_CONTROL, looping)?;
                if looping {
                    TransportFeedback::new("Blend preview loop enabled", "Loop enabled")
                } else {
                    TransportFeedback::new("Blend preview loop disabled", "Loop disabled")
                }
            }
            _ => return Ok(()),
        };

        let preview_asset = self
            .control_string(PREVIEW_ASSET_CONTROL, "value")
            .unwrap_or_else(|| "Selected sample".to_string());
        self.mutate_control_property(
            PREVIEW_STATUS_CONTROL,
            "text",
            UiValue::String(format!("{preview_asset}  |  {}", feedback.preview_status)),
        )?;
        self.mutate_control_property(
            "WorkbenchStatusReady",
            "text",
            UiValue::String(feedback.status_text.to_string()),
        )?;
        self.mutate_control_property(
            "WorkbenchStatusMessages",
            "text",
            UiValue::String("1 Message".to_string()),
        )?;
        Ok(())
    }

    fn set_timeline_time(
        &mut self,
        current_time: f64,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(
            PREVIEW_TIMELINE_CONTROL,
            "current_time",
            UiValue::Float(current_time),
        )
    }
}

struct TransportFeedback {
    status_text: &'static str,
    preview_status: &'static str,
}

impl TransportFeedback {
    const fn new(status_text: &'static str, preview_status: &'static str) -> Self {
        Self {
            status_text,
            preview_status,
        }
    }
}

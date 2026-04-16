use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DockCommand {
    FocusView {
        instance_id: String,
    },
    CloseView {
        instance_id: String,
    },
    AttachViewToDrawer {
        instance_id: String,
        slot: String,
    },
    AttachViewToDocument {
        instance_id: String,
        page_id: String,
    },
    DetachViewToWindow {
        instance_id: String,
        window_id: String,
    },
    ActivateDrawerTab {
        slot: String,
        instance_id: String,
    },
    ActivateMainPage {
        page_id: String,
    },
    SetDrawerMode {
        slot: String,
        mode: String,
    },
    SetDrawerExtent {
        slot: String,
        extent: f32,
    },
    SavePreset {
        name: String,
    },
    LoadPreset {
        name: String,
    },
    ResetToDefault,
}

use serde::Serialize;

use super::super::data::FrameRect;

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileGeometry {
    pub(in crate::ui::retained_host::host_contract) schema_version: u32,
    pub(in crate::ui::retained_host::host_contract) presenter_backend: &'static str,
    pub(in crate::ui::retained_host::host_contract) window_client_size: UiProfileSize,
    pub(in crate::ui::retained_host::host_contract) layout: UiProfileLayout,
    pub(in crate::ui::retained_host::host_contract) resize_splitters: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) document_tabs: Vec<UiProfileTabFrame>,
    pub(in crate::ui::retained_host::host_contract) drawer_tabs: Vec<UiProfileTabFrame>,
    pub(in crate::ui::retained_host::host_contract) host_page_tabs: Vec<UiProfileTabFrame>,
    pub(in crate::ui::retained_host::host_contract) activity_rail_buttons: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) viewport_frame: Option<UiProfileFrame>,
    pub(in crate::ui::retained_host::host_contract) viewport_toolbar_controls:
        Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) template_controls: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) clickable_frames: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) hit_samples: Vec<UiProfileHitSample>,
}

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileNamedFrame {
    pub(in crate::ui::retained_host::host_contract) id: String,
    pub(in crate::ui::retained_host::host_contract) kind: String,
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) frame: UiProfileFrame,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) clip: Option<UiProfileFrame>,
}

impl UiProfileNamedFrame {
    pub(in crate::ui::retained_host::host_contract) fn from_tab(tab: &UiProfileTabFrame) -> Self {
        Self {
            id: tab.id.clone(),
            kind: tab.kind.clone(),
            surface: tab.surface.clone(),
            frame: tab.frame.clone(),
            clip: None,
        }
    }
}

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileTabFrame {
    pub(in crate::ui::retained_host::host_contract) id: String,
    pub(in crate::ui::retained_host::host_contract) title: String,
    pub(in crate::ui::retained_host::host_contract) kind: String,
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) frame: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) close_frame: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) active: bool,
}

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileHitSample {
    pub(in crate::ui::retained_host::host_contract) id: String,
    pub(in crate::ui::retained_host::host_contract) kind: String,
    pub(in crate::ui::retained_host::host_contract) surface: String,
    pub(in crate::ui::retained_host::host_contract) sample: String,
    pub(in crate::ui::retained_host::host_contract) point: UiProfilePoint,
    pub(in crate::ui::retained_host::host_contract) expected_hit: bool,
    pub(in crate::ui::retained_host::host_contract) route_hit: bool,
}

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileFrame {
    pub(in crate::ui::retained_host::host_contract) x: f32,
    pub(in crate::ui::retained_host::host_contract) y: f32,
    pub(in crate::ui::retained_host::host_contract) width: f32,
    pub(in crate::ui::retained_host::host_contract) height: f32,
}

impl From<FrameRect> for UiProfileFrame {
    fn from(frame: FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

impl From<&FrameRect> for UiProfileFrame {
    fn from(frame: &FrameRect) -> Self {
        Self {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

#[derive(Clone, Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfilePoint {
    pub(in crate::ui::retained_host::host_contract) x: f32,
    pub(in crate::ui::retained_host::host_contract) y: f32,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileSize {
    pub(in crate::ui::retained_host::host_contract) width: u32,
    pub(in crate::ui::retained_host::host_contract) height: u32,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileLayout {
    pub(in crate::ui::retained_host::host_contract) center_band: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) document_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) left_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) right_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) bottom_region: UiProfileFrame,
    pub(in crate::ui::retained_host::host_contract) status_bar: UiProfileFrame,
}

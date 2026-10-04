use serde::Serialize;

use super::frame::{UiProfileFrame, UiProfileLayout, UiProfileSize};
use super::hit::UiProfileHitSample;
use super::named::UiProfileNamedFrame;
use super::native_hierarchy::UiProfileNativeHierarchy;
use super::rounded::UiProfileRoundedShape;
use super::tab::UiProfileTabFrame;
use super::text::UiProfileTextRun;

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileLogicalSize {
    pub(in crate::ui::retained_host::host_contract) width: f64,
    pub(in crate::ui::retained_host::host_contract) height: f64,
}

#[derive(Serialize)]
pub(in crate::ui::retained_host::host_contract) struct UiProfileGeometry {
    pub(in crate::ui::retained_host::host_contract) schema_version: u32,
    pub(in crate::ui::retained_host::host_contract) presenter_backend: &'static str,
    /// Monotonic successful-present identity; zero denotes the legacy one-shot artifact.
    pub(in crate::ui::retained_host::host_contract) capture_sequence: u64,
    pub(in crate::ui::retained_host::host_contract) window_client_size: UiProfileSize,
    pub(in crate::ui::retained_host::host_contract) winit_scale_factor: f32,
    pub(in crate::ui::retained_host::host_contract) window_client_logical_size:
        UiProfileLogicalSize,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) welcome_recent_frame:
        Option<UiProfileNamedFrame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) asset_browser_content_frame:
        Option<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) clickable_frames: Vec<UiProfileNamedFrame>,
    pub(in crate::ui::retained_host::host_contract) hit_samples: Vec<UiProfileHitSample>,
    pub(in crate::ui::retained_host::host_contract) rounded_shapes: Vec<UiProfileRoundedShape>,
    pub(in crate::ui::retained_host::host_contract) text_runs: Vec<UiProfileTextRun>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::ui::retained_host::host_contract) submitted_gpu_text: Option<serde_json::Value>,
    /// Optional source-bound semantic rows for the native hierarchy presenter.
    /// This remains additive under the existing schema version.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(in crate::ui::retained_host::host_contract) native_hierarchy: Vec<UiProfileNativeHierarchy>,
}

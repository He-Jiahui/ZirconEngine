use std::sync::Arc;

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

#[derive(Clone, Default)]
pub(crate) struct SceneViewportChromeData {
    pub mode: SharedString,
    pub transform_space: SharedString,
    pub pivot_mode: SharedString,
    pub projection_mode: SharedString,
    pub view_orientation: SharedString,
    pub display_mode: SharedString,
    pub grid_mode: SharedString,
    pub gizmos_enabled: bool,
    pub preview_lighting: bool,
    pub preview_skybox: bool,
    pub translate_snap: f32,
    pub rotate_snap_deg: f32,
    pub scale_snap: f32,
    pub translate_snap_label: SharedString,
    pub rotate_snap_label: SharedString,
    pub scale_snap_label: SharedString,
    pub toolbar_surface_key: SharedString,
    pub toolbar_enter_play_enabled: bool,
    pub toolbar_exit_play_enabled: bool,
    pub toolbar_is_playing: bool,
    pub toolbar_surface_frame: Option<Arc<UiSurfaceFrame>>,
    /// Complete authored paint projection measured with the same toolbar surface size.
    pub toolbar_template_nodes: ModelRc<TemplatePaneNodeData>,
}

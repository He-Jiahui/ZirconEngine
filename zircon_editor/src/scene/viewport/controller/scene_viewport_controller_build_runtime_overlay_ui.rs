use std::sync::Arc;

use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{DisplayMode, GridMode, ProjectionMode, TransformHandleKind};
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
        UiTextAlign, UiTextRenderMode, UiTextWrap,
    },
};

use super::SceneViewportController;

const VIEWPORT_HUD_TREE_ID: &str = "zircon.editor.viewport.hud";
const VIEWPORT_HUD_NODE_ID: u64 = 1;
const VIEWPORT_HUD_Z_INDEX: i32 = 10;
const VIEWPORT_HUD_MARGIN_X: f32 = 16.0;
const VIEWPORT_HUD_MARGIN_Y: f32 = 16.0;
const VIEWPORT_HUD_WIDTH: f32 = 280.0;
const VIEWPORT_HUD_MIN_WIDTH: f32 = 48.0;
const VIEWPORT_HUD_HEIGHT: f32 = 28.0;
const VIEWPORT_HUD_FONT_SIZE: f32 = 13.0;
const VIEWPORT_HUD_LINE_HEIGHT: f32 = 16.0;
const VIEWPORT_HUD_BACKGROUND: &str = "#16202ccc";
const VIEWPORT_HUD_FOREGROUND: &str = "#eef3ff";
const VIEWPORT_HUD_FONT: &str = "res://fonts/default.font.toml";
const VIEWPORT_HUD_OPACITY: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeOverlayUiExtractCacheKey {
    scene_mode_revision: u64,
    projection_mode: ProjectionMode,
    display_mode: DisplayMode,
    grid_mode: GridMode,
    viewport_size: UVec2,
}

#[derive(Default)]
pub(super) struct RuntimeOverlayUiExtractCache {
    key: Option<RuntimeOverlayUiExtractCacheKey>,
    extract: Option<Arc<UiRenderExtract>>,
}

impl RuntimeOverlayUiExtractCache {
    fn current(&self, key: RuntimeOverlayUiExtractCacheKey) -> Option<Arc<UiRenderExtract>> {
        (self.key == Some(key))
            .then(|| self.extract.as_ref().map(Arc::clone))
            .flatten()
    }

    fn publish(&mut self, key: RuntimeOverlayUiExtractCacheKey, extract: Arc<UiRenderExtract>) {
        self.key = Some(key);
        self.extract = Some(extract);
    }
}

impl SceneViewportController {
    pub(crate) fn build_runtime_overlay_ui(&self) -> Option<Arc<UiRenderExtract>> {
        let key = RuntimeOverlayUiExtractCacheKey {
            scene_mode_revision: self.state.scene_modes.revision(),
            projection_mode: self.state.settings.projection_mode,
            display_mode: self.state.settings.display_mode,
            grid_mode: self.state.settings.grid_mode,
            viewport_size: self.state.viewport.size,
        };
        if let Some(extract) = self.runtime_overlay_ui_cache.borrow().current(key) {
            return Some(extract);
        }

        let extract = Arc::new(self.build_runtime_overlay_ui_extract());
        self.runtime_overlay_ui_cache
            .borrow_mut()
            .publish(key, Arc::clone(&extract));
        Some(extract)
    }

    fn build_runtime_overlay_ui_extract(&self) -> UiRenderExtract {
        let max_width =
            (self.state.viewport.size.x as f32 - VIEWPORT_HUD_MARGIN_X - VIEWPORT_HUD_MARGIN_X)
                .max(VIEWPORT_HUD_MIN_WIDTH);
        let frame = UiFrame::new(
            VIEWPORT_HUD_MARGIN_X,
            VIEWPORT_HUD_MARGIN_Y,
            VIEWPORT_HUD_WIDTH.min(max_width),
            VIEWPORT_HUD_HEIGHT,
        );

        UiRenderExtract {
            tree_id: UiTreeId::new(VIEWPORT_HUD_TREE_ID),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(VIEWPORT_HUD_NODE_ID),
                    kind: UiRenderCommandKind::Quad,
                    frame,
                    clip_frame: None,
                    z_index: VIEWPORT_HUD_Z_INDEX,
                    style: UiResolvedStyle {
                        background_color: Some(VIEWPORT_HUD_BACKGROUND.to_string()),
                        foreground_color: Some(VIEWPORT_HUD_FOREGROUND.to_string()),
                        font: Some(VIEWPORT_HUD_FONT.to_string()),
                        font_size: VIEWPORT_HUD_FONT_SIZE,
                        line_height: VIEWPORT_HUD_LINE_HEIGHT,
                        text_align: UiTextAlign::Center,
                        wrap: UiTextWrap::None,
                        text_render_mode: UiTextRenderMode::Auto,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some(self.runtime_hud_text()),
                    image: None,
                    opacity: VIEWPORT_HUD_OPACITY,
                }],
            },
            raster_scale: 1.0,
        }
    }

    fn runtime_hud_text(&self) -> String {
        format!(
            "{} | {} | {} | {}",
            scene_mode_label(&self.active_scene_mode()),
            projection_label(self.state.settings.projection_mode),
            display_label(self.state.settings.display_mode),
            grid_label(self.state.settings.grid_mode)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::math::UVec2;

    use super::SceneViewportController;

    #[test]
    fn stable_viewport_hud_generation_reuses_the_same_allocation() {
        let controller = SceneViewportController::new(UVec2::new(1280, 720));

        let first = controller.build_runtime_overlay_ui().unwrap();
        let second = controller.build_runtime_overlay_ui().unwrap();

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn viewport_hud_key_change_publishes_one_new_allocation() {
        let mut controller = SceneViewportController::new(UVec2::new(1280, 720));
        let first = controller.build_runtime_overlay_ui().unwrap();

        controller.apply_viewport_size(UVec2::new(960, 540));
        let resized = controller.build_runtime_overlay_ui().unwrap();
        let stable = controller.build_runtime_overlay_ui().unwrap();

        assert!(!Arc::ptr_eq(&first, &resized));
        assert!(Arc::ptr_eq(&resized, &stable));
    }
}

fn scene_mode_label(mode: &SceneModeActivation) -> String {
    match mode {
        SceneModeActivation::Select => "Select".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Move) => "Move".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Rotate) => "Rotate".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Scale) => "Scale".to_string(),
        SceneModeActivation::Custom(mode_id) => mode_id.as_str().to_string(),
    }
}

fn projection_label(projection: ProjectionMode) -> &'static str {
    match projection {
        ProjectionMode::Perspective => "Persp",
        ProjectionMode::Orthographic => "Ortho",
    }
}

fn display_label(display: DisplayMode) -> &'static str {
    match display {
        DisplayMode::Shaded => "Shaded",
        DisplayMode::WireOverlay => "Wire+Shaded",
        DisplayMode::WireOnly => "Wire",
    }
}

fn grid_label(grid: GridMode) -> &'static str {
    match grid {
        GridMode::Hidden => "Grid Off",
        GridMode::VisibleNoSnap => "Grid",
        GridMode::VisibleAndSnap => "Snap Grid",
    }
}

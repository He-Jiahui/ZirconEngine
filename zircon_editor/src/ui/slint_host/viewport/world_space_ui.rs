use crate::ui::slint_host::host_contract::WorldSpaceUiSurfaceSubmission;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{
        UiPointerEventKind, UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList,
        UiResolvedStyle, UiTextAlign, UiTextRenderMode, UiTextWrap,
    },
};

use super::slint_viewport_controller::SlintViewportController;

const WORLD_SPACE_UI_TREE_ID: &str = "zircon.editor.viewport.world_space_ui";
const WORLD_SPACE_UI_NODE_ID_BASE: u64 = 50_000;
const WORLD_SPACE_UI_Z_BASE: i32 = 1_000;
const WORLD_SPACE_UI_FONT: &str = "res://fonts/default.font.toml";
const WORLD_SPACE_UI_FONT_SIZE: f32 = 12.0;
const WORLD_SPACE_UI_LINE_HEIGHT: f32 = 14.0;
const WORLD_SPACE_UI_OPACITY: f32 = 0.88;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorldSpaceUiPointerRoute {
    pub(crate) surface_id: String,
    pub(crate) node_id: String,
    pub(crate) control_id: String,
    pub(crate) point_x: f32,
    pub(crate) point_y: f32,
    pub(crate) render_order: i32,
}

impl SlintViewportController {
    #[allow(dead_code)]
    pub(crate) fn submit_world_space_ui_surfaces(
        &self,
        submissions: Vec<WorldSpaceUiSurfaceSubmission>,
    ) {
        self.lock_shared().last_world_space_ui_surfaces = submissions;
    }

    #[cfg(test)]
    pub(crate) fn last_world_space_ui_surfaces(&self) -> Vec<WorldSpaceUiSurfaceSubmission> {
        self.lock_shared().last_world_space_ui_surfaces.clone()
    }

    pub(crate) fn route_world_space_ui_pointer_event(
        &self,
        kind: UiPointerEventKind,
        x: f32,
        y: f32,
    ) -> Option<WorldSpaceUiPointerRoute> {
        let mut shared = self.lock_shared();
        let hit = topmost_world_space_ui_surface_at(&shared.last_world_space_ui_surfaces, x, y);
        let route = match kind {
            UiPointerEventKind::Down => {
                shared.world_space_ui_pointer_capture = hit.clone();
                hit
            }
            UiPointerEventKind::Move | UiPointerEventKind::Scroll => {
                shared.world_space_ui_pointer_capture.clone().or(hit)
            }
            UiPointerEventKind::Up => shared.world_space_ui_pointer_capture.take().or(hit),
        }?;

        Some(WorldSpaceUiPointerRoute {
            surface_id: route.surface_id,
            node_id: route.node_id,
            control_id: route.control_id,
            point_x: x,
            point_y: y,
            render_order: route.render_order,
        })
    }
}

pub(super) fn merge_ui_with_world_space_submissions(
    ui: Option<UiRenderExtract>,
    submissions: &[WorldSpaceUiSurfaceSubmission],
) -> Option<UiRenderExtract> {
    let Some(world_space_ui) = world_space_ui_render_extract(submissions) else {
        return ui;
    };

    match ui {
        Some(mut ui) => {
            ui.list.commands.extend(world_space_ui.list.commands);
            Some(ui)
        }
        None => Some(world_space_ui),
    }
}

fn world_space_ui_render_extract(
    submissions: &[WorldSpaceUiSurfaceSubmission],
) -> Option<UiRenderExtract> {
    let commands = submissions
        .iter()
        .enumerate()
        .filter_map(|(index, submission)| world_space_ui_render_command(index, submission))
        .collect::<Vec<_>>();

    (!commands.is_empty()).then(|| UiRenderExtract {
        tree_id: UiTreeId::new(WORLD_SPACE_UI_TREE_ID),
        list: UiRenderList { commands },
    })
}

fn world_space_ui_render_command(
    index: usize,
    submission: &WorldSpaceUiSurfaceSubmission,
) -> Option<UiRenderCommand> {
    if submission.viewport_width <= 0.0 || submission.viewport_height <= 0.0 {
        return None;
    }

    let background_color = if submission.depth_test {
        "#284f8f99"
    } else {
        "#3857aacc"
    };
    let border_color = if submission.billboard {
        "#9ed8ff"
    } else {
        "#7fa2d6"
    };

    Some(UiRenderCommand {
        node_id: UiNodeId::new(WORLD_SPACE_UI_NODE_ID_BASE + index as u64),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(
            submission.viewport_x,
            submission.viewport_y,
            submission.viewport_width,
            submission.viewport_height,
        ),
        clip_frame: None,
        z_index: WORLD_SPACE_UI_Z_BASE + submission.render_order,
        style: UiResolvedStyle {
            background_color: Some(background_color.to_string()),
            foreground_color: Some("#eef7ff".to_string()),
            border_color: Some(border_color.to_string()),
            border_width: 1.0,
            corner_radius: 6.0,
            font: Some(WORLD_SPACE_UI_FONT.to_string()),
            font_size: WORLD_SPACE_UI_FONT_SIZE,
            line_height: WORLD_SPACE_UI_LINE_HEIGHT,
            text_align: UiTextAlign::Center,
            wrap: UiTextWrap::None,
            text_render_mode: UiTextRenderMode::Auto,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some(submission.control_id.clone()),
        image: None,
        opacity: WORLD_SPACE_UI_OPACITY,
    })
}

fn topmost_world_space_ui_surface_at(
    submissions: &[WorldSpaceUiSurfaceSubmission],
    x: f32,
    y: f32,
) -> Option<WorldSpaceUiSurfaceSubmission> {
    submissions
        .iter()
        .rev()
        .find(|submission| submission.contains_viewport_point(x, y))
        .cloned()
}

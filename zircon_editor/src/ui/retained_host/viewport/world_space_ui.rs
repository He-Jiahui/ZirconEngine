use std::sync::Arc;

use crate::ui::retained_host::host_contract::WorldSpaceUiSurfaceSubmission;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{
        UiPointerEventKind, UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList,
        UiResolvedStyle, UiTextAlign, UiTextRenderMode, UiTextWrap,
    },
};

use super::retained_viewport_controller::RetainedViewportController;
use super::viewport_state::ViewportState;

const WORLD_SPACE_UI_TREE_ID: &str = "zircon.editor.viewport.world_space_ui";
const WORLD_SPACE_UI_NODE_ID_BASE: u64 = 50_000;
const WORLD_SPACE_UI_Z_BASE: i32 = 1_000;
const WORLD_SPACE_UI_FONT: &str = "res://fonts/default.font.toml";
const WORLD_SPACE_UI_FONT_SIZE: f32 = 12.0;
const WORLD_SPACE_UI_LINE_HEIGHT: f32 = 14.0;
const WORLD_SPACE_UI_OPACITY: f32 = 0.88;

#[derive(Default)]
pub(super) struct WorldSpaceUiMergeCache {
    source_generation: Option<u64>,
    base_ui: Option<Arc<UiRenderExtract>>,
    merged_ui: Option<Arc<UiRenderExtract>>,
}

impl WorldSpaceUiMergeCache {
    fn resolve(
        &mut self,
        base_ui: Option<Arc<UiRenderExtract>>,
        source_generation: u64,
        submissions: &[WorldSpaceUiSurfaceSubmission],
    ) -> Option<Arc<UiRenderExtract>> {
        if self.source_generation == Some(source_generation)
            && same_optional_extract(&self.base_ui, &base_ui)
        {
            zircon_runtime::profile_counter!(
                "editor",
                "viewport.world_space_ui_merge.cache_hit",
                1
            );
            return self.merged_ui.as_ref().map(Arc::clone);
        }

        let cached_base = base_ui.as_ref().map(Arc::clone);
        let merged_ui = merge_ui_with_world_space_submissions(base_ui, submissions);
        self.source_generation = Some(source_generation);
        self.base_ui = cached_base;
        self.merged_ui = merged_ui.as_ref().map(Arc::clone);
        zircon_runtime::profile_counter!("editor", "viewport.world_space_ui_merge.cache_hit", 0);
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.world_space_ui_merge.rebuild_count",
            1
        );
        merged_ui
    }

    fn clear(&mut self) {
        *self = Self::default();
    }
}

fn same_optional_extract(
    left: &Option<Arc<UiRenderExtract>>,
    right: &Option<Arc<UiRenderExtract>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorldSpaceUiPointerRoute {
    pub(crate) surface_id: String,
    pub(crate) node_id: String,
    pub(crate) control_id: String,
    pub(crate) point_x: f32,
    pub(crate) point_y: f32,
    pub(crate) render_order: i32,
}

impl RetainedViewportController {
    pub(crate) fn submit_world_space_ui_surfaces(
        &self,
        submissions: Vec<WorldSpaceUiSurfaceSubmission>,
    ) {
        let mut shared = self.lock_shared();
        if shared.last_world_space_ui_surfaces == submissions {
            return;
        }
        shared.last_world_space_ui_surfaces = submissions;
        shared.world_space_ui_generation = shared.world_space_ui_generation.wrapping_add(1);
        shared.world_space_ui_merge_cache.clear();
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
            UiPointerEventKind::Cancel => shared.world_space_ui_pointer_capture.take(),
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

impl ViewportState {
    pub(super) fn merge_world_space_ui(
        &mut self,
        base_ui: Option<Arc<UiRenderExtract>>,
    ) -> Option<Arc<UiRenderExtract>> {
        self.world_space_ui_merge_cache.resolve(
            base_ui,
            self.world_space_ui_generation,
            &self.last_world_space_ui_surfaces,
        )
    }
}

pub(super) fn merge_ui_with_world_space_submissions(
    ui: Option<Arc<UiRenderExtract>>,
    submissions: &[WorldSpaceUiSurfaceSubmission],
) -> Option<Arc<UiRenderExtract>> {
    let world_space_commands = world_space_ui_render_commands(submissions);
    if world_space_commands.is_empty() {
        return ui;
    }

    match ui {
        Some(ui) => {
            let mut merged = ui.as_ref().clone();
            merged.list.commands.extend(world_space_commands);
            Some(Arc::new(merged))
        }
        None => Some(Arc::new(UiRenderExtract {
            tree_id: UiTreeId::new(WORLD_SPACE_UI_TREE_ID),
            list: UiRenderList {
                commands: world_space_commands,
            },
            raster_scale: 1.0,
        })),
    }
}

fn world_space_ui_render_commands(
    submissions: &[WorldSpaceUiSurfaceSubmission],
) -> Vec<UiRenderCommand> {
    submissions
        .iter()
        .enumerate()
        .filter_map(|(index, submission)| world_space_ui_render_command(index, submission))
        .collect()
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::{
        event_ui::UiTreeId,
        surface::{UiRenderExtract, UiRenderList},
    };

    use super::{WorldSpaceUiMergeCache, WorldSpaceUiSurfaceSubmission};

    #[test]
    fn stable_world_space_generation_reuses_the_merged_allocation() {
        let mut cache = WorldSpaceUiMergeCache::default();
        let base = test_extract();
        let submissions = [test_submission()];

        let first = cache
            .resolve(Some(Arc::clone(&base)), 7, &submissions)
            .unwrap();
        let second = cache
            .resolve(Some(Arc::clone(&base)), 7, &submissions)
            .unwrap();

        assert!(!Arc::ptr_eq(&base, &first));
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn empty_world_space_generation_preserves_the_base_allocation() {
        let mut cache = WorldSpaceUiMergeCache::default();
        let base = test_extract();

        let merged = cache.resolve(Some(Arc::clone(&base)), 0, &[]).unwrap();

        assert!(Arc::ptr_eq(&base, &merged));
    }

    fn test_extract() -> Arc<UiRenderExtract> {
        Arc::new(UiRenderExtract {
            tree_id: UiTreeId::new("editor.viewport.world-space-cache-test"),
            list: UiRenderList::default(),
            raster_scale: 1.0,
        })
    }

    fn test_submission() -> WorldSpaceUiSurfaceSubmission {
        WorldSpaceUiSurfaceSubmission {
            viewport_width: 100.0,
            viewport_height: 40.0,
            control_id: "WorldPanel".to_string(),
            ..WorldSpaceUiSurfaceSubmission::default()
        }
    }
}

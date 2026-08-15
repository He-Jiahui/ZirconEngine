use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::UiSurface;

#[derive(Clone, Debug)]
pub(super) struct UiSurfaceFramePublication {
    dirty: bool,
    generation: u64,
    frame: Option<Arc<UiSurfaceFrame>>,
}

impl Default for UiSurfaceFramePublication {
    fn default() -> Self {
        Self {
            dirty: true,
            generation: 0,
            frame: None,
        }
    }
}

// Publication state is an ephemeral read cache and does not change UiSurface value equality.
impl PartialEq for UiSurfaceFramePublication {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl UiSurface {
    pub(super) fn mark_surface_frame_dirty(&mut self) {
        self.frame_publication.get_mut().dirty = true;
    }

    pub fn surface_frame(&self) -> Arc<UiSurfaceFrame> {
        let mut publication = self.frame_publication.borrow_mut();
        let transient_state_changed = publication.frame.as_deref().is_none_or(|frame| {
            frame.window_state != self.window_state || frame.focus_state != self.focus
        });

        if publication.dirty || transient_state_changed {
            publication.generation = publication.generation.saturating_add(1);
            let generation = publication.generation;
            publication.frame = Some(Arc::new(UiSurfaceFrame {
                generation,
                tree_id: self.tree.tree_id.clone(),
                window_state: self.window_state.clone(),
                arranged_tree: self.arranged_tree.clone(),
                render_extract: self.render_extract.clone(),
                hit_grid: self
                    .projected_hit_test
                    .authoritative_grid(&self.hit_test.grid)
                    .clone(),
                focus_state: self.focus.clone(),
                focus_path: self.focus_path(),
                last_rebuild: self.last_rebuild_report.debug_stats(),
                layout_engine_report: self.layout_engine_report.clone(),
                pipeline_report: self.last_rebuild_report.pipeline_report(generation),
                ecs_projection: self.ui_ecs_projection(),
            }));
            publication.dirty = false;
        }

        Arc::clone(
            publication
                .frame
                .as_ref()
                .expect("surface frame publication must exist after refresh"),
        )
    }
}

//! Canvas presets edit the authored parent-child edge in one existing undo transaction.

use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use crate::ui::asset_editor::{
    source::source_buffer::UiAssetSourceSnapshot, UiAssetEditorMode, UiDesignerSelectionModel,
};

#[path = "canvas_anchor_state/geometry.rs"]
mod geometry;

use super::{
    inspector_semantics::set_selected_slot_semantic_value,
    ui_asset_editor_session::{
        UiAssetEditorSession, UiAssetEditorSessionError, UiAssetSourceSchema,
    },
};

/// A popup's original source, selected edge and published geometry; callers cannot forge it.
#[derive(Clone)]
pub(crate) struct UiAssetCanvasAnchorPresetTicket {
    lease: Arc<()>,
    source: UiAssetSourceSnapshot,
    route_asset_id: String,
    last_valid_generation: u64,
    selection: UiDesignerSelectionModel,
    frame: Arc<UiSurfaceFrame>,
}

impl UiAssetCanvasAnchorPresetTicket {
    pub(crate) fn is_current(&self, session: &UiAssetEditorSession) -> bool {
        session
            .canvas_anchor_preset_lease
            .as_ref()
            .is_some_and(|lease| Arc::ptr_eq(lease, &self.lease))
            && session.source_buffer.is_current_snapshot(&self.source)
            && session.route.asset_id == self.route_asset_id
            && session.last_valid_source_generation == self.last_valid_generation
            && session.selection == self.selection
            && matches!(
                session.route.mode,
                UiAssetEditorMode::Design | UiAssetEditorMode::Split
            )
            && session
                .preview_host
                .as_ref()
                .is_some_and(|host| Arc::ptr_eq(&self.frame, &host.surface().surface_frame()))
    }
}

/// A committed source change and a failed projection sync are different outcomes.
pub(crate) struct UiAssetCanvasAnchorPresetCommit {
    pub(crate) changed: bool,
    pub(crate) sync_error: Option<String>,
}

impl UiAssetEditorSession {
    pub(super) fn canvas_anchor_presets_available(&self) -> bool {
        matches!(
            self.route.mode,
            UiAssetEditorMode::Design | UiAssetEditorMode::Split
        ) && self
            .selected_canvas_anchor_geometry()
            .ok()
            .flatten()
            .is_some_and(|geometry| geometry.can_rebase())
    }

    /// Capture when the real preset popup opens. A new open retires the preceding popup.
    /// This reads the current Canvas geometry and never edits source or history.
    pub(crate) fn begin_canvas_anchor_preset(
        &mut self,
    ) -> Result<Option<UiAssetCanvasAnchorPresetTicket>, UiAssetEditorSessionError> {
        self.canvas_anchor_preset_lease = None;
        if !matches!(
            self.route.mode,
            UiAssetEditorMode::Design | UiAssetEditorMode::Split
        ) {
            return Ok(None);
        }
        let Some(geometry) = self.selected_canvas_anchor_geometry()? else {
            return Ok(None);
        };
        if !geometry.can_rebase() {
            return Err(geometry_unavailable(
                self.selection
                    .primary_node_id
                    .as_deref()
                    .expect("validated selection"),
            ));
        }
        let frame = self
            .preview_host
            .as_ref()
            .expect("validated preview")
            .surface()
            .surface_frame();
        let lease = Arc::new(());
        let ticket = UiAssetCanvasAnchorPresetTicket {
            lease: Arc::clone(&lease),
            source: self.source_buffer.snapshot(),
            route_asset_id: self.route.asset_id.clone(),
            last_valid_generation: self.last_valid_source_generation,
            selection: self.selection.clone(),
            frame,
        };
        self.canvas_anchor_preset_lease = Some(lease);
        Ok(Some(ticket))
    }

    /// Admit the popup's original target once, then use the existing single undo transaction.
    /// An obsolete callback cannot cancel a newer popup or retarget the current selection.
    pub(crate) fn apply_canvas_anchor_preset(
        &mut self,
        ticket: &UiAssetCanvasAnchorPresetTicket,
        index: usize,
        shift: bool,
        control: bool,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if preset_anchors(index).is_none() {
            return Err(UiAssetEditorSessionError::InvalidCanvasAnchorPreset { index });
        }
        if !ticket.is_current(self) {
            return Err(UiAssetEditorSessionError::RetiredCanvasAnchorPreset);
        }
        self.canvas_anchor_preset_lease = None;
        self.apply_selected_canvas_anchor_preset(index, shift, control)
    }

    /// Dismissal retires only this popup; it does not mutate layout or implement focus return.
    pub(crate) fn cancel_canvas_anchor_preset(
        &mut self,
        ticket: &UiAssetCanvasAnchorPresetTicket,
    ) -> bool {
        if self
            .canvas_anchor_preset_lease
            .as_ref()
            .is_some_and(|lease| Arc::ptr_eq(lease, &ticket.lease))
        {
            self.canvas_anchor_preset_lease = None;
            true
        } else {
            false
        }
    }

    fn selected_canvas_anchor_geometry(
        &self,
    ) -> Result<Option<geometry::CanvasAnchorGeometry>, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        if !matches!(self.source_schema, UiAssetSourceSchema::V2)
            || self.selection.is_multi_select()
        {
            return Ok(None);
        }
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            return Ok(None);
        };
        let Some(parent_id) = self.selection.parent_node_id.as_deref() else {
            return Ok(None);
        };
        let host = self
            .preview_host
            .as_ref()
            .ok_or_else(|| geometry_unavailable(node_id))?;
        geometry::selected_canvas_geometry(&host.surface().tree, node_id, parent_id)
            .map(Some)
            .ok_or_else(|| geometry_unavailable(node_id))
    }

    /// Apply one row-major preset from the Designer's four-by-four anchor menu.
    ///
    /// Geometry is read synchronously from this session's current preview. The
    /// caller supplies no cached screen coordinates or independently owned frame.
    /// Shift changes the pivot after rebasing; Control then resets the offsets.
    /// The popup/input owner remains responsible for dismissal and focus return.
    /// This first adapter requires a positive-sized native Canvas edge with
    /// auto-size off. Free, component instances and collapsed/auto-size geometry
    /// leave the source unchanged and return a geometry-unavailable error.
    pub fn apply_selected_canvas_anchor_preset(
        &mut self,
        index: usize,
        shift: bool,
        control: bool,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let (minimum, maximum) = preset_anchors(index)
            .ok_or(UiAssetEditorSessionError::InvalidCanvasAnchorPreset { index })?;
        let Some(geometry) = self.selected_canvas_anchor_geometry()? else {
            return Ok(false);
        };
        let node_id = self
            .selection
            .primary_node_id
            .as_deref()
            .expect("validated selection");
        let previous = geometry.original_placement();
        let placement = geometry
            .rebase(minimum, maximum, shift, control)
            .ok_or_else(|| geometry_unavailable(node_id))?;

        // A direct synchronous command also ends any earlier popup, including a no-op.
        self.canvas_anchor_preset_lease = None;
        if placement == previous {
            return Ok(false);
        }
        // Materializing the first slot switches the runtime away from node
        // placement. Carry its unchanged nondefault axes/pivot/position too.
        let baseline = geometry.source_placement_baseline();
        let mut document = self.last_valid_document.clone();
        let maximum = placement.resolved_anchor_max();
        let previous_maximum = baseline.resolved_anchor_max();
        let materialize_maximum = baseline.anchor_max.is_none() && placement.anchor_max.is_some();
        // Preserve absent/default fields. A newly stretched range needs both
        // maximum coordinates; a point's implicit maximum follows its minimum.
        let fields = [
            (
                "layout.anchor.x",
                baseline.anchor.x != placement.anchor.x,
                placement.anchor.x,
            ),
            (
                "layout.anchor.y",
                baseline.anchor.y != placement.anchor.y,
                placement.anchor.y,
            ),
            (
                "layout.anchor_max.x",
                placement.anchor_max.is_some()
                    && (materialize_maximum || previous_maximum.x != maximum.x),
                maximum.x,
            ),
            (
                "layout.anchor_max.y",
                placement.anchor_max.is_some()
                    && (materialize_maximum || previous_maximum.y != maximum.y),
                maximum.y,
            ),
            (
                "layout.pivot.x",
                baseline.pivot.x != placement.pivot.x,
                placement.pivot.x,
            ),
            (
                "layout.pivot.y",
                baseline.pivot.y != placement.pivot.y,
                placement.pivot.y,
            ),
            (
                "layout.position.x",
                baseline.position.x != placement.position.x,
                placement.position.x,
            ),
            (
                "layout.position.y",
                baseline.position.y != placement.position.y,
                placement.position.y,
            ),
            (
                "layout.offset.left",
                baseline.offset.left != placement.offset.left,
                placement.offset.left,
            ),
            (
                "layout.offset.top",
                baseline.offset.top != placement.offset.top,
                placement.offset.top,
            ),
            (
                "layout.offset.right",
                baseline.offset.right != placement.offset.right,
                placement.offset.right,
            ),
            (
                "layout.offset.bottom",
                baseline.offset.bottom != placement.offset.bottom,
                placement.offset.bottom,
            ),
        ];
        let mut changed = false;
        for (path, needs_write, value) in fields {
            if needs_write {
                changed |= set_selected_slot_semantic_value(
                    &mut document,
                    &self.selection,
                    path,
                    &value.to_string(),
                );
            }
        }
        if !changed {
            return Ok(false);
        }
        // Do not call twelve single-field session setters: that would split one
        // preset gesture into twelve undo entries and expose intermediate layouts.
        self.apply_document_edit_with_label(document, "Changed Anchors")?;
        Ok(true)
    }
}

fn geometry_unavailable(node_id: &str) -> UiAssetEditorSessionError {
    UiAssetEditorSessionError::CanvasAnchorGeometryUnavailable {
        node_id: node_id.to_string(),
    }
}

fn preset_anchors(index: usize) -> Option<([f32; 2], [f32; 2])> {
    const AXES: [(f32, f32); 4] = [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (0.0, 1.0)];
    if index >= 16 {
        return None;
    }
    let (min_x, max_x) = AXES[index % 4];
    let (min_y, max_y) = AXES[index / 4];
    Some(([min_x, min_y], [max_x, max_y]))
}

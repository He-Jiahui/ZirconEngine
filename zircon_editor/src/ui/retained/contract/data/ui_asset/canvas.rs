use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct UiAssetCanvasNodeData {
    pub node_id: SharedString,
    pub label: SharedString,
    pub kind: SharedString,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub depth: i32,
    pub z_index: i32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetCanvasSlotTargetData {
    pub label: SharedString,
    pub detail: SharedString,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPaletteDragData {
    pub target_preview_index: i32,
    pub target_action: SharedString,
    pub target_label: SharedString,
    pub slot_target_items: ModelRc<UiAssetCanvasSlotTargetData>,
    pub candidate_items: ModelRc<SharedString>,
    pub candidate_selected_index: i32,
    pub target_chooser_active: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPreviewCanvasData {
    pub width: f32,
    pub height: f32,
    pub items: ModelRc<UiAssetCanvasNodeData>,
    pub input_result:
        Option<std::sync::Arc<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult>>,
    pub input_enabled: bool,
    pub accepts_text_input: bool,
    pub source_surface_frame:
        Option<std::sync::Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>>,
    pub source_receipts: std::sync::Arc<[zircon_runtime::ui::v2::UiV2SourceFileReceipt]>,
}

/// The canvas maps source geometry once for both native paint and inverse input coordinates.
#[derive(Clone)]
pub(crate) struct UiAssetPreviewCanvasTransform {
    pub origin_x: f32,
    pub origin_y: f32,
    pub scale: f32,
    pub clip: super::super::FrameRect,
}

impl UiAssetPreviewCanvasData {
    pub(crate) fn transform(
        &self,
        panel: &super::super::TemplatePaneNodeData,
        pane_origin: &super::super::FrameRect,
    ) -> Option<UiAssetPreviewCanvasTransform> {
        self.source_surface_frame.as_ref()?;
        let width = panel.frame.width;
        let height = panel.frame.height;
        if ![
            panel.frame.x,
            panel.frame.y,
            width,
            height,
            self.width,
            self.height,
            pane_origin.x,
            pane_origin.y,
            pane_origin.width,
            pane_origin.height,
        ]
        .iter()
        .all(|value| value.is_finite())
            || [
                width,
                height,
                self.width,
                self.height,
                pane_origin.width,
                pane_origin.height,
            ]
            .iter()
            .any(|value| *value <= 0.0)
        {
            return None;
        }
        let scale = (width / self.width).min(height / self.height).min(1.0);
        if !scale.is_finite() || scale <= 0.0 {
            return None;
        }
        let canvas = super::super::FrameRect {
            x: pane_origin.x + panel.frame.x,
            y: pane_origin.y + panel.frame.y,
            width,
            height,
        };
        let intersect = |a: &super::super::FrameRect, b: &super::super::FrameRect| {
            let edges = [
                a.x,
                a.y,
                a.x + a.width,
                a.y + a.height,
                b.x,
                b.y,
                b.x + b.width,
                b.y + b.height,
            ];
            if !edges.iter().all(|value| value.is_finite()) {
                return None;
            }
            let x = a.x.max(b.x);
            let y = a.y.max(b.y);
            let width = (a.x + a.width).min(b.x + b.width) - x;
            let height = (a.y + a.height).min(b.y + b.height) - y;
            (width > 0.0 && height > 0.0).then_some(super::super::FrameRect {
                x,
                y,
                width,
                height,
            })
        };
        let mut clip = intersect(&canvas, pane_origin)?;
        if panel.has_clip_frame {
            let inherited = super::super::FrameRect {
                x: pane_origin.x + panel.clip_frame.x,
                y: pane_origin.y + panel.clip_frame.y,
                width: panel.clip_frame.width,
                height: panel.clip_frame.height,
            };
            clip = intersect(&clip, &inherited)?;
        }
        Some(UiAssetPreviewCanvasTransform {
            // Clipping does not move the source mapping origin.
            origin_x: canvas.x + (width - self.width * scale) * 0.5,
            origin_y: canvas.y + (height - self.height * scale) * 0.5,
            scale,
            clip,
        })
    }
}

impl UiAssetPreviewCanvasTransform {
    pub(crate) fn source_point(
        &self,
        x: f32,
        y: f32,
    ) -> Option<zircon_runtime_interface::ui::layout::UiPoint> {
        if !super::super::super::frame_geometry::contains_point(&self.clip, x, y) {
            return None;
        }
        Some(zircon_runtime_interface::ui::layout::UiPoint::new(
            (x - self.origin_x) / self.scale,
            (y - self.origin_y) / self.scale,
        ))
    }
}

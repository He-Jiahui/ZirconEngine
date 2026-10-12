use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use crate::ui::workbench::snapshot::ConsoleOutputSnapshot;

use super::super::TemplatePaneNodeData;

#[derive(Clone, Default)]
pub(crate) struct ProjectOverviewData {
    pub project_name: SharedString,
    pub project_root: SharedString,
    pub assets_root: SharedString,
    pub cache_root: SharedString,
    pub default_scene_uri: SharedString,
    pub catalog_revision: SharedString,
    pub folder_count: SharedString,
    pub asset_count: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct ConsolePaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub output: ConsoleOutputSnapshot,
}

#[derive(Clone, Default)]
pub(crate) struct AssetsActivityPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub render_source_frame: Option<Arc<UiSurfaceFrame>>,
}

#[derive(Clone, Default)]
pub(crate) struct AssetBrowserPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub render_source_frame: Option<Arc<UiSurfaceFrame>>,
}

#[derive(Clone, Default)]
pub(crate) struct ProjectOverviewPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct TemplateV2PaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct GeneratedBottomPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub status: SharedString,
    pub source_surface_frame: Option<Arc<UiSurfaceFrame>>,
    pub accepts_text_input: bool,
    pub source_width: f32,
    pub source_height: f32,
    pub input_result: Option<Arc<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult>>,
}

impl GeneratedBottomPaneData {
    pub(crate) fn input_transform(
        &self,
        content: &super::super::FrameRect,
    ) -> Option<super::super::UiAssetPreviewCanvasTransform> {
        self.source_surface_frame.as_ref()?;
        if ![
            content.x,
            content.y,
            content.width,
            content.height,
            self.source_width,
            self.source_height,
        ]
        .iter()
        .all(|value| value.is_finite())
            || [
                content.width,
                content.height,
                self.source_width,
                self.source_height,
            ]
            .iter()
            .any(|value| *value <= 0.0)
        {
            return None;
        }
        let clip = super::super::FrameRect {
            x: content.x,
            y: content.y,
            width: content.width.min(self.source_width),
            height: content.height.min(self.source_height),
        };
        Some(super::super::UiAssetPreviewCanvasTransform {
            origin_x: content.x,
            origin_y: content.y,
            scale: 1.0,
            clip,
        })
    }
}

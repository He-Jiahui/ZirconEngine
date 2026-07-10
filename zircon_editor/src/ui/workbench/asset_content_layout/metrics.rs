use zircon_runtime_interface::ui::design_tokens::{EditorControlTokens, EditorDensityTokens};
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::AssetContentSurfaceProfile;
use crate::ui::workbench::snapshot::AssetViewMode;

const BROWSER_HEADER_VERTICAL_GAP_COUNT: f32 = 2.0;
const LIST_ITEM_BORDER_EDGE_COUNT: f32 = 2.0;
const THUMBNAIL_FOLDER_ROW_MULTIPLIER: f32 = 2.0;
const THUMBNAIL_ITEM_ROW_MULTIPLIER: f32 = 3.0;
const CONTENT_PADDING_EDGE_COUNT: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AssetContentLayoutMetrics {
    pub(crate) viewport_offset_y: f32,
    pub(crate) row_x: f32,
    pub(crate) row_y: f32,
    pub(crate) row_gap: f32,
    pub(crate) folder_height: f32,
    pub(crate) item_height: f32,
}

impl AssetContentLayoutMetrics {
    pub(crate) fn for_surface(
        surface: AssetContentSurfaceProfile,
        view_mode: AssetViewMode,
    ) -> Self {
        let density = EditorDensityTokens::workbench_dense();
        let controls = EditorControlTokens::workbench_dense();
        let browser_header_height =
            density.row_height + density.gap_large * BROWSER_HEADER_VERTICAL_GAP_COUNT;
        let viewport_offset_y = match surface {
            AssetContentSurfaceProfile::Activity => 0.0,
            AssetContentSurfaceProfile::Browser => browser_header_height + controls.border_width,
        };
        let (folder_height, item_height) = match view_mode {
            AssetViewMode::List => (
                density.row_height + density.gap_small,
                density.row_height
                    + density.gap_medium
                    + controls.border_width * LIST_ITEM_BORDER_EDGE_COUNT,
            ),
            AssetViewMode::Thumbnail => (
                density.row_height * THUMBNAIL_FOLDER_ROW_MULTIPLIER + density.gap_small,
                density.row_height * THUMBNAIL_ITEM_ROW_MULTIPLIER + density.gap_small,
            ),
        };

        Self {
            viewport_offset_y,
            row_x: density.gap_medium,
            row_y: density.gap_medium,
            row_gap: density.gap_medium,
            folder_height,
            item_height,
        }
    }

    pub(crate) fn viewport_frame(self, pane_size: UiSize) -> UiFrame {
        UiFrame::new(
            0.0,
            self.viewport_offset_y,
            pane_size.width.max(0.0),
            (pane_size.height - self.viewport_offset_y).max(0.0),
        )
    }

    pub(crate) fn first_row_y(self) -> f32 {
        self.viewport_offset_y + self.row_y
    }

    pub(crate) fn row_width(self, pane_width: f32) -> f32 {
        (pane_width - self.row_x * CONTENT_PADDING_EDGE_COUNT).max(0.0)
    }

    pub(crate) fn list_height(self, folder_count: usize, item_count: usize) -> f32 {
        let row_count = folder_count + item_count;
        if row_count == 0 {
            return 0.0;
        }

        self.row_y * CONTENT_PADDING_EDGE_COUNT
            + folder_count as f32 * self.folder_height
            + item_count as f32 * self.item_height
            + (row_count as f32 - 1.0) * self.row_gap
    }
}

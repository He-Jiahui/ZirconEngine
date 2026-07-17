use zircon_runtime_interface::ui::layout::UiSize;

use super::{compact_file_like_display_name, RuntimeFileNameCompaction};
use super::{AssetContentLayoutMetrics, AssetContentSurfaceProfile};
use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::snapshot::AssetViewMode;

#[test]
fn activity_content_starts_at_the_panel_origin_without_a_browser_header_gap() {
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::List,
    );
    let viewport = metrics.viewport_frame(UiSize::new(226.0, 166.0));

    assert_eq!(viewport.y, 0.0);
    assert_eq!(viewport.height, 166.0);
    assert_eq!(metrics.first_row_y(), 8.0);
    assert_eq!(metrics.item_height, 38.0);
    assert_eq!(metrics.row_width(226.0), 210.0);
}

#[test]
fn browser_list_geometry_matches_the_painted_table_header_and_rows() {
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Browser,
        AssetViewMode::List,
    );
    let viewport = metrics.viewport_frame(UiSize::new(420.0, 220.0));

    assert_eq!(viewport.y, 24.0);
    assert_eq!(viewport.height, 196.0);
    assert_eq!(metrics.first_row_y(), 24.0);
    assert_eq!(metrics.folder_height, 28.0);
    assert_eq!(metrics.item_height, 28.0);
    assert_eq!(metrics.row_gap, 0.0);
}

#[test]
fn thumbnail_geometry_is_derived_from_the_same_dense_token_profile() {
    let metrics = AssetContentLayoutMetrics::for_surface(
        AssetContentSurfaceProfile::Activity,
        AssetViewMode::Thumbnail,
    );

    assert_eq!(metrics.folder_height, 60.0);
    assert_eq!(metrics.item_height, 88.0);
    assert_eq!(metrics.list_height(1, 2), 268.0);
}

#[test]
fn shared_content_name_compaction_preserves_the_file_extension_at_runtime_width() {
    let font_size = 13.333_333;
    let max_width = 112.0;
    let compact = compact_file_like_display_name(
        "workbench_extension_accessibility_workspace.zui",
        "zui",
        RuntimeFileNameCompaction {
            max_width,
            font_size,
            min_prefix_chars: 4,
            min_tail_stem_chars: 3,
            preferred_tail_stem_chars: 6,
        },
    );

    assert!(compact.contains("..."));
    assert!(compact.ends_with(".zui"));
    assert!(measure_runtime_text_width(&compact, font_size) <= max_width + 0.01);
}

#[test]
fn shared_content_name_compaction_does_not_linearly_shape_every_prefix() {
    let source = include_str!("text.rs");
    assert!(!source.contains("for prefix_count in"));
    assert!(source.contains("largest_fitting_candidate"));
}

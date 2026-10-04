use super::*;

fn metrics() -> WorkbenchTableColumnMetrics {
    WorkbenchTableColumnMetrics {
        ratios: [0.36, 0.27, 0.19, 0.18],
        min_widths: [100.0, 60.0, 60.0, 80.0],
        drop_order: [3, 2, 1, 0],
    }
}

#[test]
fn narrow_table_allocation_keeps_the_name_column_inside_real_available_width() {
    let layout = allocate_table_columns(0.5, TableColumnLayoutTier::Regular, metrics());

    assert_eq!(layout.width(0), 0.5);
    assert_eq!(layout.width(1), 0.0);
    assert_eq!(layout.width(2), 0.0);
    assert_eq!(layout.width(3), 0.0);
    assert_eq!(layout.x_offset(1), 0.5);
}

#[test]
fn non_finite_table_width_has_no_fallback_column_extent() {
    let layout = allocate_table_columns(f32::NAN, TableColumnLayoutTier::Regular, metrics());

    assert_eq!(layout.width(0), 0.0);
    assert_eq!(layout.width(1), 0.0);
    assert_eq!(layout.width(2), 0.0);
    assert_eq!(layout.width(3), 0.0);
}

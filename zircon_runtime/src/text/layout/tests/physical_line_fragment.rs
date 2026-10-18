use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::shaping::DirectTextShapeRunProvider;
use crate::text::{TextRange, TextStyle};

use super::super::horizontal_line_fragment::HorizontalLineFragmentGeometry;
use super::{
    resolve_physical_line_visual_order, shape_horizontal_physical_line_fragment_with_provider,
};

// 混合方向样例只核对局部簇层级和排列，不替代完整布局方向验收。
#[test]
fn physical_fragment_resolves_one_post_wrap_visual_cluster_order() {
    let text = "abc אבג";

    let order = resolve_physical_line_visual_order(text, TextDirection::LeftToRight)
        .expect("mixed physical line has one canonical visual order");

    assert_eq!(order.logical_levels, vec![0, 0, 0, 0, 1, 1, 1]);
    assert_eq!(order.visual_indices, vec![0, 1, 2, 3, 6, 5, 4]);
}

// 本地文本与绝对源范围分开传入，核对塑形回执保留调用方坐标。
#[test]
fn physical_fragment_keeps_the_callers_absolute_source_range() {
    let mut provider = DirectTextShapeRunProvider::default();
    let fragment = shape_horizontal_physical_line_fragment_with_provider(
        "world",
        &TextStyle::default(),
        TextDirection::LeftToRight,
        TextRange { start: 11, end: 16 },
        &mut provider,
    )
    .into_result()
    .expect("shape final physical fragment");

    assert_eq!(
        fragment.shaped().source_range,
        TextRange { start: 11, end: 16 }
    );
    assert_eq!(fragment.grapheme_advances().len(), 5);
    assert_eq!(fragment.glyph_clusters().len(), 5);
    assert_eq!(
        fragment
            .visual_order()
            .expect("non-empty physical line stores its visual-order receipt")
            .visual_indices,
        vec![0, 1, 2, 3, 4]
    );
}

// 重新投影同一 Arc，核对共享身份及几何一致；不度量分配或塑形调用次数。
#[test]
fn horizontal_geometry_keeps_one_shaped_run_for_metrics_and_advances() {
    let mut provider = DirectTextShapeRunProvider::default();
    let style = TextStyle::default();
    let fragment = shape_horizontal_physical_line_fragment_with_provider(
        "world",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 11, end: 16 },
        &mut provider,
    )
    .into_result()
    .expect("shape final physical fragment");

    let geometry =
        HorizontalLineFragmentGeometry::from_shaped(Arc::clone(fragment.shaped()), "world", &style)
            .into_result()
            .expect("valid horizontal geometry");

    assert!(Arc::ptr_eq(geometry.shaped(), fragment.shaped()));
    assert_eq!(geometry.metrics(), fragment.metrics());
    assert_eq!(geometry.grapheme_advances(), fragment.grapheme_advances());
    assert_eq!(geometry.glyph_clusters(), fragment.glyph_clusters());
}

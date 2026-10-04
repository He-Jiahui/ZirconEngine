use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{HostControlMetrics, METRICS};

fn rect() -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 40.0,
    }
}

#[test]
fn paper_default_corner_radius_projects_from_shared_host_metrics() {
    let metrics = HostControlMetrics {
        radius_control: 6.0,
        ..METRICS
    };

    assert_eq!(
        paper_corner_radius_from_host(&TemplatePaneNodeData::default(), &rect(), metrics),
        6.0
    );
}

#[test]
fn paper_square_and_declared_radius_keep_component_semantics() {
    let metrics = HostControlMetrics {
        radius_control: 6.0,
        ..METRICS
    };
    let square = TemplatePaneNodeData {
        component_variant: "square".into(),
        ..TemplatePaneNodeData::default()
    };
    let declared = TemplatePaneNodeData {
        corner_radius: 9.0,
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        paper_corner_radius_from_host(&square, &rect(), metrics),
        0.0
    );
    assert_eq!(
        paper_corner_radius_from_host(&declared, &rect(), metrics),
        9.0
    );
}

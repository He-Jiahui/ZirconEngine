use std::rc::Rc;

use super::*;
use crate::ui::retained_host::primitives::VecModel;

#[test]
fn document_overflow_uses_published_anchor_and_hidden_frame_indices() {
    let mut presentation = overflow_presentation("document", 2);
    presentation.host_scene_data.document_dock.region_frame = frame(100.0, 40.0, 320.0, 260.0);
    presentation.host_scene_data.document_dock.overflow_frame = frame(280.0, 3.0, 28.0, 28.0);
    let state = open_state("document");

    let projection = host_dock_overflow_projection(&presentation, &state)
        .expect("published document overflow projection");
    assert_eq!(projection.anchor_frame, frame(380.0, 43.0, 28.0, 28.0));
    assert_eq!(host_dock_overflow_hidden_indices(&projection), vec![1, 2]);

    let popup = host_dock_overflow_popup_frame_with_state(&presentation, &state)
        .expect("hidden tabs should expose a popup");
    let row = host_dock_overflow_row_frame_with_state(&presentation, &popup, 0, &state);
    let hit = host_dock_overflow_row_hit_in_popup(
        &presentation,
        &popup,
        &state,
        row.x + 2.0,
        row.y + 2.0,
    )
    .expect("first hidden tab row hit");
    assert_eq!(hit.tab_index, 1);
}

#[test]
fn left_drawer_overflow_anchor_includes_activity_rail_width() {
    let mut presentation = overflow_presentation("left", 1);
    let dock = &mut presentation.host_scene_data.left_dock;
    dock.region_frame = frame(10.0, 20.0, 240.0, 300.0);
    dock.rail_before_panel = true;
    dock.rail_width_px = 36.0;
    dock.overflow_frame = frame(150.0, 2.0, 28.0, 28.0);
    let state = open_state("left");

    let projection = host_dock_overflow_projection(&presentation, &state)
        .expect("left drawer overflow projection");

    assert!(projection.drawer);
    assert_eq!(projection.anchor_frame, frame(196.0, 22.0, 28.0, 28.0));
}

#[test]
fn scroll_offset_is_bounded_by_the_same_content_extent_used_for_rows() {
    let presentation = overflow_presentation("document", 24);
    let state = open_state("document");
    let popup = host_dock_overflow_popup_frame_with_state(&presentation, &state)
        .expect("bounded overflow popup");

    let end = host_dock_overflow_scroll_offset_for_delta(&presentation, &popup, &state, f32::MAX);
    let repeated = host_dock_overflow_scroll_offset_for_delta(
        &presentation,
        &popup,
        &HostDockOverflowMenuStateData {
            scroll_offset: end,
            ..state
        },
        100.0,
    );

    assert!(end > 0.0);
    assert_eq!(repeated, end);
}

fn overflow_presentation(surface_key: &str, hidden_count: usize) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_layout.center_band_frame = frame(0.0, 0.0, 640.0, 420.0);
    presentation.host_layout.status_bar_frame = frame(0.0, 400.0, 640.0, 20.0);
    let tabs = model_rc(
        (0..=hidden_count)
            .map(|index| TabData {
                id: format!("tab-{index}").into(),
                title: format!("Tab {index}").into(),
                active: index == 0,
                ..TabData::default()
            })
            .collect(),
    );
    let frames = model_rc(
        tabs.iter()
            .enumerate()
            .map(|(index, tab)| HostChromeTabData {
                control_id: format!("DockTab{index}").into(),
                tab: tab.clone(),
                frame: if index == 0 {
                    frame(8.0, 3.0, 100.0, 28.0)
                } else {
                    FrameRect::default()
                },
                ..HostChromeTabData::default()
            })
            .collect(),
    );
    match surface_key {
        "left" => {
            presentation.host_scene_data.left_dock.surface_key = "left".into();
            presentation.host_scene_data.left_dock.tabs = tabs;
            presentation.host_scene_data.left_dock.tab_frames = frames;
        }
        _ => {
            presentation.host_scene_data.document_dock.surface_key = "document".into();
            presentation.host_scene_data.document_dock.region_frame =
                frame(0.0, 40.0, 640.0, 360.0);
            presentation.host_scene_data.document_dock.overflow_frame =
                frame(600.0, 2.0, 28.0, 28.0);
            presentation.host_scene_data.document_dock.tabs = tabs;
            presentation.host_scene_data.document_dock.tab_frames = frames;
        }
    }
    presentation
}

fn open_state(surface_key: &str) -> HostDockOverflowMenuStateData {
    HostDockOverflowMenuStateData {
        open: true,
        surface_key: surface_key.into(),
        ..HostDockOverflowMenuStateData::default()
    }
}

fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

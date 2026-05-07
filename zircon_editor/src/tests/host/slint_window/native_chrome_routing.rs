use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::slint_host::{
    FrameRect, HostChromeControlFrameData, HostChromeTabData, HostDocumentDockSurfaceData,
    HostSideDockSurfaceData, HostWindowLayoutData, TabData, UiHostContext, UiHostWindow,
};
use slint::{ModelRc, PhysicalSize, VecModel};

#[test]
fn native_host_activity_rail_click_wins_over_overlapping_drawer_header() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData {
        surface_key: "left".into(),
        region_frame: host_frame(0.0, 58.0, 120.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 120.0, 138.0),
        tab_frames: model_rc(vec![chrome_tab(
            "left.header.project",
            "Project",
            0.0,
            0.0,
            120.0,
            80.0,
        )]),
        rail_before_panel: true,
        rail_width_px: 34.0,
        rail_button_frames: model_rc(vec![control_frame(
            "ActivityRailButton0",
            3.0,
            38.0,
            30.0,
            30.0,
        )]),
        ..HostSideDockSurfaceData::default()
    };
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData::default();
    ui.set_host_presentation(presentation);

    let rail_clicks = Rc::new(RefCell::new(Vec::new()));
    let drawer_clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let rail_clicks = rail_clicks.clone();
        ui.global::<UiHostContext>()
            .on_activity_rail_pointer_clicked(move |side, x, y| {
                rail_clicks.borrow_mut().push((side.to_string(), x, y));
            });
    }
    {
        let drawer_clicks = drawer_clicks.clone();
        ui.global::<UiHostContext>()
            .on_drawer_header_pointer_clicked(move |surface_key, index, _, _, _, _| {
                drawer_clicks
                    .borrow_mut()
                    .push((surface_key.to_string(), index));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(18.0, 58.0 + 48.0);

    assert!(result.request_redraw());
    assert_eq!(
        rail_clicks.borrow().as_slice(),
        [("left".to_string(), 18.0, 48.0)]
    );
    assert_eq!(
        drawer_clicks.borrow().as_slice(),
        [],
        "collapsed side rail pixels must not be routed as drawer header tabs"
    );
}

fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, width, height - 82.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: host_frame(0.0, 58.0, 120.0, height - 82.0),
        document_region_frame: host_frame(120.0, 58.0, width - 140.0, height - 82.0),
        ..HostWindowLayoutData::default()
    }
}

fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn control_frame(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeControlFrameData {
    HostChromeControlFrameData {
        control_id: control_id.into(),
        frame: host_frame(x, y, width, height),
    }
}

fn chrome_tab(
    control_id: &str,
    title: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeTabData {
    HostChromeTabData {
        control_id: control_id.into(),
        tab: TabData {
            id: control_id.into(),
            title: title.into(),
            active: true,
            closeable: false,
            ..TabData::default()
        },
        frame: host_frame(x, y, width, height),
        close_frame: FrameRect::default(),
    }
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

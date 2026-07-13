use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize, VecModel};
use crate::ui::retained_host::{
    paint_host_frame_for_test, FrameRect, HostPageOverflowMenuStateData, TabData, UiHostContext,
    UiHostWindow,
};

const HOST_PAGE_OVERFLOW_KEYBOARD_SCREENSHOT: &str =
    "editor-window-m3-host-page-overflow-keyboard-640x420.png";

#[test]
fn host_page_overflow_keyboard_navigates_searches_and_accepts_hidden_pages() {
    let ui = host_page_overflow_window();
    let clicks = capture_host_page_clicks(&ui);

    let first = ui.dispatch_native_popup_arrow_down_for_test();
    assert!(first.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, 1);

    let second = ui.dispatch_native_popup_arrow_down_for_test();
    assert!(second.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, 2);

    let search = ui.dispatch_native_popup_text_for_test("tags");
    assert!(search.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, 3);

    let accept = ui.dispatch_native_popup_enter_for_test();
    assert!(accept.request_redraw());
    assert!(accept.requires_frame_update());
    assert!(!overflow_state(&ui).open);
    assert_eq!(clicks.borrow().as_slice(), [3]);
}

#[test]
fn host_page_overflow_keyboard_wraps_backward_and_escape_closes_without_activation() {
    let ui = host_page_overflow_window();
    let clicks = capture_host_page_clicks(&ui);

    let previous = ui.dispatch_native_popup_arrow_up_for_test();
    assert!(previous.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, 3);

    let cancel = ui.dispatch_native_popup_escape_for_test();
    assert!(cancel.request_redraw());
    assert!(cancel.requires_frame_update());
    assert!(!overflow_state(&ui).open);
    assert!(clicks.borrow().is_empty());
}

#[test]
#[ignore = "writes host-page overflow keyboard screenshot under docs/tests/editor"]
fn capture_host_page_overflow_keyboard_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");
    let ui = host_page_overflow_window_at(640, 420);
    ui.dispatch_native_popup_arrow_down_for_test();
    ui.dispatch_native_popup_arrow_down_for_test();

    let presentation = ui.get_host_presentation();
    let pixels = paint_host_frame_for_test(640, 420, &presentation);
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor");
    std::fs::create_dir_all(&output_dir).expect("editor screenshot directory should exist");
    let output_path = output_dir.join(HOST_PAGE_OVERFLOW_KEYBOARD_SCREENSHOT);
    image::save_buffer_with_format(
        &output_path,
        &pixels,
        640,
        420,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("host-page overflow keyboard screenshot should be written as PNG");
    assert!(output_path.exists());
}

fn host_page_overflow_window() -> UiHostWindow {
    host_page_overflow_window_at(420, 260)
}

fn host_page_overflow_window_at(width: u32, height: u32) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(width, height));
    let mut presentation = ui.get_host_presentation();
    presentation.host_scene_data.page_chrome.tabs = model_rc(vec![
        tab("workbench", "Workbench", true),
        tab("assets", "Assets", false),
        tab("animation", "Animation", false),
        tab("tags", "Tags", false),
    ]);
    presentation.host_scene_data.page_chrome.overflow_frame = frame(188.0, 29.0, 34.0, 28.0);
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![1, 2, 3];
    let overflow_state = HostPageOverflowMenuStateData {
        open: true,
        hovered_page_index: -1,
    };
    presentation.host_page_overflow_menu_state = overflow_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(overflow_state);
    ui
}

fn capture_host_page_clicks(ui: &UiHostWindow) -> Rc<RefCell<Vec<i32>>> {
    let clicks = Rc::new(RefCell::new(Vec::new()));
    let callback_clicks = clicks.clone();
    ui.global::<UiHostContext>().on_host_page_pointer_clicked(
        move |index, _tab_x, _tab_width, _point_x, _point_y| {
            callback_clicks.borrow_mut().push(index);
        },
    );
    clicks
}

fn overflow_state(ui: &UiHostWindow) -> HostPageOverflowMenuStateData {
    ui.get_host_presentation().host_page_overflow_menu_state
}

fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}

fn tab(id: &str, title: &str, active: bool) -> TabData {
    TabData {
        id: id.into(),
        title: title.into(),
        active,
        ..TabData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

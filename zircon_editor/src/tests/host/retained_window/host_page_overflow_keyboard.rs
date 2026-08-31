use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, PhysicalSize, VecModel};
use crate::ui::retained_host::{
    menu_popup_text_width, paint_host_frame_for_test, FrameRect, HostPageOverflowMenuStateData,
    TabData, UiHostContext, UiHostWindow,
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
fn host_page_overflow_keyboard_scrolls_a_long_hidden_tab_list_into_view() {
    let ui = host_page_overflow_window_with_hidden_tab_count(420, 260, 12);

    for _ in 0..7 {
        let result = ui.dispatch_native_popup_arrow_down_for_test();
        assert!(result.request_redraw());
    }

    let state = overflow_state(&ui);
    assert_eq!(state.hovered_page_index, 7);
    assert!(
        state.scroll_offset > 0.0,
        "moving a keyboard-selected hidden tab below the bounded viewport should scroll it into view"
    );
}

#[test]
fn host_page_overflow_pointer_scrolls_a_long_hidden_tab_list() {
    let ui = host_page_overflow_window_with_hidden_tab_count(420, 260, 12);

    let result = ui.dispatch_native_pointer_scroll_for_test(200.0, 80.0, 84.0);

    assert!(result.request_redraw());
    assert!(
        overflow_state(&ui).scroll_offset > 0.0,
        "a wheel event inside the bounded overflow popup should advance its own list"
    );
}

#[test]
fn host_page_overflow_pointer_move_tracks_and_clears_the_hovered_visible_row() {
    let ui = host_page_overflow_window_with_hidden_tab_count(420, 260, 12);

    let enter = ui.dispatch_native_pointer_move_for_test(200.0, 80.0);

    assert!(enter.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, 1);

    let leave = ui.dispatch_native_pointer_move_for_test(400.0, 220.0);

    assert!(leave.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, -1);
}

#[test]
fn host_page_overflow_pointer_scroll_retargets_hover_under_a_stationary_cursor() {
    let ui = host_page_overflow_window_with_hidden_tab_count(420, 260, 12);
    ui.dispatch_native_pointer_move_for_test(200.0, 80.0);

    let scroll = ui.dispatch_native_pointer_scroll_for_test(200.0, 80.0, 84.0);

    assert!(scroll.request_redraw());
    assert_eq!(overflow_state(&ui).hovered_page_index, 4);
}

#[test]
fn host_page_overflow_pointer_hit_activates_the_scrolled_visible_row() {
    let ui = host_page_overflow_window_with_hidden_tab_count(420, 260, 12);
    let clicks = capture_host_page_clicks(&ui);

    let scroll = ui.dispatch_native_pointer_scroll_for_test(200.0, 80.0, 84.0);
    assert!(scroll.request_redraw());

    let select = ui.dispatch_native_primary_press_for_test(200.0, 80.0);
    assert!(select.request_redraw());
    assert!(select.requires_frame_update());
    assert!(!overflow_state(&ui).open);
    assert_eq!(clicks.borrow().as_slice(), [4]);
}

#[test]
fn host_page_overflow_active_tab_uses_a_thin_indicator_instead_of_a_full_selected_fill() {
    let ui = host_page_overflow_window();
    let mut presentation = ui.get_host_presentation();
    presentation.host_scene_data.page_chrome.tabs = model_rc(vec![
        tab("workbench", "Workbench", false),
        tab("assets", "Assets", true),
        tab("animation", "Animation", false),
        tab("tags", "Tags", false),
    ]);
    ui.set_host_presentation(presentation);

    let pixels = paint_host_frame_for_test(420, 260, &ui.get_host_presentation());
    let indicator = rgba_at(&pixels, 420, 56, 80);
    let row_surface = rgba_at(&pixels, 420, 64, 80);

    assert_ne!(
        indicator, row_surface,
        "the active-page indicator must stay a distinct thin accent instead of repainting the full row"
    );
}

#[test]
#[ignore = "writes host-page overflow keyboard screenshot under docs/tests/editor"]
fn capture_host_page_overflow_keyboard_visual_artifact() {
    std::env::set_var("SLINT_BACKEND", "software");
    let ui = host_page_overflow_window_with_hidden_tab_count(640, 420, 16);
    ui.dispatch_native_pointer_scroll_for_test(200.0, 80.0, 84.0);
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
    presentation
        .host_scene_data
        .page_chrome
        .overflow_widest_title_width_px = ["Assets", "Animation", "Tags"]
        .into_iter()
        .map(menu_popup_text_width)
        .fold(0.0_f32, f32::max);
    let overflow_state = HostPageOverflowMenuStateData {
        open: true,
        hovered_page_index: -1,
        scroll_offset: 0.0,
    };
    presentation.host_page_overflow_menu_state = overflow_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>()
        .set_host_page_overflow_menu_state(overflow_state);
    ui
}

fn host_page_overflow_window_with_hidden_tab_count(
    width: u32,
    height: u32,
    hidden_tab_count: usize,
) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(width, height));
    let mut presentation = ui.get_host_presentation();
    let mut tabs = vec![tab("workbench", "Workbench", true)];
    for index in 1..=hidden_tab_count {
        tabs.push(tab(
            &format!("page-{index}"),
            &format!("Long Hidden Page {index}"),
            false,
        ));
    }
    presentation.host_scene_data.page_chrome.tabs = model_rc(tabs);
    presentation.host_scene_data.page_chrome.overflow_frame = frame(188.0, 29.0, 34.0, 28.0);
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (1..=hidden_tab_count).collect();
    presentation
        .host_scene_data
        .page_chrome
        .overflow_widest_title_width_px = (1..=hidden_tab_count)
        .map(|index| menu_popup_text_width(&format!("Long Hidden Page {index}")))
        .fold(0.0_f32, f32::max);
    presentation.host_layout.status_bar_frame =
        frame(0.0, height as f32 - 24.0, width as f32, 24.0);
    let overflow_state = HostPageOverflowMenuStateData {
        open: true,
        hovered_page_index: -1,
        scroll_offset: 0.0,
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
    ui.global::<UiHostContext>()
        .on_host_page_pointer_clicked(move |index, _close| {
            callback_clicks.borrow_mut().push(index);
        });
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

fn rgba_at(pixels: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    pixels[offset..offset + 4]
        .try_into()
        .expect("pixel sample should stay inside the captured test frame")
}

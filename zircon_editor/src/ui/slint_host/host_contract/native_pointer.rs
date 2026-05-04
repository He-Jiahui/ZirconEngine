use slint::{Model, SharedString};
use zircon_runtime_interface::ui::surface::UiPointerButton;

use super::data::{FrameRect, HostWindowPresentationData};
use super::globals::{PaneSurfaceHostContext, UiHostContext};
use super::window::UiHostWindow;

const HOST_POINTER_DOWN: i32 = 0;
const HOST_POINTER_MOVE: i32 = 1;
const HOST_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_DOWN: i32 = 0;
const VIEWPORT_POINTER_MOVE: i32 = 1;
const VIEWPORT_POINTER_UP: i32 = 2;
const VIEWPORT_POINTER_SCROLL: i32 = 3;
const VIEWPORT_POINTER_BUTTON_NONE: i32 = 0;
const VIEWPORT_POINTER_BUTTON_PRIMARY: i32 = 1;
const VIEWPORT_POINTER_BUTTON_SECONDARY: i32 = 2;
const VIEWPORT_POINTER_BUTTON_MIDDLE: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NativePointerDispatchResult {
    request_redraw: bool,
}

impl NativePointerDispatchResult {
    pub(super) fn idle() -> Self {
        Self {
            request_redraw: false,
        }
    }

    pub(super) fn dispatched() -> Self {
        Self {
            request_redraw: true,
        }
    }

    pub(crate) fn request_redraw(self) -> bool {
        self.request_redraw
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum NativePointerButtonState {
    Pressed,
    Released,
}

pub(super) fn dispatch_native_pointer_move(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let presentation = ui.get_host_presentation();
    if menu_handles_point(&presentation, x, y) {
        ui.global::<UiHostContext>().invoke_menu_pointer_moved(x, y);
        return NativePointerDispatchResult::dispatched();
    }

    if let Some(pointer) = route_pointer_to_pane(&presentation, x, y) {
        let pane_host = ui.global::<PaneSurfaceHostContext>();
        match pointer.target {
            PanePointerTarget::Hierarchy => pane_host.invoke_hierarchy_pointer_moved(
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Welcome => pane_host.invoke_welcome_recent_pointer_moved(
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetTree(mode) => pane_host.invoke_asset_tree_pointer_moved(
                mode,
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetContent(mode) => pane_host.invoke_asset_content_pointer_moved(
                mode,
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetReference(mode, list_kind) => {
                pane_host.invoke_asset_reference_pointer_moved(
                    mode,
                    list_kind,
                    pointer.local_x,
                    pointer.local_y,
                    pointer.width,
                    pointer.height,
                )
            }
            PanePointerTarget::Viewport(surface_key) => pane_host.invoke_viewport_pointer_event(
                VIEWPORT_POINTER_MOVE,
                VIEWPORT_POINTER_BUTTON_NONE,
                pointer.local_x,
                pointer.local_y,
                0.0,
            ),
            PanePointerTarget::Console
            | PanePointerTarget::Inspector
            | PanePointerTarget::BrowserAssetDetails
            | PanePointerTarget::UiAsset
            | PanePointerTarget::Other => {}
        }
        return NativePointerDispatchResult::dispatched();
    }

    NativePointerDispatchResult::idle()
}

pub(super) fn dispatch_native_pointer_button(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    button: Option<UiPointerButton>,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    let button = button.unwrap_or(UiPointerButton::Primary);
    let presentation = ui.get_host_presentation();
    let Some(button_id) = viewport_button_id(button) else {
        return NativePointerDispatchResult::idle();
    };
    if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
        if menu_handles_point(&presentation, x, y) {
            ui.global::<UiHostContext>().invoke_menu_pointer_clicked(x, y);
            return NativePointerDispatchResult::dispatched();
        }
    }

    if let Some(route) = route_top_level_chrome(&presentation, x, y) {
        if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
            dispatch_chrome_press(ui, route, x, y);
            return NativePointerDispatchResult::dispatched();
        }
    }

    if let Some(pointer) = route_pointer_to_pane(&presentation, x, y) {
        let kind = match state {
            NativePointerButtonState::Pressed => VIEWPORT_POINTER_DOWN,
            NativePointerButtonState::Released => VIEWPORT_POINTER_UP,
        };
        let host_kind = match state {
            NativePointerButtonState::Pressed => HOST_POINTER_DOWN,
            NativePointerButtonState::Released => HOST_POINTER_UP,
        };
        let pane_host = ui.global::<PaneSurfaceHostContext>();
        match pointer.target {
            PanePointerTarget::Hierarchy => {
                pane_host.invoke_hierarchy_pointer_event(
                    host_kind,
                    button_id,
                    pointer.local_x,
                    pointer.local_y,
                    pointer.width,
                    pointer.height,
                );
                if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
                    pane_host.invoke_hierarchy_pointer_clicked(
                        pointer.local_x,
                        pointer.local_y,
                        pointer.width,
                        pointer.height,
                    );
                }
            }
            PanePointerTarget::Welcome => {
                if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
                    pane_host.invoke_welcome_recent_pointer_clicked(
                        pointer.local_x,
                        pointer.local_y,
                        pointer.width,
                        pointer.height,
                    );
                }
            }
            PanePointerTarget::AssetTree(mode) => {
                if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
                    pane_host.invoke_asset_tree_pointer_clicked(
                        mode,
                        pointer.local_x,
                        pointer.local_y,
                        pointer.width,
                        pointer.height,
                    );
                }
            }
            PanePointerTarget::AssetContent(mode) => {
                pane_host.invoke_asset_content_pointer_event(
                    mode.clone(),
                    host_kind,
                    button_id,
                    pointer.local_x,
                    pointer.local_y,
                    pointer.width,
                    pointer.height,
                );
                if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
                    pane_host.invoke_asset_content_pointer_clicked(
                        mode,
                        pointer.local_x,
                        pointer.local_y,
                        pointer.width,
                        pointer.height,
                    );
                }
            }
            PanePointerTarget::AssetReference(mode, list_kind) => {
                pane_host.invoke_asset_reference_pointer_event(
                    mode.clone(),
                    list_kind.clone(),
                    host_kind,
                    button_id,
                    pointer.local_x,
                    pointer.local_y,
                    pointer.width,
                    pointer.height,
                );
                if state == NativePointerButtonState::Pressed && button == UiPointerButton::Primary {
                    pane_host.invoke_asset_reference_pointer_clicked(
                        mode,
                        list_kind,
                        pointer.local_x,
                        pointer.local_y,
                        pointer.width,
                        pointer.height,
                    );
                }
            }
            PanePointerTarget::Viewport(_) => pane_host.invoke_viewport_pointer_event(
                kind,
                button_id,
                pointer.local_x,
                pointer.local_y,
                0.0,
            ),
            PanePointerTarget::Console
            | PanePointerTarget::Inspector
            | PanePointerTarget::BrowserAssetDetails
            | PanePointerTarget::UiAsset
            | PanePointerTarget::Other => {}
        }
        return NativePointerDispatchResult::dispatched();
    }

    NativePointerDispatchResult::idle()
}

pub(super) fn dispatch_native_pointer_scroll(
    ui: &UiHostWindow,
    x: f32,
    y: f32,
    delta: f32,
) -> NativePointerDispatchResult {
    let presentation = ui.get_host_presentation();
    if menu_handles_point(&presentation, x, y) {
        ui.global::<UiHostContext>()
            .invoke_menu_pointer_scrolled(x, y, delta);
        return NativePointerDispatchResult::dispatched();
    }

    if let Some(pointer) = route_pointer_to_pane(&presentation, x, y) {
        let pane_host = ui.global::<PaneSurfaceHostContext>();
        match pointer.target {
            PanePointerTarget::Hierarchy => pane_host.invoke_hierarchy_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Welcome => pane_host.invoke_welcome_recent_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Console => pane_host.invoke_console_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::Inspector => pane_host.invoke_inspector_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::BrowserAssetDetails => pane_host.invoke_browser_asset_details_pointer_scrolled(
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetTree(mode) => pane_host.invoke_asset_tree_pointer_scrolled(
                mode,
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetContent(mode) => pane_host.invoke_asset_content_pointer_scrolled(
                mode,
                pointer.local_x,
                pointer.local_y,
                delta,
                pointer.width,
                pointer.height,
            ),
            PanePointerTarget::AssetReference(mode, list_kind) => {
                pane_host.invoke_asset_reference_pointer_scrolled(
                    mode,
                    list_kind,
                    pointer.local_x,
                    pointer.local_y,
                    delta,
                    pointer.width,
                    pointer.height,
                )
            }
            PanePointerTarget::Viewport(_) => pane_host.invoke_viewport_pointer_event(
                VIEWPORT_POINTER_SCROLL,
                VIEWPORT_POINTER_BUTTON_NONE,
                pointer.local_x,
                pointer.local_y,
                delta,
            ),
            PanePointerTarget::UiAsset | PanePointerTarget::Other => {}
        }
        return NativePointerDispatchResult::dispatched();
    }

    NativePointerDispatchResult::idle()
}

fn dispatch_chrome_press(ui: &UiHostWindow, route: ChromePointerRoute, x: f32, y: f32) {
    let host = ui.global::<UiHostContext>();
    match route {
        ChromePointerRoute::ActivityRail { side, local_x, local_y } => {
            host.invoke_activity_rail_pointer_clicked(side, local_x, local_y);
        }
        ChromePointerRoute::HostPageTab {
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
        } => host.invoke_host_page_pointer_clicked(index as i32, tab_x, tab_width, local_x, local_y),
        ChromePointerRoute::DocumentTab {
            surface_key,
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
            close,
        } => {
            if close {
                host.invoke_document_tab_close_pointer_clicked(
                    surface_key,
                    index as i32,
                    tab_x,
                    tab_width,
                    local_x,
                    local_y,
                );
            } else {
                host.invoke_document_tab_pointer_clicked(
                    surface_key,
                    index as i32,
                    tab_x,
                    tab_width,
                    local_x,
                    local_y,
                );
            }
        }
        ChromePointerRoute::DrawerHeaderTab {
            surface_key,
            index,
            tab_x,
            tab_width,
            local_x,
            local_y,
        } => host.invoke_drawer_header_pointer_clicked(
            surface_key,
            index as i32,
            tab_x,
            tab_width,
            local_x,
            local_y,
        ),
        ChromePointerRoute::FloatingWindowHeader => {
            host.invoke_floating_window_header_pointer_clicked(x, y);
        }
        ChromePointerRoute::Resize => host.invoke_host_resize_pointer_event(HOST_POINTER_DOWN, x, y),
    }
}

fn route_top_level_chrome(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
    for splitter in [
        &scene.resize_layer.left_splitter_frame,
        &scene.resize_layer.right_splitter_frame,
        &scene.resize_layer.bottom_splitter_frame,
    ] {
        if contains(splitter, x, y) {
            return Some(ChromePointerRoute::Resize);
        }
    }

    if let Some(route) = route_document_tabs("document", &scene.document_dock.header_frame, &scene.document_dock.tab_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_drawer_header("left", &scene.left_dock.region_frame, &scene.left_dock.header_frame, &scene.left_dock.tab_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_drawer_header("right", &scene.right_dock.region_frame, &scene.right_dock.header_frame, &scene.right_dock.tab_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_drawer_header("bottom", &scene.bottom_dock.region_frame, &scene.bottom_dock.header_frame, &scene.bottom_dock.tab_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_host_page_tabs(&scene.page_chrome.tab_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_activity_rail(&scene.left_dock.region_frame, true, scene.left_dock.rail_width_px, &scene.left_dock.rail_button_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_activity_rail(&scene.right_dock.region_frame, false, scene.right_dock.rail_width_px, &scene.right_dock.rail_button_frames, x, y) {
        return Some(route);
    }

    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if contains(&translated(&window.header_frame, window.frame.x, window.frame.y), x, y) {
            if let Some(route) = route_document_tabs(
                window.window_id.as_str(),
                &translated(&window.header_frame, window.frame.x, window.frame.y),
                &window.tab_frames,
                x,
                y,
            ) {
                return Some(route);
            }
            return Some(ChromePointerRoute::FloatingWindowHeader);
        }
    }

    None
}

fn route_host_page_tabs(
    tabs: &slint::ModelRc<super::data::HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        if contains(&tab.frame, x, y) {
            return Some(ChromePointerRoute::HostPageTab {
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x,
                local_y: y,
            });
        }
    }
    None
}

fn route_document_tabs(
    surface_key: &str,
    header_frame: &FrameRect,
    tabs: &slint::ModelRc<super::data::HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        let close_frame = translated(&tab.close_frame, header_frame.x, header_frame.y);
        if contains(&close_frame, x, y) {
            return Some(ChromePointerRoute::DocumentTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_frame.x,
                local_y: y - header_frame.y,
                close: true,
            });
        }
        let tab_frame = translated(&tab.frame, header_frame.x, header_frame.y);
        if contains(&tab_frame, x, y) {
            return Some(ChromePointerRoute::DocumentTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_frame.x,
                local_y: y - header_frame.y,
                close: false,
            });
        }
    }
    None
}

fn route_drawer_header(
    surface_key: &str,
    region: &FrameRect,
    header: &FrameRect,
    tabs: &slint::ModelRc<super::data::HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let header_origin = translated(header, region.x, region.y);
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        let tab_frame = translated(&tab.frame, header_origin.x, header_origin.y);
        if contains(&tab_frame, x, y) {
            return Some(ChromePointerRoute::DrawerHeaderTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_origin.x,
                local_y: y - header_origin.y,
            });
        }
    }
    None
}

fn route_activity_rail(
    region: &FrameRect,
    rail_before_panel: bool,
    rail_width: f32,
    buttons: &slint::ModelRc<super::data::HostChromeControlFrameData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    if !contains(region, x, y) || rail_width <= 0.0 {
        return None;
    }
    let rail_x = if rail_before_panel {
        region.x
    } else {
        region.x + (region.width - rail_width).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: region.y,
        width: rail_width.min(region.width.max(0.0)),
        height: region.height,
    };
    if !contains(&rail, x, y) {
        return None;
    }
    for row in 0..buttons.row_count() {
        let button = buttons.row_data(row)?;
        let button_frame = translated(&button.frame, rail.x, rail.y);
        if contains(&button_frame, x, y) {
            return Some(ChromePointerRoute::ActivityRail {
                side: if rail_before_panel { "left" } else { "right" }.into(),
                local_x: x - rail.x,
                local_y: y - rail.y,
            });
        }
    }
    None
}

fn route_pointer_to_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    let scene = &presentation.host_scene_data;
    for row in (0..scene.floating_layer.floating_windows.row_count()).rev() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        let content = floating_window_content_frame(&window.frame, &window.header_frame);
        if contains(&content, x, y) {
            return pane_route_from_pane(
                &window.active_pane,
                &content,
                x,
                y,
                Some(window.window_id.as_str()),
            );
        }
    }
    for (pane, content, surface_key) in [
        (
            &scene.document_dock.pane,
            translated(
                &scene.document_dock.content_frame,
                scene.document_dock.region_frame.x,
                scene.document_dock.region_frame.y,
            ),
            Some(scene.document_dock.surface_key.as_str()),
        ),
        (
            &scene.left_dock.pane,
            side_dock_content_frame(&scene.left_dock),
            Some(scene.left_dock.surface_key.as_str()),
        ),
        (
            &scene.right_dock.pane,
            side_dock_content_frame(&scene.right_dock),
            Some(scene.right_dock.surface_key.as_str()),
        ),
        (
            &scene.bottom_dock.pane,
            translated(
                &scene.bottom_dock.content_frame,
                scene.bottom_dock.region_frame.x,
                scene.bottom_dock.region_frame.y,
            ),
            Some(scene.bottom_dock.surface_key.as_str()),
        ),
    ] {
        if let Some(route) = pane_route_from_pane(pane, &content, x, y, surface_key) {
            return Some(route);
        }
    }
    None
}

fn pane_route_from_pane(
    pane: &super::data::PaneData,
    content: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&str>,
) -> Option<PanePointerRoute> {
    if !contains(content, x, y) {
        return None;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        let toolbar = FrameRect {
            x: content.x,
            y: content.y,
            width: content.width,
            height: toolbar_height,
        };
        if contains(&toolbar, x, y) {
            return Some(PanePointerRoute::new(
                PanePointerTarget::Viewport(surface_key.unwrap_or("document").into()),
                &toolbar,
                x,
                y,
            ));
        }
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    let target = match pane.kind.as_str() {
        "Hierarchy" => PanePointerTarget::Hierarchy,
        "Welcome" => PanePointerTarget::Welcome,
        "Console" => PanePointerTarget::Console,
        "Inspector" => PanePointerTarget::Inspector,
        "Assets" => PanePointerTarget::AssetTree("activity".into()),
        "AssetBrowser" => PanePointerTarget::AssetTree("browser".into()),
        "Scene" | "Game" => PanePointerTarget::Viewport(surface_key.unwrap_or("document").into()),
        "UiAssetEditor" => PanePointerTarget::UiAsset,
        _ => PanePointerTarget::Other,
    };
    Some(PanePointerRoute::new(target, &body, x, y))
}

fn menu_handles_point(presentation: &HostWindowPresentationData, x: f32, y: f32) -> bool {
    let scene = &presentation.host_scene_data;
    if contains(&scene.menu_chrome_frame(), x, y) {
        return true;
    }
    if scene.menu_chrome.menu_frames.row_count() == 0 {
        return contains(
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: presentation.host_layout.status_bar_frame.width,
                height: scene.menu_chrome.top_bar_height_px.max(0.0),
            },
            x,
            y,
        );
    }
    (0..scene.menu_chrome.menu_frames.row_count()).any(|row| {
        scene
            .menu_chrome
            .menu_frames
            .row_data(row)
            .is_some_and(|control| contains(&control.frame, x, y))
    })
}

trait HostSceneMenuFrame {
    fn menu_chrome_frame(&self) -> FrameRect;
}

impl HostSceneMenuFrame for super::data::HostWindowSceneData {
    fn menu_chrome_frame(&self) -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: self.layout.status_bar_frame.width.max(self.layout.center_band_frame.width),
            height: self.menu_chrome.top_bar_height_px.max(0.0),
        }
    }
}

fn side_dock_content_frame(dock: &super::data::HostSideDockSurfaceData) -> FrameRect {
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    translated(&dock.content_frame, panel_x, dock.region_frame.y)
}

fn floating_window_content_frame(frame: &FrameRect, header: &FrameRect) -> FrameRect {
    FrameRect {
        x: frame.x + 1.0,
        y: frame.y + header.height.max(0.0) + 1.0,
        width: (frame.width - 2.0).max(0.0),
        height: (frame.height - header.height.max(0.0) - 2.0).max(0.0),
    }
}

fn translated(frame: &FrameRect, origin_x: f32, origin_y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

fn contains(frame: &FrameRect, x: f32, y: f32) -> bool {
    frame.width > 0.0
        && frame.height > 0.0
        && x >= frame.x
        && y >= frame.y
        && x < frame.x + frame.width
        && y < frame.y + frame.height
}

fn viewport_button_id(button: UiPointerButton) -> Option<i32> {
    match button {
        UiPointerButton::Primary => Some(VIEWPORT_POINTER_BUTTON_PRIMARY),
        UiPointerButton::Secondary => Some(VIEWPORT_POINTER_BUTTON_SECONDARY),
        UiPointerButton::Middle => Some(VIEWPORT_POINTER_BUTTON_MIDDLE),
    }
}

enum ChromePointerRoute {
    ActivityRail {
        side: SharedString,
        local_x: f32,
        local_y: f32,
    },
    HostPageTab {
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    DocumentTab {
        surface_key: SharedString,
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
        close: bool,
    },
    DrawerHeaderTab {
        surface_key: SharedString,
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    FloatingWindowHeader,
    Resize,
}

struct PanePointerRoute {
    target: PanePointerTarget,
    local_x: f32,
    local_y: f32,
    width: f32,
    height: f32,
}

impl PanePointerRoute {
    fn new(target: PanePointerTarget, frame: &FrameRect, x: f32, y: f32) -> Self {
        Self {
            target,
            local_x: x - frame.x,
            local_y: y - frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

enum PanePointerTarget {
    Hierarchy,
    Welcome,
    Console,
    Inspector,
    BrowserAssetDetails,
    AssetTree(SharedString),
    AssetContent(SharedString),
    AssetReference(SharedString, SharedString),
    Viewport(SharedString),
    UiAsset,
    Other,
}

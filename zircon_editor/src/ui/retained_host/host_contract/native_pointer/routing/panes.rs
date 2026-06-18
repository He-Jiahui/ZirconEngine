use super::super::super::data::{FrameRect, HostWindowPresentationData, PaneData};
use super::super::super::surface_hit_test;
use super::{
    geometry::{contains, floating_window_content_frame, side_dock_content_frame, translated},
    PanePointerRoute, PanePointerTarget,
};

pub(in crate::ui::retained_host::host_contract::native_pointer) fn route_pointer_to_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    route_pointer_to_pane_with_mode(presentation, x, y, PaneRouteMode::Default)
}

pub(in crate::ui::retained_host::host_contract::native_pointer) fn route_pointer_move_to_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    route_pointer_to_pane_with_mode(presentation, x, y, PaneRouteMode::PointerMove)
}

fn route_pointer_to_pane_with_mode(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
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
                mode,
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
        if let Some(route) = pane_route_from_pane(pane, &content, x, y, surface_key, mode) {
            return Some(route);
        }
    }
    None
}

fn pane_route_from_pane(
    pane: &PaneData,
    content: &FrameRect,
    x: f32,
    y: f32,
    surface_key: Option<&str>,
    mode: PaneRouteMode,
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
            let surface_key = surface_key.unwrap_or("document");
            if let Some(hit) = surface_hit_test::hit_test_viewport_toolbar(
                surface_key,
                &pane.viewport,
                &toolbar,
                x,
                y,
            ) {
                return Some(PanePointerRoute::new(
                    PanePointerTarget::ViewportToolbar(hit),
                    &toolbar,
                    x,
                    y,
                ));
            }
            return Some(PanePointerRoute::new(
                PanePointerTarget::Viewport(surface_key.into()),
                &toolbar,
                x,
                y,
            ));
        }
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    if mode.allows_template_hit_for_move(pane) {
        if let Some(hit) = surface_hit_test::hit_test_pane_template_node(pane, &body, x, y) {
            return Some(PanePointerRoute::new(
                PanePointerTarget::TemplateNode(hit),
                &body,
                x,
                y,
            ));
        }
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

#[derive(Clone, Copy)]
enum PaneRouteMode {
    Default,
    PointerMove,
}

impl PaneRouteMode {
    fn allows_template_hit_for_move(self, pane: &PaneData) -> bool {
        match self {
            Self::Default => true,
            Self::PointerMove => !matches!(
                pane.kind.as_str(),
                "Hierarchy" | "Welcome" | "Assets" | "AssetBrowser"
            ),
        }
    }
}

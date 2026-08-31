use crate::ui::retained_host::{callback_dispatch, PaneData};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::hit_controls::{viewport_toolbar_hit_control_id, viewport_toolbar_hit_route_key};

const VIEWPORT_TOOLBAR_HEIGHT: f32 = 28.0;

pub(super) fn viewport_toolbar_size_for_width(width: f32) -> UiSize {
    UiSize::new(width.max(1.0), VIEWPORT_TOOLBAR_HEIGHT)
}

pub(super) fn attach_viewport_toolbar_surface_frame_to_pane(
    viewport_toolbar_bridge: &mut callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    surface_key: &str,
    toolbar_size: UiSize,
    pane: &mut PaneData,
) {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") || !pane.show_toolbar {
        pane.viewport.toolbar_surface_frame = None;
        return;
    }

    let cached_surface_frame = {
        let viewport = &pane.viewport;
        let hit_route_key = viewport_toolbar_hit_route_key(viewport);
        viewport_toolbar_bridge
            .surface_frame_from_cached_layout_for_projection_controls_with_hit_route_key(
                surface_key,
                toolbar_size,
                &hit_route_key,
                |projection_control_id| {
                    Some(viewport_toolbar_hit_control_id(
                        viewport,
                        projection_control_id,
                    ))
                },
            )
    };
    if let Some(surface_frame) = cached_surface_frame {
        pane.viewport.toolbar_surface_frame = Some(surface_frame);
        return;
    }

    if viewport_toolbar_bridge
        .recompute_layout(toolbar_size)
        .is_err()
    {
        pane.viewport.toolbar_surface_frame = None;
        return;
    }

    let surface_frame = {
        let viewport = &pane.viewport;
        let hit_route_key = viewport_toolbar_hit_route_key(viewport);
        viewport_toolbar_bridge.surface_frame_for_projection_controls_with_hit_route_key(
            surface_key,
            toolbar_size,
            &hit_route_key,
            |projection_control_id| {
                Some(viewport_toolbar_hit_control_id(
                    viewport,
                    projection_control_id,
                ))
            },
        )
    };
    pane.viewport.toolbar_surface_frame = Some(surface_frame);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::tests::editor_event::support::env_lock;

    #[test]
    fn stable_pane_attachment_skips_layout_and_surface_rebuild() {
        let _guard = env_lock().lock().unwrap();
        let mut bridge = callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()
            .expect("viewport toolbar template should load");
        let mut pane = PaneData {
            kind: "Scene".into(),
            show_toolbar: true,
            ..PaneData::default()
        };
        let toolbar_size = viewport_toolbar_size_for_width(1280.0);

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.main",
            toolbar_size,
            &mut pane,
        );
        let first = pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("first attachment should publish a toolbar frame");
        let recomputes_after_first = bridge.layout_recompute_count();
        let mappings_after_first = bridge.hit_control_projection_count();
        assert!(mappings_after_first > 0);

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.main",
            toolbar_size,
            &mut pane,
        );
        let repeated = pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("stable attachment should preserve a toolbar frame");
        assert!(Arc::ptr_eq(&first, &repeated));
        assert_eq!(bridge.layout_recompute_count(), recomputes_after_first);
        assert_eq!(
            bridge.hit_control_projection_count(),
            mappings_after_first,
            "an exact retained hit must not remap every toolbar control"
        );

        pane.viewport.display_mode = "Wireframe".into();
        pane.viewport.translate_snap = 50.0;
        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.main",
            toolbar_size,
            &mut pane,
        );
        let presentation_only_change = pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("presentation-only changes should preserve a toolbar frame");
        assert!(Arc::ptr_eq(&first, &presentation_only_change));
        assert_eq!(
            bridge.hit_control_projection_count(),
            mappings_after_first,
            "non-routing viewport state must not invalidate hit-control projection"
        );

        for _ in 0..1_000 {
            attach_viewport_toolbar_surface_frame_to_pane(
                &mut bridge,
                "scene.main",
                toolbar_size,
                &mut pane,
            );
        }
        let stable_storm_frame = pane
            .viewport
            .toolbar_surface_frame
            .as_ref()
            .expect("stable attachment storm should preserve a toolbar frame");
        assert!(Arc::ptr_eq(&first, stable_storm_frame));
        assert_eq!(
            bridge.hit_control_projection_count(),
            mappings_after_first,
            "1,000 stable attachments must not perform per-control route mapping"
        );

        pane.viewport.mode = "Transform.Rotate".into();
        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.main",
            toolbar_size,
            &mut pane,
        );
        let remapped = pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("changed routing state should publish a toolbar frame");
        assert!(!Arc::ptr_eq(&first, &remapped));
        assert_eq!(
            bridge.layout_recompute_count(),
            recomputes_after_first,
            "route-only changes must reproject cached geometry without relayout"
        );
        let mappings_after_route_change = bridge.hit_control_projection_count();
        assert!(mappings_after_route_change > mappings_after_first);

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.main",
            toolbar_size,
            &mut pane,
        );
        let repeated_remap = pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("stable remapped routing should preserve a toolbar frame");
        assert!(Arc::ptr_eq(&remapped, &repeated_remap));
        assert_eq!(
            bridge.hit_control_projection_count(),
            mappings_after_route_change,
            "a stable route key must bypass control-id reconstruction"
        );

        let resized_toolbar = viewport_toolbar_size_for_width(900.0);
        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.main",
            resized_toolbar,
            &mut pane,
        );
        let resized = pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("resized attachment should publish a toolbar frame");
        assert!(!Arc::ptr_eq(&remapped, &resized));
        assert_eq!(
            bridge.layout_recompute_count(),
            recomputes_after_first.saturating_add(1),
            "surface-size changes must recompute template layout exactly once"
        );
    }

    #[test]
    fn route_only_state_storm_never_recomputes_toolbar_layout() {
        let _guard = env_lock().lock().unwrap();
        let mut bridge = callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()
            .expect("viewport toolbar template should load");
        let mut pane = PaneData {
            kind: "Scene".into(),
            show_toolbar: true,
            ..PaneData::default()
        };
        let toolbar_size = viewport_toolbar_size_for_width(1280.0);

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.storm",
            toolbar_size,
            &mut pane,
        );
        let layout_recompute_count = bridge.layout_recompute_count();
        let modes = [
            ("Select", "mode.select"),
            ("Transform.Move", "mode.move"),
            ("Transform.Rotate", "mode.rotate"),
            ("Transform.Scale", "mode.scale"),
        ];

        for index in 0..1_000 {
            let (mode, expected_control_id) = modes[index % modes.len()];
            pane.viewport.mode = mode.to_string();
            attach_viewport_toolbar_surface_frame_to_pane(
                &mut bridge,
                "scene.storm",
                toolbar_size,
                &mut pane,
            );
            let frame = pane
                .viewport
                .toolbar_surface_frame
                .as_ref()
                .expect("route-only attachment should preserve a toolbar frame");
            assert!(frame
                .hit_grid
                .entries
                .iter()
                .any(|entry| entry.control_id.as_deref() == Some(expected_control_id)));
        }

        assert_eq!(bridge.layout_recompute_count(), layout_recompute_count);
    }

    #[test]
    fn cached_surface_geometry_remains_authoritative_across_different_pane_widths() {
        let _guard = env_lock().lock().unwrap();
        let mut bridge = callback_dispatch::BuiltinViewportToolbarTemplateBridge::new()
            .expect("viewport toolbar template should load");
        let mut wide_pane = PaneData {
            kind: "Scene".into(),
            show_toolbar: true,
            ..PaneData::default()
        };
        let mut narrow_pane = wide_pane.clone();
        let wide_size = viewport_toolbar_size_for_width(1280.0);
        let narrow_size = viewport_toolbar_size_for_width(640.0);

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.wide",
            wide_size,
            &mut wide_pane,
        );
        let first_wide = wide_pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("wide pane should publish a toolbar frame");

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.narrow",
            narrow_size,
            &mut narrow_pane,
        );
        let narrow = narrow_pane
            .viewport
            .toolbar_surface_frame
            .clone()
            .expect("narrow pane should publish a toolbar frame");
        assert!(!Arc::ptr_eq(&first_wide, &narrow));
        let recomputes_after_both_sizes = bridge.layout_recompute_count();

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.wide",
            wide_size,
            &mut wide_pane,
        );
        let repeated_wide = wide_pane
            .viewport
            .toolbar_surface_frame
            .as_ref()
            .expect("wide pane should recover its retained toolbar frame");
        assert!(Arc::ptr_eq(&first_wide, repeated_wide));
        assert_eq!(
            bridge.layout_recompute_count(),
            recomputes_after_both_sizes,
            "returning to a retained pane must not relayout the last visited pane geometry"
        );

        let mut second_narrow_pane = narrow_pane.clone();
        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "game.narrow",
            narrow_size,
            &mut second_narrow_pane,
        );
        let second_narrow = second_narrow_pane
            .viewport
            .toolbar_surface_frame
            .as_ref()
            .expect("second narrow pane should publish its own toolbar frame");
        assert!(!Arc::ptr_eq(&narrow, second_narrow));
        assert_eq!(
            control_frame(&narrow, "display.cycle"),
            control_frame(second_narrow, "display.cycle")
        );
        assert_eq!(
            bridge.layout_recompute_count(),
            recomputes_after_both_sizes,
            "a new surface at a retained size must reuse the shared layout geometry"
        );

        attach_viewport_toolbar_surface_frame_to_pane(
            &mut bridge,
            "scene.wide",
            narrow_size,
            &mut wide_pane,
        );
        let resized_wide = wide_pane
            .viewport
            .toolbar_surface_frame
            .as_ref()
            .expect("resized wide pane should publish narrow retained geometry");
        assert!(!Arc::ptr_eq(&first_wide, resized_wide));
        assert_eq!(
            control_frame(&narrow, "display.cycle"),
            control_frame(resized_wide, "display.cycle")
        );
        assert_eq!(
            bridge.layout_recompute_count(),
            recomputes_after_both_sizes,
            "resizing to an already retained width must not relayout"
        );
    }

    fn control_frame(
        frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
        control_id: &str,
    ) -> zircon_runtime_interface::ui::layout::UiFrame {
        frame
            .hit_grid
            .entries
            .iter()
            .find(|entry| entry.control_id.as_deref() == Some(control_id))
            .map(|entry| entry.frame)
            .unwrap_or_else(|| panic!("missing toolbar hit control `{control_id}`"))
    }
}

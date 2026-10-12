use super::*;
use crate::tests::editor_event::support::env_lock;
use crate::ui::retained_host::callback_dispatch::{
    BuiltinWorkbenchWindowTemplateSurfaceBridge, WorkbenchCommandPaletteOpenState,
};
use crate::ui::retained_host::host_contract::native_keyboard::workbench_popup_accept_is_owned;
use crate::ui::retained_host::host_contract::surface_hit_test::{
    hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
};
use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes;
use winit::keyboard::{Key, NamedKey};
use zircon_runtime_interface::ui::{component::UiValue, layout::UiSize};

#[test]
fn actual_authored_scene_picker_overlay_paints_and_routes_outside_chrome_then_closes() {
    let _guard = env_lock().lock().unwrap();
    let size = UiSize::new(1280.0, 800.0);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(size).unwrap();
    bridge
        .open_command_palette_with_chrome(
            WorkbenchCommandPaletteOpenState {
                query: "res://scenes/review.scene.toml".to_owned(),
                commands: UiValue::Array(vec![UiValue::String(
                    "scene-picker-create-confirm|label=Create Scene".to_owned(),
                )]),
                filtered_commands: UiValue::Array(vec![UiValue::String(
                    "scene-picker-create-confirm".to_owned(),
                )]),
                selected_command_id: "scene-picker-create-confirm".to_owned(),
                focused_index: 0,
                catalog_generation: 0,
                total_match_count: 1,
                window_offset: 0,
            },
            "scene-picker-create",
            "New scene URI",
            "Enter a scene URI",
            "Create scene",
            "Create an empty scene in the current project",
        )
        .unwrap();
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes =
        to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    presentation.host_layout.center_band_frame = FrameRect {
        x: 0.0,
        y: 120.0,
        width: 1280.0,
        height: 660.0,
    };
    presentation.host_layout.status_bar_frame = FrameRect {
        x: 0.0,
        y: 780.0,
        width: 1280.0,
        height: 20.0,
    };
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 1280.0,
        height: 800.0,
    };
    let overlays = RootOverlayRows::from_presentation(&presentation, true, false);
    let palette_row = presentation
        .workbench_window_nodes
        .iter()
        .position(|node| node.control_id.as_str() == "WorkbenchCommandPalette")
        .expect("actual window palette should be visible in final native projection");
    let palette = presentation
        .workbench_window_nodes
        .get(palette_row)
        .unwrap();
    assert!(palette.popup_open);
    assert!(overlays.rows.contains(&palette_row));
    assert!(!RootOverlayRows::chrome_exclusion(&presentation)
        .transform_row(palette_row, palette.clone(), bounds.clone())
        .is_some());
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);
    // Locate the real projected option geometry through the production hit owner.
    let hit = (120..780)
        .find_map(|y| {
            let x = palette.frame.x + palette.frame.width * 0.5;
            hit_test_workbench_window_template_node_with_index(&presentation, &index, x, y as f32)
                .filter(|hit| {
                    hit.control_id.as_str() == "WorkbenchCommandPalette"
                        && hit.value_text.as_str() == "scene-picker-create-confirm"
                })
                .map(|hit| (x as usize, y, hit))
        })
        .expect("real palette option must be hit outside both chrome clips");
    assert_eq!(hit.2.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(hit.2.commit_action_id.as_str(), "CommandPalette/Commit");
    let background = [241, 17, 193, 255];
    let mut overlay_frame = HostRgbaFrame::filled(1280, 800, background);
    draw_authored_root_overlays(&mut overlay_frame, &presentation, &bounds);
    let offset = (hit.1 * 1280 + hit.0) * 4;
    assert_ne!(&overlay_frame.as_bytes()[offset..offset + 4], &background);
    let mut final_frame = HostRgbaFrame::filled(1280, 800, background);
    super::super::draw_componentized_workbench_window(&mut final_frame, &presentation);
    assert_eq!(
        &final_frame.as_bytes()[offset..offset + 4],
        &overlay_frame.as_bytes()[offset..offset + 4],
        "foreground overlay must survive dock composition at its actual hit geometry"
    );
    let ui = UiHostWindow::new().unwrap();
    ui.set_host_presentation(presentation.clone());
    assert!(workbench_popup_accept_is_owned(
        &ui,
        &Key::Named(NamedKey::Enter)
    ));
    bridge.close_command_palette().unwrap();
    presentation.workbench_window_nodes =
        to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    assert!(
        RootOverlayRows::from_presentation(&presentation, true, false)
            .rows
            .is_empty()
    );
    let mut closed = HostRgbaFrame::filled(1280, 800, background);
    draw_authored_root_overlays(&mut closed, &presentation, &bounds);
    assert!(closed
        .as_bytes()
        .chunks_exact(4)
        .all(|pixel| pixel == background));
    ui.set_host_presentation(presentation);
    assert!(
        !workbench_popup_accept_is_owned(&ui, &Key::Named(NamedKey::Enter)),
        "closed palette must release foreground keyboard ownership"
    );
}

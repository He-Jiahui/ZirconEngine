use super::*;

#[test]
#[ignore = "isolated native Editor lane; requires ZIRCON_PREVIEW168_TEST_ROOT"]
fn game_pointer_owner_unknown_right_up_keeps_fresh_hit_after_known_tail() {
    let (before, first_unknown, known_tail, next_unknown) = in_game_pointer_owner_fixture(
        |host, ui, entity, check_edit| {
            let owner = GamePointerOwner::enter(host);
            let points = game_pointer_owner_points(ui);
            let mut drive = ui.winit_pointer_driver_for_test();
            let before = mouse_edges(&owner.input());
            let pose = owner.pose(entity);
            let inside_up =
                || game_pointer_edge(MouseButton::Right, ElementState::Released, points.drag);

            drive(inside_up()); // no RMB Down observed by this window
            let first_unknown = mouse_edges(&owner.input());
            probe_game_pointer_owner(&mut drive, &owner, entity, points.probe, false);

            drive(game_pointer_edge(
                MouseButton::Right,
                ElementState::Pressed,
                points.status,
            ));
            assert_eq!(
                mouse_edges(&owner.input()),
                first_unknown,
                "fixture: the same-window status band RMB Down must stay outside Game"
            );
            drive(inside_up()); // known non-Game tail, only this edge is suppressed
            let known_tail = mouse_edges(&owner.input());
            drive(inside_up()); // no new Down: there is no remaining known tail
            let next_unknown = mouse_edges(&owner.input());
            probe_game_pointer_owner(&mut drive, &owner, entity, points.down, false);
            assert_eq!(
                owner.pose(entity),
                pose,
                "release-only input must not change the Play camera"
            );
            assert!(host.borrow().runtime.play_preview_view_focused().unwrap());
            check_edit();
            eprintln!("PREVIEW168_OWNER_RMB_HISTORY owner={:?} edges={before:?}->{first_unknown:?}->{known_tail:?}->{next_unknown:?}", owner.identity);
            owner.stop(host);
            (before, first_unknown, known_tail, next_unknown)
        },
    );

    let released_once = |(presses, releases)| (presses, releases + 1);
    assert_eq!(first_unknown, released_once(before), "PREVIEW168_OWNER_UNKNOWN_RMB: a fresh in-Game Up with no observed RMB Down keeps its ordinary route");
    assert_eq!(next_unknown, released_once(known_tail), "PREVIEW168_OWNER_UNKNOWN_RMB: the known tail must not suppress a second Up with no new Down");
    assert_eq!(
        known_tail, first_unknown,
        "PREVIEW168_OWNER_KNOWN_RMB_ONCE: only the known non-Game tail must stay out of Play"
    );
}

fn adjacent_authored_hierarchy_point(ui: &UiHostWindow) -> UiPoint {
    use crate::ui::retained_host::host_contract::{
        componentized_workbench_regions::{authored_hierarchy, point_in_frame},
        hit_test_workbench_window_template_node_with_index,
    };

    let generation = ui.get_host_presentation_generation();
    let presentation = generation.structure();
    assert!(!presentation.host_shell.native_floating_window_mode);
    assert!(!presentation.close_prompt.visible && !presentation.asset_deletion_blocker.visible);
    assert!(!host_context(ui).host_popup_occludes_workbench_tooltip());
    assert!(presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .is_empty());
    let dock = presentation
        .host_scene_data
        .document_surfaces()
        .iter()
        .find(|dock| dock.pane.kind.as_str() == "Game")
        .expect("fixture: Game document");
    let game =
        authored_level_viewport(presentation, dock).expect("fixture: authored Game viewport");
    let hierarchy = authored_hierarchy(presentation)
        .expect("fixture: no ordinary authored Hierarchy sibling in this Play window");
    let viewport = hierarchy.viewport;
    let size = ui.window().size();
    for y in [0.81, 0.68, 0.46, 0.28] {
        for x in [0.59, 0.28, 0.82] {
            let point = UiPoint::new(
                viewport.x + viewport.width * x,
                viewport.y + viewport.height * y,
            );
            if point.x >= 0.0
                && point.x < size.width as f32
                && point.y >= 0.0
                && point.y < size.height as f32
                && point_in_frame(&viewport, point.x, point.y)
                && !point_in_frame(&game.clip, point.x, point.y)
                && !point_in_frame(&presentation.host_layout.status_bar_frame, point.x, point.y)
                && hit_test_workbench_window_template_node_with_index(
                    presentation,
                    generation.workbench_hit_index(),
                    point.x,
                    point.y,
                )
                .is_none()
            {
                eprintln!(
                    "PREVIEW168_ADJACENT_PANE pane={} point={point:?} viewport={viewport:?}",
                    hierarchy.pane.id
                );
                return point;
            }
        }
    }
    panic!("fixture: no exposed ordinary Hierarchy hit outside Game on the committed surface");
}

fn adjacent_play_reset_requests(owner: &GamePointerOwner) -> (usize, usize) {
    use zircon_runtime_host::foreign_output::{
        RuntimeForeignOutputKind::HostRequests, RuntimeForeignOutputState,
        RuntimeOwnedOutputReleaser, HOST_REQUEST_OUTPUT_BUDGET,
    };
    use zircon_runtime_interface::{
        ZrOwnedResultV2, ZrRuntimeCursorGrabModeV1, ZrRuntimeCursorHostRequestKindV1,
        ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeImeHostRequestKindV1,
        ZIRCON_RUNTIME_ABI_VERSION_V1,
    };

    assert_eq!(owner.gateway.identity(), owner.identity);
    let handle = owner.identity.runtime_session();
    // The frozen Play fixture owns a live session created by this linked provider.
    let api = unsafe {
        zircon_runtime::dynamic_api::zircon_runtime_get_api_v8(std::ptr::null()).as_ref()
    }
    .expect("fixture: linked Runtime V8");
    let drain = api
        .drain_host_requests
        .expect("fixture: V8 host requests available");
    let release = api.release_allocation.expect("fixture: V8 output releaser");
    let releaser = unsafe { RuntimeOwnedOutputReleaser::new(handle, release) };
    let output_state = RuntimeForeignOutputState::default();
    let (mut ime_disabled, mut grab_released) = (0, 0);
    for _ in 0..4 {
        let mut output = ZrOwnedResultV2::empty();
        let status = unsafe { drain(handle, &mut output) };
        // The real provider produced the allocation above; the existing host helpers always
        // consume its sole release authority before returning the owned JSON result.
        let output = unsafe {
            output_state.ensure_call_succeeded(
                status,
                output,
                releaser,
                HostRequests,
                "fixture: drain Play host requests",
                "fixture: free Play request output",
            )
        }
        .unwrap();
        let batch = unsafe {
            output_state.decode_json::<ZrRuntimeHostRequestBatchV1, _>(
                output,
                releaser,
                HostRequests,
                HOST_REQUEST_OUTPUT_BUDGET,
                "fixture: decode Play host requests",
                "fixture: free Play request output",
                |batch| {
                    if batch.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1 {
                        Ok(batch.requests.len())
                    } else {
                        Err("unexpected host request batch ABI")
                    }
                },
            )
        }
        .unwrap();
        let Some(batch) = batch else {
            return (ime_disabled, grab_released);
        };
        for request in batch.requests {
            match request {
                ZrRuntimeHostRequestV1::Ime(request)
                    if request.kind == ZrRuntimeImeHostRequestKindV1::Disable =>
                {
                    ime_disabled += 1
                }
                ZrRuntimeHostRequestV1::Cursor(request)
                    if request.kind == ZrRuntimeCursorHostRequestKindV1::SetGrabMode
                        && request.grab_mode == Some(ZrRuntimeCursorGrabModeV1::None) =>
                {
                    grab_released += 1
                }
                _ => {}
            }
        }
    }
    panic!("fixture: unexpected unbounded host requests during isolated pointer probe");
}

#[test]
#[ignore = "isolated native Editor lane; requires ZIRCON_PREVIEW168_TEST_ROOT"]
fn game_pointer_adjacent_hierarchy_up_finishes_original_orbit_without_focus_reset() {
    let (before, after, ended, reset) = in_game_pointer_owner_fixture(
        |host, ui, entity, check_edit| {
            let owner = GamePointerOwner::enter(host);
            let points = game_pointer_owner_points(ui);
            let mut drive = ui.winit_pointer_driver_for_test();
            let armed = arm_game_pointer_owner(&mut drive, &owner, entity, &points);
            let sibling = adjacent_authored_hierarchy_point(ui);
            let _queued_before = adjacent_play_reset_requests(&owner);
            drive(game_pointer_edge(
                MouseButton::Right,
                ElementState::Released,
                sibling,
            ));
            let after = owner.input();
            let reset = adjacent_play_reset_requests(&owner);
            assert!(
                host.borrow().runtime.play_preview_view_focused().unwrap(),
                "fixture: adjacent release must not leave Game focus"
            );
            let camera_after_up = owner.pose(entity);
            drive(game_pointer_move(points.probe));
            let probe = owner.input();
            assert_eq!(probe.pointer_move_count, after.pointer_move_count + 1);
            assert_eq!(mouse_edges(&probe), mouse_edges(&after));
            let camera_after_move = owner.pose(entity);
            assert_eq!(camera_after_move.0, camera_after_up.0);
            let ended = camera_after_move == camera_after_up;

            // Calibrate the reset witness on this same live owner, after the measured interval.
            drive(WindowEvent::Focused(false));
            let blur = adjacent_play_reset_requests(&owner);
            assert!(blur.0 > 0 && blur.1 > 0, "fixture: true native-focus handler must produce the Runtime reset requests: {blur:?}");
            assert_eq!(mouse_edges(&owner.input()), mouse_edges(&after));
            check_edit();
            eprintln!("PREVIEW168_ADJACENT_HIERARCHY_UP owner={:?} edges={:?}->{:?} moves={}->{} ended={ended} reset={reset:?} actual_blur={blur:?}", owner.identity, mouse_edges(&armed), mouse_edges(&after), after.pointer_move_count, probe.pointer_move_count);
            owner.stop(host);
            (mouse_edges(&armed), mouse_edges(&after), ended, reset)
        },
    );
    assert_eq!((after, ended, reset), ((before.0, before.1 + 1), true, (0, 0)),
        "PREVIEW168_ADJACENT_HIERARCHY_UP: ordinary sibling release owes one real RMB Up, a stopped orbit and no focus reset requests");
}

#[test]
#[ignore = "isolated native Editor lane; requires ZIRCON_PREVIEW168_TEST_ROOT"]
fn game_pointer_adjacent_open_menu_does_not_block_uncovered_game_right_down() {
    use crate::ui::retained_host::host_contract::hit_test_workbench_window_template_node_with_index;

    let (before, pressed, released, orbited) = in_game_pointer_owner_fixture(
        |host, ui, entity, check_edit| {
            let owner = GamePointerOwner::enter(host);
            let points = game_pointer_owner_points(ui);
            let mut drive = ui.winit_pointer_driver_for_test();
            let menu_button = {
                let host = host.borrow();
                let frame = host
                    .menu_pointer_layout
                    .button_frames
                    .first()
                    .expect("fixture: real root menu button");
                assert!(frame.width > 0.0 && frame.height > 0.0);
                UiPoint::new(frame.x + frame.width * 0.5, frame.y + frame.height * 0.5)
            };
            let click_menu =
                || host_context(ui).invoke_menu_pointer_clicked(menu_button.x, menu_button.y);
            assert!(ui.get_menu_state().open_menu_index < 0);
            click_menu(); // real workbench callback opens the published ordinary menu
            assert_eq!(
                ui.get_menu_state().open_menu_index,
                0,
                "fixture: first menu must actually open"
            );
            let down = {
                let generation = ui.get_host_presentation_generation();
                let presentation = generation.structure();
                assert!(
                    !presentation.close_prompt.visible
                        && !presentation.asset_deletion_blocker.visible
                );
                assert!(
                    !generation.dock_overflow_menu_state().open
                        && !generation.page_overflow_menu_state().open
                );
                assert!(presentation
                    .host_scene_data
                    .floating_layer
                    .floating_windows
                    .is_empty());
                // Match the live root-popup horizontal extent, not the menu's broader
                // primary-dismiss/motion region. No submenu was opened by the header click.
                let chrome = &presentation.host_scene_data.menu_chrome;
                let header = chrome
                    .menu_frames
                    .get(0)
                    .expect("fixture: published menu header");
                let menu = chrome.menus.get(0).expect("fixture: published root popup");
                assert!(generation.menu_state().open_submenu_path.is_empty());
                let shell_width = presentation
                    .host_layout
                    .status_bar_frame
                    .width
                    .max(presentation.host_scene_data.layout.status_bar_frame.width)
                    .max(presentation.host_scene_data.layout.center_band_frame.width)
                    .max(1.0);
                let width = menu
                    .popup_width_px
                    .max(header.frame.width)
                    .min(shell_width)
                    .max(1.0);
                let popup_right =
                    crate::ui::retained_host::popup_anchor_metrics::clamp_popup_x_to_bounds(
                        header.frame.x - generation.menu_state().menu_bar_scroll_px,
                        0.0,
                        shell_width,
                        width,
                    ) + width;
                [points.probe, points.drag, points.down]
                    .into_iter()
                    .find(|point| {
                        point.x > popup_right
                            && hit_test_workbench_window_template_node_with_index(
                                presentation,
                                generation.workbench_hit_index(),
                                point.x,
                                point.y,
                            )
                            .is_none()
                    })
                    .expect(
                        "fixture: ordinary menu left no Game clip point free of template controls",
                    )
            };
            assert!(host.borrow().runtime.play_preview_view_focused().unwrap());
            let before = mouse_edges(&owner.input());
            drive(game_pointer_edge(
                MouseButton::Right,
                ElementState::Pressed,
                down,
            ));
            let pressed = mouse_edges(&owner.input());
            // The legacy menu may consume Move via its dismiss region; compare only the RMB Down
            // while the popup is open, then resume the same physical pair after closing it.
            if ui.get_menu_state().open_menu_index >= 0 {
                click_menu();
            }
            assert!(
                ui.get_menu_state().open_menu_index < 0,
                "fixture: same real menu closed"
            );
            let pose = owner.pose(entity);
            let moves = owner.input().pointer_move_count;
            let drag = if down.x == points.drag.x {
                points.probe
            } else {
                points.drag
            };
            drive(game_pointer_move(drag));
            assert_eq!(owner.input().pointer_move_count, moves + 1);
            let drag_pose = owner.pose(entity);
            assert_eq!(drag_pose.0, pose.0);
            let orbited = drag_pose != pose;
            drive(game_pointer_edge(
                MouseButton::Right,
                ElementState::Released,
                drag,
            ));
            let released = mouse_edges(&owner.input());
            probe_game_pointer_owner(&mut drive, &owner, entity, points.down, false);
            check_edit();
            eprintln!("PREVIEW168_ADJACENT_OPEN_MENU owner={:?} down={down:?} edges={before:?}->{pressed:?}->{released:?} orbited={orbited}", owner.identity);
            owner.stop(host);
            (before, pressed, released, orbited)
        },
    );
    assert_eq!((pressed, released, orbited), ((before.0 + 1, before.1), (before.0 + 1, before.1 + 1), true),
        "PREVIEW168_ADJACENT_OPEN_MENU: the ordinary menu must not block a fresh RMB Down on uncovered Game");
}

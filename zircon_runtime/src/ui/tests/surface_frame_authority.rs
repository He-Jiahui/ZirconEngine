use crate::ui::{
    dispatch::UiPointerDispatcher,
    surface::{hit_test_surface_frame, UiSurface},
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerEvent},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAlignment, UiAlignment2D, UiContainerKind,
        UiFrame, UiGridBoxConfig, UiGridSlotPlacement, UiLayoutEngineBackend,
        UiLayoutEngineFallbackReason, UiLayoutEngineFamily, UiLayoutEngineSupport,
        UiLinearBoxConfig, UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiPoint, UiSize,
        UiSizeBoxConfig, UiSlot, UiSlotKind, UiWrapBoxConfig,
    },
    surface::{UiPointerButton, UiPointerEventKind},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

const ROOT_ID: UiNodeId = UiNodeId::new(1);
const BACK_ID: UiNodeId = UiNodeId::new(2);
const FRONT_ID: UiNodeId = UiNodeId::new(3);

#[test]
fn surface_frame_render_hit_and_pointer_dispatch_share_arranged_authority() {
    let mut surface = overlapping_button_surface();
    let point = UiPoint::new(48.0, 36.0);
    let frame = surface.surface_frame();

    assert_eq!(frame.tree_id, UiTreeId::new("surface.frame.authority"));
    assert_eq!(frame.arranged_tree.tree_id, frame.tree_id);
    assert_eq!(frame.render_extract.tree_id, frame.tree_id);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should be rendered from the arranged tree");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should be entered into the hit grid");

    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(render_front.z_index, arranged_front.z_index);
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(
        hit_front.clip_frame,
        arranged_front
            .frame
            .intersection(arranged_front.clip_frame)
            .expect("front arranged frame should intersect its clip")
    );
    assert_eq!(hit_front.z_index, arranged_front.z_index);
    assert_eq!(hit_front.paint_order, arranged_front.paint_order);
    assert_eq!(hit_front.control_id.as_deref(), Some("front.button"));

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.stacked, vec![FRONT_ID, BACK_ID]);
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the same surface hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.target, frame_hit.path.target);
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_native_flex_surface_frame_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_flex_button_surface();
    surface.compute_layout(UiSize::new(124.0, 40.0)).unwrap();
    let point = UiPoint::new(48.0, 12.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Taffy flex pass");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy frame");

    assert_eq!(arranged_front.frame, UiFrame::new(44.0, 0.0, 80.0, 40.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_flex_linear_slot_sizing_button_surface();
    surface.compute_layout(UiSize::new(180.0, 40.0)).unwrap();
    let point = UiPoint::new(140.0, 12.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_back = frame
        .arranged_tree
        .get(BACK_ID)
        .expect("back control should be arranged by Taffy slot sizing");
    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by Taffy slot sizing");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy slot-sizing frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy slot-sizing frame");

    assert_eq!(arranged_back.frame, UiFrame::new(0.0, 0.0, 120.0, 40.0));
    assert_eq!(arranged_front.frame, UiFrame::new(120.0, 0.0, 60.0, 40.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy slot-sizing hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_vertical_flex_linear_slot_sizing_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_vertical_flex_linear_slot_sizing_button_surface();
    surface.compute_layout(UiSize::new(60.0, 180.0)).unwrap();
    let point = UiPoint::new(30.0, 150.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_back = frame
        .arranged_tree
        .get(BACK_ID)
        .expect("back control should be arranged by Taffy vertical slot sizing");
    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by Taffy vertical slot sizing");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy vertical slot-sizing frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy vertical slot-sizing frame");

    assert_eq!(arranged_back.frame, UiFrame::new(0.0, 0.0, 60.0, 120.0));
    assert_eq!(arranged_front.frame, UiFrame::new(0.0, 120.0, 60.0, 60.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy vertical slot-sizing hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_flex_slot_policy_fallback_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_flex_slot_policy_fallback_button_surface();
    surface.compute_layout(UiSize::new(124.0, 40.0)).unwrap();
    let point = UiPoint::new(30.0, 24.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Flex);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::SlotFramePolicy)
    );

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Zircon flex fallback");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Zircon flex fallback frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Zircon flex fallback frame");

    assert_eq!(arranged_front.frame, UiFrame::new(10.0, 19.0, 40.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Zircon flex fallback hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_wrap_surface_frame_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_wrap_button_surface();
    surface.compute_layout(UiSize::new(90.0, 44.0)).unwrap();
    let point = UiPoint::new(24.0, 28.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Wrap);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Taffy wrap pass");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy wrap frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy wrap frame");

    assert_eq!(arranged_front.frame, UiFrame::new(0.0, 22.0, 50.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy wrap-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn taffy_grid_slot_frame_policy_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = taffy_grid_slot_button_surface();
    surface.compute_layout(UiSize::new(124.0, 82.0)).unwrap();
    let point = UiPoint::new(80.0, 65.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(root_selection.request.family, UiLayoutEngineFamily::Grid);
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::Taffy
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(root_selection.fallback_reason, None);

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Taffy grid pass");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Taffy grid frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Taffy grid frame");

    assert_eq!(arranged_front.frame, UiFrame::new(73.0, 61.0, 40.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Taffy grid-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

#[test]
fn zircon_size_box_fallback_feeds_render_hit_and_pointer_dispatch() {
    let mut surface = zircon_size_box_button_surface();
    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();
    let point = UiPoint::new(40.0, 60.0);
    let frame = surface.surface_frame();

    let root_selection = frame
        .layout_engine_report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(ROOT_ID))
        .expect("root should report layout engine selection");
    assert_eq!(
        root_selection.request.family,
        UiLayoutEngineFamily::Container
    );
    assert_eq!(
        root_selection.selected_backend,
        UiLayoutEngineBackend::LegacyZircon
    );
    assert_eq!(root_selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root_selection.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );

    let arranged_front = frame
        .arranged_tree
        .get(FRONT_ID)
        .expect("front control should be arranged by the Zircon SizeBox fallback");
    let render_front = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == FRONT_ID)
        .expect("front control should render from the arranged Zircon SizeBox frame");
    let hit_front = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == FRONT_ID)
        .expect("front control should enter hit grid from the arranged Zircon SizeBox frame");

    assert_eq!(arranged_front.frame, UiFrame::new(30.0, 54.0, 40.0, 16.0));
    assert_eq!(render_front.frame, arranged_front.frame);
    assert_eq!(render_front.clip_frame, Some(arranged_front.clip_frame));
    assert_eq!(hit_front.frame, arranged_front.frame);
    assert_eq!(hit_front.z_index, arranged_front.z_index);

    let frame_hit = hit_test_surface_frame(&frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(FRONT_ID));
    assert_eq!(frame_hit.path.root_to_leaf, vec![ROOT_ID, FRONT_ID]);
    assert_eq!(frame_hit.path.bubble_route, vec![FRONT_ID, ROOT_ID]);

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(FRONT_ID, UiPointerEventKind::Down, |context| {
        assert_eq!(context.route.hit_path.target, Some(FRONT_ID));
        assert_eq!(context.route.hit_path.bubble_route, vec![FRONT_ID, ROOT_ID]);
        UiPointerDispatchEffect::handled()
    });

    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .expect("pointer dispatch should route through the Zircon SizeBox-derived hit path");

    assert_eq!(dispatch.handled_by, Some(FRONT_ID));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

fn overlapping_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                BACK_ID,
                "root/back",
                "back.button",
                UiFrame::new(16.0, 16.0, 96.0, 56.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                FRONT_ID,
                "root/front",
                "front.button",
                UiFrame::new(32.0, 24.0, 96.0, 56.0),
                10,
            ),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn taffy_flex_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.taffy"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 4.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node(BACK_ID, "root/back", "back.button", 40.0, 0),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node(FRONT_ID, "root/front", "front.button", 80.0, 10),
        )
        .unwrap();
    surface
}

fn taffy_flex_linear_slot_sizing_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "surface.frame.authority.taffy.flex.slot_sizing",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                BACK_ID,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                FRONT_ID,
                "root/front",
                "front.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                10,
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, BACK_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );
    surface
}

fn taffy_vertical_flex_linear_slot_sizing_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "surface.frame.authority.taffy.vertical_flex.slot_sizing",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 0.0 }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                BACK_ID,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            button_node(
                FRONT_ID,
                "root/front",
                "front.button",
                UiFrame::new(0.0, 0.0, 0.0, 0.0),
                10,
            ),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, BACK_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(2.0),
        ),
    );
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Linear).with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::Stretch).with_value(1.0),
        ),
    );
    surface
}

fn taffy_flex_slot_policy_fallback_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "surface.frame.authority.taffy.flex.slot_policy_fallback",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(UiLinearBoxConfig {
                gap: 0.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 40.0, 16.0, 10),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Linear)
            .with_padding(UiMargin::new(10.0, 5.0, 10.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );
    surface
}

fn taffy_wrap_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.taffy.wrap"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::WrapBox(UiWrapBoxConfig {
                horizontal_gap: 4.0,
                vertical_gap: 6.0,
                item_min_width: 1.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(BACK_ID, "root/back", "back.button", 40.0, 16.0, 0),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 50.0, 16.0, 10),
        )
        .unwrap();
    surface
}

fn taffy_grid_slot_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.taffy.grid"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::GridBox(UiGridBoxConfig {
                columns: 2,
                rows: 2,
                column_gap: 4.0,
                row_gap: 6.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 40.0, 16.0, 10),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Grid)
            .with_grid_placement(UiGridSlotPlacement::new(1, 1))
            .with_padding(UiMargin::new(2.0, 3.0, 4.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );
    surface
}

fn zircon_size_box_button_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("surface.frame.authority.size_box"));
    surface.tree.insert_root(
        UiTreeNode::new(ROOT_ID, UiNodePath::new("root"))
            .with_container(UiContainerKind::SizeBox(UiSizeBoxConfig {
                aspect_ratio: 2.0,
            }))
            .with_input_policy(UiInputPolicy::Ignore)
            .with_state_flags(root_state()),
    );
    surface
        .tree
        .insert_child(
            ROOT_ID,
            layout_button_node_with_size(FRONT_ID, "root/front", "front.button", 40.0, 16.0, 10),
        )
        .unwrap();
    surface.tree.slots.push(
        UiSlot::new(ROOT_ID, FRONT_ID, UiSlotKind::Container)
            .with_padding(UiMargin::new(10.0, 5.0, 10.0, 5.0))
            .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::End)),
    );
    surface
}

fn button_node(
    node_id: UiNodeId,
    node_path: &'static str,
    control_id: &'static str,
    frame: UiFrame,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(node_id, UiNodePath::new(node_path))
        .with_frame(frame)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            attributes: toml::from_str(
                r##"
text = "Run"
opacity = 1.0

[background]
color = "#2f6f5e"
"##,
            )
            .unwrap(),
            ..Default::default()
        })
}

fn layout_button_node(
    node_id: UiNodeId,
    node_path: &'static str,
    control_id: &'static str,
    width: f32,
    z_index: i32,
) -> UiTreeNode {
    button_node(
        node_id,
        node_path,
        control_id,
        UiFrame::new(0.0, 0.0, width, 0.0),
        z_index,
    )
    .with_constraints(BoxConstraints {
        width: fixed_axis(width),
        height: AxisConstraint::default(),
    })
}

fn layout_button_node_with_size(
    node_id: UiNodeId,
    node_path: &'static str,
    control_id: &'static str,
    width: f32,
    height: f32,
    z_index: i32,
) -> UiTreeNode {
    button_node(
        node_id,
        node_path,
        control_id,
        UiFrame::new(0.0, 0.0, width, height),
        z_index,
    )
    .with_constraints(BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    })
}

fn fixed_axis(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: value,
        preferred: value,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn root_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

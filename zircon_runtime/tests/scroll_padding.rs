#![cfg(feature = "ui")]

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher,
    surface::{hit_test_surface_frame, UiSurface},
    v2::{UiV2DocumentCompiler, UiV2SurfaceBuilder, UiZuiAssetLoader},
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchResult, UiPointerEvent},
    event_ui::UiTreeId,
    layout::{UiContainerKind, UiMargin, UiPoint, UiSize},
    surface::{UiPointerEventKind, UiRenderCommandKind},
};

const WIDTH: f32 = 120.0;
const ROW: f32 = 40.0;
const ZUI: &str = r##"
[asset]
kind = "view"
id = "res://ui/tests/scroll_padding.zui"
version = 2

[root]
node = "scroll"

[nodes.scroll]
component = "ScrollableBox"
props = { input_hoverable = true }
layout = { container = { kind = "ScrollableBox", axis = "Vertical" }, input_policy = "Receive", padding = { top = PAD, bottom = PAD } }
children = [{ node = "first" }, { node = "last" }]

[nodes.first]
component = "Button"
control_id = "First"
props = { text = "First", background_color = "#334155" }
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" } }

[nodes.last]
component = "Button"
control_id = "Last"
props = { text = "Last", background_color = "#334155" }
layout = { width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" } }
"##;

fn surface_with_padding(pad: u32, height: f32) -> UiSurface {
    let document = UiZuiAssetLoader::load_zui_str(&ZUI.replace("PAD", &pad.to_string())).unwrap();
    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("scroll-padding"),
        &document,
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(WIDTH, height)).unwrap();
    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    let UiContainerKind::ScrollableBox(config) = root.container else {
        panic!("authored scroll owner must survive document expansion");
    };
    assert!(config.virtualization.is_none());
    let pad = pad as f32;
    assert_eq!(root.layout_padding, UiMargin::new(0.0, pad, 0.0, pad));
    surface
}

fn wheel_to_end(surface: &mut UiSurface) -> UiPointerDispatchResult {
    let point = UiPoint::new(20.0, 35.0);
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(60.0),
        )
        .unwrap()
}

fn scroll_offset(surface: &UiSurface) -> f32 {
    surface
        .tree
        .node(surface.tree.roots[0])
        .unwrap()
        .scroll_state
        .unwrap()
        .offset
}

fn log_frame(stamp: &str, surface: &UiSurface, wheel: Option<&UiPointerDispatchResult>) {
    let root_id = surface.tree.roots[0];
    let root = surface.tree.node(root_id).unwrap();
    let frame = surface.surface_frame();
    let row = |control| {
        let id = surface.unique_control_node_id(control);
        let arranged = id.and_then(|id| frame.arranged_tree.get(id).map(|node| node.frame));
        let quad = id.and_then(|id| {
            frame
                .render_extract
                .list
                .commands
                .iter()
                .find(|command| command.node_id == id && command.kind == UiRenderCommandKind::Quad)
                .map(|command| command.frame)
        });
        (arranged, quad)
    };
    eprintln!(
        "scroll-padding {stamp}: shell={:?} padding={:?} measured={:?} scroll={:?} wheel={:?} first={:?} last={:?}",
        frame.arranged_tree.get(root_id).map(|node| node.frame),
        root.layout_padding,
        root.layout_cache.content_size,
        root.scroll_state,
        wheel.map(|result| (result.handled_by, result.diagnostics.scroll_defaulted)),
        row("First"),
        row("Last"),
    );
}

fn assert_fitted_rows(surface: &UiSurface, pad: f32) {
    let frame = surface.surface_frame();
    for (control, y) in [("First", pad), ("Last", pad + ROW)] {
        let id = surface.unique_control_node_id(control).unwrap();
        let arranged = frame.arranged_tree.get(id).unwrap().frame;
        assert_eq!((arranged.y, arranged.height), (y, ROW), "{control}");
        assert!(
            frame.render_extract.list.commands.iter().any(|command| {
                command.node_id == id
                    && command.kind == UiRenderCommandKind::Quad
                    && command.frame == arranged
            }),
            "{control} paint must use the published layout frame"
        );
        let hit = hit_test_surface_frame(&frame, UiPoint::new(20.0, y + 25.0));
        assert_eq!(
            hit.top_hit,
            Some(id),
            "{control} hit must use the same frame"
        );
    }
    assert_eq!(scroll_offset(surface), 0.0);
}

#[test]
fn scroll_padding_exact_fit_does_not_scroll() {
    let mut surface = surface_with_padding(10, 100.0);
    log_frame("fit-before", &surface, None);
    assert_fitted_rows(&surface, 10.0);
    let wheel = wheel_to_end(&mut surface);
    surface.rebuild_dirty(UiSize::new(WIDTH, 100.0)).unwrap();
    log_frame("fit-after-wheel", &surface, Some(&wheel));
    assert_fitted_rows(&surface, 10.0);
    assert!(!wheel.diagnostics.scroll_defaulted);
}

#[test]
fn scroll_padding_overflow_clamps_to_start_as_the_viewport_grows() {
    let mut surface = surface_with_padding(10, 70.0);
    log_frame("overflow-before", &surface, None);
    let wheel = wheel_to_end(&mut surface);
    surface.rebuild_dirty(UiSize::new(WIDTH, 70.0)).unwrap();
    log_frame("overflow-end", &surface, Some(&wheel));
    let end_offset = scroll_offset(&surface);
    let old_frame = surface.surface_frame();

    surface.rebuild_dirty(UiSize::new(WIDTH, 100.0)).unwrap();
    log_frame("overflow-after-resize", &surface, None);
    assert_eq!(wheel.handled_by, Some(surface.tree.roots[0]));
    assert!(wheel.diagnostics.scroll_defaulted);
    assert_fitted_rows(&surface, 10.0);
    assert_eq!(end_offset, 30.0);
    let first = surface.unique_control_node_id("First").unwrap();
    assert_eq!(old_frame.arranged_tree.get(first).unwrap().frame.y, -20.0);
}

#[test]
fn scroll_padding_zero_padding_keeps_existing_end_and_fit() {
    let mut surface = surface_with_padding(0, 70.0);
    let wheel = wheel_to_end(&mut surface);
    surface.rebuild_dirty(UiSize::new(WIDTH, 70.0)).unwrap();
    log_frame("zero-padding-end", &surface, Some(&wheel));
    assert!(wheel.diagnostics.scroll_defaulted);
    assert_eq!(scroll_offset(&surface), 10.0);

    surface.rebuild_dirty(UiSize::new(WIDTH, 80.0)).unwrap();
    let wheel = wheel_to_end(&mut surface);
    surface.rebuild_dirty(UiSize::new(WIDTH, 80.0)).unwrap();
    log_frame("zero-padding-fit", &surface, Some(&wheel));
    assert_fitted_rows(&surface, 0.0);
    assert!(!wheel.diagnostics.scroll_defaulted);
}

fn scroll_padding_control_surface(source: &str, size: UiSize) -> UiSurface {
    let document = UiZuiAssetLoader::load_zui_str(source).unwrap();
    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("scroll-padding-controls"),
        &document,
        &compiled,
    )
    .unwrap();
    surface.compute_layout(size).unwrap();
    surface
}

fn scroll_padding_dispatch(
    surface: &mut UiSurface,
    point: UiPoint,
    delta: f32,
) -> UiPointerDispatchResult {
    surface
        .dispatch_pointer_event(
            &UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )
        .unwrap()
}

#[test]
fn scroll_padding_horizontal_uses_left_and_right_not_top_and_bottom() {
    let source = r#"
[asset]
kind = "view"
id = "res://ui/tests/scroll_padding_horizontal.zui"
version = 2

[root]
node = "scroll"

[nodes.scroll]
component = "ScrollableBox"
props = { input_hoverable = true }
layout = { container = { kind = "ScrollableBox", axis = "Horizontal" }, input_policy = "Receive", padding = { left = 7, right = 13, top = 3, bottom = 5 } }
children = [{ node = "first" }, { node = "last" }]

[nodes.first]
component = "Button"
control_id = "First"
props = { text = "First" }
layout = { width = { min = 40, preferred = 40, max = 40, stretch = "Fixed" }, height = { min = 24, preferred = 24, max = 24, stretch = "Fixed" } }

[nodes.last]
component = "Button"
control_id = "Last"
props = { text = "Last" }
layout = { width = { min = 40, preferred = 40, max = 40, stretch = "Fixed" }, height = { min = 24, preferred = 24, max = 24, stretch = "Fixed" } }
"#;
    let size = UiSize::new(90.0, 60.0);
    let mut surface = scroll_padding_control_surface(source, size);
    let root_id = surface.tree.roots[0];
    let first = surface.unique_control_node_id("First").unwrap();
    let last = surface.unique_control_node_id("Last").unwrap();
    let before = surface.surface_frame();
    let point = UiPoint::new(20.0, 15.0);
    let wheel = scroll_padding_dispatch(&mut surface, point, 60.0);
    surface.rebuild_dirty(size).unwrap();
    let after = surface.surface_frame();
    log_frame("horizontal-end", &surface, Some(&wheel));

    let root = surface.tree.node(root_id).unwrap();
    assert_eq!(root.layout_padding, UiMargin::new(7.0, 3.0, 13.0, 5.0));
    assert_eq!(before.arranged_tree.get(first).unwrap().frame.x, 7.0);
    assert_eq!(wheel.route.target, Some(first));
    assert_eq!(wheel.handled_by, Some(root_id));
    assert!(wheel.diagnostics.scroll_defaulted);
    assert_eq!(scroll_offset(&surface), 10.0);
    let first_frame = after.arranged_tree.get(first).unwrap().frame;
    let last_frame = after.arranged_tree.get(last).unwrap().frame;
    assert_eq!((first_frame.x, first_frame.y), (-3.0, 3.0));
    assert_eq!(
        (last_frame.x, last_frame.y, last_frame.right()),
        (37.0, 3.0, 77.0)
    );
    assert_eq!(hit_test_surface_frame(&after, point).top_hit, Some(first));
}

#[test]
fn scroll_padding_exact_fit_inner_leaves_first_wheel_to_outer_owner() {
    let source = r#"
[asset]
kind = "view"
id = "res://ui/tests/scroll_padding_nested.zui"
version = 2

[root]
node = "outer"

[nodes.outer]
component = "ScrollableBox"
control_id = "Outer"
props = { input_hoverable = true }
layout = { container = { kind = "ScrollableBox", axis = "Vertical" }, input_policy = "Receive" }
children = [{ node = "inner" }, { node = "tail" }]

[nodes.inner]
component = "ScrollableBox"
control_id = "Inner"
props = { input_hoverable = true }
layout = { container = { kind = "ScrollableBox", axis = "Vertical" }, input_policy = "Receive", padding = { top = 10, bottom = 10 }, width = { stretch = "Stretch" }, height = { min = 100, preferred = 100, max = 100, stretch = "Fixed" } }
children = [{ node = "first" }, { node = "last" }]

[nodes.first]
component = "Button"
control_id = "First"
props = { text = "First" }
layout = { width = { stretch = "Stretch" }, height = { min = 40, preferred = 40, max = 40, stretch = "Fixed" } }

[nodes.last]
component = "Button"
control_id = "Last"
props = { text = "Last" }
layout = { width = { stretch = "Stretch" }, height = { min = 40, preferred = 40, max = 40, stretch = "Fixed" } }

[nodes.tail]
component = "Container"
control_id = "Tail"
layout = { input_policy = "Ignore", width = { stretch = "Stretch" }, height = { min = 80, preferred = 80, max = 80, stretch = "Fixed" } }
"#;
    let size = UiSize::new(WIDTH, 100.0);
    let mut surface = scroll_padding_control_surface(source, size);
    let outer = surface.unique_control_node_id("Outer").unwrap();
    let inner = surface.unique_control_node_id("Inner").unwrap();
    let first = surface.unique_control_node_id("First").unwrap();
    let last = surface.unique_control_node_id("Last").unwrap();
    let tail = surface.unique_control_node_id("Tail").unwrap();
    let before = surface.surface_frame();
    let point = UiPoint::new(20.0, 25.0);
    let wheel = scroll_padding_dispatch(&mut surface, point, 12.0);
    surface.rebuild_dirty(size).unwrap();
    let after = surface.surface_frame();
    let inner_state = surface.tree.node(inner).unwrap().scroll_state.unwrap();
    let outer_state = surface.tree.node(outer).unwrap().scroll_state.unwrap();
    let inner_frame = after.arranged_tree.get(inner).unwrap().frame;
    let first_frame = after.arranged_tree.get(first).unwrap().frame;
    let last_frame = after.arranged_tree.get(last).unwrap().frame;
    let tail_frame = after.arranged_tree.get(tail).unwrap().frame;
    eprintln!(
        "scroll-padding nested: route={:?} owner={:?} inner={inner_state:?}/{inner_frame:?} outer={outer_state:?} first={first_frame:?} last={last_frame:?} tail={tail_frame:?}",
        wheel.route.stacked, wheel.handled_by,
    );

    assert_eq!(before.arranged_tree.get(first).unwrap().frame.y, 10.0);
    assert_eq!(before.arranged_tree.get(tail).unwrap().frame.y, 100.0);
    assert_eq!(wheel.route.target, Some(first));
    let inner_position = wheel
        .route
        .stacked
        .iter()
        .position(|&id| id == inner)
        .unwrap();
    let outer_position = wheel
        .route
        .stacked
        .iter()
        .position(|&id| id == outer)
        .unwrap();
    assert!(inner_position < outer_position);
    assert_eq!(wheel.handled_by, Some(outer));
    assert!(wheel.diagnostics.scroll_defaulted);
    assert_eq!((inner_state.offset, outer_state.offset), (0.0, 12.0));
    assert_eq!((inner_frame.y, tail_frame.y), (-12.0, 88.0));
    assert_eq!(
        (first_frame.y - inner_frame.y, last_frame.y - inner_frame.y),
        (10.0, 50.0)
    );
    assert_eq!(hit_test_surface_frame(&after, point).top_hit, Some(first));
}

#![cfg(feature = "ui")]

use std::sync::Arc;

use zircon_runtime::ui::{
    icon_atlas::builtin_icon_supported,
    surface::{
        hit_test_surface_frame, UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface,
    },
    v2::{UiV2SurfaceBuilder, UiZuiAssetLoader},
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiTreeId},
    layout::UiSize,
    surface::{
        UiBrushPayload, UiPaintElement, UiPaintPayload, UiRenderResourceKind, UiSurfaceFrame,
    },
};

const VIEWPORT: UiSize = UiSize::new(320.0, 220.0);
const LEAF_TEXT: &str = "Opacity witness";
const BUILTIN_ICON: &str = "check-circle";
const LEAF_BACKGROUND: &str = "#2c526d80";
const LEAF_FOREGROUND: &str = "#f0c87740";

struct Fixture {
    surface: UiSurface,
    root: UiNodeId,
    parent: UiNodeId,
    middle: UiNodeId,
    leaf: UiNodeId,
    sibling: UiNodeId,
}

fn fixture(
    root: Option<f32>,
    parent: Option<f32>,
    middle: Option<f32>,
    leaf: Option<f32>,
) -> Fixture {
    fn opacity_prop(value: Option<f32>) -> String {
        value
            .map(|value| format!(", opacity = {value:?}"))
            .unwrap_or_default()
    }

    let root_opacity = opacity_prop(root);
    let parent_opacity = opacity_prop(parent);
    let middle_opacity = opacity_prop(middle);
    let leaf_opacity = opacity_prop(leaf);
    let source = format!(
        r##"
[asset]
kind = "view"
id = "res://ui/tests/parent_opacity.zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"
control_id = "OpacityRoot"
props = {{ background_color = "#111c28"{root_opacity} }}
layout = {{ width = {{ stretch = "Stretch" }}, height = {{ stretch = "Stretch" }}, container = {{ kind = "VerticalBox", gap = 12 }} }}
children = [{{ node = "parent" }}, {{ node = "sibling" }}]

[nodes.parent]
component = "VerticalBox"
control_id = "FadingParent"
props = {{ background_color = "#432860"{parent_opacity} }}
layout = {{ width = {{ stretch = "Stretch" }}, height = {{ min = 120, preferred = 120, max = 120, stretch = "Fixed" }}, container = {{ kind = "VerticalBox" }} }}
children = [{{ node = "middle" }}]

[nodes.middle]
component = "VerticalBox"
control_id = "OpacityMiddle"
props = {{ background_color = "#345678"{middle_opacity} }}
layout = {{ width = {{ stretch = "Stretch" }}, height = {{ min = 72, preferred = 72, max = 72, stretch = "Fixed" }}, container = {{ kind = "VerticalBox" }} }}
children = [{{ node = "leaf" }}]

[nodes.leaf]
component = "Label"
control_id = "PaintedLeaf"
classes = ["opacity-leaf"]
props = {{ text = "{LEAF_TEXT}", icon = "{BUILTIN_ICON}", input_clickable = true{leaf_opacity} }}
layout = {{ width = {{ stretch = "Stretch" }}, height = {{ min = 32, preferred = 32, max = 32, stretch = "Fixed" }} }}

[nodes.sibling]
component = "VerticalBox"
control_id = "FollowingSibling"
props = {{ background_color = "#374b5a", opacity = 0.625 }}
layout = {{ width = {{ stretch = "Stretch" }}, height = {{ min = 24, preferred = 24, max = 24, stretch = "Fixed" }} }}

[[stylesheets]]
id = "test-colors"

[[stylesheets.rules]]
selector = ".opacity-leaf"
set = {{ self = {{ background_color = "{LEAF_BACKGROUND}", foreground_color = "{LEAF_FOREGROUND}" }} }}
"##
    );
    let document = UiZuiAssetLoader::load_zui_str(&source).expect("parse public ZUI view");
    let mut surface =
        UiV2SurfaceBuilder::build_surface(UiTreeId::new("test.parent_opacity"), &document)
            .expect("compile and build retained ZUI surface");
    surface
        .compute_layout(VIEWPORT)
        .expect("run public UI layout");
    let id = |name: &str| {
        surface
            .unique_control_node_id(name)
            .unwrap_or_else(|| panic!("missing unique ZUI control {name}"))
    };
    let [root, parent, middle, leaf, sibling] = [
        "OpacityRoot",
        "FadingParent",
        "OpacityMiddle",
        "PaintedLeaf",
        "FollowingSibling",
    ]
    .map(id);
    let frame = surface.surface_frame();
    for (node, expected_parent) in [
        (root, None),
        (parent, Some(root)),
        (middle, Some(parent)),
        (leaf, Some(middle)),
        (sibling, Some(root)),
    ] {
        let arranged = frame.arranged_tree.get(node).expect("public arranged node");
        assert_eq!(arranged.parent, expected_parent);
        assert!(arranged.is_render_visible());
        assert!(arranged.frame.intersection(arranged.clip_frame).is_some());
    }
    Fixture {
        surface,
        root,
        parent,
        middle,
        leaf,
        sibling,
    }
}

fn one_paint<'a, T>(
    elements: &'a [UiPaintElement],
    node: UiNodeId,
    kind: &str,
    mut select: impl FnMut(&'a UiPaintPayload) -> Option<T>,
) -> (&'a UiPaintElement, T) {
    let mut found = elements
        .iter()
        .filter(|element| element.node_id == node)
        .filter_map(|element| select(&element.payload).map(|value| (element, value)));
    let result = found
        .next()
        .unwrap_or_else(|| panic!("missing {kind} paint for {node:?}"));
    assert!(
        found.next().is_none(),
        "duplicate {kind} paint for {node:?}"
    );
    result
}

fn solid_fill(elements: &[UiPaintElement], node: UiNodeId) -> (&UiPaintElement, &str) {
    one_paint(elements, node, "solid background", |payload| {
        if let UiPaintPayload::Brush { brushes } = payload {
            if let Some(UiBrushPayload::Solid(fill)) = &brushes.fill {
                return Some(fill.color.as_str());
            }
        }
        None
    })
}

fn fill_opacities<const N: usize>(frame: &UiSurfaceFrame, nodes: [UiNodeId; N]) -> [f32; N] {
    let elements = frame.render_extract.list.to_paint_elements();
    nodes.map(|node| solid_fill(&elements, node).0.effects.opacity)
}

fn assert_alphas<const N: usize>(label: &str, actual: [f32; N], expected: [f32; N]) {
    println!("{label}: observed {actual:?}; expected {expected:?}");
    for (index, (actual, expected)) in actual.into_iter().zip(expected).enumerate() {
        assert!(
            (actual - expected).abs() < 1.0e-6,
            "{label}: alpha[{index}] {actual} != {expected}"
        );
    }
}

#[test]
fn parent_opacity_unset_and_explicit_one_are_identity() {
    for root_opacity in [None, Some(1.0)] {
        let scene = fixture(root_opacity, None, Some(1.0), None);
        assert_alphas(
            "unset/identity root, parent, middle and leaf",
            fill_opacities(
                &scene.surface.surface_frame(),
                [scene.root, scene.parent, scene.middle, scene.leaf],
            ),
            [1.0, 1.0, 1.0, 1.0],
        );
    }
    let scene = fixture(Some(0.8), None, None, None);
    assert_alphas(
        "root self opacity applied once",
        fill_opacities(&scene.surface.surface_frame(), [scene.root]),
        [0.8],
    );
}

#[test]
fn parent_opacity_single_ancestor_multiplies_local_alpha_once() {
    let scene = fixture(None, Some(0.5), Some(0.5), None);
    assert_alphas(
        "parent, direct child",
        fill_opacities(&scene.surface.surface_frame(), [scene.parent, scene.middle]),
        [0.5, 0.25],
    );
}

#[test]
fn parent_opacity_two_ancestors_restore_the_following_sibling() {
    let scene = fixture(None, Some(0.5), Some(0.5), Some(0.8));
    let elements = scene
        .surface
        .surface_frame()
        .render_extract
        .list
        .to_paint_elements();
    let parent = solid_fill(&elements, scene.parent).0;
    let middle = solid_fill(&elements, scene.middle).0;
    let leaf = solid_fill(&elements, scene.leaf).0;
    let sibling = solid_fill(&elements, scene.sibling).0;
    assert!(parent.paint_order < middle.paint_order);
    assert!(middle.paint_order < leaf.paint_order);
    assert!(leaf.paint_order < sibling.paint_order);
    assert_alphas(
        "parent, middle, leaf, following sibling",
        [
            parent.effects.opacity,
            middle.effects.opacity,
            leaf.effects.opacity,
            sibling.effects.opacity,
        ],
        [0.5, 0.25, 0.2, 0.625],
    );
}

#[test]
fn parent_opacity_zero_removes_visual_contribution_but_not_picking() {
    let shown = fixture(None, Some(1.0), None, Some(1.0));
    let hidden = fixture(None, Some(0.0), None, Some(1.0));
    let shown_frame = shown.surface.surface_frame();
    let hidden_frame = hidden.surface.surface_frame();
    let shown_leaf = shown_frame.arranged_tree.get(shown.leaf).unwrap();
    let hidden_leaf = hidden_frame.arranged_tree.get(hidden.leaf).unwrap();
    assert_eq!(shown_leaf.frame, hidden_leaf.frame);
    let point = shown_leaf.frame.center();
    assert_eq!(
        hit_test_surface_frame(&shown_frame, point).top_hit,
        Some(shown.leaf)
    );
    assert_eq!(
        hit_test_surface_frame(&hidden_frame, point).top_hit,
        Some(hidden.leaf)
    );
    assert_alphas(
        "visible leaf control",
        fill_opacities(&shown_frame, [shown.leaf]),
        [1.0],
    );

    // A zero-opacity primitive may be retained or culled. It must not leave visible paint.
    let elements = hidden_frame.render_extract.list.to_paint_elements();
    for element in elements.iter().filter(|element| {
        element.node_id == hidden.leaf && !matches!(element.payload, UiPaintPayload::Empty)
    }) {
        assert_alphas(
            "leaf primitive under zero parent",
            [element.effects.opacity],
            [0.0],
        );
    }
}

fn styled_leaf_alphas(scene: &Fixture) -> [f32; 5] {
    // The public paint boundary intentionally carries raw color and opacity separately.
    // Decode only the known #RRGGBBAA alpha byte for the combined numeric oracle.
    let color_alpha = |rgba: &str| f32::from(u8::from_str_radix(&rgba[7..9], 16).unwrap()) / 255.0;
    let elements = scene
        .surface
        .surface_frame()
        .render_extract
        .list
        .to_paint_elements();
    let (background, background_color) = solid_fill(&elements, scene.leaf);
    let (text_element, text) = one_paint(&elements, scene.leaf, "text", |payload| match payload {
        UiPaintPayload::Text { text } => Some(text),
        _ => None,
    });
    let (icon_element, icon) = one_paint(&elements, scene.leaf, "built-in icon brush", |payload| {
        if let UiPaintPayload::Brush { brushes } = payload {
            if let Some(UiBrushPayload::Image(image)) = &brushes.fill {
                return Some(image);
            }
        }
        None
    });
    assert_eq!(background_color, LEAF_BACKGROUND);
    assert_eq!(text.source_text, LEAF_TEXT);
    let text_color = text.color.as_deref().expect("styled text color");
    assert_eq!(text_color, LEAF_FOREGROUND);
    assert_eq!(icon.resource.kind, UiRenderResourceKind::Icon);
    assert_eq!(icon.resource.id, BUILTIN_ICON);
    assert!(builtin_icon_supported(BUILTIN_ICON));
    assert!(
        icon.tint.is_none(),
        "neutral icon brush does not carry a tint"
    );
    let background_opacity = background.effects.opacity;
    let text_opacity = text_element.effects.opacity;
    [
        background_opacity,
        text_opacity,
        icon_element.effects.opacity,
        color_alpha(background_color) * background_opacity,
        color_alpha(text_color) * text_opacity,
    ]
}

#[test]
fn parent_opacity_preserves_style_alpha_and_fades_each_paint_once() {
    let own_only = fixture(None, None, None, Some(0.8));
    let inherited = fixture(None, Some(0.5), None, Some(0.8));
    assert_alphas(
        "self only: background/text/icon effects, background/text alphas",
        styled_leaf_alphas(&own_only),
        [0.8, 0.8, 0.8, 0.40156863, 0.20078431],
    );
    assert_alphas(
        "with ancestor: background/text/icon effects, background/text alphas",
        styled_leaf_alphas(&inherited),
        [0.4, 0.4, 0.4, 0.20078431, 0.10039216],
    );
}

#[test]
fn parent_opacity_parent_only_update_republishes_descendant_paint() {
    let mut scene = fixture(None, Some(1.0), None, Some(0.8));
    let ids = [scene.parent, scene.middle, scene.leaf, scene.sibling];
    let before = scene.surface.surface_frame();
    let baseline = [1.0, 1.0, 0.8, 0.625];
    assert_alphas(
        "before parent-only update",
        fill_opacities(&before, ids),
        baseline,
    );

    let mutation = scene
        .surface
        .mutate_property(UiPropertyMutationRequest::new(
            scene.parent,
            "opacity",
            UiValue::Float(0.5),
        ))
        .expect("mutate only the parent's authored opacity");
    assert_eq!(
        mutation.status,
        UiPropertyMutationStatus::Accepted,
        "{mutation:?}"
    );
    let report = scene
        .surface
        .rebuild_dirty(VIEWPORT)
        .expect("rebuild the dirty surface");
    let after = scene.surface.surface_frame();
    assert!(
        !report.layout_recomputed && report.render_rebuilt && !report.hit_grid_rebuilt,
        "opacity-only rebuild report: {report:?}"
    );
    assert!(Arc::ptr_eq(&before.arranged_tree, &after.arranged_tree));
    assert!(Arc::ptr_eq(&before.hit_grid, &after.hit_grid));
    assert!(after.domain_generations.render > before.domain_generations.render);
    assert_alphas(
        "old frame remains immutable",
        fill_opacities(&before, ids),
        baseline,
    );
    assert_alphas(
        "new parent/middle/leaf and unaffected sibling",
        fill_opacities(&after, ids),
        [0.5, 0.5, 0.4, 0.625],
    );
}

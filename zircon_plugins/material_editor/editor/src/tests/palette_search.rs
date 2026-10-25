use super::{
    search_material_graph_node_palette, MaterialGraphNodeSearch,
    MaterialGraphNodeSearchNavigation as Navigation,
};
use zircon_editor::core::{
    asset::AssetTypeId,
    editor_authoring_extension::{GraphNodeDescriptor, GraphNodePaletteDescriptor},
};
use zircon_runtime_interface::resource::ResourceKind;

fn ids(nodes: &[GraphNodeDescriptor]) -> Vec<&str> {
    nodes.iter().map(GraphNodeDescriptor::id).collect()
}

#[test]
fn material_graph_search_empty_query_preserves_registered_actions_and_pins() {
    let registered = super::plugin::material_node_palette();
    for query in ["", "  \t\n", "\u{3000}"] {
        assert_eq!(
            search_material_graph_node_palette(query).as_slice(),
            registered.nodes()
        );
    }
    assert_eq!(
        ids(registered.nodes()),
        [
            "output",
            "texture_sample",
            "scalar_parameter",
            "vector_parameter",
            "add",
            "multiply"
        ]
    );
}

#[test]
fn material_graph_search_ands_words_across_caption_category_and_schema_key() {
    assert_eq!(
        ids(&search_material_graph_node_palette("  PaRaMeTeR scalar ")),
        ["scalar_parameter"]
    );
    assert_eq!(
        ids(&search_material_graph_node_palette("math")),
        ["add", "multiply"]
    );
    assert_eq!(
        ids(&search_material_graph_node_palette("texture_sample")),
        ["texture_sample"]
    );
    assert!(search_material_graph_node_palette("scalar texture").is_empty());
    assert!(search_material_graph_node_palette("function").is_empty());
}

#[test]
fn material_graph_search_uses_unicode_captions_without_inventing_aliases() {
    let palette = GraphNodePaletteDescriptor::new(
        "test.material.localized",
        AssetTypeId::from_resource_kind(ResourceKind::MaterialGraph),
    )
    .with_node(GraphNodeDescriptor::new(
        "scalar_parameter",
        "标量参数 ÅNGSTRÖM",
        "参数",
    ))
    .with_node(GraphNodeDescriptor::new(
        "vector_parameter",
        "向量参数",
        "参数",
    ));
    let found = super::palette_search::matching_actions(&palette, "标量 ångström");
    assert_eq!(ids(&found), ["scalar_parameter"]);
    assert_eq!(found[0].display_name(), "标量参数 ÅNGSTRÖM");
    assert!(super::palette_search::matching_actions(&palette, "噪声").is_empty());
}

#[test]
fn material_graph_search_refresh_retains_action_identity_and_rejects_stale_rows() {
    let mut search = MaterialGraphNodeSearch::default();
    assert!(search.select_id("multiply"));
    search.set_query("math");
    assert_eq!(search.selected().unwrap().id(), "multiply");
    assert!(!search.select_id("texture_sample"));
    assert_eq!(search.selected().unwrap().id(), "multiply");
    search.set_query("scalar");
    assert_eq!(search.selected().unwrap().id(), "scalar_parameter");
}

#[test]
fn material_graph_search_keyboard_steps_and_pages_clamp_to_real_results() {
    let mut search = MaterialGraphNodeSearch::new("math");
    search.navigate(Navigation::Previous);
    assert_eq!(search.selected().unwrap().id(), "add");
    search.navigate(Navigation::Next);
    search.navigate(Navigation::Next);
    assert_eq!(search.selected().unwrap().id(), "multiply");
    search.navigate(Navigation::PageUp);
    assert_eq!(search.selected().unwrap().id(), "add");
    search.navigate(Navigation::PageDown);
    assert_eq!(search.selected().unwrap().id(), "multiply");
    search.navigate(Navigation::First);
    assert_eq!(search.selected().unwrap().id(), "add");
    search.navigate(Navigation::Last);
    assert_eq!(search.selected().unwrap().id(), "multiply");
}

#[test]
fn material_graph_search_empty_result_cannot_activate_and_does_not_close_search() {
    let mut search = MaterialGraphNodeSearch::new("unimplemented function");
    assert!(search.matches().is_empty());
    assert!(search.selected().is_none());
    for direction in [Navigation::Next, Navigation::PageUp, Navigation::Last] {
        search.navigate(direction);
        assert!(search.selected().is_none());
    }
    assert!(search.activate().is_none());
    assert!(search.is_open());
    search.set_query("multiply");
    assert_eq!(search.activate().unwrap().id(), "multiply");
}

#[test]
fn material_graph_search_activation_returns_exact_registered_descriptor_once() {
    let mut search = MaterialGraphNodeSearch::new("vector");
    let expected = search.matches()[0].clone();
    assert_eq!(search.activate(), Some(expected));
    assert!(!search.is_open());
    assert!(search.selected().is_none());
    assert!(search.activate().is_none());
    assert!(!search.select_id("output"));
    search.set_query("output");
    assert_eq!(search.query(), "vector");
}

#[test]
fn material_graph_search_cancel_does_not_create_an_action_or_reopen_on_input() {
    let mut search = MaterialGraphNodeSearch::new("parameter");
    search.cancel();
    assert!(search.activate_id("scalar_parameter").is_none());
    search.navigate(Navigation::Last);
    search.set_query("output");
    assert_eq!(search.query(), "parameter");
    assert!(!search.is_open());
    assert!(search.activate().is_none());
    assert_eq!(
        MaterialGraphNodeSearch::default().selected().unwrap().id(),
        "output"
    );
}

#[test]
fn material_graph_search_pointer_activates_clicked_identity_exactly_once() {
    let mut search = MaterialGraphNodeSearch::new("math");
    assert_eq!(search.selected().unwrap().id(), "add");
    let clicked = search
        .matches()
        .iter()
        .find(|action| action.id() == "multiply")
        .unwrap()
        .clone();
    assert_eq!(search.activate_id("multiply"), Some(clicked));
    assert!(!search.is_open());
    assert!(search.activate_id("add").is_none());
    assert!(search.activate_id("multiply").is_none());
    assert!(search.activate().is_none());
}

#[test]
fn material_graph_search_stale_pointer_preserves_current_results_and_selection() {
    let mut search = MaterialGraphNodeSearch::new("vector");
    let stale_row_id = search.selected().unwrap().id().to_owned();
    search.set_query("math");
    assert!(search.select_id("multiply"));
    let before = search.clone();
    assert!(search.activate_id(&stale_row_id).is_none());
    assert_eq!(search, before);
    assert_eq!(search.query(), "math");
    assert!(search.is_open());
    assert_eq!(search.selected().unwrap().id(), "multiply");
    assert_eq!(search.activate().unwrap().id(), "multiply");
}

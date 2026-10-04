use zircon_runtime_interface::resource::ResourceKind;

use super::builtin_asset_type_definition;

#[test]
fn builtin_ui_assets_share_the_document_toolkit_route() {
    for kind in [
        ResourceKind::UiLayout,
        ResourceKind::UiWidget,
        ResourceKind::UiStyle,
    ] {
        let toolkit = builtin_asset_type_definition(kind)
            .expect("built-in UI asset type should be registered")
            .toolkit()
            .expect("built-in UI asset type should declare a document toolkit");

        assert_eq!(toolkit.view_id(), "editor.ui_asset");
        assert_eq!(
            toolkit.open_operation().as_str(),
            "view.editor.ui_asset.open"
        );
    }
}

#[test]
fn builtin_animation_assets_have_their_document_toolkit_routes() {
    for (kind, view_id, operation) in [
        (
            ResourceKind::AnimationSequence,
            "editor.animation_sequence",
            "timeline_sequence.authoring.open",
        ),
        (
            ResourceKind::AnimationGraph,
            "editor.animation_graph",
            "animation_graph.authoring.open_graph",
        ),
        (
            ResourceKind::AnimationStateMachine,
            "editor.animation_graph",
            "animation_graph.authoring.open_state_machine",
        ),
    ] {
        let toolkit = builtin_asset_type_definition(kind)
            .expect("built-in animation asset type should be registered")
            .toolkit()
            .expect("built-in animation asset type should declare a document toolkit");

        assert_eq!(toolkit.view_id(), view_id);
        assert_eq!(toolkit.open_operation().as_str(), operation);
    }
}

#[test]
fn builtin_lookup_does_not_construct_an_owned_asset_type_id() {
    let source = include_str!("../builtin.rs");
    let owned_lookup = [".get(&AssetTypeId::", "from_resource_kind(kind))"].concat();
    assert!(!source.contains(&owned_lookup));
}

use super::*;

use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiPrototypeStoreBuilder};
use zircon_runtime_interface::ui::template::UiCompiledNodeId;

const ROOT_ASSET: &str = "res://ui/views/main.zui";
const WIDGET_ASSET: &str = "res://ui/widgets/search_box.zui";

const ROOT_PROTOTYPE: &str = r#"
[asset]
kind = "layout"
id = "res://ui/views/main.zui"
version = 3

[imports]
widgets = ["res://ui/widgets/search_box.zui#SearchBox"]

[root]
node = "search"

[nodes.search]
kind = "reference"
component_ref = "res://ui/widgets/search_box.zui#SearchBox"

[[nodes.search.bindings]]
id = "instance/onSubmit"
event = "Click"
route = "Route.Submit"
"#;

const WIDGET_PROTOTYPE: &str = r#"
[asset]
kind = "widget"
id = "res://ui/widgets/search_box.zui"
version = 3

[components.SearchBox]
root = "search_root"

[nodes.search_root]
kind = "native"
type = "Button"

[[nodes.search_root.bindings]]
id = "SearchInput/onChange"
event = "Change"
route = "Route.Change"
"#;

#[test]
fn compiled_binding_ownership_indexes_imported_nodes_and_caller_bindings() {
    let root = UiAssetLoader::load_flat_prototype_toml_str(ROOT_PROTOTYPE).unwrap();
    let widget = UiAssetLoader::load_flat_prototype_toml_str(WIDGET_PROTOTYPE).unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(root);
    let _ = builder.insert(widget);
    let store = builder.build().unwrap();
    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset(ROOT_ASSET, &store)
        .unwrap();
    let program = compiled.template_instance().binding_program();
    let instance_handle = program
        .iter_bindings()
        .find(|binding| program.binding_name(binding.handle) == Some("instance/onSubmit"))
        .map(|binding| binding.handle)
        .unwrap();
    let widget_handle = program
        .iter_bindings()
        .find(|binding| program.binding_name(binding.handle) == Some("SearchInput/onChange"))
        .map(|binding| binding.handle)
        .unwrap();

    assert_eq!(
        program.node_asset_id(program.binding(instance_handle).unwrap().node_id),
        Some(WIDGET_ASSET)
    );
    assert_eq!(
        program.node_asset_id(program.binding(widget_handle).unwrap().node_id),
        Some(WIDGET_ASSET)
    );
    assert_eq!(program.binding_asset_id(instance_handle), Some(ROOT_ASSET));
    assert_eq!(program.binding_asset_id(widget_handle), Some(WIDGET_ASSET));

    let tree_id = tree_id("runtime.ui.binding-ownership");
    let mut index = UiAssetSurfaceIndex::new();
    index.record_binding_program(tree_id.clone(), &program);

    assert_eq!(
        index
            .compiled_nodes_for_asset(WIDGET_ASSET)
            .map(|target| target.node_id)
            .collect::<Vec<_>>(),
        vec![UiCompiledNodeId::new(0)]
    );
    assert_eq!(
        index
            .bindings_for_asset(ROOT_ASSET)
            .map(|target| target.handle)
            .collect::<Vec<_>>(),
        vec![instance_handle]
    );
    assert_eq!(
        index
            .bindings_for_asset(WIDGET_ASSET)
            .map(|target| target.handle)
            .collect::<Vec<_>>(),
        vec![widget_handle]
    );
    assert_eq!(
        index
            .surfaces_for_asset(WIDGET_ASSET)
            .cloned()
            .collect::<Vec<_>>(),
        vec![tree_id]
    );
}

#[test]
fn compiled_binding_ownership_accepts_legacy_program_without_owner_tables() {
    const ROOT_ASSET: &str = "res://ui/views/legacy.zui";

    let document = UiAssetLoader::load_toml_str(
        r#"
[asset]
kind = "layout"
id = "res://ui/views/legacy.zui"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"

[[root.bindings]]
id = "legacy/onClick"
event = "Click"
"#,
    )
    .unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let program = compiled.template_instance().binding_program();
    let handle = program
        .handle_for_node_binding(UiCompiledNodeId::new(0), 0)
        .unwrap();

    let mut legacy = serde_json::to_value(program).unwrap();
    {
        let legacy_object = legacy.as_object_mut().unwrap();
        legacy_object.remove("asset_ids");
        for node in legacy_object
            .get_mut("nodes")
            .and_then(serde_json::Value::as_array_mut)
            .unwrap()
        {
            node.as_object_mut().unwrap().remove("owner_asset_id");
        }
        for binding in legacy_object
            .get_mut("bindings")
            .and_then(serde_json::Value::as_array_mut)
            .unwrap()
        {
            binding.as_object_mut().unwrap().remove("owner_asset_id");
        }
    }

    let restored: zircon_runtime_interface::ui::template::UiCompiledBindingProgram =
        serde_json::from_value(legacy).unwrap();
    assert!(restored.is_well_formed());
    assert_eq!(restored.asset_count(), 1);
    assert_eq!(
        restored.node_asset_id(UiCompiledNodeId::new(0)),
        Some(ROOT_ASSET)
    );
    assert_eq!(restored.binding_asset_id(handle), Some(ROOT_ASSET));
}

#[test]
fn replacing_binding_ownership_preserves_independent_surface_assets() {
    const RESOURCE_ASSET: &str = "res://ui/icons/search.svg";

    let first = compile_imported_program(ROOT_PROTOTYPE);
    let replacement_source = ROOT_PROTOTYPE.replace("Route.Submit", "Route.SubmitV2");
    let replacement = compile_imported_program(&replacement_source);
    assert_ne!(first.generation(), replacement.generation());

    let tree_id = tree_id("runtime.ui.binding-ownership-resource-union");
    let mut index = UiAssetSurfaceIndex::new();
    index.record_surface_assets(tree_id.clone(), [ROOT_ASSET, RESOURCE_ASSET]);
    index.record_binding_program(tree_id.clone(), &first);
    index.record_binding_program(tree_id.clone(), &replacement);

    assert_eq!(
        index.assets_for_surface(&tree_id),
        [ROOT_ASSET.to_string(), RESOURCE_ASSET.to_string()]
    );
    assert_eq!(
        index
            .surfaces_for_asset(RESOURCE_ASSET)
            .cloned()
            .collect::<Vec<_>>(),
        vec![tree_id]
    );
    assert_eq!(
        index
            .bindings_for_asset(ROOT_ASSET)
            .map(|target| target.handle)
            .collect::<Vec<_>>(),
        replacement
            .iter_bindings()
            .filter(|binding| replacement.binding_asset_id(binding.handle) == Some(ROOT_ASSET))
            .map(|binding| binding.handle)
            .collect::<Vec<_>>()
    );
}

fn compile_imported_program(
    root_source: &str,
) -> zircon_runtime_interface::ui::template::UiCompiledBindingProgram {
    let root = UiAssetLoader::load_flat_prototype_toml_str(root_source).unwrap();
    let widget = UiAssetLoader::load_flat_prototype_toml_str(WIDGET_PROTOTYPE).unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(root);
    let _ = builder.insert(widget);
    let store = builder.build().unwrap();
    UiDocumentCompiler::default()
        .compile_prototype_asset(ROOT_ASSET, &store)
        .unwrap()
        .into_template_instance()
        .binding_program()
        .clone()
}

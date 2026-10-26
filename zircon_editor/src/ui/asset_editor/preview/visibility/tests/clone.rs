use std::sync::Arc;

use crate::ui::asset_editor::preview::preview_host::UiAssetPreviewHost;
use zircon_runtime::ui::v2::{UiV2AssetLoader, UiV2DocumentCompiler, UiV2PrototypeStore};
use zircon_runtime_interface::ui::layout::UiSize;

use super::{
    UiPreviewVisibilityError as Error, UiPreviewVisibilityOutcome as Outcome,
    UiPreviewVisibilitySource, UiPreviewVisibilityTarget,
};

#[test]
fn cloned_host_cannot_inherit_gesture_even_while_sharing_the_cached_frame() {
    let uri = "asset://ui/clone_probe.zui";
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "ui.test.clone_probe"
version = 2
[root]
node = "root"
[nodes.root]
component = "Panel"
children = [{ node = "child" }]
[nodes.child]
component = "Label"
props = { text = "Clone lease" }
"#,
    )
    .unwrap();
    let mut store = UiV2PrototypeStore::new();
    store.insert_with_source_path(document.clone(), uri);
    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&document, &store).unwrap();
    let mut host =
        UiAssetPreviewHost::new_v2(UiSize::new(800.0, 600.0), &document, &compiled).unwrap();
    let gesture = host
        .begin_visibility_gesture(
            UiPreviewVisibilitySource {
                document_uri: uri.to_string(),
                asset_id: document.asset.id.clone(),
                source_revision: 0,
                last_valid_generation: 0,
            },
            UiPreviewVisibilityTarget {
                source_path: uri.to_string(),
                root_node_id: "root".to_string(),
                source_node_id: "child".to_string(),
                instance_path: Vec::new(),
            },
        )
        .unwrap();
    let initial = host.surface().surface_frame();
    let mut clone = host.clone();
    assert!(Arc::ptr_eq(&initial, &clone.surface().surface_frame()));
    assert!(matches!(
        clone.apply_visibility_gesture(&gesture, &initial, 1, Some(false)),
        Outcome::Rejected(Error::RetiredGesture)
    ));
    assert!(matches!(
        host.apply_visibility_gesture(&gesture, &initial, 1, Some(false)),
        Outcome::Reported { .. }
    ));
}

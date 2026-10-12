use std::path::PathBuf;
use std::sync::Arc;

use super::NativePluginDiscoveryRefreshInput;

#[test]
fn load_manifest_input_clones_share_export_root() {
    let input =
        NativePluginDiscoveryRefreshInput::load_manifest(PathBuf::from("export/plugins/native"));
    let cloned = input.clone();

    let (
        NativePluginDiscoveryRefreshInput::LoadManifest {
            export_root: original,
        },
        NativePluginDiscoveryRefreshInput::LoadManifest {
            export_root: cloned,
        },
    ) = (&input, &cloned)
    else {
        panic!("load-manifest constructor preserves the input variant");
    };

    assert!(Arc::ptr_eq(original, cloned));
}

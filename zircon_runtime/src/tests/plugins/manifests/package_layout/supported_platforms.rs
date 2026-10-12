use super::super::{for_each_static_plugin_manifest, non_empty_string_array_values};
use super::arrays::assert_unique_entries;

#[test]
fn plugin_tomls_declare_known_supported_platforms() {
    for_each_static_plugin_manifest(|relative_path, table| {
        if table.get("supported_platforms").is_none() {
            return;
        }
        let platforms =
            non_empty_string_array_values(table, relative_path, "top-level", "supported_platforms");
        assert_unique_entries(
            relative_path,
            "top-level",
            "supported_platforms",
            &platforms,
        );

        for platform in platforms {
            assert!(
                matches!(
                    platform,
                    "windows"
                        | "linux"
                        | "macos"
                        | "android"
                        | "ios"
                        | "web_gpu"
                        | "wasm"
                        | "headless"
                ),
                "plugin manifest {relative_path:?} top-level supported platform `{platform}` should be a known export target platform"
            );
        }
    });
}

use super::super::for_each_static_plugin_manifest;
use super::{shape::assert_dot_namespaced_option_key, traversal::visit_option_rows};

#[test]
fn plugin_tomls_declare_option_keys_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_option_rows(
            table,
            relative_path,
            false,
            &mut |_option, key, option_context| {
                assert_dot_namespaced_option_key(relative_path, option_context, key);
            },
        );
    });
}

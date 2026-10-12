use super::super::{
    for_each_static_plugin_manifest, visit_asset_importer_required_capabilities,
    visit_option_required_capabilities,
};
use super::capabilities::static_declared_capabilities;
use super::capability_assertions::assert_declared_or_host_capability;

#[test]
fn plugin_tomls_declare_required_capability_gates_are_declared_or_host_owned() {
    let declared_capabilities = static_declared_capabilities();

    for_each_static_plugin_manifest(|relative_path, table| {
        visit_option_required_capabilities(table, relative_path, &mut |key, capability| {
            let context = format!("plugin option `{key}` required_capability");
            assert_declared_or_host_capability(
                relative_path,
                &context,
                capability,
                &declared_capabilities,
            );
        });
        visit_asset_importer_required_capabilities(
            table,
            relative_path,
            &mut |importer_id, capability| {
                let context = format!("asset importer `{importer_id}` required_capabilities");
                assert_declared_or_host_capability(
                    relative_path,
                    &context,
                    capability,
                    &declared_capabilities,
                );
            },
        );
    });
}

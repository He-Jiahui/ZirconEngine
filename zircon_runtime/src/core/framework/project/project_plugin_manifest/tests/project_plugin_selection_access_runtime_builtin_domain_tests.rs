use crate::builtin::RuntimePluginId;

use super::ProjectPluginSelection;

#[test]
fn default_ui_selection_is_builtin_but_explicit_external_ui_is_not() {
    let ui = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Ui, true, true);
    assert!(ui.is_runtime_builtin_domain());
    assert!(!ui
        .with_runtime_crate("zircon_plugin_ui_runtime")
        .is_runtime_builtin_domain());
    assert!(
        !ProjectPluginSelection::runtime_plugin(RuntimePluginId::Rendering, true, true)
            .is_runtime_builtin_domain()
    );
}

use super::super::super::types::PendingOptionalFeatureManifest;

pub(in super::super) fn push_optional_feature_module(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    name: &mut Option<String>,
    kind: &mut Option<zircon_runtime::plugin::PluginModuleKind>,
    crate_name: &mut Option<String>,
    target_modes: &mut Vec<zircon_runtime::RuntimeTargetMode>,
    capabilities: &mut Vec<String>,
) {
    let Some(name) = name.take() else {
        return;
    };
    feature
        .as_mut()
        .expect("optional feature module should have a parent feature")
        .modules
        .push((
            name,
            kind.take()
                .expect("optional feature module should declare kind"),
            crate_name
                .take()
                .expect("optional feature module should declare crate_name"),
            std::mem::take(target_modes),
            std::mem::take(capabilities),
        ));
}

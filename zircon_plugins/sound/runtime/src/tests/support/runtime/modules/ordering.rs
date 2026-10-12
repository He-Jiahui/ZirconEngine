use super::super::super::types::OptionalFeatureModuleSignature;

pub(super) fn sort_module_signatures(modules: &mut [OptionalFeatureModuleSignature]) {
    modules.sort_unstable_by_key(|module| module.0.clone());
}

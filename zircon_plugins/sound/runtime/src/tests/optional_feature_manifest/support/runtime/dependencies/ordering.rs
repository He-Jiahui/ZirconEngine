use super::super::super::types::OptionalFeatureDependencySignature;

pub(super) fn sort_dependency_signatures(dependencies: &mut [OptionalFeatureDependencySignature]) {
    dependencies.sort_unstable();
}

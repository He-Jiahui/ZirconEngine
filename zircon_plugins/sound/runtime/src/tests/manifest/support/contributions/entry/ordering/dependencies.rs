use super::super::super::StaticDependency;

pub(super) fn sort_static_dependencies(dependencies: &mut [StaticDependency]) {
    dependencies.sort_unstable();
}

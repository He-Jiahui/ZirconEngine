use super::super::super::StaticModule;

pub(super) fn sort_static_modules(modules: &mut [StaticModule]) {
    modules.sort_unstable_by(|left, right| left.0.cmp(&right.0));
}

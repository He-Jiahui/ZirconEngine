#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::plugin::runtime_plugin) struct RuntimePluginPackageValidationMetrics {
    pub projection_builds: usize,
    pub standalone_feature_projection_builds: usize,
    pub embedded_feature_projection_views: usize,
    pub identity_rows_indexed: usize,
    pub membership_probes: usize,
}

#[cfg(test)]
std::thread_local! {
    static OBSERVED_PACKAGE_PROJECTION_BUILDS: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
}

#[cfg(test)]
pub(super) fn observe_package_projection_build() {
    OBSERVED_PACKAGE_PROJECTION_BUILDS.with(|builds| builds.set(builds.get() + 1));
}

#[cfg(test)]
pub(in crate::plugin::runtime_plugin) fn begin_package_projection_build_observation() {
    OBSERVED_PACKAGE_PROJECTION_BUILDS.with(|builds| builds.set(0));
}

#[cfg(test)]
pub(in crate::plugin::runtime_plugin) fn observed_package_projection_builds() -> usize {
    OBSERVED_PACKAGE_PROJECTION_BUILDS.with(std::cell::Cell::get)
}

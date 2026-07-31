#[derive(Default)]
struct FeatureProjectionBuildObservation {
    standalone_builds: usize,
    embedded_views: usize,
}

std::thread_local! {
    static OBSERVATION: std::cell::RefCell<FeatureProjectionBuildObservation> = const {
        std::cell::RefCell::new(FeatureProjectionBuildObservation {
            standalone_builds: 0,
            embedded_views: 0,
        })
    };
}

pub(in crate::plugin::runtime_plugin) fn begin_feature_projection_build_observation() {
    OBSERVATION.with(|observation| *observation.borrow_mut() = Default::default());
}

pub(super) fn observe_standalone_feature_projection_build() {
    OBSERVATION.with(|observation| observation.borrow_mut().standalone_builds += 1);
}

pub(super) fn observe_embedded_feature_projection_view() {
    OBSERVATION.with(|observation| observation.borrow_mut().embedded_views += 1);
}

pub(in crate::plugin::runtime_plugin) fn observed_standalone_feature_projection_builds() -> usize {
    OBSERVATION.with(|observation| observation.borrow().standalone_builds)
}

pub(in crate::plugin::runtime_plugin) fn observed_embedded_feature_projection_views() -> usize {
    OBSERVATION.with(|observation| observation.borrow().embedded_views)
}

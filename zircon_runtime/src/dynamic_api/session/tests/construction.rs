use super::{
    activate_registered_modules, apply_profile_time_policy,
    rebase_frame_clock_after_session_activation, store_profile_submission_config, CoreRuntime,
    RenderProfileBundle, RenderSubmissionConfig, RuntimeDynamicSessionProfile,
    RENDER_PROFILE_CONFIG_KEY,
};
use crate::core::FrameClockFirstTickPolicy;

#[test]
fn pipelined_runtime_profile_stores_the_render_submission_config_before_activation() {
    let runtime = CoreRuntime::new();
    let core = runtime.handle();

    store_profile_submission_config(&core, RuntimeDynamicSessionProfile::RuntimePipelined)
        .expect("pipelined runtime profile should store the render submission config");

    let profile = core
        .load_config::<RenderProfileBundle>(RENDER_PROFILE_CONFIG_KEY)
        .expect("pipelined runtime profile should be readable before module activation");
    assert_eq!(
        profile.submission_config(),
        RenderSubmissionConfig::pipelined()
    );
}

#[test]
fn construction_commits_the_selected_product_time_policy_before_module_activation() {
    let runtime = CoreRuntime::new();
    let policy = RuntimeDynamicSessionProfile::Headless.product_time_policy();

    apply_profile_time_policy(&runtime, policy)
        .expect("built-in headless policy should apply to a new runtime");

    assert_eq!(runtime.time_policy(), policy.time_policy());
    assert_eq!(runtime.time_policy_generation(), 1);
}

#[test]
fn successful_session_activation_rebases_the_frame_clock() {
    let runtime = CoreRuntime::new();
    activate_registered_modules(&runtime).expect("empty module activation should succeed");

    let receipt = rebase_frame_clock_after_session_activation(&runtime);

    assert_eq!(receipt.generation(), 1);
    assert_eq!(
        receipt.first_tick_policy(),
        FrameClockFirstTickPolicy::MeasureFromRebase
    );
}

#[test]
fn standard_runtime_profile_does_not_override_the_default_submission_config() {
    let runtime = CoreRuntime::new();
    let core = runtime.handle();

    store_profile_submission_config(&core, RuntimeDynamicSessionProfile::Runtime)
        .expect("standard runtime profile should not require render submission configuration");

    assert_eq!(core.load_config_value(RENDER_PROFILE_CONFIG_KEY), None);
}

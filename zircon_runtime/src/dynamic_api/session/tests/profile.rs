use std::time::Duration;

use super::RuntimeDynamicSessionProfile;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::time::ProductTimeProfile;

#[test]
fn pipelined_runtime_profile_selects_client_render_bridge_and_pipeline() {
    let profile = RuntimeDynamicSessionProfile::from_bytes(b"runtime-pipelined")
        .expect("pipelined runtime profile should parse");

    assert_eq!(profile, RuntimeDynamicSessionProfile::RuntimePipelined);
    assert!(profile.uses_render_bridge());
    assert!(profile.pipelined_render());
    assert_eq!(profile.target_mode(), RuntimeTargetMode::ClientRuntime);
}

#[test]
fn runtime_session_profiles_select_versioned_product_time_policies() {
    for (profile, expected_product_profile, expected_budget) in [
        (
            RuntimeDynamicSessionProfile::Runtime,
            ProductTimeProfile::Client,
            8,
        ),
        (
            RuntimeDynamicSessionProfile::RuntimePipelined,
            ProductTimeProfile::Client,
            8,
        ),
        (
            RuntimeDynamicSessionProfile::Editor,
            ProductTimeProfile::Editor,
            4,
        ),
        (
            RuntimeDynamicSessionProfile::Dev,
            ProductTimeProfile::Client,
            8,
        ),
        (
            RuntimeDynamicSessionProfile::Minimal,
            ProductTimeProfile::Test,
            1,
        ),
        (
            RuntimeDynamicSessionProfile::Headless,
            ProductTimeProfile::Headless,
            16,
        ),
    ] {
        let policy = profile.product_time_policy();
        assert_eq!(policy.profile(), expected_product_profile);
        assert_eq!(policy.max_fixed_steps_per_frame(), expected_budget);
        assert_eq!(
            policy.time_policy().fixed_timestep(),
            Duration::from_micros(15_625)
        );
        policy
            .validate()
            .expect("session profile must select a valid time policy");
    }
}

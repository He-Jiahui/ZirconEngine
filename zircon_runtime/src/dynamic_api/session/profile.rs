use crate::core::framework::{
    platform::RuntimeTargetMode,
    time::{ProductTimePolicy, ProductTimeProfile},
};
use crate::core::runtime::ProductTimePolicies;
use crate::diagnostic_log::{DiagnosticStoreLogSchedule, DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT};

const RUNTIME_SESSION_PROFILE_RUNTIME: &[u8] = b"runtime";
const RUNTIME_SESSION_PROFILE_RUNTIME_PIPELINED: &[u8] = b"runtime-pipelined";
const RUNTIME_SESSION_PROFILE_EDITOR: &[u8] = b"editor";
const RUNTIME_SESSION_PROFILE_DEV: &[u8] = b"dev";
const RUNTIME_SESSION_PROFILE_MINIMAL: &[u8] = b"minimal";
const RUNTIME_SESSION_PROFILE_HEADLESS: &[u8] = b"headless";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeDynamicSessionProfile {
    Runtime,
    RuntimePipelined,
    Editor,
    Dev,
    Minimal,
    Headless,
}

impl RuntimeDynamicSessionProfile {
    pub(super) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [] | RUNTIME_SESSION_PROFILE_RUNTIME => Some(Self::Runtime),
            RUNTIME_SESSION_PROFILE_RUNTIME_PIPELINED => Some(Self::RuntimePipelined),
            RUNTIME_SESSION_PROFILE_EDITOR => Some(Self::Editor),
            RUNTIME_SESSION_PROFILE_DEV => Some(Self::Dev),
            RUNTIME_SESSION_PROFILE_MINIMAL => Some(Self::Minimal),
            RUNTIME_SESSION_PROFILE_HEADLESS => Some(Self::Headless),
            _ => None,
        }
    }

    pub(super) fn product_time_policy(self) -> ProductTimePolicy {
        let profile = match self {
            Self::Runtime | Self::RuntimePipelined | Self::Dev => ProductTimeProfile::Client,
            Self::Editor => ProductTimeProfile::Editor,
            Self::Minimal => ProductTimeProfile::Test,
            Self::Headless => ProductTimeProfile::Headless,
        };
        ProductTimePolicies::for_profile(profile)
    }

    pub(super) fn diagnostic_log_schedule(self) -> DiagnosticStoreLogSchedule {
        match self {
            Self::Dev => DiagnosticStoreLogSchedule::repeating(DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT),
            Self::Runtime
            | Self::RuntimePipelined
            | Self::Editor
            | Self::Minimal
            | Self::Headless => DiagnosticStoreLogSchedule::disabled(),
        }
    }

    pub(super) fn uses_render_bridge(self) -> bool {
        matches!(
            self,
            Self::Runtime | Self::RuntimePipelined | Self::Editor | Self::Dev
        )
    }

    pub(super) const fn pipelined_render(self) -> bool {
        matches!(self, Self::RuntimePipelined)
    }

    pub(super) fn target_mode(self) -> RuntimeTargetMode {
        match self {
            Self::Editor => RuntimeTargetMode::EditorHost,
            Self::Runtime | Self::RuntimePipelined | Self::Dev | Self::Minimal | Self::Headless => {
                RuntimeTargetMode::ClientRuntime
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}

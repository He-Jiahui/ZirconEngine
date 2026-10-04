use std::sync::Arc;

use crate::core::framework::platform::{
    ApplicationLifecycleState, PlatformHostBackend, PlatformHostBackendKind,
    PlatformHostDescriptor, PlatformHostEvidence, PlatformHostLifecycleState,
    PlatformHostObservedCapabilities, PlatformHostThreadAffinity,
};
use crate::platform::test_support::{platform_driver, platform_manager};
use crate::platform::{
    PlatformConfig, PlatformFeatureSelection, PlatformRuntimeCapabilityStatus, PlatformTarget,
    WindowBackend,
};

use super::super::PlatformManager;

#[test]
fn manager_exposes_driver_owned_immutable_display_topology() {
    let manager = platform_manager();

    let topology = manager.display_topology_snapshot();

    assert_eq!(topology.generation().get(), 1);
    assert!(topology.is_empty());
}

#[test]
fn manager_exposes_host_facts_without_exposing_backend_objects() {
    let manager = platform_manager();

    let host = manager.platform_host_snapshot();

    assert_eq!(host.lifecycle(), PlatformHostLifecycleState::Uninstalled);
    assert_eq!(host.instance(), None);
    assert_eq!(host.descriptor(), None);
}

#[test]
fn manager_exposes_application_lifecycle_as_an_immutable_snapshot() {
    let manager = platform_manager();

    let lifecycle = manager.application_lifecycle_snapshot();

    assert_eq!(lifecycle.state(), ApplicationLifecycleState::Cold);
    assert!(!lifecycle.allows_runtime_updates());
}

#[derive(Debug)]
struct ReadyPlatformHostBackend;

impl PlatformHostBackend for ReadyPlatformHostBackend {
    fn descriptor(&self) -> PlatformHostDescriptor {
        PlatformHostDescriptor::new(
            PlatformHostBackendKind::Winit,
            PlatformHostThreadAffinity::MainThreadOnly,
        )
    }

    fn request_quiesce(
        &self,
        _request: crate::core::framework::platform::PlatformHostQuiesceRequest,
    ) -> Result<(), crate::core::framework::platform::PlatformHostBackendRequestError> {
        Ok(())
    }
}

fn enabled_desktop_config() -> PlatformConfig {
    PlatformConfig {
        enabled: true,
        target: PlatformTarget::Windows,
        target_mode: crate::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        features: PlatformFeatureSelection::bevy_default_platform(),
    }
}

#[test]
fn runtime_capabilities_fail_closed_until_the_platform_host_is_observed_ready() {
    let driver = platform_driver();
    let manager = PlatformManager::new(driver.shared());
    let config = enabled_desktop_config();

    assert!(matches!(
        manager.runtime_capability_report(&config).window_backend(),
        PlatformRuntimeCapabilityStatus::HostUnavailable {
            lifecycle: PlatformHostLifecycleState::Uninstalled,
            provider: None,
            ..
        }
    ));

    let starting = driver
        .install_platform_host(Arc::new(ReadyPlatformHostBackend))
        .expect("platform host installs");
    let instance = starting.instance().expect("installed host has an instance");
    assert!(matches!(
        manager.runtime_capability_report(&config).window_backend(),
        PlatformRuntimeCapabilityStatus::HostUnavailable {
            lifecycle: PlatformHostLifecycleState::Starting,
            provider: Some(provider),
            ..
        } if provider == instance
    ));

    driver
        .publish_platform_host_ready(
            instance,
            PlatformHostEvidence::new(PlatformHostObservedCapabilities::new(true, true, true)),
        )
        .expect("host publishes observed capability evidence");
    let report = manager.runtime_capability_report(&config);
    assert!(matches!(
        report.window_backend(),
        PlatformRuntimeCapabilityStatus::Ready {
            value: WindowBackend::Winit,
            provider,
            generation: _,
        } if provider == instance
    ));
    assert!(report.monitor_inventory().is_ready());
    assert!(report.window_events().is_ready());
    assert!(report.window_lifecycle().is_ready());
    assert!(report.window_metrics().is_ready());
}

#[test]
fn disabled_platform_config_never_projects_a_runtime_ready_capability() {
    let driver = platform_driver();
    let manager = PlatformManager::new(driver.shared());
    let mut config = enabled_desktop_config();
    config.enabled = false;

    assert_eq!(
        manager.runtime_capability_report(&config).window_backend(),
        PlatformRuntimeCapabilityStatus::Disabled
    );
}

#[test]
fn runtime_capabilities_require_their_specific_host_observation() {
    let driver = platform_driver();
    let manager = PlatformManager::new(driver.shared());
    let starting = driver
        .install_platform_host(Arc::new(ReadyPlatformHostBackend))
        .expect("platform host installs");
    let instance = starting.instance().expect("installed host has an instance");
    driver
        .publish_platform_host_ready(
            instance,
            PlatformHostEvidence::new(PlatformHostObservedCapabilities::new(true, true, false)),
        )
        .expect("host publishes partial observation evidence");

    let report = manager.runtime_capability_report(&enabled_desktop_config());
    assert!(report.window_backend().is_ready());
    assert!(report.window_events().is_ready());
    assert!(matches!(
        report.monitor_inventory(),
        PlatformRuntimeCapabilityStatus::NotObserved { .. }
    ));
}

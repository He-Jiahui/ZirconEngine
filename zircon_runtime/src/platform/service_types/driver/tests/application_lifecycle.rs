use crate::core::framework::platform::{
    ApplicationActivationState, ApplicationLifecycleState, ApplicationSurfaceAvailability,
};
use crate::platform::test_support::platform_driver;
use crate::platform::{ApplicationLifecycleServiceError, PlatformApplicationSuspendError};

#[test]
fn application_lifecycle_keeps_activation_and_surface_availability_orthogonal() {
    let driver = platform_driver();

    let initial = driver.application_lifecycle_snapshot();
    assert_eq!(initial.state(), ApplicationLifecycleState::Cold);
    assert_eq!(initial.activation(), ApplicationActivationState::Unknown);
    assert_eq!(
        initial.surface_availability(),
        ApplicationSurfaceAvailability::Unavailable
    );

    driver
        .publish_application_activation(ApplicationActivationState::Inactive)
        .expect("application activation does not start the application");
    let resume = driver
        .request_application_resume()
        .expect("cold application requests resume");
    let duplicate_resume = driver
        .request_application_resume()
        .expect("resume is single-flight");
    assert_eq!(resume.id(), duplicate_resume.id());
    assert_eq!(
        driver.application_lifecycle_snapshot().state(),
        ApplicationLifecycleState::WillResume
    );

    let running = driver
        .publish_application_running(resume)
        .expect("matching resume receipt enters running");
    assert_eq!(running.state(), ApplicationLifecycleState::Running);
    assert!(running.allows_runtime_updates());
    assert_eq!(running.activation(), ApplicationActivationState::Inactive);
    assert_eq!(
        running.surface_availability(),
        ApplicationSurfaceAvailability::Unavailable
    );

    driver
        .publish_application_surface_availability(ApplicationSurfaceAvailability::Available)
        .expect("surface availability is an independent observed fact");
    let suspend = driver
        .begin_application_suspend_after_quiesce()
        .expect("running application begins its surface-safe suspend transaction");
    assert!(suspend.retiring_leases().is_empty());
    assert_eq!(
        driver.begin_application_suspend_after_quiesce(),
        Err(PlatformApplicationSuspendError::Lifecycle(
            ApplicationLifecycleServiceError::InvalidState {
                operation: "begin application suspend",
                state: ApplicationLifecycleState::WillSuspend,
            }
        ))
    );

    let suspended = driver
        .publish_application_suspended(suspend.operation())
        .expect("surface release receipt enters suspended");
    assert_eq!(suspended.state(), ApplicationLifecycleState::Suspended);
    assert!(!suspended.allows_runtime_updates());
    assert_eq!(suspended.activation(), ApplicationActivationState::Inactive);
    assert_eq!(
        suspended.surface_availability(),
        ApplicationSurfaceAvailability::Unavailable
    );
}

#[test]
fn application_exit_is_terminal_and_rejects_later_activation_changes() {
    let driver = platform_driver();

    let exiting = driver
        .begin_application_exit()
        .expect("cold application may begin exit");

    assert_eq!(exiting.state(), ApplicationLifecycleState::Exiting);
    assert_eq!(
        exiting.surface_availability(),
        ApplicationSurfaceAvailability::Unavailable
    );
    assert!(driver
        .publish_application_activation(ApplicationActivationState::Active)
        .is_err());
}

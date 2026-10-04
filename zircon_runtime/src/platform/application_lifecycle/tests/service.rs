use super::*;

#[test]
fn suspended_application_rejects_late_surface_availability() {
    let service = ApplicationLifecycleService::default();
    let resume = service
        .request_resume()
        .expect("a cold application must admit its first resume operation");
    service
        .publish_surface_availability(ApplicationSurfaceAvailability::Available)
        .expect("the resume path may publish an available surface");
    service
        .publish_running(resume)
        .expect("the matching resume operation must enter running");

    let suspend = service
        .request_suspend()
        .expect("a running application must admit suspension");
    let suspended = service
        .publish_suspended(suspend)
        .expect("the matching suspend operation must enter suspended");

    assert_eq!(
        service.publish_surface_availability(ApplicationSurfaceAvailability::Available),
        Err(ApplicationLifecycleServiceError::InvalidState {
            operation: "publish surface availability",
            state: ApplicationLifecycleState::Suspended,
        })
    );
    assert_eq!(service.snapshot(), suspended);
}

#[test]
fn suspending_application_rejects_late_surface_availability_before_terminal_receipt() {
    let service = ApplicationLifecycleService::default();
    let resume = service
        .request_resume()
        .expect("a cold application must admit its first resume operation");
    service
        .publish_running(resume)
        .expect("the matching resume operation must enter running");
    service
        .request_suspend()
        .expect("a running application must admit suspension");
    let suspending = service.snapshot();

    assert_eq!(
        service.publish_surface_availability(ApplicationSurfaceAvailability::Available),
        Err(ApplicationLifecycleServiceError::InvalidState {
            operation: "publish surface availability",
            state: ApplicationLifecycleState::WillSuspend,
        })
    );
    assert_eq!(service.snapshot(), suspending);
}

#[test]
fn suspended_application_retains_unavailable_surface_fact() {
    let service = ApplicationLifecycleService::default();
    let resume = service
        .request_resume()
        .expect("a cold application must admit its first resume operation");
    service
        .publish_running(resume)
        .expect("the matching resume operation must enter running");
    let suspend = service
        .request_suspend()
        .expect("a running application must admit suspension");
    service
        .publish_suspended(suspend)
        .expect("the matching suspend operation must enter suspended");

    let snapshot = service
        .publish_surface_availability(ApplicationSurfaceAvailability::Unavailable)
        .expect("unavailable surface publication remains idempotent while suspended");
    assert_eq!(snapshot.state(), ApplicationLifecycleState::Suspended);
    assert_eq!(
        snapshot.surface_availability(),
        ApplicationSurfaceAvailability::Unavailable
    );
}

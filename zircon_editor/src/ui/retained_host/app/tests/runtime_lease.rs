use zircon_runtime::core::manager::RENDER_FRAMEWORK_NAME;
use zircon_runtime::core::{CoreError, CoreRuntime};

use super::RetainedHostRuntimeLease;

#[test]
fn retained_host_lease_keeps_viewport_access_live_after_external_core_is_released() {
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let lease = RetainedHostRuntimeLease::new(core.clone());
    let access = lease.viewport_render_framework_access();
    drop(core);
    drop(runtime);

    assert!(matches!(
        access.render_framework_handle(),
        Err(CoreError::MissingService(name)) if name == RENDER_FRAMEWORK_NAME
    ));
}

use std::sync::Arc;

use zircon_core::{CoreError, ServiceFactory, ServiceObject};

pub fn factory(
    builder: impl Fn(&zircon_core::CoreHandle) -> Result<ServiceObject, CoreError>
        + Send
        + Sync
        + 'static,
) -> ServiceFactory {
    Arc::new(builder)
}

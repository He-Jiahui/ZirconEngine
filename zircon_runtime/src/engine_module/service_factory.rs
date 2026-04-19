use std::sync::Arc;

use crate::core::{CoreError, ServiceFactory, ServiceObject};

pub fn factory(
    builder: impl Fn(&crate::core::CoreHandle) -> Result<ServiceObject, CoreError>
        + Send
        + Sync
        + 'static,
) -> ServiceFactory {
    Arc::new(builder)
}

use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::framework::sound::{
    SoundDynamicEventDelivery, SoundDynamicEventHandlerDescriptor,
};

type SoundDynamicEventExecutorCallback =
    dyn Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub(crate) struct SoundDynamicEventExecutor {
    callback: Arc<SoundDynamicEventExecutorCallback>,
}

impl SoundDynamicEventExecutor {
    pub(crate) fn new<F>(callback: F) -> Self
    where
        F: Fn(&SoundDynamicEventDelivery) -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(callback),
        }
    }

    pub(crate) fn execute(&self, delivery: &SoundDynamicEventDelivery) -> Result<(), String> {
        (self.callback)(delivery)
    }
}

impl fmt::Debug for SoundDynamicEventExecutor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SoundDynamicEventExecutor")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SoundDynamicEventExecutorKey {
    pub(crate) plugin_id: String,
    pub(crate) handler_id: String,
}

impl SoundDynamicEventExecutorKey {
    pub(crate) fn new(plugin_id: impl Into<String>, handler_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            handler_id: handler_id.into(),
        }
    }

    pub(crate) fn from_handler(handler: &SoundDynamicEventHandlerDescriptor) -> Self {
        Self::new(handler.plugin_id.clone(), handler.handler_id.clone())
    }
}

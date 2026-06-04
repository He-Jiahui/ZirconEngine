use zircon_runtime::core::framework::sound::{
    SoundDynamicEventCatalog, SoundDynamicEventDelivery, SoundDynamicEventDescriptor,
    SoundDynamicEventExecutionReport, SoundDynamicEventHandlerDescriptor,
    SoundDynamicEventInvocation, SoundDynamicEventManager, SoundError,
};

use super::super::DefaultSoundManager;

impl SoundDynamicEventManager for DefaultSoundManager {
    fn dynamic_event_catalog(&self) -> Result<SoundDynamicEventCatalog, SoundError> {
        self.dynamic_event_catalog_impl()
    }

    fn register_dynamic_event(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError> {
        self.register_dynamic_event_impl(descriptor)
    }

    fn unregister_dynamic_event(&self, event_id: &str) -> Result<(), SoundError> {
        self.unregister_dynamic_event_impl(event_id)
    }

    fn dynamic_event_handlers(
        &self,
    ) -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError> {
        self.dynamic_event_handlers_impl()
    }

    fn register_dynamic_event_handler(
        &self,
        handler: SoundDynamicEventHandlerDescriptor,
    ) -> Result<(), SoundError> {
        self.register_dynamic_event_handler_impl(handler)
    }

    fn unregister_dynamic_event_handler(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError> {
        self.unregister_dynamic_event_handler_impl(plugin_id, handler_id)
    }

    fn submit_dynamic_event(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError> {
        self.submit_dynamic_event_impl(invocation)
    }

    fn drain_dynamic_events(&self) -> Result<Vec<SoundDynamicEventInvocation>, SoundError> {
        self.drain_dynamic_events_impl()
    }

    fn dispatch_dynamic_events(&self) -> Result<Vec<SoundDynamicEventDelivery>, SoundError> {
        self.dispatch_dynamic_events_impl()
    }

    fn execute_dynamic_events(&self) -> Result<SoundDynamicEventExecutionReport, SoundError> {
        self.execute_dynamic_events_impl()
    }
}

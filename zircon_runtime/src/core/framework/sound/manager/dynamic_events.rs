use super::super::{
    SoundDynamicEventCatalog, SoundDynamicEventDelivery, SoundDynamicEventDescriptor,
    SoundDynamicEventExecutionReport, SoundDynamicEventHandlerDescriptor,
    SoundDynamicEventInvocation, SoundError,
};

pub trait SoundDynamicEventManager {
    fn dynamic_event_catalog(&self) -> Result<SoundDynamicEventCatalog, SoundError>;
    fn register_dynamic_event(
        &self,
        descriptor: SoundDynamicEventDescriptor,
    ) -> Result<(), SoundError>;
    fn unregister_dynamic_event(&self, event_id: &str) -> Result<(), SoundError>;
    fn dynamic_event_handlers(&self)
        -> Result<Vec<SoundDynamicEventHandlerDescriptor>, SoundError>;
    fn register_dynamic_event_handler(
        &self,
        handler: SoundDynamicEventHandlerDescriptor,
    ) -> Result<(), SoundError>;
    fn unregister_dynamic_event_handler(
        &self,
        plugin_id: &str,
        handler_id: &str,
    ) -> Result<(), SoundError>;
    fn submit_dynamic_event(
        &self,
        invocation: SoundDynamicEventInvocation,
    ) -> Result<(), SoundError>;
    fn drain_dynamic_events(&self) -> Result<Vec<SoundDynamicEventInvocation>, SoundError>;
    fn dispatch_dynamic_events(&self) -> Result<Vec<SoundDynamicEventDelivery>, SoundError>;
    fn execute_dynamic_events(&self) -> Result<SoundDynamicEventExecutionReport, SoundError>;
}

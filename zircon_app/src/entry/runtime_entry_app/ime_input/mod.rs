mod admission;
mod composition;
mod composition_v2;
mod deletion;
mod lifecycle;
mod native;
mod routing;

pub(super) use admission::RuntimeImeInputAdmission;
pub(super) use composition_v2::{
    ImeCompositionClauseAvailability, ImeCompositionProducerError, NativeImeCompositionInput,
    RuntimeImeCompositionContextAllocator, RuntimeImeCompositionProducer,
};
pub(super) use lifecycle::{focus_changed_events, focus_changed_events_with_native_source};
pub(super) use native::native_ime_provider_available;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

use std::collections::BTreeMap;

use thiserror::Error;
use zircon_runtime_interface::ui::{
    binding::{
        UiBindingConversionDescriptor, UiBindingConversionHandle, UiBindingConversionId,
        UiBindingConversionProviderError, UiBindingConversionProviderGeneration,
        UiBindingConversionSlot,
    },
    component::{UiValue, UiValueKind},
};

pub type UiBindingConversionFunction =
    fn(&UiValue) -> Result<UiValue, UiBindingConversionProviderError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingConversionRegistryError {
    #[error(
        "binding conversion {id} generation {generation:?} is already registered with a different signature"
    )]
    ProviderGenerationCollision {
        id: UiBindingConversionId,
        generation: UiBindingConversionProviderGeneration,
    },
    #[error(
        "binding conversion {id} provider generation regressed from {current:?} to {requested:?}"
    )]
    ProviderGenerationRegression {
        id: UiBindingConversionId,
        current: UiBindingConversionProviderGeneration,
        requested: UiBindingConversionProviderGeneration,
    },
    #[error("binding conversion registry exceeded its u32 slot capacity")]
    SlotCapacityExceeded,
    #[error("binding conversion handle {handle:?} is stale or unloaded")]
    StaleHandle { handle: UiBindingConversionHandle },
    #[error(
        "binding conversion handle {handle:?} expected {expected_source:?}->{expected_destination:?} but registered {actual_source:?}->{actual_destination:?}"
    )]
    SignatureMismatch {
        handle: UiBindingConversionHandle,
        expected_source: UiValueKind,
        expected_destination: UiValueKind,
        actual_source: UiValueKind,
        actual_destination: UiValueKind,
    },
    #[error(
        "binding conversion handle {handle:?} expected input {expected:?} but received {actual:?}"
    )]
    InputTypeMismatch {
        handle: UiBindingConversionHandle,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error(
        "binding conversion handle {handle:?} promised output {expected:?} but returned {actual:?}"
    )]
    OutputTypeMismatch {
        handle: UiBindingConversionHandle,
        expected: UiValueKind,
        actual: UiValueKind,
    },
    #[error("binding conversion handle {handle:?} provider failed: {error}")]
    ProviderFailed {
        handle: UiBindingConversionHandle,
        error: UiBindingConversionProviderError,
    },
}

#[derive(Clone, Debug)]
struct RegisteredBindingConversion {
    descriptor: UiBindingConversionDescriptor,
    function: UiBindingConversionFunction,
}

#[derive(Clone, Debug, Default)]
pub struct UiBindingConversionRegistry {
    slots: Vec<Option<RegisteredBindingConversion>>,
    slots_by_id: BTreeMap<UiBindingConversionId, UiBindingConversionSlot>,
    revision: u64,
}

impl UiBindingConversionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn registered_count(&self) -> usize {
        self.slots_by_id.len()
    }

    pub fn register(
        &mut self,
        descriptor: UiBindingConversionDescriptor,
        function: UiBindingConversionFunction,
    ) -> Result<UiBindingConversionHandle, UiBindingConversionRegistryError> {
        if let Some(slot) = self.slots_by_id.get(&descriptor.id).copied() {
            let existing = self.slots[slot.get() as usize]
                .as_ref()
                .expect("active conversion IDs must reference an occupied slot");
            if existing.descriptor == descriptor {
                return Ok(handle_for(slot, &existing.descriptor));
            }
            if descriptor.provider_generation < existing.descriptor.provider_generation {
                return Err(
                    UiBindingConversionRegistryError::ProviderGenerationRegression {
                        id: descriptor.id,
                        current: existing.descriptor.provider_generation,
                        requested: descriptor.provider_generation,
                    },
                );
            }
            if descriptor.provider_generation == existing.descriptor.provider_generation {
                return Err(
                    UiBindingConversionRegistryError::ProviderGenerationCollision {
                        id: descriptor.id,
                        generation: descriptor.provider_generation,
                    },
                );
            }

            let handle = handle_for(slot, &descriptor);
            self.slots[slot.get() as usize] = Some(RegisteredBindingConversion {
                descriptor,
                function,
            });
            self.revision = self.revision.saturating_add(1);
            return Ok(handle);
        }

        let slot_index = u32::try_from(self.slots.len())
            .map_err(|_| UiBindingConversionRegistryError::SlotCapacityExceeded)?;
        let slot = UiBindingConversionSlot::new(slot_index);
        let handle = handle_for(slot, &descriptor);
        self.slots_by_id.insert(descriptor.id.clone(), slot);
        self.slots.push(Some(RegisteredBindingConversion {
            descriptor,
            function,
        }));
        self.revision = self.revision.saturating_add(1);
        Ok(handle)
    }

    pub fn resolve(
        &self,
        handle: UiBindingConversionHandle,
    ) -> Result<&UiBindingConversionDescriptor, UiBindingConversionRegistryError> {
        let registered = self
            .slots
            .get(handle.slot().get() as usize)
            .and_then(Option::as_ref)
            .filter(|registered| {
                registered.descriptor.provider_generation == handle.provider_generation()
            })
            .ok_or(UiBindingConversionRegistryError::StaleHandle { handle })?;
        Ok(&registered.descriptor)
    }

    pub fn resolve_typed(
        &self,
        handle: UiBindingConversionHandle,
        source: UiValueKind,
        destination: UiValueKind,
    ) -> Result<&UiBindingConversionDescriptor, UiBindingConversionRegistryError> {
        let descriptor = self.resolve(handle)?;
        if descriptor.signature.source != source || descriptor.signature.destination != destination
        {
            return Err(UiBindingConversionRegistryError::SignatureMismatch {
                handle,
                expected_source: source,
                expected_destination: destination,
                actual_source: descriptor.signature.source,
                actual_destination: descriptor.signature.destination,
            });
        }
        Ok(descriptor)
    }

    pub fn execute(
        &self,
        handle: UiBindingConversionHandle,
        input: &UiValue,
    ) -> Result<UiValue, UiBindingConversionRegistryError> {
        let registered = self
            .slots
            .get(handle.slot().get() as usize)
            .and_then(Option::as_ref)
            .filter(|registered| {
                registered.descriptor.provider_generation == handle.provider_generation()
            })
            .ok_or(UiBindingConversionRegistryError::StaleHandle { handle })?;
        let signature = registered.descriptor.signature;
        let actual_input = input.kind();
        if !kind_matches(signature.source, actual_input) {
            return Err(UiBindingConversionRegistryError::InputTypeMismatch {
                handle,
                expected: signature.source,
                actual: actual_input,
            });
        }
        let output = (registered.function)(input)
            .map_err(|error| UiBindingConversionRegistryError::ProviderFailed { handle, error })?;
        let actual_output = output.kind();
        if !kind_matches(signature.destination, actual_output) {
            return Err(UiBindingConversionRegistryError::OutputTypeMismatch {
                handle,
                expected: signature.destination,
                actual: actual_output,
            });
        }
        Ok(output)
    }

    pub fn unregister(
        &mut self,
        handle: UiBindingConversionHandle,
    ) -> Result<UiBindingConversionDescriptor, UiBindingConversionRegistryError> {
        let descriptor = self.resolve(handle)?.clone();
        self.slots[handle.slot().get() as usize] = None;
        self.slots_by_id.remove(&descriptor.id);
        self.revision = self.revision.saturating_add(1);
        Ok(descriptor)
    }
}

fn kind_matches(expected: UiValueKind, actual: UiValueKind) -> bool {
    expected == UiValueKind::Any || expected == actual
}

fn handle_for(
    slot: UiBindingConversionSlot,
    descriptor: &UiBindingConversionDescriptor,
) -> UiBindingConversionHandle {
    UiBindingConversionHandle::new(slot, descriptor.provider_generation)
}

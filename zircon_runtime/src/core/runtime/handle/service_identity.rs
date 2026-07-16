use super::super::descriptors::RegistryName;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RegisteredServiceIdentity {
    index: u32,
    generation: u32,
    service: RegistryName,
}

impl RegisteredServiceIdentity {
    pub(crate) fn new(index: u32, generation: u32, service: RegistryName) -> Self {
        Self {
            index,
            generation,
            service,
        }
    }

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }

    pub(crate) fn service(&self) -> &RegistryName {
        &self.service
    }
}

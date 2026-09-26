use super::super::descriptors::RegistryName;

// 注册名、槽位索引和代次共同标识一次服务实例生命周期；两类句柄共用此失效依据。
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

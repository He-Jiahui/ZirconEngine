use crate::core::resource::ResourceId;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum GpuResourceHandle {
    Texture(ResourceId),
    Model(ResourceId),
}

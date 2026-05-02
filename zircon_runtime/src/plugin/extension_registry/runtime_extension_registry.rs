use crate::core::ManagerDescriptor;
use crate::core::ModuleDescriptor;
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::{ComponentTypeDescriptor, UiComponentDescriptor};

#[derive(Clone, Debug, Default)]
pub struct RuntimeExtensionRegistry {
    pub(super) managers: Vec<ManagerDescriptor>,
    pub(super) modules: Vec<ModuleDescriptor>,
    pub(super) render_features: Vec<RenderFeatureDescriptor>,
    pub(super) render_pass_executors: Vec<RenderPassExecutorRegistration>,
    pub(super) virtual_geometry_runtime_providers: Vec<VirtualGeometryRuntimeProviderRegistration>,
    pub(super) components: Vec<ComponentTypeDescriptor>,
    pub(super) ui_components: Vec<UiComponentDescriptor>,
}

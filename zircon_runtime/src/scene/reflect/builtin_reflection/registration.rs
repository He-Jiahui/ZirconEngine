use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::components::{
    ActiveSelf, AmbientLight, CameraComponent, DirectionalLight, LocalTransform, MeshRenderer,
    Mobility, Name, PointLight, RectLight, RenderLayerMask, RigidBodyComponent, SpotLight,
};
use crate::scene::{derived_component_registration, TypeRegistry};

use super::{active_in_hierarchy, hierarchy};

pub(in crate::scene::reflect) fn register(registry: &mut TypeRegistry) -> Result<(), ReflectError> {
    registry.register(derived_component_registration::<Name>()?)?;
    registry.register(hierarchy::registration()?)?;
    registry.register(derived_component_registration::<LocalTransform>()?)?;
    registry.register(derived_component_registration::<ActiveSelf>()?)?;
    registry.register(active_in_hierarchy::registration()?)?;
    registry.register(derived_component_registration::<RenderLayerMask>()?)?;
    registry.register(derived_component_registration::<Mobility>()?)?;
    registry.register(derived_component_registration::<CameraComponent>()?)?;
    registry.register(derived_component_registration::<MeshRenderer>()?)?;
    registry.register(derived_component_registration::<AmbientLight>()?)?;
    registry.register(derived_component_registration::<DirectionalLight>()?)?;
    registry.register(derived_component_registration::<PointLight>()?)?;
    registry.register(derived_component_registration::<RectLight>()?)?;
    registry.register(derived_component_registration::<SpotLight>()?)?;
    registry.register(derived_component_registration::<RigidBodyComponent>()?)?;
    Ok(())
}

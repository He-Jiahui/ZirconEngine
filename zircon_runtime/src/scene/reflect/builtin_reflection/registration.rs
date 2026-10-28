use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::components::{
    ActiveSelf, AmbientLight, AnimationPlayerComponent, CameraComponent, ColliderComponent,
    DirectionalLight, JointComponent, LocalTransform, MeshRenderer, Mobility, Name, PointLight,
    PostProcessSettingsComponent, PostProcessVolumeComponent, RectLight, RenderLayerMask,
    RigidBodyComponent, SpotLight, Sprite2dComponent,
};
use crate::scene::{derived_component_registration, TypeRegistry};

use super::{active_in_hierarchy, hierarchy, script_bindings};

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
    registry.register(derived_component_registration::<ColliderComponent>()?)?;
    registry.register(derived_component_registration::<JointComponent>()?)?;
    registry.register(derived_component_registration::<
        PostProcessSettingsComponent,
    >()?)?;
    registry.register(derived_component_registration::<PostProcessVolumeComponent>()?)?;
    registry.register(derived_component_registration::<AnimationPlayerComponent>()?)?;
    registry.register(derived_component_registration::<Sprite2dComponent>()?)?;
    registry.register(script_bindings::registration()?)?;
    Ok(())
}

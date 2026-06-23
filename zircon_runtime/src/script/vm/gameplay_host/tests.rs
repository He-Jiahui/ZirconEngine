use std::sync::{Arc, Mutex};

use super::{
    register_gameplay_host_module,
    script_bindings::{script_binding_property_matches, SCRIPT_BINDINGS_COMPONENT},
    GAMEPLAY_MODULE,
};
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::WorldHandle;
use crate::core::framework::script::ScriptHostValue;
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{ResourceHandle, ResourceId};
use crate::core::CoreRuntime;
use crate::scene::components::NodeKind;
use crate::scene::{LevelMetadata, LevelSystem, World};
use crate::script::{
    with_script_runtime_call_context, CapabilitySet, HostExportRegistry, HostRegistry,
    ScriptRuntimeCallContext,
};

mod combat_lifecycle;
mod component_state;
mod property_animation;
mod spawn_transform;

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    let delta = (actual - expected).abs();
    assert!(
        delta.max_element() <= 0.0001,
        "expected {expected:?}, received {actual:?}"
    );
}

fn assert_quat_close(actual: Quat, expected: Quat) {
    let delta = (actual.to_array(), expected.to_array());
    let max_component_delta = delta
        .0
        .iter()
        .zip(delta.1.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        max_component_delta <= 0.0001,
        "expected {expected:?}, received {actual:?}"
    );
}

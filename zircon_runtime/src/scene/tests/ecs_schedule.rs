use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::framework::render::{
    CameraRenderDescriptor, DisplayMode, ProjectionMode, RenderCameraClear, RenderCameraClearColor,
    RenderExposureMode, RenderExtractContext, RenderLayerSet, RenderMaterialAlphaMode, RenderPhase,
    RenderViewportRect, RenderVirtualGeometryDebugState, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportCameraSnapshot, ViewportRenderSettings,
};
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::math::{Transform, UVec2, Vec3};
use crate::core::runtime::modules::{TasksModule, TimeModule};
use crate::core::CoreRuntime;
use crate::engine_module::EngineModule;
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::{CameraComponent, MeshRenderer, Mobility};
use crate::scene::ecs::{
    CommandsParam, Component, EventPayloadProfile, EventPayloadStorage, EventStore,
    EventSubscription, EventSubscriptionStatus, Events, FunctionRuntimeSceneSystem,
    InternalSceneSystem, ResourceStore, SceneSystemDescriptor, SceneSystemMetadata,
    SceneSystemTickPolicy, Schedule, SystemOrderingConstraint, SystemRef, SystemSetRegistry,
    SystemStage, SystemState, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES,
    EVENT_INLINE_PAYLOAD_MAX_BYTES,
};
use crate::scene::{create_default_level, module_descriptor, NodeKind, World};

fn scene_test_runtime() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    // Core owns the services but starts with an empty module dependency graph.
    for descriptor in [
        TasksModule.descriptor(),
        TimeModule.descriptor(),
        crate::asset::module_descriptor(),
        module_descriptor(),
    ] {
        runtime.register_module(descriptor).unwrap();
    }
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime
}

#[path = "ecs_schedule/conflict_graph.rs"]
mod conflict_graph;
#[path = "ecs_schedule/fixed_update.rs"]
mod fixed_update;
#[path = "ecs_schedule/parallel_executor.rs"]
mod parallel_executor;
#[path = "ecs_schedule/render_extract.rs"]
mod render_extract;
#[path = "ecs_schedule/resources_events.rs"]
mod resources_events;
#[path = "ecs_schedule/schedule_plan.rs"]
mod schedule_plan;
#[path = "ecs_schedule/unreal_stage_contract.rs"]
mod unreal_stage_contract;
#[path = "ecs_schedule/world_driver.rs"]
mod world_driver;
#[path = "ecs_schedule/world_time_controller.rs"]
mod world_time_controller;

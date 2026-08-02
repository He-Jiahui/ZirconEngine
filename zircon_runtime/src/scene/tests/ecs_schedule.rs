use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::CoreRuntime;
use crate::core::framework::render::{
    CameraRenderDescriptor, DisplayMode, ProjectionMode, RenderCameraClear, RenderCameraClearColor,
    RenderExposureMode, RenderExtractContext, RenderLayerSet, RenderMaterialAlphaMode, RenderPhase,
    RenderViewportRect, RenderVirtualGeometryDebugState, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportCameraSnapshot, ViewportRenderSettings,
};
use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::math::{Transform, UVec2, Vec3};
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::{CameraComponent, MeshRenderer, Mobility};
use crate::scene::ecs::{
    CommandsParam, Component, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES,
    EVENT_INLINE_PAYLOAD_MAX_BYTES, EventPayloadProfile, EventPayloadStorage, EventStore,
    EventSubscription, EventSubscriptionStatus, Events, FunctionRuntimeSceneSystem,
    InternalSceneSystem, ResourceStore, SceneSystemDescriptor, SceneSystemMetadata, Schedule,
    SystemOrderingConstraint, SystemRef, SystemSetRegistry, SystemStage, SystemState,
};
use crate::scene::{NodeKind, World, create_default_level, module_descriptor};
use crate::scene::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};

mod conflict_graph;
mod fixed_update;
mod parallel_executor;
mod render_extract;
mod resources_events;
mod schedule_plan;
mod world_driver;

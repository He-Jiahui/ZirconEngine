use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::framework::render::{
    CameraRenderDescriptor, DisplayMode, ProjectionMode, RenderCameraClear, RenderCameraClearColor,
    RenderExposureMode, RenderExtractContext, RenderLayerSet, RenderMaterialAlphaMode, RenderPhase,
    RenderViewportRect, RenderVirtualGeometryDebugState, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportCameraSnapshot, ViewportRenderSettings,
};
use crate::core::math::{Transform, UVec2, Vec3};
use crate::core::CoreRuntime;
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::components::{CameraComponent, MeshRenderer, Mobility};
use crate::scene::ecs::{
    CommandsParam, Component, EventPayloadProfile, EventPayloadStorage, EventStore,
    EventSubscription, EventSubscriptionStatus, Events, FunctionRuntimeSceneSystem,
    InternalSceneSystem, ResourceStore, SceneSystemDescriptor, SceneSystemMetadata, Schedule,
    SystemOrderingConstraint, SystemRef, SystemSetRegistry, SystemStage, SystemState,
    EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES,
};
use crate::scene::{create_default_level, module_descriptor, NodeKind, World, SCENE_MODULE_NAME};
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

#[path = "builtin_scene_mode.rs"]
mod builtin_scene_mode;
#[path = "builtin_scene_mode_registry.rs"]
mod builtin_scene_mode_registry;
#[path = "editor_scene_mode.rs"]
mod editor_scene_mode;
#[path = "input_outcome.rs"]
mod input_outcome;
#[path = "isolated_scene_mode.rs"]
mod isolated_scene_mode;
#[path = "scene_mode_activation.rs"]
mod scene_mode_activation;
#[path = "scene_mode_activation_error.rs"]
mod scene_mode_activation_error;
#[path = "scene_mode_ctx.rs"]
mod scene_mode_ctx;
#[path = "scene_mode_factory.rs"]
mod scene_mode_factory;
#[path = "scene_mode_input_effect.rs"]
mod scene_mode_input_effect;
#[path = "scene_mode_registration.rs"]
mod scene_mode_registration;
#[path = "scene_mode_registry.rs"]
mod scene_mode_registry;
#[path = "scene_mode_registry_error.rs"]
mod scene_mode_registry_error;
#[path = "scene_mode_stack.rs"]
mod scene_mode_stack;
#[path = "scene_mode_stack_error.rs"]
mod scene_mode_stack_error;
#[path = "viewport_overlay_builder.rs"]
mod viewport_overlay_builder;

pub(crate) use builtin_scene_mode_registry::builtin_scene_mode_registry;
pub use editor_scene_mode::EditorSceneMode;
pub use input_outcome::InputOutcome;
pub use scene_mode_activation::SceneModeActivation;
pub(crate) use scene_mode_activation::{SELECT_SCENE_MODE_ID, TRANSFORM_SCENE_MODE_ID};
pub(crate) use scene_mode_activation_error::SceneModeActivationError;
pub use scene_mode_ctx::SceneModeCtx;
pub use scene_mode_factory::SceneModeFactory;
pub(crate) use scene_mode_input_effect::SceneModeInputEffect;
pub use scene_mode_registration::SceneModeRegistration;
pub use scene_mode_registry::SceneModeRegistry;
pub use scene_mode_registry_error::SceneModeRegistryError;
pub use scene_mode_stack::SceneModeStack;
pub use scene_mode_stack_error::SceneModeStackError;
pub use viewport_overlay_builder::ViewportOverlayBuilder;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

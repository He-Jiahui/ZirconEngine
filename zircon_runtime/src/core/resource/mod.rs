//! Resource foundation layer: locators, ids, typed handles, registry, and runtime state.

mod data;
mod diagnostic;
mod handle;
mod id;
mod identity;
mod io;
mod lease;
mod locator;
mod manager;
mod marker;
mod record;
mod registry;
mod runtime;
mod state;

pub use data::ResourceData;
pub use diagnostic::{ResourceDiagnostic, ResourceDiagnosticSeverity};
pub use handle::{ResourceHandle, UntypedResourceHandle};
pub use id::ResourceId;
pub use identity::{AssetReference, AssetUuid};
pub use io::{ResourceIo, ResourceIoError};
pub use lease::ResourceLease;
pub use locator::{ResourceLocator, ResourceLocatorError, ResourceScheme};
pub use manager::ResourceManager;
pub use marker::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, FontMarker, MaterialMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceKind, ResourceMarker, SceneMarker, ShaderMarker, SoundMarker, TextureMarker,
    UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};
pub use record::{ResourceEvent, ResourceEventKind, ResourceRecord};
pub use registry::ResourceRegistry;
pub use runtime::{Resource, ResourceRuntimeInfo, RuntimeResourceState};
pub use state::ResourceState;

#[cfg(test)]
mod tests;

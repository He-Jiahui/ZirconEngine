//! Shared resource identity and status DTOs for editor/runtime contracts.

#[path = "../../../zircon_runtime/src/core/resource/diagnostic.rs"]
mod diagnostic;
#[path = "../../../zircon_runtime/src/core/resource/handle/resource_handle.rs"]
mod resource_handle;
#[path = "../../../zircon_runtime/src/core/resource/id.rs"]
mod resource_id;
#[path = "../../../zircon_runtime/src/core/resource/identity/asset_reference.rs"]
mod asset_reference;
#[path = "../../../zircon_runtime/src/core/resource/identity/asset_uuid.rs"]
mod asset_uuid;
#[path = "../../../zircon_runtime/src/core/resource/identity/stable_uuid.rs"]
mod stable_uuid;
#[path = "../../../zircon_runtime/src/core/resource/locator.rs"]
mod locator;
#[path = "../../../zircon_runtime/src/core/resource/marker.rs"]
mod marker;
#[path = "../../../zircon_runtime/src/core/resource/record/resource_event.rs"]
mod resource_event;
#[path = "../../../zircon_runtime/src/core/resource/record/resource_event_kind.rs"]
mod resource_event_kind;
#[path = "../../../zircon_runtime/src/core/resource/record/resource_record.rs"]
mod resource_record;
#[path = "../../../zircon_runtime/src/core/resource/state.rs"]
mod state;
#[path = "../../../zircon_runtime/src/core/resource/handle/untyped_handle.rs"]
mod untyped_handle;

pub use asset_reference::AssetReference;
pub use asset_uuid::AssetUuid;
pub use diagnostic::{ResourceDiagnostic, ResourceDiagnosticSeverity};
pub use locator::{ResourceLocator, ResourceLocatorError, ResourceScheme};
pub use marker::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, FontMarker, MaterialMarker, ModelMarker, PhysicsMaterialMarker,
    ResourceKind, ResourceMarker, SceneMarker, ShaderMarker, SoundMarker, TextureMarker,
    UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};
pub use resource_event::ResourceEvent;
pub use resource_event_kind::ResourceEventKind;
pub use resource_handle::ResourceHandle;
pub use resource_id::ResourceId;
pub use resource_record::ResourceRecord;
pub use state::ResourceState;
pub use untyped_handle::UntypedResourceHandle;

pub(crate) use stable_uuid::stable_uuid_from_components;

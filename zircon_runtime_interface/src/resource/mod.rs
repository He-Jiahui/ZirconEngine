//! Shared resource identity and status DTOs for editor/runtime contracts.

mod asset_reference;
mod asset_uuid;
mod diagnostic;
mod locator;
mod marker;
mod resource_event;
mod resource_event_kind;
mod resource_handle;
mod resource_id;
mod resource_record;
mod stable_uuid;
mod state;
mod untyped_handle;

pub use asset_reference::AssetReference;
pub use asset_uuid::AssetUuid;
pub use diagnostic::{ResourceDiagnostic, ResourceDiagnosticSeverity};
pub use locator::{ResourceLocator, ResourceLocatorError, ResourceScheme};
pub use marker::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, DataMarker, FontMarker, MaterialGraphMarker, MaterialMarker,
    ModelMarker, NavMeshMarker, NavigationSettingsMarker, PhysicsMaterialMarker, PrefabMarker,
    ResourceKind, ResourceMarker, SceneMarker, ShaderMarker, SoundMarker, TerrainLayerStackMarker,
    TerrainMarker, TextureMarker, TileMapMarker, TileSetMarker, UiLayoutMarker, UiStyleMarker,
    UiWidgetMarker,
};
pub use resource_event::ResourceEvent;
pub use resource_event_kind::ResourceEventKind;
pub use resource_handle::ResourceHandle;
pub use resource_id::ResourceId;
pub use resource_record::ResourceRecord;
pub use state::ResourceState;
pub use untyped_handle::UntypedResourceHandle;

pub(crate) use stable_uuid::stable_uuid_from_components;

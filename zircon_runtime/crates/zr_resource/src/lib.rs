//! Resource foundation layer: locators, ids, typed handles, registry, and runtime state.

mod data;
mod error;
mod event_stream;
pub mod io;

#[doc(hidden)]
pub mod assembly;
mod lease;
mod management_generation;
mod manager;
mod mutation;
mod readiness_generation;
mod registry;
mod runtime;
mod snapshot;
#[cfg(test)]
mod test_profile;

pub use data::ResourceData;
pub use error::{ResourceRegistryError, ResourceResult};
pub use event_stream::{
    ResourceEventGap, ResourceEventReceiver, ResourceEventRecvError, ResourceEventRecvTimeoutError,
    ResourceEventStreamDiagnostics, ResourceEventTryRecvError,
};
pub use lease::ResourceLease;
pub use management_generation::{
    ResourceManagementGeneration, ResourceManagementGenerationDiagnostics,
    ResourceManagementGenerationIdentity, ResourceManagementKindSummary, ResourceManagementPage,
    ResourceManagementQuery, ResourceManagementRow, ResourceManagementRowIdentity,
    ResourceManagementScan, ResourceManagementSummary,
};
pub(crate) use management_generation::{
    ResourceManagementIdShard, ResourceManagementLocatorShard,
    resource_management_id_maps_from_ordered_pages, resource_management_pages_from_sorted_rows,
    resource_management_row_order,
};
pub use manager::{ResourceManager, ResourceProjectionSnapshot, ResourceRegistryReadGuard};
pub(crate) use mutation::ResourceMutationOperation;
pub use mutation::{ResourceMutationBatch, ResourceMutationReceipt};
pub use readiness_generation::{
    ResourceReadinessGeneration, ResourceReadinessGenerationDiagnostics,
    ResourceReadinessGenerationIdentity, ResourceReadinessRow, ResourceReadinessRowIdentity,
    ResourceReadinessState,
};
pub use registry::ResourceRegistry;
pub use runtime::{Resource, ResourceRuntimeInfo, RuntimeResourceState};
pub use snapshot::ResourceSnapshot;

// Interface-owned resource identities and stable locator protocol.
pub use zircon_runtime_interface::resource::{
    AssetReference, AssetUuid, ResourceId, ResourceLocator, ResourceLocatorError, ResourceScheme,
    STABLE_UUID_ALGORITHM_VERSION,
};

// Interface-owned resource records, handles, state, and event DTOs.
pub use zircon_runtime_interface::resource::{
    ResourceDiagnostic, ResourceDiagnosticSeverity, ResourceEvent, ResourceEventKind,
    ResourceHandle, ResourceRecord, ResourceState, UntypedResourceHandle,
};

// Resource classification remains shared with interface and editor consumers.
pub use zircon_runtime_interface::resource::{ResourceKind, ResourceMarker};

// Typed asset markers project the shared resource vocabulary without a second runtime definition.
pub use zircon_runtime_interface::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, DataMarker, FontMarker, MaterialGraphMarker, MaterialMarker,
    MeshMarker, ModelMarker, NavMeshMarker, NavigationSettingsMarker, PhysicsMaterialMarker,
    PrefabMarker, SceneMarker, ShaderMarker, SoundMarker, TerrainLayerStackMarker, TerrainMarker,
    TextureMarker, TileMapMarker, TileSetMarker, UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};

#[cfg(test)]
mod tests;

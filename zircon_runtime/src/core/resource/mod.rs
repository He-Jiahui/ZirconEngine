//! Curated Runtime projection of the canonical Resource foundation.

pub mod io;

pub use zr_resource::ResourceData;
pub use zr_resource::ResourceLease;
pub use zr_resource::ResourceRegistry;
pub use zr_resource::ResourceSnapshot;
pub use zr_resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, DataMarker, FontMarker, MaterialGraphMarker, MaterialMarker,
    MeshMarker, ModelMarker, NavMeshMarker, NavigationSettingsMarker, PhysicsMaterialMarker,
    PrefabMarker, SceneMarker, ShaderMarker, SoundMarker, TerrainLayerStackMarker, TerrainMarker,
    TextureMarker, TileMapMarker, TileSetMarker, UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};
pub use zr_resource::{
    AssetReference, AssetUuid, ResourceId, ResourceLocator, ResourceLocatorError, ResourceScheme,
    STABLE_UUID_ALGORITHM_VERSION,
};
pub use zr_resource::{Resource, ResourceRuntimeInfo, RuntimeResourceState};
pub use zr_resource::{
    ResourceDiagnostic, ResourceDiagnosticSeverity, ResourceEvent, ResourceEventKind,
    ResourceHandle, ResourceRecord, ResourceState, UntypedResourceHandle,
};
pub use zr_resource::{
    ResourceEventGap, ResourceEventReceiver, ResourceEventRecvError, ResourceEventRecvTimeoutError,
    ResourceEventStreamDiagnostics, ResourceEventTryRecvError,
};
pub use zr_resource::{ResourceKind, ResourceMarker};
pub use zr_resource::{
    ResourceManagementGeneration, ResourceManagementGenerationDiagnostics,
    ResourceManagementGenerationIdentity, ResourceManagementKindSummary, ResourceManagementPage,
    ResourceManagementQuery, ResourceManagementRow, ResourceManagementRowIdentity,
    ResourceManagementScan, ResourceManagementSummary,
};
pub use zr_resource::{ResourceManager, ResourceProjectionSnapshot, ResourceRegistryReadGuard};
pub use zr_resource::{ResourceMutationBatch, ResourceMutationReceipt};
pub use zr_resource::{
    ResourceReadinessGeneration, ResourceReadinessGenerationDiagnostics,
    ResourceReadinessGenerationIdentity, ResourceReadinessRow, ResourceReadinessRowIdentity,
    ResourceReadinessState,
};
pub use zr_resource::{ResourceRegistryError, ResourceResult};

pub(crate) use zr_resource::assembly::{
    PreparedResourceMutation, ResourceManagerAssemblyExt, ResourceReadinessGenerationAssemblyExt,
    ResourceRegistryAssemblyExt, ResourceRegistryStaging, approximate_event_bytes,
};

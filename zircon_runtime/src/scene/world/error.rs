use thiserror::Error;
use zircon_runtime_interface::reflect::ReflectError;

use crate::scene::{
    ecs::{EntityRegistryError, ObserverId, StorageError},
    EntityId,
};

pub type SceneResult<T> = std::result::Result<T, SceneError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SceneError {
    #[error("cannot {operation} missing entity {entity}")]
    MissingEntity {
        operation: &'static str,
        entity: EntityId,
    },
    #[error("entity {entity} is missing required {component} for {operation}")]
    MissingRequiredComponent {
        operation: &'static str,
        entity: EntityId,
        component: &'static str,
    },
    #[error("entity {entity} already exists")]
    DuplicateEntity { entity: EntityId },
    #[error("observer {} is not registered", observer.index())]
    MissingObserver { observer: ObserverId },
    #[error("entity {entity} cannot advance the world identity allocator")]
    EntityIdExhausted { entity: EntityId },
    #[error("node name cannot be empty")]
    EmptyNodeName,
    #[error("joint on entity {entity} cannot connect to itself")]
    JointConnectsToSelf { entity: EntityId },
    #[error("entity {entity} cannot become its own parent")]
    EntityCannotParentItself { entity: EntityId },
    #[error("entity {child} cannot use missing parent {parent}")]
    MissingParent { child: EntityId, parent: EntityId },
    #[error("reparenting entity {child} under parent {parent} would create a hierarchy cycle")]
    HierarchyCycle { child: EntityId, parent: EntityId },
    #[error("entity {entity} cannot become Dynamic while it owns Static children")]
    DynamicMobilityWithStaticChildren { entity: EntityId },
    #[error("entity {entity} cannot become Static under Dynamic parent {parent}")]
    StaticMobilityUnderDynamicParent { entity: EntityId, parent: EntityId },
    #[error("static entity {entity} cannot update transform at runtime")]
    StaticTransformMutation { entity: EntityId },
    #[error("static entity {entity} cannot be reparented during runtime mutation")]
    StaticReparentMutation { entity: EntityId },
    #[error(transparent)]
    EntityRegistry(#[from] EntityRegistryError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Reflect(#[from] ReflectError),
    #[error("component type {type_id} must be prefixed by plugin id {plugin_id}")]
    ComponentTypePluginPrefixMismatch { type_id: String, plugin_id: String },
    #[error("component type {type_id} already registered")]
    DuplicateComponentType { type_id: String },
    #[error("dynamic component type `{component_id}` is not registered")]
    UnregisteredDynamicComponentType { component_id: String },
    #[error(
        "plugin `{plugin_id}` cannot unload while dynamic components are active: {active_components}"
    )]
    PluginComponentsActive {
        plugin_id: String,
        active_components: String,
    },
    #[error("unknown property `{property_path}`")]
    UnknownDynamicComponentProperty { property_path: String },
    #[error("property `{property_path}` cannot be written to a dynamic component")]
    DynamicComponentPropertyUnsupportedValue { property_path: String },
    #[error("dynamic component `{component_id}` is not an object")]
    DynamicComponentNotObject { component_id: String },
    #[error("dynamic component type `{component_id}` does not declare property `{property}`")]
    UndeclaredDynamicComponentProperty {
        component_id: String,
        property: String,
    },
    #[error("dynamic component property `{component_id}.{property}` is not editable")]
    NonEditableDynamicComponentProperty {
        component_id: String,
        property: String,
    },
    #[error("property `{property_path}` is not available on entity {entity}")]
    PropertyUnavailable {
        entity: EntityId,
        property_path: String,
    },
    #[error("property `{property_path}` expects {expected} segments, found {actual}")]
    PropertySegmentCount {
        property_path: String,
        expected: usize,
        actual: usize,
    },
    #[error("unknown property `{property_path}`")]
    UnknownProperty { property_path: String },
    #[error("entity {entity} does not expose property `{property_path}`")]
    MissingPropertyComponent {
        entity: EntityId,
        property_path: String,
    },
    #[error("property `{property_path}` expected {expected}")]
    PropertyTypeMismatch {
        property_path: String,
        expected: String,
    },
    #[error("unknown {axis_kind} in property `{property_path}`")]
    UnknownPropertyAxis {
        property_path: String,
        axis_kind: &'static str,
    },
    #[error("property `{property_path}` rejects zero-length quaternion")]
    ZeroLengthQuaternion { property_path: String },
    #[error("entity {entity} has zero {axis}-scale in its transform")]
    ZeroScaleTransform {
        entity: EntityId,
        axis: &'static str,
    },
    #[error("property `{property_path}` expected finite {expected}")]
    NonFinitePropertyValue {
        property_path: String,
        expected: &'static str,
    },
    #[error("property `{property_path}` has invalid resource id: {source_message}")]
    InvalidPropertyResourceId {
        property_path: String,
        source_message: String,
    },
    #[error("unsupported {kind} `{value}`")]
    UnsupportedPropertyValue { kind: &'static str, value: String },
    #[error("property {property_path} is read-only {reason}")]
    ReadOnlyProperty {
        property_path: String,
        reason: &'static str,
    },
    #[error("property `{property_path}` has an invalid {index_kind}")]
    InvalidPropertyIndex {
        property_path: String,
        index_kind: &'static str,
    },
    #[error("bundle commit requires final-state validation")]
    BundleFinalStateNotValidated,
    #[error("bundle cannot contain duplicate component types")]
    DuplicateBundleComponentType,
    #[error("bundle accepts at most {limit} staged component values")]
    BundleComponentLimitExceeded { limit: usize },
    #[error("bundle preflight can reserve at most {limit} component types")]
    BundleTypeReservationLimitExceeded { limit: usize },
    #[error("bundle transaction invariant failed: {reason}")]
    BundleTransactionInvariant { reason: &'static str },
    #[error("detached entity batch invariant failed: {reason}")]
    DetachedEntityBatchInvariant { reason: &'static str },
    #[error("{0}")]
    Message(String),
}

impl SceneError {
    pub(crate) fn missing_entity(operation: &'static str, entity: EntityId) -> Self {
        Self::MissingEntity { operation, entity }
    }
}

impl From<String> for SceneError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}

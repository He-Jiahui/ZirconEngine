/// Marker for values stored in the runtime world's typed resource table.
///
/// The marker is a neutral framework contract: concrete ECS storage and scheduling remain owned by
/// the scene domain, while framework DTOs and plugins can declare resource eligibility without
/// importing that implementation domain.
pub trait SceneResource: 'static + Send + Sync {}

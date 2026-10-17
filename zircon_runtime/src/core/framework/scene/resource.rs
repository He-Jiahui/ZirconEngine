/// Marker for values stored in the runtime world's typed resource table.
///
/// The marker is a neutral framework contract: concrete ECS storage and scheduling remain owned by
/// the scene domain, while framework DTOs and plugins can declare resource eligibility without
/// importing that implementation domain.
///
/// 资源值须满足 `'static + Send + Sync`，可供 World 的类型化资源表持有。
/// 该标记不插入资源实例或安装反射适配器，实例插入/移除和借用期由 World API 管理。
/// 参见 ECS 的 `Resource` 重导出及 `World::insert_resource`、`World::get_resource_mut`。
pub trait SceneResource: 'static + Send + Sync {}

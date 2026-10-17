//! Runtime registry service object storage slot.

use std::any::Any;
use std::sync::Arc;

/// 注册表持有的类型擦除服务实例；解析端再按请求类型下转。
///
/// 调用许可与旧句柄失效由 ServiceEntry 的代际和调用门禁控制，而非此 Arc 本身。
/// 这里的 `'static` 类型擦除不授予永久调用权；实例的最后一个共享所有者释放时才析构。
pub type ServiceObject = Arc<dyn Any + Send + Sync>;

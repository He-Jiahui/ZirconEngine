use std::sync::Arc;

use super::super::weak::CoreWeak;
use super::ServiceObject;
use crate::core::CoreError;

/// 驱动和管理器的延迟构造入口；弱句柄避免工厂被注册表持有时形成 Runtime 引用环。
///
/// 依赖由解析器先行满足；工厂可临时升级并解析依赖；失败通过 CoreError 返回解析调用链。
///
/// 输入弱句柄仅借用到本次调用结束，返回的类型擦除实例须拥有自身数据。
/// Core 在依赖解析后、注册表锁外执行工厂；成功返回仍须通过槽位代次与状态的发布检查。
/// 该解析入口会将工厂错误包装为初始化错误，将 unwind 映射为工厂 panic 错误。
pub type ServiceFactory = Arc<dyn Fn(&CoreWeak) -> Result<ServiceObject, CoreError> + Send + Sync>;

use std::sync::Arc;

use super::super::weak::CoreWeak;
use super::ServiceObject;
use crate::core::CoreError;

/// 驱动和管理器的延迟构造入口；弱句柄避免工厂被注册表持有时形成 Runtime 引用环。
///
/// 依赖由解析器先行满足；工厂可临时升级并解析依赖；失败通过 CoreError 返回解析调用链。
pub type ServiceFactory = Arc<dyn Fn(&CoreWeak) -> Result<ServiceObject, CoreError> + Send + Sync>;

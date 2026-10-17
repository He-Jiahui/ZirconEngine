use std::sync::Arc;

use super::super::contexts::PluginContext;
use super::ServiceObject;
use crate::core::CoreError;

/// 插件实例的延迟创建入口；Core 在依赖解析后传入当前 PluginContext。
///
/// 闭包可能在没有包路径的内建注册场景执行，应只按上下文实际提供的能力访问宿主。
///
/// 输入上下文只借用到本次调用结束；需留存回访入口时可克隆其中的弱句柄。
/// 返回值须拥有自身数据，不能借用这次上下文；Core 解析器还会重验槽位状态后才发布。
/// 在该解析入口，工厂返回错误会包装为初始化错误，unwind 会映射为工厂 panic 错误。
pub type PluginFactory =
    Arc<dyn Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync>;

use std::sync::Arc;

use super::super::contexts::PluginContext;
use super::ServiceObject;
use crate::core::CoreError;

/// 插件实例的延迟创建入口；Core 在依赖解析后传入当前 PluginContext。
///
/// 闭包可能在没有包路径的内建注册场景执行，应只按上下文实际提供的能力访问宿主。
pub type PluginFactory =
    Arc<dyn Fn(&PluginContext) -> Result<ServiceObject, CoreError> + Send + Sync>;

//! 将包级与模块级清单构造器分开维护，集中组织模块和事件消费者的构造依赖。
mod module;
mod package;

use super::PluginEventConsumerManifest;

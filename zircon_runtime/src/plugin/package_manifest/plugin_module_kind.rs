//! 模块运行域的清单标识；注册排序、原生入口选择与目标校验据此分流。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 声明模块所属运行域；它不表示该域在当前宿主已提供可执行入口。
pub enum PluginModuleKind {
    Runtime,
    Editor,
    Native,
    Vm,
}

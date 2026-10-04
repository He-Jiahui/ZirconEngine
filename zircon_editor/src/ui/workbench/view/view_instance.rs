//! 项目工作区持久化和运行时注册表共用的pane实例快照；布局位置会在恢复后依据布局树校正。
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ViewDescriptorId, ViewHost, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 实例身份与描述符身份分别标识具体pane和其种类；反序列化不执行交叉校验。
pub struct ViewInstance {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
    pub serializable_payload: Value,
    pub dirty: bool,
    pub host: ViewHost,
}

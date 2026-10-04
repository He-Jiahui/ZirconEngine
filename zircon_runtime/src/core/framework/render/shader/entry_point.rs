use serde::{Deserialize, Serialize};

use super::RenderShaderStage;

/// 资源导入后供 pass 构建器校验的入口名称与阶段；应先确认阶段匹配，再交给后端创建管线。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderShaderEntryPointDescriptor {
    pub name: String,
    pub stage: RenderShaderStage,
}

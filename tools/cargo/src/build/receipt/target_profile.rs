//! 构建配置与 Cargo 图的收据身份。
//! 构建 owner 记录 target/profile/代码生成标志与依赖图；验收通过闭包检测产物是否对应预期配置。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetProfile {
    pub target_triple: String,
    pub cargo_profile: String,
    pub codegen_flags_digest: String,
    pub cargo_graph_digest: String,
}

use serde::{Deserialize, Serialize};

/// 声明服务禁用、模拟或仅查询的运行意图；实际可用性另由后端状态报告。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsSimulationMode {
    #[default]
    Disabled,
    Simulate,
    QueryOnly,
}

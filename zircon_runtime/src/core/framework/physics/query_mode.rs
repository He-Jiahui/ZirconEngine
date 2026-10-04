use serde::{Deserialize, Serialize};

/// 控制结果数量与顺序：首个依遍历顺序，最近依查询度量，全部按度量排序。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsQueryMode {
    First,
    #[default]
    Closest,
    All,
}

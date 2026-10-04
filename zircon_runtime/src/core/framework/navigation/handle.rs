use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// 管理器装载网格后分配的运行时句柄；查询可指定句柄，省略时由管理器选取默认已加载网格。
pub struct NavMeshHandle(pub u64);

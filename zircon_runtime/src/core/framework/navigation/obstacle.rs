use serde::{Deserialize, Serialize};

use crate::core::math::Real;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NavMeshObstacleShape {
    Box,
    Capsule,
}

impl Default for NavMeshObstacleShape {
    fn default() -> Self {
        Self::Box
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
/// 场景障碍契约；静态烘焙和运行时 TileCache 会分别读取 carve，避障系统读取 avoidance_enabled。
pub struct NavMeshObstacleDescriptor {
    pub shape: NavMeshObstacleShape,
    pub center: [Real; 3],
    pub size: [Real; 3],
    pub radius: Real,
    pub height: Real,
    pub avoidance_enabled: bool,
    pub carve: bool,
    // TODO: [CR-NAVIGATION-0004] 核实移动阈值、静止时间和仅静止雕刻的契约；当前障碍采集与 TileCache 同步只读取 carve，未读取这组三字段。
    pub move_threshold: Real,
    pub time_to_stationary: Real,
    pub carve_only_stationary: bool,
}

impl Default for NavMeshObstacleDescriptor {
    fn default() -> Self {
        Self {
            shape: NavMeshObstacleShape::Box,
            center: [0.0, 0.0, 0.0],
            size: [1.0, 1.0, 1.0],
            radius: 0.5,
            height: 2.0,
            avoidance_enabled: true,
            carve: false,
            move_threshold: 0.1,
            time_to_stationary: 0.5,
            carve_only_stationary: true,
        }
    }
}

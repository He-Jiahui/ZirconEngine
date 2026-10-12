#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 探针场景数据的整数编码；位置使用带偏移的有符号坐标，半径使用正值尺度。
pub struct HybridGiResolveProbeSceneData {
    position_x_q: u32,
    position_y_q: u32,
    position_z_q: u32,
    radius_q: u32,
}

impl HybridGiResolveProbeSceneData {
    pub fn new(position_x_q: u32, position_y_q: u32, position_z_q: u32, radius_q: u32) -> Self {
        Self {
            position_x_q,
            position_y_q,
            position_z_q,
            radius_q,
        }
    }

    pub fn position_x_q(&self) -> u32 {
        self.position_x_q
    }

    pub fn position_y_q(&self) -> u32 {
        self.position_y_q
    }

    pub fn position_z_q(&self) -> u32 {
        self.position_z_q
    }

    pub fn radius_q(&self) -> u32 {
        self.radius_q
    }
}

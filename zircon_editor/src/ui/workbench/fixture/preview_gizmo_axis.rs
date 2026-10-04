//! 预览样本的坐标轴序列化形状；写入编辑器快照前需转换为viewport轴身份。
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PreviewGizmoAxis {
    X,
    Y,
    Z,
}

use serde::{Deserialize, Serialize};

/// 区分语义图标与普通图像引用，供 Runtime 和 Editor 选择各自的资源查找及后备绘制路径。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiVisualAssetRef {
    Icon(String),
    Image(String),
}

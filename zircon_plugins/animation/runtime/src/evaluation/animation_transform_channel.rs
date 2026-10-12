//! 评估错误中的平移、旋转和缩放通道标识，供资源作者定位不合法键数据。
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationTransformChannel {
    Translation,
    Rotation,
    Scale,
}

impl fmt::Display for AnimationTransformChannel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Translation => "translation",
            Self::Rotation => "rotation",
            Self::Scale => "scale",
        })
    }
}

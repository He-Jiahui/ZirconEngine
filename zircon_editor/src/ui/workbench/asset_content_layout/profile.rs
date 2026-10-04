#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 资产指针与内容布局的宿主类别；字符串解析拒绝未知模式以免套用错误几何。
pub(crate) enum AssetContentSurfaceProfile {
    Activity,
    Browser,
}

impl AssetContentSurfaceProfile {
    pub(crate) fn from_surface_mode(surface_mode: &str) -> Option<Self> {
        match surface_mode {
            "activity" => Some(Self::Activity),
            "browser" => Some(Self::Browser),
            _ => None,
        }
    }
}

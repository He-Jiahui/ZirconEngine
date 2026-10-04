use crate::core::framework::render::ViewportIconId;

use super::ViewportIconSource;

/// 无图标资源的默认注入源；场景辅助线仍由几何回退绘制，普通运行入口无需携带编辑器素材。
#[derive(Debug, Default)]
pub(crate) struct EmptyViewportIconSource;

impl ViewportIconSource for EmptyViewportIconSource {
    fn bytes(&self, _id: ViewportIconId) -> Option<&'static [u8]> {
        None
    }
}

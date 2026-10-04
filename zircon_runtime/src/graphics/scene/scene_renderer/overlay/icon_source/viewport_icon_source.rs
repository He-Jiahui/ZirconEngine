use crate::core::framework::render::ViewportIconId;

/// 将宿主图标素材注入视口绘制；返回字节会被解码为亮度遮罩，再由场景图标颜色着色。
/// 字节必须可跨帧共享；缺失结果会被缓存，替换素材需重建图标缓存。
pub(crate) trait ViewportIconSource: Send + Sync + 'static {
    fn bytes(&self, id: ViewportIconId) -> Option<&'static [u8]>;
}
